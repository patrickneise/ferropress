use crate::engine;
use crate::models::{ProjectPaths, ServeMode, SiteConfig};
use anyhow::{Context, Result};
use axum::Router;
use std::net::SocketAddr;
use std::process::Stdio;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
use tower_http::services::{ServeDir, ServeFile};
use tower_livereload::LiveReloadLayer;

pub async fn execute(mode: ServeMode) -> Result<()> {
    let paths = ProjectPaths::default();
    let config = SiteConfig::load(&paths.config).context("Failed to load site.toml")?;
    let token = CancellationToken::new();

    paths.clean_dist().context("Failed to clean dist/")?;
    paths
        .create_dist_folders()
        .context("Failed to create dist/")?;

    // livereload
    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();

    let serve_dir =
        ServeDir::new(&paths.dist).fallback(ServeFile::new(paths.dist.join("404.html")));

    let mut app = Router::new().fallback_service(serve_dir);
    if mode == ServeMode::Dev {
        app = app.layer(livereload);
    }

    // In prod, build CSS once up front.
    if mode == ServeMode::Prod {
        engine::build_css(&paths).context("Tailwind build failed")?;
    }

    // Copy + render once before starting the server
    engine::copy_static_assets(&paths).context("Copying static assets failed")?;
    engine::render_site(&paths, &config).context("Rendering site failed")?;

    if mode == ServeMode::Dev {
        spawn_tailwind_watch(paths.clone(), token.clone());
        spawn_watcher(paths.clone(), reloader.clone(), token.clone());
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind {}", addr))?;

    println!("🌍 Listening on {} (open http://localhost:3000)", addr);

    let server_token = token.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            println!("\n🛑 Quenching the flames...");
            server_token.cancel();
        })
        .await
        .context("Server exited with error")?;

    Ok(())
}

fn spawn_tailwind_watch(paths: ProjectPaths, token: CancellationToken) {
    tokio::spawn(async move {
        let tw_path = match engine::assets::get_tailwind_exe() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("🛑 Tailwind not available: {:#}", e);
                return;
            }
        };

        let input_css = paths.input_css_file();
        let output_css = paths.output_css_file();

        let mut child = match Command::new(tw_path)
            .arg("-i")
            .arg(&input_css)
            .arg("-o")
            .arg(&output_css)
            .arg("--watch")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("🛑 Failed to start Tailwind: {:#}", e);
                return;
            }
        };

        tokio::select! {
            _ = token.cancelled() => { let _ = child.kill().await; }
            _ = child.wait() => {}
        }
    });
}

fn spawn_watcher(
    watch_paths: ProjectPaths,
    reloader: tower_livereload::Reloader,
    token: CancellationToken,
) {
    tokio::task::spawn_blocking(move || {
        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("🛑 Watcher init failed: {:#}", e);
                return;
            }
        };

        for (path, mode) in [
            (&watch_paths.content, RecursiveMode::Recursive),
            (&watch_paths.templates, RecursiveMode::Recursive),
            (&watch_paths.static_files, RecursiveMode::Recursive),
            (&watch_paths.config, RecursiveMode::NonRecursive),
        ] {
            if let Err(e) = watcher.watch(path, mode) {
                eprintln!("🛑 Failed to watch {}: {:#}", path.display(), e);
                return;
            }
        }

        while !token.is_cancelled() {
            match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(Ok(event)) => {
                    if event.kind.is_access() {
                        continue;
                    }

                    // debounce
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    while rx.try_recv().is_ok() {}

                    println!("♻️  Change detected. Recasting...");

                    let result: Result<()> = (|| {
                        let latest_config = SiteConfig::load(&watch_paths.config)
                            .context("Failed to load site.toml")?;
                        engine::copy_static_assets(&watch_paths)
                            .context("Copying static assets failed")?;
                        engine::render_site(&watch_paths, &latest_config)
                            .context("Rendering site failed")?;
                        Ok(())
                    })();

                    match result {
                        Ok(()) => reloader.reload(),
                        Err(e) => eprintln!("🛑 Build failed: {:#}", e),
                    }
                }
                Ok(Err(e)) => eprintln!("⚠️  Watch event error: {:#}", e),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}

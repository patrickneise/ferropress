use crate::engine;
use crate::models::{ProjectPaths, SiteConfig};
use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use std::process::Stdio;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
use tower_http::services::{ServeDir, ServeFile};
use tower_livereload::LiveReloadLayer;

pub async fn execute(watch: bool) -> Result<()> {
    let paths = ProjectPaths::default();
    let config = SiteConfig::load(&paths.config)?;
    let token = CancellationToken::new();

    // start from clean slate and build structure
    paths.clean_dist()?;
    paths.create_dist_folders()?;

    // setup web server
    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();

    let serve_dir =
        ServeDir::new(&paths.dist).fallback(ServeFile::new(paths.dist.join("404.html")));

    let mut app = Router::new().fallback_service(serve_dir);
    if watch {
        app = app.layer(livereload);
    }

    // start sidecars (Tailwind & Watcher)
    if watch {
        // tailwind sidecar
        let tw_token = token.clone();
        let tw_paths = paths.clone();
        tokio::spawn(async move {
            let tw_path = engine::assets::get_tailwind_exe().unwrap();

            let input_css = tw_paths.input_css_file();
            let output_css = tw_paths.output_css_file();
            let mut child = Command::new(tw_path)
                .args([
                    "-i",
                    input_css.to_str().unwrap(),
                    "-o",
                    output_css.to_str().unwrap(),
                    "--watch",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("Failed to start Tailwind");

            tokio::select! {
                _ = tw_token.cancelled() => { let _ = child.kill().await; }
                _ = child.wait() => {}
            }
        });

        // file watcher sidecar
        let watch_paths = paths.clone();
        let watch_reloader = reloader.clone();
        let watch_token = token.clone();

        tokio::task::spawn_blocking(move || {
            use notify::{Config, RecursiveMode, Watcher};
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = notify::RecommendedWatcher::new(tx, Config::default()).unwrap();

            watcher
                .watch(&watch_paths.content, RecursiveMode::Recursive)
                .ok();
            watcher
                .watch(&watch_paths.templates, RecursiveMode::Recursive)
                .ok();
            watcher
                .watch(&watch_paths.static_files, RecursiveMode::Recursive)
                .ok();
            watcher
                .watch(&watch_paths.config, RecursiveMode::NonRecursive)
                .ok();

            while !watch_token.is_cancelled() {
                if let Ok(Ok(event)) = rx.recv_timeout(std::time::Duration::from_millis(200)) {
                    if event.kind.is_access() {
                        continue;
                    }

                    // debounce
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    while rx.try_recv().is_ok() {} // Drain buffer

                    println!("♻️  Change detected. Recasting...");

                    // copy static assets and render site
                    let rebuild = || -> Result<()> {
                        let latest_config = SiteConfig::load(&watch_paths.config)?;
                        engine::copy_static_assets(&watch_paths)?;
                        engine::render_site(&watch_paths, &latest_config)?;
                        Ok(())
                    };

                    if let Err(e) = rebuild() {
                        eprintln!("🛑 Build failed: {:?}", e);
                    } else {
                        // Give Tailwind sidecar a moment, then refresh
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        watch_reloader.reload();
                    }
                }
            }
        });
    } else {
        engine::build_css(&paths)?;
    }

    engine::copy_static_assets(&paths)?;
    engine::render_site(&paths, &config)?;

    // run server with graceful shutdown
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("🔥 THE HEARTH IS GLOWING (Dev Mode: {})", watch);
    println!("🌍 URL: http://localhost:3000");

    let server_token = token.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.ok();
            println!("\n🛑 Quenching the flames...");
            server_token.cancel()
        })
        .await?;

    std::process::exit(0);
}

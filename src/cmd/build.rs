use crate::engine;
use crate::models::{ProjectPaths, SiteConfig};
use anyhow::{Context, Result};

pub async fn execute() -> Result<()> {
    let start = std::time::Instant::now();

    let paths = ProjectPaths::default();
    let config = SiteConfig::load(&paths.config).context("Failed to load site.toml")?;

    println!("📦 THE CASTING: Preparing Production Build...");

    // start with clean slate and proper structure
    paths.clean_dist().context("Failed to clean dist/")?;
    paths
        .create_dist_folders()
        .context("Failed to create dist/")?;

    // build and copy assets then render pages
    engine::build_css(&paths).context("Tailwind build failed")?;
    engine::copy_static_assets(&paths).context("Copying static assets failed")?;
    engine::render_site(&paths, &config).context("Rendering site failed")?;

    println!("🏆 CASTING COMPLETE in {:?}.", start.elapsed());
    Ok(())
}

use crate::engine;
use crate::models::{ProjectPaths, SiteConfig};
use anyhow::Result;

pub fn execute() -> Result<()> {
    let start = std::time::Instant::now();

    let paths = ProjectPaths::default();
    let config = SiteConfig::load(&paths.config)?;

    println!("📦 THE CASTING: Preparing Production Build...");

    // start with clean slate and proper structure
    paths.clean_dist()?;
    paths.create_dist_folders()?;

    // build and copy assets then render pages
    engine::build_css(&paths)?;
    engine::copy_static_assets(&paths)?;
    engine::render_site(&paths, &config)?;

    let duration = start.elapsed();
    println!("🏆 CASTING COMPLETE in {:?}.", duration);
    Ok(())
}

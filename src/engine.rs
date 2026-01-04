pub mod assets;
pub mod parser;
pub mod render;
pub mod utils;

use crate::{
    engine::render::Renderer,
    models::{ProjectPaths, RenderedPage, SiteConfig},
};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Stdio;

pub fn render_site(paths: &ProjectPaths, config: &SiteConfig) -> Result<()> {
    // initialize renderer
    let renderer = Renderer::new(&paths.templates)?;

    // process content
    let posts = parser::parse_all_posts(&paths.content)?;
    let rendered_posts = renderer.render_all_posts(&posts, config)?;

    let rendered_pages = renderer.render_all_pages(&posts, paths, config)?;

    // write HTML files
    write_pages(&paths.dist, rendered_posts)?;
    write_pages(&paths.dist, rendered_pages)?;

    Ok(())
}

pub fn build_css(paths: &ProjectPaths) -> Result<()> {
    paths.create_dist_folders()?;

    println!("🎨 TAILWIND: Weaving v4 styles...");
    let tailwind_path = crate::engine::assets::get_tailwind_exe()?;

    let input_css = paths.input_css_file();
    let output_css = paths.output_css_file();

    let status = std::process::Command::new(tailwind_path)
        .arg("-i")
        .arg(&input_css)
        .arg("-o")
        .arg(&output_css)
        .arg("--minify")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .context("TailwindCSS execution failed")?;

    if !status.success() {
        anyhow::bail!("TailwindCSS exited with status: {}", status);
    }

    Ok(())
}

pub fn copy_static_assets(paths: &ProjectPaths) -> Result<()> {
    paths.create_dist_folders()?;

    if paths.static_files.exists() {
        for entry in fs::read_dir(&paths.static_files)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.file_name().unwrap_or_default() == "css" {
                continue;
            }

            let dst = paths.dist_static().join(entry.file_name());
            if path.is_dir() {
                copy_dir_all(&path, &dst)?;
            } else {
                fs::copy(&path, &dst)?;
            }
        }
    }
    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

fn write_pages(dist_root: &Path, pages: Vec<RenderedPage>) -> Result<()> {
    for page in pages {
        let slug = page.slug.trim_matches('/');

        // Pretty URL rules:
        // - "index" -> /index.html
        // - "404"   -> /404.html   (so ServeFile fallback works)
        // - otherwise -> /<slug>/index.html
        let output_path = if slug == "index" {
            dist_root.join("index.html")
        } else if slug == "404" {
            dist_root.join("404.html")
        } else {
            dist_root.join(slug).join("index.html")
        };

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        fs::write(&output_path, page.html)
            .with_context(|| format!("Failed to write page: {:?}", output_path))?;
    }

    Ok(())
}

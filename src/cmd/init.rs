use crate::models::{ProjectPaths, SiteConfig};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub const DEFAULT_LAYOUT: &str = include_str!("../../defaults/templates/layouts/base.html");
pub const DEFAULT_INDEX: &str = include_str!("../../defaults/templates/pages/index.html");
pub const DEFAULT_404: &str = include_str!("../../defaults/templates/pages/404.html");
pub const DEFAULT_POST_TEMPLATE: &str = include_str!("../../defaults/templates/post.html");
pub const DEFAULT_CSS: &str = include_str!("../../defaults/static/css/input.css");
pub const DEFAULT_HTMX: &str = include_str!("../../defaults/static/js/htmx.min.js");
pub const EXAMPLE_POST_1: &str = include_str!("../../defaults/content/posts/hello.md");
pub const EXAMPLE_POST_2: &str = include_str!("../../defaults/content/posts/2025/hello.md");

pub fn execute(force: bool, name: Option<String>) -> Result<()> {
    // determine project root
    let root = match name {
        Some(n) => PathBuf::from(n),
        None => {
            if force {
                PathBuf::from(".")
            } else {
                PathBuf::from("my-forge")
            }
        }
    };

    // safety check to avoid overwriting project
    if root.exists() && !force {
        anyhow::bail!(
            "❌ Directory '{}' already exists. Use --force to overwrite or choose a new name.",
            root.display()
        );
    }

    // create root if it doesn't exist
    if !root.exists() {
        fs::create_dir_all(&root)?;
    }

    // build source structure
    let paths = ProjectPaths::from_root(&root);
    let dirs = [
        &paths.content,
        &paths.templates,
        &paths.static_files,
        &paths.static_files.join("css"),
        &paths.static_files.join("js"),
    ];
    for dir in dirs {
        fs::create_dir_all(dir).with_context(|| format!("Failed to create {:?}", dir))?;
    }

    println!("⚒️  FERROPRESS: Drafting the Blueprints...");

    // generate site.toml from SiteConfig::default()
    let default_config = SiteConfig::default();
    let config_str = toml::to_string_pretty(&default_config)
        .context("Failed to serialize default SiteConfig to TOML")?;

    // define blueprints (source content, desination path)
    let blueprints: Vec<(&str, PathBuf)> = vec![
        (&config_str, paths.config.clone()),
        (
            DEFAULT_LAYOUT,
            paths.templates.join("layouts").join("base.html"),
        ),
        (
            DEFAULT_INDEX,
            paths.templates.join("pages").join("index.html"),
        ),
        (DEFAULT_404, paths.templates.join("pages").join("404.html")),
        (DEFAULT_POST_TEMPLATE, paths.templates.join("post.html")),
        (
            DEFAULT_CSS,
            paths.static_files.join("css").join("input.css"),
        ),
        (
            DEFAULT_HTMX,
            paths.static_files.join("js").join("htmx.min.js"),
        ),
        (EXAMPLE_POST_1, paths.content.join("posts").join("hello.md")),
        (
            EXAMPLE_POST_2,
            paths.content.join("posts").join("2025").join("hello.md"),
        ),
    ];

    // execute blueprints
    for (content, dest) in blueprints {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, content)?;
    }

    match init_git(&root) {
        Ok(true) => {
            let gitignore = "/dist\n/target\n.DS_Store\n";
            fs::write(root.join(".gitignore"), gitignore)?;
            println!("🌱 Git repository initialized with .gitignore");
        }
        Ok(false) => {
            println!("⚠️  Git executable not found. Skipping repository initialization.");
        }
        Err(e) => {
            println!(
                "⚠️  Git init failed, but the forge was still created: {}",
                e
            );
        }
    }

    println!("✅ SUCCESS: Workshop ready. Try running 'ferropress preview'!");
    Ok(())
}

fn init_git(root: &PathBuf) -> Result<bool> {
    // check for `git` binary
    let git_check = Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match git_check {
        Ok(status) if status.success() => {
            let output = Command::new("git")
                .arg("init")
                .arg("-b")
                .arg("main")
                .arg(root)
                .output()
                .context("Failed to execute git init")?;

            Ok(output.status.success())
        }
        _ => Ok(false), // Git not found or command failed to start
    }
}

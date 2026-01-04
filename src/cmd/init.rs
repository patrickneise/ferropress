use crate::models::{ProjectPaths, SiteConfig};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub const DEFAULT_LAYOUT: &str = include_str!("../../defaults/templates/layouts/base.html");
pub const DEFAULT_INDEX: &str = include_str!("../../defaults/templates/pages/index.html");
pub const DEFAULT_404: &str = include_str!("../../defaults/templates/pages/404.html");
pub const DEFAULT_POST_TEMPLATE: &str = include_str!("../../defaults/templates/post.html");
pub const DEFAULT_CSS: &str = include_str!("../../defaults/static/css/input.css");
pub const DEFAULT_HTMX: &str = include_str!("../../defaults/static/js/htmx.min.js");
pub const EXAMPLE_POST_1: &str = include_str!("../../defaults/content/posts/hello.md");
pub const EXAMPLE_POST_2: &str = include_str!("../../defaults/content/posts/2025/hello.md");

/// Initialize a FerroPress site at `path`.
/// - `overwrite`: overwrite only the scaffold files FerroPress manages
/// - `clean`: remove existing scaffold directories/files before initializing (dangerous)
pub async fn execute(path: PathBuf, overwrite: bool, clean: bool) -> Result<()> {
    let root = path;

    // create root if needed
    if !root.exists() {
        fs::create_dir_all(&root)
            .with_context(|| format!("Failed to create directory {}", root.display()))?;
    }

    let paths = ProjectPaths::from_root(&root);

    if clean {
        clean_scaffold(&paths)?;
    } else if !overwrite {
        // Safe default: refuse if any of our managed scaffold files already exist
        // (prevents accidental clobber in an existing repo)
        ensure_no_scaffold_conflicts(&paths)?;
    }

    println!(
        "⚒️  FERROPRESS: Drafting the Blueprints in {}...",
        root.display()
    );

    // Build site.toml from default config
    let default_config = SiteConfig::default();
    let config_str = toml::to_string_pretty(&default_config)
        .context("Failed to serialize default SiteConfig to TOML")?;

    // define blueprints (content, desination)
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

    for (content, dest) in blueprints {
        // Shouldn't happen because ensure_no_scaffold_conflicts() catches it,
        // but keep this as a safety net.
        if dest.exists() && !overwrite && !clean {
            anyhow::bail!(
                "Refusing to overwrite existing file {}. Re-run with --overwrite.",
                dest.display()
            )
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        fs::write(&dest, content)
            .with_context(|| format!("Failed to write file {}", dest.display()))?;
    }

    // Initialize git only if there's not .git already
    match init_git(&root) {
        Ok(true) => {
            upsert_gitignore(&root)?;
            println!("🌱 Git repository initialized (main) + .gitignore updated");
        }
        Ok(false) => {
            // git not found OR already a git repo
        }
        Err(e) => {
            println!("⚠️  Git init failed (site still created): {:#}", e);
        }
    }

    println!("✅ SUCCESS: Workshop ready.");
    if root.as_path() == Path::new(".") {
        println!("Next: ferropress preview");
    } else {
        println!("Next: cd {} && ferropress preview", root.display());
    }

    Ok(())
}

/// Refuse to overwrite if any managed scaffold files already exist.
fn ensure_no_scaffold_conflicts(paths: &ProjectPaths) -> Result<()> {
    let managed = managed_scaffold_files(paths);
    let conflicts: Vec<PathBuf> = managed.into_iter().filter(|p| p.exists()).collect();

    if !conflicts.is_empty() {
        let mut msg = String::from("Scaffold already exists:\n");
        for c in conflicts {
            msg.push_str(&format!("  - {}\n", c.display()));
        }
        msg.push_str("Re-run with --overwrite to overwrite scaffold files, or --clean to remove scaffold first.");
        anyhow::bail!(msg);
    }

    Ok(())
}

/// Remove known scaffold directories/files (dangerous but bounded).
fn clean_scaffold(paths: &ProjectPaths) -> Result<()> {
    // Prefer boudned deletion: remove only known top-level scaffold dirs/fils.
    // (Safer than nuking the entire root directory.)
    let to_remove_files = [paths.config.clone()];
    for f in to_remove_files {
        if f.exists() {
            fs::remove_file(&f)
                .with_context(|| format!("Failed to remove file {}", f.display()))?;
        }
    }

    let to_remove_dirs = [
        paths.content.clone(),
        paths.templates.clone(),
        paths.static_files.clone(),
    ];
    for d in to_remove_dirs {
        if d.exists() {
            fs::remove_dir_all(&d)
                .with_context(|| format!("Failed to remove directory {}", d.display()))?;
        }
    }

    Ok(())
}

fn managed_scaffold_files(paths: &ProjectPaths) -> Vec<PathBuf> {
    vec![
        paths.config.clone(),
        paths.templates.join("layouts").join("base.html"),
        paths.templates.join("pages").join("index.html"),
        paths.templates.join("pages").join("404.html"),
        paths.templates.join("post.html"),
        paths.static_files.join("css").join("input.css"),
        paths.static_files.join("js").join("htmx.min.js"),
        paths.content.join("posts").join("hello.md"),
        paths.content.join("posts").join("2025").join("hello.md"),
    ]
}

fn init_git(root: &Path) -> Result<bool> {
    // Skip if already a git repo
    if root.join(".git").exists() {
        return Ok(false);
    }

    // check for git binary
    let git_check = Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match git_check {
        Ok(status) if status.success() => {
            let output = Command::new("git")
                .args(["init", "-b", "main"])
                .output()
                .context("Failed to execute git init")?;

            Ok(output.status.success())
        }
        _ => Ok(false), // Git not found or command failed to start
    }
}

fn upsert_gitignore(root: &Path) -> Result<()> {
    let path = root.join(".gitignore");
    let block = "/dist\n/target\n.DS_Store\n";

    if path.exists() {
        let existing = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if !existing.contains("/dist") {
            let mut new_contents = existing;
            if !new_contents.ends_with('\n') {
                new_contents.push('\n');
            }
            new_contents.push_str(block);
            fs::write(&path, new_contents)
                .with_context(|| format!("Failed to write {}", path.display()))?;
        } else {
            fs::write(&path, block)
                .with_context(|| format!("Failed to write {}", path.display()))?;
        }
    }

    Ok(())
}

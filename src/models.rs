use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

// ----------------------
// SiteConfig
// ----------------------
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteConfig {
    pub title: String,
    pub author: String,
    pub footer_text: String,
    pub navbar_links: Vec<NavbarLink>,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "My New Forge".to_string(),
            author: "Ironmaster".to_string(),
            footer_text: "Forged with FerroPress".to_string(),
            navbar_links: vec![
                NavbarLink {
                    label: "Home".into(),
                    url: "/".into(),
                },
                NavbarLink {
                    label: "Posts".into(),
                    url: "/posts".into(),
                },
            ],
        }
    }
}

impl SiteConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|_| {
            anyhow::anyhow!(
                "Could not find {:?}. Did you run 'init' to draft the blueprints?",
                path
            )
        })?;

        let config: Self = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse site.toml: {}", e))?;

        Ok(config)
    }

    pub fn base_context(&self) -> tera::Context {
        let mut ctx = tera::Context::new();
        ctx.insert("site", self);

        // Compute year once per build
        let current_year = chrono::Local::now().format("%Y").to_string();
        ctx.insert("current_year", &current_year);

        ctx
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NavbarLink {
    pub label: String,
    pub url: String,
}

// ----------------------
// Posts
// ----------------------
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostMetadata {
    pub title: String,
    pub date: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub metadata: PostMetadata,
    pub content: String,
    pub slug: String,
}

// ----------------------
// Pages
// ----------------------
#[derive(Debug, Serialize, Clone)]
pub struct RenderedPage {
    pub slug: String,
    pub html: String,
}

// ----------------------
// Directory Management
// ----------------------
#[derive(Clone)]
pub struct ProjectPaths {
    pub content: PathBuf,
    pub templates: PathBuf,
    pub dist: PathBuf,
    pub static_files: PathBuf,
    pub config: PathBuf,
}

impl Default for ProjectPaths {
    fn default() -> Self {
        Self {
            content: PathBuf::from("content"),
            templates: PathBuf::from("templates"),
            dist: PathBuf::from("dist"),
            static_files: PathBuf::from("static"),
            config: PathBuf::from("site.toml"),
        }
    }
}

impl ProjectPaths {
    pub fn from_root<P: Into<PathBuf>>(root: P) -> Self {
        let root = root.into();
        Self {
            content: root.join("content"),
            templates: root.join("templates"),
            dist: root.join("dist"),
            static_files: root.join("static"),
            config: root.join("site.toml"),
        }
    }

    pub fn current() -> Self {
        Self::from_root(".")
    }

    pub fn dist_posts(&self) -> PathBuf {
        self.dist.join("posts")
    }

    pub fn dist_static(&self) -> PathBuf {
        self.dist.join("static")
    }

    pub fn dist_css(&self) -> PathBuf {
        self.dist_static().join("css")
    }

    pub fn dist_js(&self) -> PathBuf {
        self.dist_static().join("js")
    }

    pub fn input_css_file(&self) -> PathBuf {
        self.static_files.join("css").join("input.css")
    }

    pub fn output_css_file(&self) -> PathBuf {
        self.dist_css().join("style.css")
    }

    pub fn clean_dist(&self) -> Result<()> {
        if self.dist.exists() {
            fs::remove_dir_all(&self.dist)
                .with_context(|| format!("Failed to remove dist directory at {:?}", self.dist))?;
            println!("🧹 Dist directory cleared.");
        }
        Ok(())
    }

    pub fn create_dist_folders(&self) -> Result<()> {
        // Create the root dist
        fs::create_dir_all(&self.dist)
            .with_context(|| format!("Failed to create dist directory at {:?}", self.dist))?;

        // Create subdirectories
        fs::create_dir_all(self.dist_posts())
            .with_context(|| format!("Failed to create posts directory at {:?}", self.dist))?;
        fs::create_dir_all(self.dist_css())
            .with_context(|| format!("Failed to create css directory at {:?}", self.dist))?;
        fs::create_dir_all(self.dist_js())
            .with_context(|| format!("Failed to create js directory at {:?}", self.dist))?;

        Ok(())
    }
}

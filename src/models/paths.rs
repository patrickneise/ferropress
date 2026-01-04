use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug)]
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
        fs::create_dir_all(&self.dist)
            .with_context(|| format!("Failed to create dist directory at {:?}", self.dist))?;

        fs::create_dir_all(self.dist_posts()).with_context(|| {
            format!(
                "Failed to create posts directory at {:?}",
                self.dist_posts()
            )
        })?;
        fs::create_dir_all(self.dist_css())
            .with_context(|| format!("Failed to create css directory at {:?}", self.dist_css()))?;
        fs::create_dir_all(self.dist_js())
            .with_context(|| format!("Failed to create js directory at {:?}", self.dist_js()))?;

        Ok(())
    }
}

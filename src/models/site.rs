use anyhow::{Context, Result};
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteConfig {
    pub title: String,
    pub author: String,
    pub footer_text: String,
    #[serde(default)]
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
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Could not read config at {:?}. Did you run 'init'?", path))?;

        let config: Self = toml::from_str(&content)
            .with_context(|| "Failed to parse site.toml (invalid TOML)".to_string())?;

        Ok(config)
    }

    pub fn base_context(&self) -> tera::Context {
        let mut ctx = tera::Context::new();
        ctx.insert("site", self);

        let current_year: i32 = Local::now().year();
        ctx.insert("current_year", &current_year);

        ctx
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NavbarLink {
    pub label: String,
    pub url: String,
}

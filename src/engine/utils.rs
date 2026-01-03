use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Slugify;

impl Slugify {
    /// Strips an explicit prefix, removes extension, and formats for URL.
    pub fn from_path(path: &Path, prefix: &Path) -> String {
        // strip the explicit prefix (e.g., 'content' or 'templates/pages')
        let relative_path = path.strip_prefix(prefix).unwrap_or(path);

        // remove extension, lowercase, and replace spaces
        relative_path
            .with_extension("")
            .to_string_lossy()
            .to_lowercase()
            .replace('\\', "/")
            .replace(' ', "-")
            .trim_matches('/')
            .to_string()
    }
}

pub fn walk_dir(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    // read_dir only iterates the immediate directory
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            files.extend(walk_dir(&path, extension)?);
        } else if path.extension().is_some_and(|ext| ext == extension) {
            files.push(path);
        }
    }
    Ok(files)
}

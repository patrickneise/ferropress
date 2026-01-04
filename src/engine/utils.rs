use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct Slugify;

impl Slugify {
    pub fn from_path(path: &Path, prefix: &Path) -> Result<String> {
        let relative = path
            .strip_prefix(prefix)
            .with_context(|| format!("Path {:?} is not under prefix {:?}", path, prefix))?;

        Ok(relative
            .with_extension("")
            .to_string_lossy()
            .to_lowercase()
            .replace('\\', "/")
            .replace(' ', "-")
            .trim_matches('/')
            .to_string())
    }
}

pub fn walk_dir(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        })
        .collect();

    files.sort();
    Ok(files)
}

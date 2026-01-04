use crate::engine::utils;
use crate::models::{Post, PostMetadata};
use anyhow::{Context, Result};
use gray_matter::Matter;
use std::fs;
use std::path::Path;

pub fn parse_post(path: &Path, prefix: &Path) -> Result<Post> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("Failed to read file at {:?}", path))?;

    let matter = Matter::<gray_matter::engine::YAML>::new();
    let result = matter
        .parse::<PostMetadata>(&raw)
        .with_context(|| format!("Failed to parse front matter in {:?}", path))?;

    let metadata = result
        .data
        .with_context(|| format!("Invalid post front matter in {:?}", path))?;

    let slug = utils::Slugify::from_path(path, prefix)?;

    Ok(Post {
        metadata,
        content: result.content,
        slug,
    })
}

pub fn parse_all_posts(content_dir: &Path) -> Result<Vec<Post>> {
    let posts_dir = content_dir.join("posts");
    let mut posts = Vec::new();

    if !posts_dir.exists() {
        return Ok(posts);
    }

    for entry in utils::walk_dir(&posts_dir, "md")? {
        let post = parse_post(&entry, content_dir)?;
        posts.push(post);
    }

    posts.sort_by(|a, b| b.metadata.date.cmp(&a.metadata.date));
    Ok(posts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_valid_post() {
        let dir = tempdir().unwrap();
        let posts_dir = dir.path().join("posts");
        fs::create_dir(&posts_dir).unwrap();

        let file_path = posts_dir.join("hello-world.md");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(
            file,
            "---
title: Hello World
date: 2026-01-01
tags: [rust, forge]
---
This is the body."
        )
        .unwrap();

        let post = parse_post(&file_path, dir.path()).unwrap();

        assert_eq!(post.metadata.title, "Hello World");
        assert_eq!(
            post.metadata.date,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
        assert_eq!(post.metadata.tags, vec!["rust", "forge"]);
        assert_eq!(post.slug, "posts/hello-world");
        assert_eq!(post.content.trim(), "This is the body.");
    }
}

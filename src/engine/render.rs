use crate::engine::utils;
use crate::models::{Post, ProjectPaths, RenderedPage, SiteConfig};
use anyhow::{Context, Result};
use std::path::Path;
use tera::Tera;

pub struct Renderer {
    pub tera: Tera,
}

impl Renderer {
    pub fn new(template_dir: &Path) -> Result<Self> {
        let tera = Tera::new(&format!("{}/**/*", template_dir.display()))
            .context("Failed to initialize Tera templates")?;
        Ok(Self { tera })
    }

    pub fn render_all_posts(
        &self,
        posts: &[Post],
        config: &SiteConfig,
    ) -> Result<Vec<RenderedPage>> {
        let mut rendered = Vec::new();

        for post in posts {
            let mut ctx = config.base_context();
            ctx.insert("post", &post.metadata);
            ctx.insert("content", &markdown_to_html(&post.content));

            let html = self.tera.render("post.html", &ctx).with_context(|| {
                format!("Failed to render post template for slug: {}", post.slug)
            })?;

            rendered.push(RenderedPage {
                slug: post.slug.clone(),
                html,
            });
        }
        Ok(rendered)
    }

    pub fn render_all_pages(
        &self,
        posts: &[Post],
        paths: &ProjectPaths,
        config: &SiteConfig,
    ) -> Result<Vec<RenderedPage>> {
        let mut rendered = Vec::new();
        let pages_dir = paths.templates.join("pages");

        if !pages_dir.exists() {
            return Ok(rendered);
        }

        // Fixed: walk_dir returns a Result, so we need to handle it
        for entry in utils::walk_dir(&pages_dir, "html")? {
            // Slug is the output path (e.g., "about")
            let slug = utils::Slugify::from_path(&entry, &pages_dir)?;

            let mut ctx = config.base_context();
            ctx.insert("posts", posts);

            // template_name is the internal Tera key (e.g., "pages/about.html")
            let template_path = entry
                .strip_prefix(&paths.templates)
                .with_context(|| format!("Path {:?} is outside of template directory", entry))?;

            let template_name = template_path.to_string_lossy();

            let html = self
                .tera
                .render(&template_name, &ctx)
                .with_context(|| format!("Failed to render page template: {:?}", entry))?;

            rendered.push(RenderedPage { slug, html })
        }
        Ok(rendered)
    }
}

fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

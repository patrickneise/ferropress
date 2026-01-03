# 🔥 FerroPress

**FerroPress** is a lightweight, high-performance static site generator (SSG) forged in Rust. It combines the speed of a compiled engine with a modern development experience, featuring integrated Tailwind CSS v4 support and instant hot-reloading.

## ✨ Features

- **Lightning Fast**: Written in Rust for near-instant site "casting."
- **The Hearth (Dev Server)**: A built-in development server with tower-livereload for real-time browser syncing.
- **Tailwind Integrated**: Automatic management of the Tailwind CSS v4 sidecar—no Node.js required.
- **Atomic Blueprints**: Simple TOML-based configuration and Markdown-driven content.
- **Git-Ready**: Automated repository initialization and .gitignore generation.

## 🚀 Getting Started

### 1. Installation

Ensure you have the Rust toolchain intalled. Clone this repository and build the binary:

```bash
cargo install --path .
```

### 2. Initialize a New Project

Create a new site structure with a single command:

```bash
ferropress init my-blog
cd my-blog
```

*This creates your blueprints (content, templates, static assets) and initlizes a Git repo.*

### 3. Start the Hearth (Development)

Launch the development server to see your changes in real-time:

```bash
ferropress preview
```

Open [http://localhost:3000](http://localhost:3000) in your browser. Any change to your Markdown or Templates will trigger an automatic "recast" and browser refresh.

### 4. Final Casting (Production)

When you are ready to publish, generate a minified, production-ready build:

```bash
ferropress build
```

Your final site will be waiting in the `dist/` directory.

## Project Structure

```
my-forge/
├── content/          # Your Markdown files (.md)
├── templates/        # Tera HTML templates (.html)
├── static/           # CSS, JS, and Image assets
│   └── css/
│       └── input.css # Tailwind entry point
├── dist/             # The generated site (Production output)
└── site.toml         # Your forge configuration
```

## 🔧 Configuration

The `site.toml` file controls your site's metadata. You can hot-reload these values during development without restarting the server.

```toml
title = "My New Forge"
author = "Ironmaster"
footer_text = "Forged with Ferropress"

[[navbar_links]]
label = "Home"
url = "/"

[[navbar_links]]
label = "Posts"
url = "/posts"
```

## 📜 Commands

| Command | Description |
| ------- | ----------- |
| init [name] | Creates a new project folder and structure. |
| preview | Starts the dev server with file watching and Tailwind --watch. |
| build | Cleans the dist folder and performs a full production cast. | 
| serve | Serves the dist folder without watching (one-shot build). |

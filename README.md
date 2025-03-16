# Rust-Hexo (Static Blog Generator)

![Rust-Hexo](https://img.shields.io/badge/Rust--Hexo-v0.1.0-brightgreen)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![License](https://img.shields.io/badge/License-MIT-blue)

Rust-Hexo is a static blog generator inspired by Hexo, reimplemented in Rust to provide faster generation speed and more powerful extensibility.

## Features

- **High Performance**: Utilizes Rust's high-performance characteristics for faster generation compared to traditional static blog generators
- **Modern Themes**: Built-in beautiful default theme with responsive design
- **Rich Plugin Support**: Includes various useful plugins such as mathematical formulas, code highlighting, word count, and more
- **Enhanced Markdown**: Supports code block syntax highlighting, mathematical formula rendering, tables, and other rich Markdown syntax
- **Live Preview**: Supports file watching and live preview for an improved writing experience
- **Search Functionality**: Built-in full-text search for easy content discovery
- **Multi-platform Support**: Cross-platform compatibility, works on Windows, macOS, and Linux

## Documentation

- [User Guide](docs/user-guide_en.md)
- [Theme Development](docs/theme-development.md)
- [Plugin Development Guide](docs/plugin-development_en.md)
- [API Reference](docs/api-reference.md)

## Installation

### Compile from Source

```bash
# Clone the repository
git clone https://github.com/your-username/rust-hexo.git
cd rust-hexo

# Compile the project
cargo build --release

# Install to your system
cargo install --path .
```

### Or Install Directly with Cargo

```bash
cargo install rust-hexo
```

## Quick Start

### Create a New Blog

```bash
# Initialize a new blog with default theme and configuration
rust-hexo init my-blog

# Use a custom title
rust-hexo init my-blog --title "My Personal Blog"

# Enter the blog directory
cd my-blog
```

### Create a New Article

```bash
# Create a new post
rust-hexo new "My First Article"

# Create a new page
rust-hexo new --page "About Me"
```

### Generate Static Files

```bash
# Generate static files
rust-hexo generate

# Generate and watch for file changes
rust-hexo generate --watch
```

### Local Preview

```bash
# Start local server
rust-hexo server

# Specify port and watch for file changes
rust-hexo server --port 8080 --watch
```

## Command Details

### `init` - Initialize a New Blog

```bash
rust-hexo init <NAME> [--title <TITLE>]
```

- `NAME`: Blog directory name
- `--title`: Blog title, defaults to directory name

The initialization command automatically creates a complete blog structure, including:
- Pre-configured `_config.yml` file with common plugin and feature settings
- Built-in default theme with responsive design and multiple features
- Basic directory structure and example articles
- Configuration examples for common plugins

### `new` - Create a New Article or Page

```bash
rust-hexo new <TITLE> [--page] [--path <PATH>]
```

- `TITLE`: Article or page title
- `--page`: Create a page instead of an article
- `--path`: Specify the article or page path

### `generate` - Generate Static Files

```bash
rust-hexo generate [--watch] [--deploy]
```

- `--watch`: Watch for file changes and automatically regenerate
- `--deploy`: Automatically deploy after generation

### `server` - Start Local Server

```bash
rust-hexo server [--port <PORT>] [--watch]
```

- `--port`: Server port, defaults to 4000
- `--watch`: Watch for file changes and automatically regenerate

### `clean` - Clean Generated Files

```bash
rust-hexo clean
```

### `deploy` - Deploy Website

```bash
rust-hexo deploy
```

### `plugin` - Plugin Management

```bash
rust-hexo plugin <COMMAND>
```

- `EnableHotReload`: Enable plugin hot reloading
- `DisableHotReload`: Disable plugin hot reloading
- `List`: List loaded plugins

## Directory Structure

```
.
├── _config.yml           # Site configuration file
├── scaffolds/            # Template directory
├── source/               # Resource folder
|   ├── _posts/           # Posts directory
|   └── _pages/           # Pages directory
├── themes/               # Themes directory
|   └── default/          # Default theme
|       ├── layout/       # Layout templates
|       └── source/       # Theme resources
|           ├── css/      # Style files
|           ├── js/       # Script files
|           └── images/   # Image resources
└── plugins/              # Plugins directory
```

## Configuration Details

### Site Configuration

The default `_config.yml` file already includes common configuration items:

```yaml
# Site Information
title: My Blog
subtitle: 'Testing Rust-Hexo Features'
description: 'This is a blog site for testing Rust-Hexo'
author: 'Rust-Hexo User'
language: en
timezone: America/New_York

# URL Configuration
url: http://example.com
root: /
permalink: :year/:month/:day/:title/

# Directory Configuration
source_dir: source
public_dir: public
tag_dir: tags
category_dir: categories
archive_dir: archives

# Writing Configuration
new_post_name: :title.md
default_layout: post
titlecase: false
external_link: true
post_asset_folder: true

# Theme
theme: default

# Plugin Configuration
plugins:
  - word-count
  - syntax-highlight
  - math
  - search
  - comments

# Search Functionality
search:
  enable: true
  path: search.json
  field: post
  content: true
```

You can customize these configuration items as needed.

### Front Matter

Each article or page's Markdown file can include YAML format front matter at the beginning:

```markdown
---
title: My First Article
date: 2023-06-18 12:34:56
tags: [rust, blog]
categories: [programming]
---

This is the article summary

<!-- more -->

This is the article main content...
```

## Built-in Plugin Features

Rust-Hexo includes various built-in plugins that can be enabled or configured in the `_config.yml` file:

### Mathematical Formula Rendering (math)

Supports both KaTeX and MathJax rendering engines for writing mathematical formulas using LaTeX syntax in articles:

```markdown
Inline formula: $E=mc^2$

Block formula:
$$
\sum_{i=1}^n i = \frac{n(n+1)}{2}
$$
```

Configuration example:
```yaml
math:
  engine: katex  # or mathjax
  inline: true
  block: true
```

### Code Highlighting (syntax-highlight)

Automatically adds syntax highlighting to code blocks, supporting multiple programming languages:

```markdown
```rust
fn main() {
    println!("Hello, Rust-Hexo!");
}
```
```

Configuration example:
```yaml
syntax_highlight:
  enable: true
  line_number: true
  copy_button: true
  theme: github-light
```

### Word Count (word-count)

Automatically counts article words and estimated reading time:

Configuration example:
```yaml
word_count:
  enable: true
  wordcount: true
  min2read: true
  avg_time: 300
```

### Search Functionality (search)

Adds full-text search functionality to the site:

Configuration example:
```yaml
search:
  enable: true
  path: search.json
  field: post
  content: true
```

### Comment System (comments)

Supports both Giscus and Disqus comment systems:

Configuration example:
```yaml
comments:
  enable: true
  system: giscus
  giscus:
    repo: username/repo-name
    repo_id: YOUR_REPO_ID
    category: Announcements
    category_id: YOUR_CATEGORY_ID
```

## Theme System

Rust-Hexo comes with a beautiful and practical default theme, while also supporting custom themes. The theme directory structure is as follows:

```
themes/mytheme/
├── layout/         # Layout templates
|   ├── layout.html # Base layout
|   ├── index.html  # Home page layout
|   ├── post.html   # Article page layout
|   ├── tag.html    # Tag page layout
|   └── category.html # Category page layout
└── source/         # Theme resource files
    ├── css/        # Style files
    ├── js/         # Script files
    └── images/     # Image resources
```

Using a theme:

1. Download or create a theme in the `themes` directory
2. Set `theme: mytheme` in the site's `_config.yml` file

## Plugin Development

Rust-Hexo supports extending functionality through plugins. Check the [Plugin Development Guide](docs/plugin-development_en.md) to learn how to develop your own plugins to add more features to your blog.

## Contributing

Contributions are welcome - code contributions, bug reports, or suggestions! Please participate in project development through GitHub Issues or Pull Requests.

## License

This project is licensed under the MIT License.

## Acknowledgements

- [Hexo](https://hexo.io/) - Provided inspiration and design ideas
- [Rust Community](https://www.rust-lang.org/) - Provided an excellent programming language and tools

---

🚀 Developed and maintained by the Rust-Hexo Team 
# Rust-Hexo User Guide

## Table of Contents

- [Introduction](#introduction)
- [Installation & Setup](#installation--setup)
- [Creating a New Blog](#creating-a-new-blog)
- [Adding Content](#adding-content)
- [Using Themes](#using-themes)
- [Configuration](#configuration)
- [Deployment](#deployment)
- [Using Plugins](#using-plugins)
- [Frequently Asked Questions](#frequently-asked-questions)

## Introduction

Rust-Hexo is a static blog generator written in Rust, inspired by Hexo. This guide will help you understand how to use Rust-Hexo to create and manage your blog.

## Installation & Setup

### Prerequisites

- Rust (version 1.70 or newer)
- Git (optional, for deployment)

### Installing Rust-Hexo

You can install Rust-Hexo from source:

```bash
# Clone the repository
git clone https://github.com/your-username/rust-hexo.git
cd rust-hexo

# Build the project
cargo build --release

# Optional: Add to your PATH for system-wide access
# For example, on Linux/macOS:
# sudo cp target/release/rust-hexo /usr/local/bin/
```

## Creating a New Blog

To create a new blog site:

```bash
rust-hexo init my-blog
```

This will create a new directory with the following structure:

```
my-blog/
├── _config.yml          # Site configuration file
├── source/              # Source files
│   ├── _posts/          # Blog posts
│   └── _pages/          # Static pages
├── themes/              # Themes
│   └── default/         # Default theme
└── plugins/             # Plugins directory
```

## Adding Content

### Creating a New Post

```bash
cd my-blog
rust-hexo new "My First Post"
```

This creates a new markdown file in `source/_posts/` with a front-matter template:

```markdown
---
title: My First Post
date: 2023-01-01 12:00:00
categories:
  - General
tags:
  - Getting Started
---

Write your post content here.
```

### Creating a New Page

```bash
rust-hexo new --page "About"
```

This creates a new page in `source/_pages/`.

### Front-matter

Front-matter is a YAML block at the beginning of a markdown file that defines metadata about the post or page:

```yaml
---
title: Title of Your Post
date: 2023-01-01 12:00:00
updated: 2023-01-02 12:00:00
categories:
  - Category1
  - Category2
tags:
  - Tag1
  - Tag2
permalink: custom-permalink
layout: custom-layout
draft: false
---
```

Available front-matter options:

| Option | Description |
|--------|-------------|
| title | The title of the post or page |
| date | Published date |
| updated | Last updated date |
| categories | Post categories |
| tags | Post tags |
| permalink | Custom permalink (overrides the default format) |
| layout | Custom layout template |
| draft | If true, the post will not be rendered in production |

## Using Themes

Rust-Hexo comes with a default theme. To use a different theme:

1. Place the theme in the `themes` directory
2. Update `_config.yml` to use the new theme:

```yaml
theme: theme-name
```

## Configuration

The main configuration file is `_config.yml` in the root directory of your blog:

```yaml
# Site Information
title: Your Blog Title
subtitle: 'Your Blog Subtitle'
description: 'Site description'
author: 'Your Name'
language: en-US
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
filename_case: 0
render_drafts: false
post_asset_folder: true

# Category & Tag
default_category: uncategorized

# Date / Time format
date_format: YYYY-MM-DD
time_format: HH:mm:ss

# Pagination
per_page: 10
pagination_dir: page

# Theme
theme: default

# Plugins
plugins:
  - word-count
  - syntax-highlight
  - math
  - search
  - comments

# Plugin-specific configurations
# Example: Math plugin configuration
math:
  engine: katex
  inline: true
  block: true
  katex:
    macros: true
    auto_render: true
    mhchem: true

# Comments configuration
comments:
  enable: true
  system: giscus
  giscus:
    repo: owner/repo
    repo_id: repo_id
    category: Announcements
    category_id: category_id
```

## Deployment

### Generating Static Files

```bash
rust-hexo generate
```

This generates a static site in the `public` directory.

### Starting a Local Server

To preview your site locally:

```bash
rust-hexo server
```

This starts a local server at http://localhost:4000 by default.

### Deploying to a Server

After generating the static files, you can deploy them to any web server:

```bash
# Example: Copying to a server via rsync
rsync -avz --delete public/ user@server:/path/to/site/
```

## Using Plugins

Rust-Hexo supports plugins to extend functionality. Plugins are enabled in the `_config.yml` file:

```yaml
plugins:
  - word-count
  - syntax-highlight
  - math
  - search
  - comments
```

### Word Count Plugin

Counts words in posts and estimates reading time:

```yaml
word_count:
  show_word_count: true
  show_read_time: true
  words_per_minute: 200
```

### Math Plugin

Renders mathematical expressions using KaTeX or MathJax:

```yaml
math:
  engine: katex  # or mathjax
  inline: true
  block: true
  katex:
    macros: true
    auto_render: true
    mhchem: true
```

Example usage in markdown:

```markdown
Inline math: $E = mc^2$

Block math:
$$
\frac{d}{dx}e^x = e^x
$$
```

### Comments Plugin

Integrates with comment systems:

```yaml
comments:
  enable: true
  system: giscus  # or disqus
  giscus:
    repo: owner/repo
    repo_id: repo_id
    category: Announcements
    category_id: category_id
```

## Frequently Asked Questions

*To be populated based on common user questions* 
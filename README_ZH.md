# Rust-Hexo (静态博客生成器)

![Rust-Hexo](https://img.shields.io/badge/Rust--Hexo-v0.1.0-brightgreen)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)
![License](https://img.shields.io/badge/License-MIT-blue)

Rust-Hexo 是一个受 Hexo 启发的静态博客生成器，使用 Rust 语言重新实现，提供了更快的生成速度和更强大的扩展性。

## 特点

- **高性能**: 利用 Rust 的高性能特性，生成速度比传统静态博客生成器更快
- **现代化主题**: 内置美观的默认主题，支持响应式设计
- **丰富的插件支持**: 内置多种实用插件，包括数学公式、代码高亮、字数统计等
- **Markdown 增强**: 支持代码块语法高亮、数学公式渲染、表格等丰富的Markdown语法
- **实时预览**: 支持文件监视和实时预览，提升写作体验
- **搜索功能**: 内置全文搜索功能，方便内容查找
- **多平台支持**: 跨平台支持，可在 Windows、macOS 和 Linux 上运行

## 文档

- [用户指南](docs/user-guide.md)
- [主题开发](docs/theme-development.md)
- [插件开发指南](docs/plugin-development.md)
- [API 参考](docs/api-reference.md)

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/your-username/rust-hexo.git
cd rust-hexo

# 编译项目
cargo build --release

# 安装到系统
cargo install --path .
```

### 或使用 Cargo 直接安装

```bash
cargo install rust-hexo
```

## 快速开始

### 创建新博客

```bash
# 初始化新博客，自带默认主题和配置
rust-hexo init my-blog

# 使用自定义标题
rust-hexo init my-blog --title "我的个人博客"

# 进入博客目录
cd my-blog
```

### 创建新文章

```bash
# 创建新文章
rust-hexo new "我的第一篇文章"

# 创建新页面
rust-hexo new --page "关于我"
```

### 生成静态文件

```bash
# 生成静态文件
rust-hexo generate

# 生成并监视文件变化
rust-hexo generate --watch
```

### 本地预览

```bash
# 启动本地服务器
rust-hexo server

# 指定端口并监视文件变化
rust-hexo server --port 8080 --watch
```

## 命令详解

### `init` - 初始化新博客

```bash
rust-hexo init <NAME> [--title <TITLE>]
```

- `NAME`: 博客目录名称
- `--title`: 博客标题，默认使用目录名称

初始化命令会自动创建一个完整的博客结构，包括：
- 预配置的 `_config.yml` 文件，包含常用插件和功能设置
- 内置的默认主题，支持响应式设计和多种功能
- 基本的目录结构和示例文章
- 常用插件的配置示例

### `new` - 创建新文章或页面

```bash
rust-hexo new <TITLE> [--page] [--path <PATH>]
```

- `TITLE`: 文章或页面标题
- `--page`: 创建页面而不是文章
- `--path`: 指定文章或页面路径

### `generate` - 生成静态文件

```bash
rust-hexo generate [--watch] [--deploy]
```

- `--watch`: 监视文件变化并自动重新生成
- `--deploy`: 生成后自动部署

### `server` - 启动本地服务器

```bash
rust-hexo server [--port <PORT>] [--watch]
```

- `--port`: 服务器端口，默认为 4000
- `--watch`: 监视文件变化并自动重新生成

### `clean` - 清理生成的文件

```bash
rust-hexo clean
```

### `deploy` - 部署网站

```bash
rust-hexo deploy
```

### `plugin` - 插件管理

```bash
rust-hexo plugin <COMMAND>
```

- `EnableHotReload`: 启用插件热重载
- `DisableHotReload`: 禁用插件热重载
- `List`: 列出已加载的插件

## 目录结构

```
.
├── _config.yml           # 站点配置文件
├── scaffolds/            # 模板目录
├── source/               # 资源文件夹
|   ├── _posts/           # 文章目录
|   └── _pages/           # 页面目录
├── themes/               # 主题目录
|   └── default/          # 默认主题
|       ├── layout/       # 布局模板
|       └── source/       # 主题资源
|           ├── css/      # 样式文件
|           ├── js/       # 脚本文件
|           └── images/   # 图片资源
└── plugins/              # 插件目录
```

## 配置说明

### 站点配置

默认的 `_config.yml` 文件已经包含了常用的配置项：

```yaml
# 站点信息
title: 我的博客
subtitle: '测试Rust-Hexo功能'
description: '这是一个用于测试Rust-Hexo的博客站点'
author: 'Rust-Hexo用户'
language: zh-CN
timezone: Asia/Shanghai

# URL配置
url: http://example.com
root: /
permalink: :year/:month/:day/:title/

# 目录配置
source_dir: source
public_dir: public
tag_dir: tags
category_dir: categories
archive_dir: archives

# 写作配置
new_post_name: :title.md
default_layout: post
titlecase: false
external_link: true
post_asset_folder: true

# 主题
theme: default

# 插件配置
plugins:
  - word-count
  - syntax-highlight
  - math
  - search
  - comments

# 搜索功能
search:
  enable: true
  path: search.json
  field: post
  content: true
```

你可以根据需要自定义这些配置项。

### 文章前置数据

每篇文章或页面的 Markdown 文件开头可以包含 YAML 格式的前置数据：

```markdown
---
title: 我的第一篇文章
date: 2023-06-18 12:34:56
tags: [rust, blog]
categories: [programming]
---

这是文章摘要

<!-- more -->

这是文章正文内容...
```

## 内置插件功能

Rust-Hexo 包含多种内置插件，可以在 `_config.yml` 文件中启用或配置：

### 数学公式渲染 (math)

支持 KaTeX 和 MathJax 两种渲染引擎，可以在文章中使用 LaTeX 语法编写数学公式：

```markdown
行内公式: $E=mc^2$

块级公式:
$$
\sum_{i=1}^n i = \frac{n(n+1)}{2}
$$
```

配置示例：
```yaml
math:
  engine: katex  # 或 mathjax
  inline: true
  block: true
```

### 代码高亮 (syntax-highlight)

自动为代码块添加语法高亮，支持多种编程语言：

```markdown
```rust
fn main() {
    println!("Hello, Rust-Hexo!");
}
```
```

配置示例：
```yaml
syntax_highlight:
  enable: true
  line_number: true
  copy_button: true
  theme: github-light
```

### 字数统计 (word-count)

自动统计文章字数和预计阅读时间：

配置示例：
```yaml
word_count:
  enable: true
  wordcount: true
  min2read: true
  avg_time: 300
```

### 搜索功能 (search)

为站点添加全文搜索功能：

配置示例：
```yaml
search:
  enable: true
  path: search.json
  field: post
  content: true
```

### 评论系统 (comments)

支持 Giscus 和 Disqus 两种评论系统：

配置示例：
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

## 主题系统

Rust-Hexo 自带一个美观实用的默认主题，同时也支持自定义主题。主题目录结构如下：

```
themes/mytheme/
├── layout/         # 布局模板
|   ├── layout.html # 基础布局
|   ├── index.html  # 首页布局
|   ├── post.html   # 文章页布局
|   ├── tag.html    # 标签页布局
|   └── category.html # 分类页布局
└── source/         # 主题资源文件
    ├── css/        # 样式文件
    ├── js/         # 脚本文件
    └── images/     # 图片资源
```

使用主题：

1. 下载或创建主题到 `themes` 目录
2. 在站点 `_config.yml` 文件中设置 `theme: mytheme`

## 插件开发

Rust-Hexo 支持通过插件扩展功能。查看[插件开发指南](docs/plugin-development.md)了解如何开发自己的插件，为博客添加更多功能。

## 贡献

欢迎贡献代码、报告问题或提供建议！请通过 GitHub Issues 或 Pull Requests 参与项目开发。

## 许可证

本项目采用 MIT 许可证。

## 致谢

- [Hexo](https://hexo.io/) - 提供了灵感和设计思路
- [Rust 社区](https://www.rust-lang.org/) - 提供了卓越的编程语言和工具

---

🚀 由 Rust-Hexo 团队开发和维护
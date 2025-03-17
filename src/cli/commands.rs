use crate::core::Engine;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::Write;
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 指定站点目录
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 初始化新的博客站点
    Init(InitArgs),
    
    /// 创建新的文章
    New(NewArgs),
    
    /// 生成静态文件
    Generate(GenArgs),
    
    /// 启动本地服务器
    Server(ServerArgs),
    
    /// 清理生成的文件
    Clean,
    
    /// 部署站点
    Deploy,
    
    /// 插件管理
    Plugin(PluginArgs),
}

#[derive(Args)]
pub struct InitArgs {
    /// 站点目录名称
    #[arg(value_name = "NAME")]
    pub name: String,
    
    /// 站点标题
    #[arg(short, long)]
    pub title: Option<String>,
}

#[derive(Args)]
pub struct NewArgs {
    /// 文章标题
    pub title: String,
    
    /// 文章/页面路径
    #[arg(short = 'f', long)]
    pub path: Option<String>,
    
    /// 创建页面而不是文章
    #[arg(short, long)]
    pub page: bool,
}

#[derive(Args)]
pub struct GenArgs {
    /// 部署到远程服务器
    #[arg(short, long)]
    pub deploy: bool,
    
    /// 监视文件变化并自动重新生成
    #[arg(short, long)]
    pub watch: bool,
}

#[derive(Args)]
pub struct ServerArgs {
    /// 服务器端口
    #[arg(short, long, default_value = "4000")]
    pub port: u16,
    
    /// 监视文件变化并自动重新生成
    #[arg(short, long)]
    pub watch: bool,
}

#[derive(Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    command: PluginCommands,
}

#[derive(Subcommand)]
pub enum PluginCommands {
    /// 启用插件热重载
    EnableHotReload,
    /// 禁用插件热重载
    DisableHotReload,
    /// 列出已加载的插件
    List,
}

// 嵌入的默认配置模板
const DEFAULT_CONFIG_TEMPLATE: &str = r#"# 站点信息
title: {title}
subtitle: '测试Rust-Hexo功能'
description: '这是一个用于测试Rust-Hexo的博客站点'
author: 'Rust-Hexo用户'
language: zh-CN
timezone: Asia/Shanghai

# URL配置
url: http://example.com
root: /
permalink: :year/:month/:day/:title/
permalink_defaults:

# 目录配置
source_dir: source
public_dir: public
tag_dir: tags
category_dir: categories
archive_dir: archives
code_dir: downloads/code
i18n_dir: :lang
skip_render:

# 写作配置
new_post_name: :title.md
default_layout: post
titlecase: false
external_link: true
filename_case: 0
render_drafts: false
post_asset_folder: true
relative_link: false
future: true
highlight:
  enable: true
  line_number: true
  auto_detect: false
  tab_replace:

# 分类 & 标签
default_category: uncategorized
category_map:
tag_map:

# 日期 / 时间格式
date_format: YYYY-MM-DD
time_format: HH:mm:ss

# 分页配置
per_page: 10
pagination_dir: page

# 主题配置
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
  format: html
"#;

// 嵌入的默认主题文件
mod default_theme {
    // 主题CSS文件
    pub const STYLE_CSS: &str = include_str!("../../embed/theme/default/source/css/style.css");
    
    // 主题布局文件
    pub const LAYOUT_HTML: &str = include_str!("../../embed/theme/default/layout/layout.html");
    pub const INDEX_HTML: &str = include_str!("../../embed/theme/default/layout/index.html");
    pub const POST_HTML: &str = include_str!("../../embed/theme/default/layout/post.html");
    pub const CATEGORY_HTML: &str = include_str!("../../embed/theme/default/layout/category.html");
    pub const TAG_HTML: &str = include_str!("../../embed/theme/default/layout/tag.html");
}

// 初始化网站文件结构，包括创建默认主题和示例文件
fn initialize_site_structure(site_path: &PathBuf, site_title: &str) -> Result<()> {
    // 创建目录结构
    let source_dir = site_path.join("source");
    let posts_dir = source_dir.join("_posts");
    let theme_dir = site_path.join("themes").join("default");
    let theme_layout_dir = theme_dir.join("layout");
    let theme_source_dir = theme_dir.join("source");
    let theme_css_dir = theme_source_dir.join("css");
    let theme_js_dir = theme_source_dir.join("js");
    let theme_images_dir = theme_source_dir.join("images");
    let scaffolds_dir = site_path.join("scaffolds");
    let plugins_dir = site_path.join("plugins");

    // 创建所有必要的目录
    for dir in &[
        &source_dir, &posts_dir, &theme_dir, &theme_layout_dir, &theme_source_dir,
        &theme_css_dir, &theme_js_dir, &theme_images_dir, &scaffolds_dir, &plugins_dir
    ] {
        fs::create_dir_all(dir)?;
    }

    // 创建默认配置文件
    let config_content = DEFAULT_CONFIG_TEMPLATE.replace("{title}", site_title);
    fs::write(site_path.join("_config.yml"), config_content)?;

    // 创建默认主题文件
    fs::write(theme_css_dir.join("style.css"), default_theme::STYLE_CSS)?;
    fs::write(theme_layout_dir.join("layout.html"), default_theme::LAYOUT_HTML)?;
    fs::write(theme_layout_dir.join("index.html"), default_theme::INDEX_HTML)?;
    fs::write(theme_layout_dir.join("post.html"), default_theme::POST_HTML)?;
    fs::write(theme_layout_dir.join("category.html"), default_theme::CATEGORY_HTML)?;
    fs::write(theme_layout_dir.join("tag.html"), default_theme::TAG_HTML)?;

    // 创建示例博文
    let hello_post = posts_dir.join("hello-world.md");
    let hello_content = r#"---
title: Hello World
date: 2023-01-01 12:00:00
categories:
  - 入门指南
tags:
  - Rust-Hexo
  - 指南
---

# 欢迎使用Rust-Hexo

这是您使用Rust-Hexo创建的第一篇博客文章。您可以编辑此文件来开始您的博客之旅！

## 快速开始

### 创建新文章

``` bash
rust-hexo new "我的新文章"
```

### 生成静态文件

``` bash
rust-hexo generate
```

### 启动本地服务器

``` bash
rust-hexo server
```

更多信息请访问[文档](https://github.com/your-username/rust-hexo)。
"#;
    fs::write(hello_post, hello_content)?;

    // 创建脚手架模板
    let post_scaffold = scaffolds_dir.join("post.md");
    let post_scaffold_content = r#"---
title: {{ title }}
date: {{ date }}
categories:
tags:
---
"#;
    fs::write(post_scaffold, post_scaffold_content)?;

    let page_scaffold = scaffolds_dir.join("page.md");
    let page_scaffold_content = r#"---
title: {{ title }}
date: {{ date }}
---
"#;
    fs::write(page_scaffold, page_scaffold_content)?;

    Ok(())
}

/// 执行命令
pub async fn execute(cli: Cli) -> Result<()> {
    let site_path = cli.path.clone();
    
    let mut engine = Engine::new(site_path.clone())?;
    
    match cli.command {
        Commands::Init(args) => {
            // 使用提供的目录名称
            let site_path = site_path.join(&args.name);
            
            // 如果目录不为空，询问用户是否继续
            if site_path.exists() && site_path.read_dir()?.next().is_some() {
                println!("Directory is not empty. Do you want to continue? (y/N)");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Operation cancelled.");
                    return Ok(());
                }
            }
            
            // 创建站点目录
            fs::create_dir_all(&site_path)?;
            
            // 获取站点标题
            let site_title = args.title.unwrap_or_else(|| args.name.clone());
            
            // 初始化网站文件结构
            initialize_site_structure(&site_path, &site_title)?;
            
            // 创建引擎实例，但不调用init()方法
            // 因为initialize_site_structure已经创建了必要的文件结构
            // let engine = Engine::new(site_path.clone())?;
            
            info!("Initialized new site at: {}", site_path.display());
        }
        Commands::New(args) => {
            if args.page {
                let path = args.path.unwrap_or_else(|| "page".to_string());
                engine.new_page(&args.title, &path).await?;
            } else {
                engine.new_post(&args.title, args.path.as_deref()).await?;
            }
        }
        Commands::Generate(args) => {
            // 确保引擎已初始化
            engine.init()?;
            
            let public_dir = engine.public_dir.clone();
            engine.generate(&public_dir)?;
            
            if args.watch {
                // 启动监视模式
                println!("Watching for changes. Press Ctrl+C to stop.");
                engine.watch().await?;
                
                // 等待用户中断
                tokio::signal::ctrl_c().await?;
                engine.unwatch();
            }
            
            if args.deploy {
                engine.deploy().await?;
            }
        }
        Commands::Server(args) => {
            // 生成静态文件
            let public_dir = engine.public_dir.clone();
            engine.generate(&public_dir)?;
            
            if args.watch {
                // 在后台启动监视任务
                let engine_clone = engine.clone();
                tokio::spawn(async move {
                    if let Err(e) = engine_clone.watch().await {
                        eprintln!("Error watching files: {}", e);
                    }
                });
            }
            
            // 启动服务器
            engine.server(args.port).await?;
            
            // 等待用户中断
            tokio::signal::ctrl_c().await?;
            if args.watch {
                engine.unwatch();
            }
        }
        Commands::Clean => {
            engine.clean().await?;
        }
        Commands::Deploy => {
            // 先生成静态文件，再部署
            let public_dir = engine.public_dir.clone();
            engine.generate(&public_dir)?;
            engine.deploy().await?;
        }
        Commands::Plugin(args) => {
            match args.command {
                PluginCommands::EnableHotReload => {
                    if let Err(e) = engine.start_plugin_hot_reload() {
                        println!("启用插件热重载失败: {}", e);
                    } else {
                        println!("插件热重载已启用");
                        
                        // 重新生成静态文件
                        let public_dir = engine.public_dir.clone();
                        engine.generate(&public_dir)?;
                    }
                }
                PluginCommands::DisableHotReload => {
                    engine.disable_plugin_hot_reload();
                    println!("插件热重载已禁用。");
                }
                PluginCommands::List => {
                    // 列出已加载的插件
                    let plugins = engine.plugin_manager.get_all_plugins()?;
                    println!("已加载的插件列表:");
                    for plugin in plugins {
                        println!("  - {} v{}", plugin.name(), plugin.version());
                    }
                    
                    // 重新生成静态文件
                    let public_dir = engine.public_dir.clone();
                    engine.generate(&public_dir)?;
                }
            }
        }
    }
    
    Ok(())
} 
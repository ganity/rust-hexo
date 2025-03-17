use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::mpsc;
use std::any::Any;

use anyhow::{Context as AnyhowContext, Result};
use chrono::{Utc, TimeZone};
use colored::Colorize;
use gray_matter::Matter;
use gray_matter::engine::YAML;
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use rayon::prelude::*;
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;
use yaml_rust2::{YamlLoader, Yaml};
use serde_yaml::{Mapping, Value};
use gray_matter::Pod;
use pulldown_cmark::{html, Options, Parser};
use slug;

use crate::models::config::Config;
use crate::models::{Category, Page, Post, SiteConfig, Tag};
use crate::plugins::{PluginManager, PluginHook, PluginContext, ContentType};
use crate::theme::renderer::ThemeRenderer;
use crate::core::generator::HtmlGenerator;

/// Hexo引擎的核心实现
#[derive(Clone)]
pub struct Engine {
    /// 基础目录
    pub base_dir: PathBuf,
    /// 源文件目录
    pub source_dir: PathBuf,
    /// 公共目录（输出）
    pub public_dir: PathBuf,
    /// 主题目录
    pub theme_dir: PathBuf,
    /// 脚手架目录
    pub scaffold_dir: PathBuf,
    /// 站点配置
    pub config: Config,
    /// 主题配置
    pub theme_config: HashMap<String, Value>,
    /// 所有文章
    pub posts: Arc<RwLock<Vec<Post>>>,
    /// 所有页面
    pub pages: Arc<RwLock<Vec<Page>>>,
    /// 所有分类
    pub categories: Arc<RwLock<Vec<Category>>>,
    /// 所有标签
    pub tags: Arc<RwLock<Vec<Tag>>>,
    /// 是否处于监听状态
    is_watching: Arc<RwLock<bool>>,
    /// 插件管理器
    pub plugin_manager: PluginManager,
    /// 主题渲染器
    theme_renderer: Option<ThemeRenderer>,
    /// 文件监视器
    file_watcher: Arc<RwLock<Option<Box<dyn Any + Send + Sync>>>>,
}

// 手动实现Sync，因为所有的字段都是Sync的
unsafe impl Sync for Engine {}

impl Engine {
    /// 创建一个新的引擎实例
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        info!("初始化 Hexo 引擎...");
        info!("工作目录: {}", base_dir.display());
        
        // 创建必要的目录
        let source_dir = base_dir.join("source");
        let public_dir = base_dir.join("public");
        let theme_dir = base_dir.join("themes").join("default");
        let scaffold_dir = base_dir.join("scaffolds");

        // 检查必要目录是否存在，如果不存在且不是在initialize_site_structure之后，再创建
        // 这样可以避免重复创建目录
        // if !source_dir.exists() {
        //     info!("创建 source 目录");
        //     fs::create_dir_all(&source_dir)
        //         .with_context(|| format!("创建目录失败: {}", source_dir.display()))?;
        // }
        
        // 配置文件
        let config_path = base_dir.join("_config.yml");
        let config = if config_path.exists() {
            Config::load(&config_path)?
        } else {
            let config = Config::default();
            // config.save(&config_path)?;
            // info!("创建默认配置文件");
            config
        };
        
        // 克隆base_dir以便在后续使用
        let base_dir_clone = base_dir.clone();
        
        Ok(Self {
            base_dir: base_dir.clone(),
            source_dir,
            public_dir,
            theme_dir,
            scaffold_dir,
            config,
            theme_config: HashMap::new(),
            posts: Arc::new(RwLock::new(Vec::new())),
            pages: Arc::new(RwLock::new(Vec::new())),
            categories: Arc::new(RwLock::new(Vec::new())),
            tags: Arc::new(RwLock::new(Vec::new())),
            is_watching: Arc::new(RwLock::new(false)),
            plugin_manager: PluginManager::new(base_dir_clone, PluginContext::default()),
            theme_renderer: None,
            file_watcher: Arc::new(RwLock::new(None)),
        })
    }
    
    /// 创建插件上下文
    fn create_plugin_context(&self) -> PluginContext {
        info!("创建插件上下文...");
        let posts = self.posts.read().unwrap().clone();
        let pages = self.pages.read().unwrap().clone();
        let categories = self.categories.read().unwrap().clone();
        let tags = self.tags.read().unwrap().clone();
        
        // 创建 base_url，如果 url 或 root 为 None，则使用默认值
        let base_url = match (&self.config.url, &self.config.root) {
            (Some(url), Some(root)) => format!("{}{}", url, root),
            (Some(url), None) => url.clone(),
            (None, Some(root)) => root.clone(),
            (None, None) => String::from("/"),
        };
        
        PluginContext {
            base_dir: self.base_dir.clone(),
            plugins_dir: self.base_dir.join("plugins"),
            theme_dir: self.theme_dir.clone(),
            base_url,
            output_dir: self.public_dir.clone(),
            config: self.config.clone(),
            posts,
            pages,
            categories,
            tags,
            current_post: None,
            current_page: None,
        }
    }
    
    /// 加载主题配置
    fn load_theme_config(theme_dir: &Path, site_config: &Config) -> Result<HashMap<String, Value>> {
        let theme_config_path = theme_dir.join("_config.yml");
        let mut theme_config = HashMap::new();
        
        if theme_config_path.exists() {
            let config_str = std::fs::read_to_string(&theme_config_path)
                .with_context(|| format!("Failed to read theme config file: {}", theme_config_path.display()))?;
            
            theme_config = serde_yaml::from_str(&config_str)
                .with_context(|| "Failed to parse theme _config.yml")?;
        }
        
        // 合并站点配置中的主题配置
        if let Some(site_theme_config) = &site_config.theme_config {
            if let Value::Mapping(mapping) = site_theme_config {
                for (key, value) in mapping {
                    if let Some(key_str) = key.as_str() {
                        theme_config.insert(key_str.to_string(), value.clone());
                    }
                }
            }
        }
        
        Ok(theme_config)
    }
    
    /// 初始化引擎
    pub fn init(&mut self) -> Result<()> {
        info!("初始化 Hexo 引擎...");
        
        // 初始化插件管理器
        let plugin_context = self.create_plugin_context();
        self.plugin_manager.set_context(plugin_context);
        
        // 确保插件目录存在
        let plugins_dir = self.base_dir.join("plugins");
        if !plugins_dir.exists() {
            info!("创建插件目录: {}", plugins_dir.display());
            fs::create_dir_all(&plugins_dir)?;
        }
        
        self.plugin_manager.init()?;
        
        // 初始化主题渲染器
        let mut theme_renderer = ThemeRenderer::new(&self.base_dir, self.config.clone())?;
        
        // 将插件功能注册到主题渲染器
        if let Err(e) = self.plugin_manager.register_to_theme_renderer(&mut theme_renderer) {
            warn!("注册插件功能到主题渲染器失败: {}", e);
            // 继续执行，不中断初始化过程
        }
        
        // 保存主题渲染器
        self.theme_renderer = Some(theme_renderer);
        
        info!("{}", "Initialization complete.".green());
        Ok(())
    }
    
    /// 处理内容
    fn process_content(&self, content: &str, content_type: ContentType) -> String {
        // 使用插件处理内容
        match self.plugin_manager.process_content(content, content_type) {
            Ok(processed) => processed,
            Err(e) => {
                warn!("内容处理出错: {}", e);
                content.to_string()
            }
        }
    }
    
    /// 处理Markdown内容
    fn process_markdown(&self, content: &str) -> String {
        self.process_content(content, ContentType::Markdown)
    }
    
    /// 处理HTML内容
    fn process_html(&self, content: &str) -> String {
        self.process_content(content, ContentType::HTML)
    }
    
    /// 加载配置文件
    pub fn load_config(&mut self) -> Result<()> {
        let config_path = self.base_dir.join("_config.yml");
        let config_str = std::fs::read_to_string(&config_path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        self.config = config.clone();
        
        // 更新插件上下文中的配置
        let plugin_context = self.create_plugin_context();
        self.plugin_manager.set_context(plugin_context);
        
        Ok(())
    }
    
    /// 运行引擎
    pub async fn run(&mut self) -> Result<()> {
        debug!("运行引擎...");
        
        // 调用插件钩子：生成前
        if let Err(e) = self.plugin_manager.execute_hook(&PluginHook::BeforeGenerate) {
            warn!("执行生成前钩子失败: {}", e);
            // 继续执行，不中断整个过程
        }
        
        // 读取文章和页面
        self.load_posts_and_pages()?;
        self.process_categories_and_tags()?;
        
        // 调用插件钩子：路由生成前
        if let Err(e) = self.plugin_manager.execute_hook(&PluginHook::BeforeRouteGenerate) {
            warn!("执行路由生成前钩子失败: {}", e);
            // 继续执行，不中断整个过程
        }
        
        // 生成静态文件
        let public_dir = self.public_dir.clone();
        self.generate(&public_dir)?;
        
        // 调用插件钩子：生成后
        if let Err(e) = self.plugin_manager.execute_hook(&PluginHook::AfterGenerate) {
            warn!("执行生成后钩子失败: {}", e);
            // 继续执行，不中断整个过程
        }
        
        Ok(())
    }
    
    /// 加载文章和页面
    fn load_posts_and_pages(&self) -> Result<()> {
        info!("加载文章和页面...");
        
        // 加载文章
        let posts_dir = self.source_dir.join("_posts");
        info!("从 {} 加载文章", posts_dir.display());
        
        let mut found_posts = Vec::new();
        let matter = Matter::<YAML>::new();
        
        if posts_dir.exists() {
            for entry in WalkDir::new(&posts_dir) {
                let entry = entry?;
                let path = entry.path();
                
                // 只处理 .md 文件
                if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                
                // 读取文件内容
                let content = std::fs::read_to_string(path)?;
                
                // 使用插件处理Markdown内容
                let processed_content = self.process_markdown(&content);
                
                // 解析 Front Matter
                let result = matter.parse(&processed_content);
                
                // 获取 YAML 数据
                let yaml_data = if let Some(data) = result.data {
                    data
                } else {
                    continue;
                };
                
                // 获取标题
                let title = if let Ok(title) = yaml_data["title"].as_string() {
                    title
                } else {
                    // 如果没有标题，使用文件名
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled")
                        .to_string()
                };
                
                // 解析日期
                let date = if let Ok(date_str) = yaml_data["date"].as_string() {
                    match chrono::DateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S %z") {
                        Ok(dt) => dt.with_timezone(&Utc),
                        Err(_) => {
                            // 尝试另一种格式
                            match chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S") {
                                Ok(dt) => Utc.from_utc_datetime(&dt),
                                Err(_) => Utc::now(), // 如果无法解析，使用当前时间
                            }
                        }
                    }
                } else {
                    // 如果没有日期，使用文件的修改时间
                    let metadata = std::fs::metadata(path)?;
                    let modified = metadata.modified()?;
                    let system_time: chrono::DateTime<Utc> = modified.into();
                    system_time
                };
                
                // 创建前置数据的HashMap
                let mut front_matter = HashMap::new();
                if let Ok(hash) = yaml_data.as_hashmap() {
                    for (k, v) in hash {
                        let value = match v {
                            Pod::String(s) => Value::String(s),
                            Pod::Integer(i) => Value::Number(serde_yaml::Number::from(i)),
                            Pod::Float(f) => Value::Number(serde_yaml::Number::from(f)),
                            Pod::Boolean(b) => Value::Bool(b),
                            Pod::Array(arr) => Value::Sequence(arr.into_iter().map(pod_to_value).collect()),
                            Pod::Hash(map) => {
                                let mut yaml_map = Mapping::new();
                                for (map_k, map_v) in map {
                                    yaml_map.insert(Value::String(map_k), pod_to_value(map_v));
                                }
                                Value::Mapping(yaml_map)
                            },
                            Pod::Null => Value::Null,
                        };
                        front_matter.insert(k, value);
                    }
                }
                
                // 创建文章的URL路径
                let filename = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let url_path = format!("posts/{}.html", filename);
                
                // 将Markdown转换为HTML
                let html_content = crate::utils::markdown::render(&result.content)?;
                
                // 使用插件处理HTML内容
                let final_content = self.process_html(&html_content);
                
                // 创建新的文章对象
                let post = Post {
                    title,
                    date,
                    updated: Some(date), // 默认使用相同的时间
                    comments: true,
                    layout: if let Ok(hash) = yaml_data.as_hashmap() { if let Some(layout_pod) = hash.get("layout") { if let Ok(layout) = layout_pod.as_string() { layout } else { "post".to_string() } } else { "post".to_string() } } else { "post".to_string() },
                    content: html_content,  // 使用已经渲染好的HTML内容
                    rendered_content: Some(final_content),  // 存储处理后的内容
                    source: path.to_path_buf(),
                    path: format!("posts/{}.html", filename),
                    permalink: url_path,
                    excerpt: None, // TODO: 实现摘要提取
                    url: None,
                    categories: Vec::new(), // 稍后处理
                    tags: Vec::new(),       // 稍后处理
                    front_matter,
                };
                
                found_posts.push(post);
            }
        }
        
        // 更新文章列表
        if !found_posts.is_empty() {
            // 按日期排序
            found_posts.sort_by(|a, b| b.date.cmp(&a.date));
            
            let mut posts = self.posts.write().unwrap();
            *posts = found_posts;
            
            info!("加载了 {} 篇文章", posts.len());
        }
        
        // TODO: 加载页面
        
        Ok(())
    }
    
    /// 处理分类和标签
    fn process_categories_and_tags(&self) -> Result<()> {
        info!("处理分类和标签...");
        
        let mut posts = self.posts.write().unwrap();
        let mut categories_map = HashMap::new();
        let mut tags_map = HashMap::new();
        
        // 打印文章总数
        info!("开始处理 {} 篇文章的分类和标签", posts.len());
        
        // 收集所有分类和标签，同时更新文章对象
        for post in posts.iter_mut() {
            let mut post_categories = Vec::new();
            let mut post_tags = Vec::new();
            
            // 打印文章信息
            info!("处理文章: {}, 检查前言数据", post.title);
            
            // 处理分类
            if let Some(Value::Sequence(cats)) = post.front_matter.get("categories") {
                info!("文章 {} 有序列化分类: {:?}", post.title, cats);
                for cat_value in cats {
                    if let Value::String(cat_name) = cat_value {
                        post_categories.push(cat_name.clone());
                        categories_map.entry(cat_name.clone())
                            .or_insert_with(Vec::new)
                            .push(post.clone());
                    }
                }
            } else if let Some(Value::String(cat)) = post.front_matter.get("categories") {
                info!("文章 {} 有字符串分类: {}", post.title, cat);
                post_categories.push(cat.clone());
                categories_map.entry(cat.clone())
                    .or_insert_with(Vec::new)
                    .push(post.clone());
            } else {
                info!("文章 {} 没有分类", post.title);
            }
            
            // 处理标签
            if let Some(Value::Sequence(tags)) = post.front_matter.get("tags") {
                info!("文章 {} 有序列化标签: {:?}", post.title, tags);
                for tag_value in tags {
                    if let Value::String(tag_name) = tag_value {
                        post_tags.push(tag_name.clone());
                        tags_map.entry(tag_name.clone())
                            .or_insert_with(Vec::new)
                            .push(post.clone());
                    }
                }
            } else if let Some(Value::String(tag)) = post.front_matter.get("tags") {
                info!("文章 {} 有字符串标签: {}", post.title, tag);
                post_tags.push(tag.clone());
                tags_map.entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(post.clone());
            } else {
                info!("文章 {} 没有标签", post.title);
            }
            
            // 更新文章对象的分类和标签
            info!("更新文章 {} 的分类为: {:?}", post.title, post_categories);
            info!("更新文章 {} 的标签为: {:?}", post.title, post_tags);
            post.categories = post_categories;
            post.tags = post_tags;
        }
        
        // 更新分类列表
        let categories: Vec<Category> = categories_map.into_iter()
            .map(|(name, posts)| {
                let slug = slug::slugify(&name);
                Category {
                    name: name.clone(),
                    slug,
                    path: format!("categories/{}", name.to_lowercase()),
                    parent: None,
                    post_count: posts.len(),
                }
            })
            .collect();
        
        // 更新标签列表
        let tags: Vec<Tag> = tags_map.into_iter()
            .map(|(name, posts)| {
                let slug = slug::slugify(&name);
                Tag {
                    name: name.clone(),
                    slug,
                    path: format!("tags/{}", name.to_lowercase()),
                    post_count: posts.len(),
                }
            })
            .collect();
        
        // 更新存储
        {
            let mut categories_store = self.categories.write().unwrap();
            *categories_store = categories;
            info!("处理了 {} 个分类", categories_store.len());
            
            // 打印所有分类
            for category in categories_store.iter() {
                info!("分类: {}, 路径: {}, 文章数: {}", category.name, category.path, category.post_count);
            }
        }
        
        {
            let mut tags_store = self.tags.write().unwrap();
            *tags_store = tags;
            info!("处理了 {} 个标签", tags_store.len());
            
            // 打印所有标签
            for tag in tags_store.iter() {
                info!("标签: {}, 路径: {}, 文章数: {}", tag.name, tag.path, tag.post_count);
            }
        }
        
        Ok(())
    }
    
    /// 创建站点配置
    fn create_site_config(&self) -> Result<SiteConfig> {
        use crate::models::types::SiteConfig;
        
        // 从配置和主题配置构建站点配置
        Ok(SiteConfig {
            title: self.config.title.clone(),
            subtitle: self.config.subtitle.clone(),
            description: self.config.description.clone(),
            author: self.config.author.clone(),
            language: self.config.language.clone().unwrap_or_else(|| "en".to_string()),
            timezone: self.config.timezone.clone(),
            url: self.config.url.clone().unwrap_or_else(|| "http://localhost".to_string()),
            root: self.config.root.clone().unwrap_or_else(|| "/".to_string()),
            per_page: self.config.per_page.unwrap_or(10) as usize,
            theme: self.config.theme.clone().unwrap_or_else(|| "default".to_string()),
            deploy: None,
            theme_config: Some(self.theme_config.clone()),
            comments: None,
            search: None,
            extras: HashMap::new(),
        })
    }
    
    /// 生成静态网站
    pub fn generate(&mut self, output_dir: &Path) -> Result<()> {
        info!("开始生成静态网站");
        let output_dir = output_dir.to_path_buf();
        
        // 检查输出目录是否存在，不存在则创建
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir)?;
        }
        
        // 确保已加载文章
        if self.posts.read().unwrap().is_empty() {
            self.load_posts_and_pages()?;
        } else {
            // 在文件监视模式下，文章可能已被修改，需要重新加载
            // 通过检查 is_watching 状态判断
            if *self.is_watching.read().unwrap() {
                debug!("检测到监视模式，重新加载文章和页面");
                self.load_posts_and_pages()?;
            }
        }
        
        // 处理分类和标签
        info!("处理分类和标签数据");
        self.process_categories_and_tags()?;
        
        // 初始化插件上下文
        self.plugin_manager.set_context(self.create_plugin_context());
        
        // 确保插件已初始化
        info!("检查插件管理器初始化状态: initialized={}, plugins_count={}",
              self.plugin_manager.is_initialized(),
              self.plugin_manager.get_all_plugins()?.len());
              
        if !self.plugin_manager.is_initialized() {
            info!("插件管理器尚未初始化，开始初始化...");
            self.plugin_manager.init()?;
        } else {
            info!("插件管理器已初始化，跳过初始化步骤");
        }
        
        info!("插件管理器状态: 已初始化 = {}", self.plugin_manager.is_initialized());
        let plugin_count = self.plugin_manager.get_all_plugins()?.len();
        info!("已加载 {} 个插件", plugin_count);
        
        // 获取所有文章
        let posts = self.posts.read().unwrap().clone();
        
        // 调用HTML生成器，使用已初始化的插件管理器
        info!("创建HTML生成器，使用已初始化的插件管理器");
        let generator = HtmlGenerator::new_with_plugin_manager(
            output_dir,
            self.config.clone(),
            self.plugin_manager.clone()
        );
        
        // 生成HTML文件
        generator.generate(&posts)?;
        
        info!("静态网站生成完成");
        Ok(())
    }
    
    /// 复制静态文件（如CSS和JS）到输出目录
    fn copy_static_files(&self) -> Result<()> {
        let static_dir = self.source_dir.join("static");
        let output_dir = self.public_dir.clone();
        
        if static_dir.exists() {
            for entry in walkdir::WalkDir::new(&static_dir) {
                let entry = entry?;
                let src_path = entry.path();
                
                if src_path.is_file() {
                    let rel_path = src_path.strip_prefix(&static_dir)?;
                    let dest_path = output_dir.join(rel_path);
                    
                    // 确保目标目录存在
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    // 复制文件
                    fs::copy(src_path, dest_path)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// 复制主题静态资源
    fn copy_theme_assets(&self) -> Result<()> {
        let theme_source = self.theme_dir.join("source");
        let theme_dest = self.public_dir.join("assets");
        
        // 如果主题源目录不存在，跳过
        if !theme_source.exists() {
            return Ok(());
        }
        
        // 创建目标目录
        fs::create_dir_all(&theme_dest)?;
        
        // 复制所有文件
        for entry in walkdir::WalkDir::new(&theme_source) {
            let entry = entry?;
            let src_path = entry.path();
            
            if src_path.is_file() {
                let rel_path = src_path.strip_prefix(&theme_source)?;
                let dest_path = theme_dest.join(rel_path);
                
                // 确保目标目录存在
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                // 复制文件
                fs::copy(src_path, dest_path)?;
            }
        }
        
        info!("主题资源已复制");
        Ok(())
    }
    
    /// 清理插件资源
    pub fn cleanup(&mut self) -> Result<()> {
        info!("清理资源...");
        
        // 调用插件钩子：清理
        if let Err(e) = self.plugin_manager.execute_hook(&PluginHook::Clean) {
            warn!("执行清理钩子失败: {}", e);
            // 继续执行，不中断整个过程
        }
        
        // 清理插件资源
        if let Err(e) = self.plugin_manager.cleanup() {
            warn!("清理插件资源失败: {}", e);
            // 继续执行，不中断整个过程
        }
        
        info!("资源清理完成");
        Ok(())
    }

    /// 创建新页面
    pub async fn new_page(&self, title: &str, path: &str) -> Result<()> {
        info!("创建新页面: {} 在路径 {}", title, path);
        // TODO: 实现页面创建逻辑
        Ok(())
    }

    /// 创建新文章
    pub async fn new_post(&self, title: &str, path: Option<&str>) -> Result<()> {
        info!("创建新文章: {}", title);
        // 生成slug化的文件名
        let slug = slug::slugify(title);
        let filename = format!("{}.md", slug);
        
        // 确定目标路径
        let target_path = match path {
            Some(p) => {
                let mut path = self.source_dir.join("_posts").join(p);
                if path.is_dir() {
                    path.push(filename);
                }
                path
            }
            None => self.source_dir.join("_posts").join(filename),
        };

        // 创建父目录（如果不存在）
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("创建目录失败: {}", parent.display()))?;
            }
        }

        // 检查文件是否已存在
        if target_path.exists() {
            return Err(anyhow::anyhow!("文件已存在: {}", target_path.display()));
        }

        // 生成Front Matter内容
        let front_matter = format!(
            "---\n\
            title: {}\n\
            date: {}\n\
            ---\n\n\
            # {}\n\n\
            在这里开始你的创作...\n",
            title,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            title
        );

        // 写入文件
        fs::write(&target_path, front_matter)
            .with_context(|| format!("写入文件失败: {}", target_path.display()))?;

        info!("成功创建文章: {}", target_path.display());
        Ok(())
    }

    /// 开始监视文件变化
    pub async fn watch(&self) -> Result<()> {
        info!("{}", "Watching for file changes...".green());
        
        // 设置监听状态
        {
            let mut is_watching = self.is_watching.write().unwrap();
            *is_watching = true;
        }
        
        // 使用notify库监听文件变化
        use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
        use std::sync::mpsc;
        use std::time::Duration;
        
        info!("创建文件监视器，基础目录: {:?}", self.base_dir);
        
        // 创建通道以接收文件系统事件
        let (tx, rx) = mpsc::channel();
        
        // 创建一个监视器，使用明确的配置
        let mut watcher_config = Config::default();
        // 注意：notify 6.x 版本的 poll_interval 不接受参数
        // 使用推荐的自动配置
        
        info!("初始化监视器，配置: {:?}", watcher_config);
        
        // 创建一个监视器
        let mut watcher = match RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        // 直接在回调中打印信息，以便确认事件触发
                        println!("收到文件事件: {:?}", event);
                        let _ = tx.send(event);
                    },
                    Err(e) => {
                        println!("监视错误: {:?}", e);
                    }
                }
            },
            watcher_config,
        ) {
            Ok(w) => {
                info!("成功创建监视器");
                w
            },
            Err(e) => {
                error!("创建监视器失败: {:?}", e);
                return Err(anyhow::anyhow!("创建文件监视器失败: {}", e));
            }
        };
        
        // 监视源目录和所有子目录
        info!("正在监控目录: {:?}", self.source_dir);
        match watcher.watch(&self.source_dir, RecursiveMode::Recursive) {
            Ok(_) => info!("成功添加源目录到监控"),
            Err(e) => error!("监控源目录失败: {:?}", e),
        }
        
        // 如果主题目录存在，也监视它
        if self.theme_dir.exists() {
            info!("正在监控主题目录: {:?}", self.theme_dir);
            match watcher.watch(&self.theme_dir, RecursiveMode::Recursive) {
                Ok(_) => info!("成功添加主题目录到监控"),
                Err(e) => error!("监控主题目录失败: {:?}", e),
            }
        }
        
        // 尝试明确地监控一些特定的子目录，以增加监控范围
        let source_posts_dir = self.source_dir.join("_posts");
        if source_posts_dir.exists() {
            info!("明确监控文章目录: {:?}", source_posts_dir);
            match watcher.watch(&source_posts_dir, RecursiveMode::Recursive) {
                Ok(_) => info!("成功添加文章目录到监控"),
                Err(e) => warn!("监控文章目录失败 (可能已被监控): {:?}", e),
            }
        }
        
        // 关键修改：保存 watcher 对象到 Engine 结构体中，避免它被销毁
        // 这样可以确保 watcher 保持存活，并继续发送事件到通道
        {
            let watcher_box: Box<dyn Any + Send + Sync> = Box::new(watcher);
            *self.file_watcher.write().unwrap() = Some(watcher_box);
            info!("文件监视器已保存到引擎实例中，确保其生命周期持续整个监控过程");
        }
        
        // 创建一个引擎的克隆，用于生成
        let mut engine = self.clone();
        
        // 在后台启动监视任务
        tokio::spawn(async move {
            // 创建一个防抖动计时器，避免频繁生成
            let mut last_event = std::time::Instant::now();
            let debounce_time = Duration::from_millis(1000);
            // let mut event_count = 0;
            
            info!("启动文件监控循环");
            
            loop {
                // 添加周期性日志，以便跟踪循环是否仍在运行
                // if event_count % 100 == 0 {
                //     info!("监控循环运行中... 已处理事件数: {}", event_count);
                // }
                
                // 读取事件，设置超时
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(event) => {
                        // event_count += 1;
                        // info!("收到事件 #{}: {:?}", event_count, event);
                        
                        // 检查事件路径
                        if let Some(path) = event.paths.get(0) {
                            info!("事件路径: {:?}", path);
                            
                            // 输出文件是否存在的信息
                            if path.exists() {
                                info!("文件存在: {:?}", path);
                                // 如果是文件，尝试获取一些元数据
                                if path.is_file() {
                                    match std::fs::metadata(path) {
                                        Ok(meta) => info!("文件元数据: 大小={}字节, 只读={}", meta.len(), meta.permissions().readonly()),
                                        Err(e) => warn!("无法获取文件元数据: {:?}", e),
                                    }
                                }
                            } else {
                                info!("文件不存在（可能已删除）: {:?}", path);
                            }
                        } else {
                            warn!("事件没有包含路径信息");
                        }
                        
                        // 只处理创建、修改和删除事件
                        match event.kind {
                            EventKind::Create(_) | 
                            EventKind::Modify(_) | 
                            EventKind::Remove(_) => {
                                info!("有效的事件类型: {:?}", event.kind);
                                
                                // 检查是否是我们关心的文件类型
                                let mut is_relevant = false;
                                for path in &event.paths {
                                    // 详细检查路径
                                    info!("检查路径: {}", path.display());
                                    
                                    // 对于目录，直接认为是相关变化
                                    if path.is_dir() {
                                        is_relevant = true;
                                        info!("检测到目录变化: {}", path.display());
                                        break;
                                    }
                                    
                                    // 对于文件，更详细地检查扩展名
                                    if let Some(ext) = path.extension() {
                                        let ext_str = ext.to_string_lossy().to_lowercase();
                                        info!("文件扩展名: {}", ext_str);
                                        
                                        if ext_str == "md" || ext_str == "markdown" || 
                                           ext_str == "yml" || ext_str == "yaml" || 
                                           ext_str == "html" || ext_str == "css" || ext_str == "js" {
                                            is_relevant = true;
                                            info!("检测到相关文件变化: {}", path.display());
                                            break;
                                        }
                                    } else {
                                        info!("文件没有扩展名: {}", path.display());
                                    }
                                    
                                    // 特别处理 _config.yml 文件
                                    if path.file_name().map_or(false, |name| name == "_config.yml") {
                                        is_relevant = true;
                                        info!("检测到配置文件变化: {}", path.display());
                                        break;
                                    }
                                }
                                
                                if is_relevant {
                                    // 更新最后事件时间
                                    last_event = std::time::Instant::now();
                                    info!("设置重新生成计时器，{}毫秒后将重新生成", debounce_time.as_millis());
                                } else {
                                    info!("忽略不相关的文件变化");
                                }
                            },
                            _ => {
                                info!("忽略事件类型: {:?}", event.kind);
                            }
                        }
                    },
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // 周期性地输出更明确的超时信息
                        // if event_count % 50 == 0 {
                        //     info!("监控超时，等待文件变化... (上次事件距今: {}毫秒)", last_event.elapsed().as_millis());
                        // }
                        
                        // 如果自上次事件以来已过去debounce时间，且有事件发生，则重新生成
                        if last_event.elapsed() >= debounce_time {
                            // 移除了额外的条件限制，只要超过防抖时间就重新生成
                            let elapsed = last_event.elapsed().as_millis();
                            
                            if elapsed < 10000 { // 只有在过去10秒内有事件时才重新生成
                                info!("检测到文件变化，重新生成... (上次事件距今: {}毫秒)", elapsed);
                                
                                // 重新生成前重新加载文章
                                info!("重新加载文章内容...");
                                if let Err(e) = engine.load_posts_and_pages() {
                                    error!("重新加载文章失败: {}", e);
                                }
                                
                                // 重新生成静态文件
                                let public_dir = engine.public_dir.clone();
                                if let Err(e) = engine.generate(&public_dir) {
                                    error!("重新生成失败: {}", e);
                                } else {
                                    info!("重新生成成功");
                                }
                                
                                // 重置最后事件时间，使用足够长的时间以避免连续触发
                                last_event = std::time::Instant::now() - Duration::from_secs(10);
                            }
                        }
                        
                        // 检查是否仍在监视
                        if !*engine.is_watching.read().unwrap() {
                            info!("监视已停止，退出监控循环");
                            break;
                        }
                    },
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        error!("监控通道已断开，退出监控循环");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }

    /// 停止监视文件变化
    pub fn unwatch(&self) {
        info!("停止监视文件变化");
        // TODO: 实现停止监控逻辑
        *self.is_watching.write().unwrap() = false;
    }

    /// 部署网站
    pub async fn deploy(&self) -> Result<()> {
        info!("部署网站");
        // TODO: 实现部署逻辑
        Ok(())
    }

    /// 启动本地服务器
    pub async fn server(&mut self, port: u16) -> Result<()> {
        info!("启动本地服务器在端口 {}", port);
        
        // 确保生成了静态文件
        let public_dir = self.public_dir.clone();
        
        // 检查生成目录是否存在，如果不存在或为空则先生成
        if !public_dir.exists() || public_dir.read_dir()?.next().is_none() {
            info!("生成目录不存在或为空，先生成静态文件");
            self.generate(&public_dir)?;
        }
        
        // 创建服务器实例
        let server = super::server::Server::new(public_dir, port);
        
        // 启动服务器
        info!("启动Web服务器在 http://localhost:{}", port);
        server.start().await?;
        
        Ok(())
    }

    /// 清理生成的文件
    pub async fn clean(&self) -> Result<()> {
        info!("清理生成的文件");
        // TODO: 实现清理逻辑
        Ok(())
    }

    /// 启动插件热重载
    pub fn start_plugin_hot_reload(&mut self) -> Result<()> {
        info!("启动插件热重载");
        self.plugin_manager.start_hot_reload()
    }

    /// 禁用插件热重载
    pub fn disable_plugin_hot_reload(&mut self) {
        info!("禁用插件热重载");
        self.plugin_manager.stop_hot_reload();
    }
}

// 工具函数：将Pod值转换为serde_yaml::Value
fn pod_to_value(pod: Pod) -> Value {
    match pod {
        Pod::String(s) => Value::String(s),
        Pod::Integer(i) => Value::Number(serde_yaml::Number::from(i)),
        Pod::Float(f) => Value::Number(serde_yaml::Number::from(f)),
        Pod::Boolean(b) => Value::Bool(b),
        Pod::Array(arr) => Value::Sequence(arr.into_iter().map(pod_to_value).collect()),
        Pod::Hash(map) => {
            let mut mapping = Mapping::new();
            for (k, v) in map {
                mapping.insert(Value::String(k), pod_to_value(v));
            }
            Value::Mapping(mapping)
        },
        Pod::Null => Value::Null,
    }
} 
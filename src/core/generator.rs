use std::{
    fs::{self, File},
    io::{self, Write as IoWrite},
    path::{Path, PathBuf},
    collections::HashMap,
    fmt::Write,
};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Datelike, FixedOffset, TimeZone, Local};
use pulldown_cmark::{html, Parser};
use tracing::{debug, info, warn};
use atom_syndication::{Feed, FeedBuilder, Entry, Text};
use rss::{Channel, ChannelBuilder, Item, Guid};
use serde_json::{json, Value};
use walkdir::WalkDir;
use url::Url;
use tera::{Context, Tera};

use crate::{
    models::{
        types::Post,
        config::Config,
    },
    plugins::{
        PluginManager,
        PluginContext,
        PluginError,
        PluginHook,
    },
    core::search::SearchIndexGenerator,
};

/// HTML 生成器
pub struct HtmlGenerator {
    /// 输出目录
    pub output_dir: PathBuf,
    /// 站点配置
    pub config: Config,
    /// 插件管理器
    plugin_manager: PluginManager,
}

impl HtmlGenerator {
    /// 创建新的 HTML 生成器
    pub fn new(output_dir: PathBuf, config: Config, base_dir: PathBuf) -> Self {
        let plugin_manager = PluginManager::new(base_dir.clone(), PluginContext::default());
        Self {
            output_dir,
            config,
            plugin_manager,
        }
    }
    
    /// 使用已初始化的插件管理器创建HTML生成器
    pub fn new_with_plugin_manager(output_dir: PathBuf, config: Config, plugin_manager: PluginManager) -> Self {
        Self {
            output_dir,
            config,
            plugin_manager,
        }
    }
    
    /// 生成所有内容
    pub fn generate(&self, posts: &[Post]) -> Result<()> {
        info!("开始生成静态网站...");
        
        // 调用插件钩子：生成前
        self.plugin_manager.execute_hook(&PluginHook::BeforeGenerate)?;
        
        // 确保输出目录存在
        fs::create_dir_all(&self.output_dir)?;
        
        // 复制主题资源文件
        self.copy_theme_assets()?;
        
        // 生成文章页面
        self.generate_posts(posts)?;
        
        // 生成索引页面（带分页）
        let per_page = self.config.per_page.unwrap_or(10) as usize;
        self.generate_paginated_index(posts, per_page)?;
        
        // 生成分类页面
        self.generate_categories(posts)?;
        
        // 生成标签页面
        self.generate_tags(posts)?;
        
        // 生成归档页面
        self.generate_archives(posts)?;
        
        // 生成 RSS feed
        self.generate_rss_feed(posts)?;
        
        // 生成 Atom feed
        self.generate_atom_feed(posts)?;
        
        // 生成搜索索引
        self.generate_search_index(posts)?;
        
        // 生成成功，调用生成后钩子
        self.plugin_manager.execute_hook(&PluginHook::AfterGenerate)?;
        
        info!("Generated HTML files successfully");
        
        Ok(())
    }
    
    /// 清理临时文件和资源
    pub fn cleanup(&self) -> Result<()> {
        // 调用清理钩子
        self.plugin_manager.execute_hook(&PluginHook::Clean)?;
        
        // 清理插件资源
        self.plugin_manager.cleanup()?;
        
        Ok(())
    }
    
    /// 复制主题资源文件
    fn copy_theme_assets(&self) -> Result<()> {
        let theme_name = self.config.theme.clone().unwrap_or_default();
        let theme_source = PathBuf::from("themes").join(&theme_name).join("source");
        
        if theme_source.exists() {
            info!("Copying theme assets from {:?}", theme_source);
            
            // 遍历主题资源目录
            for entry in WalkDir::new(&theme_source)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let source_path = entry.path();
                let relative_path = source_path.strip_prefix(&theme_source)?;
                let target_path = self.output_dir.join(relative_path);
                
                // 确保目标目录存在
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                // 复制文件
                fs::copy(source_path, target_path)?;
            }
            
            info!("Theme assets copied successfully");
        } else {
            warn!("Theme source directory not found: {:?}", theme_source);
        }
        
        Ok(())
    }
    
    /// 生成所有文章页面
    fn generate_posts(&self, posts: &[Post]) -> Result<()> {
        info!("Generating post pages...");
        for post in posts {
            self.generate_post(post)?;
        }
        Ok(())
    }
    
    /// 生成分页的索引页面
    fn generate_paginated_index(&self, posts: &[Post], page_size: usize) -> Result<()> {
        info!("Generating paginated index pages...");
        
        let total_posts = posts.len();
        let total_pages = (total_posts + page_size - 1) / page_size;
        
        for page_num in 1..=total_pages {
            let start_idx = (page_num - 1) * page_size;
            let end_idx = std::cmp::min(start_idx + page_size, total_posts);
            let page_posts: Vec<&Post> = posts[start_idx..end_idx].iter().collect();
            
            let file_name = if page_num == 1 {
                "index.html".to_string()
            } else {
                format!("page/{}/index.html", page_num)
            };
            
            let output_file = self.output_dir.join(&file_name);
            if let Some(parent) = output_file.parent() {
                fs::create_dir_all(parent)?;
            }
            
            self.generate_index_page(&page_posts, page_num, total_pages, &output_file)?;
        }
        
        Ok(())
    }
    
    /// 生成分类页面
    fn generate_categories(&self, posts: &[Post]) -> Result<()> {
        info!("Generating category pages...");
        
        // 按分类对文章进行分组
        let mut categories: HashMap<String, Vec<&Post>> = HashMap::new();
        for post in posts {
            for category in &post.categories {
                categories.entry(category.clone())
                    .or_default()
                    .push(post);
            }
        }
        
        // 创建分类目录
        let categories_dir = self.output_dir.join("categories");
        fs::create_dir_all(&categories_dir)?;
        
        // 生成分类索引页面
        self.generate_categories_index(&categories)?;
        
        // 生成每个分类的页面
        for (category, category_posts) in categories {
            let category_dir = categories_dir.join(&category);
            fs::create_dir_all(&category_dir)?;
            
            // 生成分类文章列表页面
            let mut content = String::with_capacity(4096);
            self.write_html_header(&mut content, &format!("Category: {}", category))?;
            
            writeln!(&mut content, "    <div class=\"category-posts\">")?;
            writeln!(&mut content, "        <h1>Category: {}</h1>", category)?;
            writeln!(&mut content, "        <p>Posts: {}</p>", category_posts.len())?;
            
            // 添加文章列表
            for post in category_posts {
                writeln!(&mut content, "        <article class=\"post-item\">")?;
                writeln!(&mut content, "            <h2><a href=\"/{}\">{}</a></h2>", post.path, post.title)?;
                writeln!(&mut content, "            <time>{}</time>", post.date.format("%Y-%m-%d"))?;
                writeln!(&mut content, "        </article>")?;
            }
            
            writeln!(&mut content, "    </div>")?;
            
            self.write_html_footer(&mut content)?;
            
            // 写入文件
            fs::write(category_dir.join("index.html"), content)?;
        }
        
        Ok(())
    }
    
    /// 生成标签页面
    fn generate_tags(&self, posts: &[Post]) -> Result<()> {
        info!("Generating tag pages...");
        
        // 按标签对文章进行分组
        let mut tags: HashMap<String, Vec<&Post>> = HashMap::new();
        for post in posts {
            for tag in &post.tags {
                tags.entry(tag.clone())
                    .or_default()
                    .push(post);
            }
        }
        
        // 创建标签目录
        let tags_dir = self.output_dir.join("tags");
        fs::create_dir_all(&tags_dir)?;
        
        // 生成标签索引页面
        self.generate_tags_index(&tags)?;
        
        // 生成每个标签的页面
        for (tag, tag_posts) in tags {
            let tag_dir = tags_dir.join(&tag);
            fs::create_dir_all(&tag_dir)?;
            
            // 生成标签文章列表页面
            let mut content = String::with_capacity(4096);
            self.write_html_header(&mut content, &format!("Tag: {}", tag))?;
            
            writeln!(&mut content, "    <div class=\"tag-posts\">")?;
            writeln!(&mut content, "        <h1>Tag: {}</h1>", tag)?;
            writeln!(&mut content, "        <p>Posts: {}</p>", tag_posts.len())?;
            
            // 添加文章列表
            for post in tag_posts {
                writeln!(&mut content, "        <article class=\"post-item\">")?;
                writeln!(&mut content, "            <h2><a href=\"/{}\">{}</a></h2>", post.path, post.title)?;
                writeln!(&mut content, "            <time>{}</time>", post.date.format("%Y-%m-%d"))?;
                writeln!(&mut content, "        </article>")?;
            }
            
            writeln!(&mut content, "    </div>")?;
            
            self.write_html_footer(&mut content)?;
            
            // 写入文件
            fs::write(tag_dir.join("index.html"), content)?;
        }
        
        Ok(())
    }
    
    /// 生成归档页面
    fn generate_archives(&self, posts: &[Post]) -> Result<()> {
        info!("Generating archive pages...");
        
        // 按年月对文章进行分组
        let mut archives: HashMap<(i32, u32), Vec<&Post>> = HashMap::new();
        for post in posts {
            let year = post.date.year();
            let month = post.date.month();
            archives.entry((year, month))
                .or_default()
                .push(post);
        }
        
        // 创建归档目录
        let archives_dir = self.output_dir.join("archives");
        fs::create_dir_all(&archives_dir)?;
        
        // 生成归档索引页面
        self.generate_archives_index(&archives)?;
        
        // 生成每个年月的归档页面
        for ((year, month), archive_posts) in &archives {
            let year_dir = archives_dir.join(year.to_string());
            fs::create_dir_all(&year_dir)?;
            
            // 生成月度归档页面
            let mut content = String::with_capacity(4096);
            self.write_html_header(&mut content, &format!("Archive: {}-{:02}", year, month))?;
            
            writeln!(&mut content, "    <div class=\"archive-posts\">")?;
            writeln!(&mut content, "        <h1>Archive: {}-{:02}</h1>", year, month)?;
            writeln!(&mut content, "        <p>Posts: {}</p>", archive_posts.len())?;
            
            // 添加文章列表
            for post in archive_posts {
                writeln!(&mut content, "        <article class=\"post-item\">")?;
                writeln!(&mut content, "            <h2><a href=\"/{}\">{}</a></h2>", post.path, post.title)?;
                writeln!(&mut content, "            <time>{}</time>", post.date.format("%Y-%m-%d"))?;
                writeln!(&mut content, "        </article>")?;
            }
            
            writeln!(&mut content, "    </div>")?;
            
            self.write_html_footer(&mut content)?;
            
            // 写入文件
            fs::write(year_dir.join(format!("{:02}.html", month)), content)?;
        }
        
        Ok(())
    }
    
    /// 生成分类索引页面
    fn generate_categories_index(&self, categories: &HashMap<String, Vec<&Post>>) -> Result<()> {
        let mut content = String::with_capacity(4096);
        self.write_html_header(&mut content, "分类")?;
        
        writeln!(&mut content, "    <div class=\"categories-list\">")?;
        writeln!(&mut content, "        <h1>分类</h1>")?;
        
        // 按分类名称排序
        let mut category_names: Vec<&String> = categories.keys().collect();
        category_names.sort();
        
        for category_name in category_names {
            let posts = &categories[category_name];
            writeln!(&mut content, "        <div class=\"category-item\">")?;
            writeln!(&mut content, "            <h2><a href=\"/categories/{}\">{}</a></h2>", 
                     category_name, category_name)?;
            writeln!(&mut content, "            <span class=\"post-count\">{} 篇文章</span>", posts.len())?;
            writeln!(&mut content, "        </div>")?;
        }
        
        // 获取所有文章并去重
        let mut all_posts = Vec::new();
        for (_, posts) in categories {
            for &post in posts {
                all_posts.push(post);
            }
        }
        
        // 去重
        all_posts.sort_by(|a, b| a.path.cmp(&b.path));
        all_posts.dedup_by(|a, b| a.path == b.path);
        
        // 按日期排序
        all_posts.sort_by(|a, b| b.date.cmp(&a.date));
        
        // 添加所有文章列表
        writeln!(&mut content, "        <div class=\"all-posts\">")?;
        writeln!(&mut content, "            <h2>全部分类文章</h2>")?;
        
        for post in &all_posts {
            writeln!(&mut content, "            <article class=\"post-item\">")?;
            writeln!(&mut content, "                <h3><a href=\"/{}\">{}</a></h3>", post.path, post.title)?;
            writeln!(&mut content, "                <div class=\"post-meta\">")?;
            writeln!(&mut content, "                    <time>{}</time>", post.date.format("%Y-%m-%d"))?;
            
            // 显示分类
            if !post.categories.is_empty() {
                write!(&mut content, " | 分类: ")?;
                for (i, category) in post.categories.iter().enumerate() {
                    if i > 0 {
                        write!(&mut content, ", ")?;
                    }
                    write!(&mut content, "<a href=\"/categories/{}\">{}</a>", category, category)?;
                }
            }
            
            // 显示标签
            if !post.tags.is_empty() {
                write!(&mut content, " | 标签: ")?;
                for (i, tag) in post.tags.iter().enumerate() {
                    if i > 0 {
                        write!(&mut content, ", ")?;
                    }
                    write!(&mut content, "<a href=\"/tags/{}\">{}</a>", tag, tag)?;
                }
            }
            
            writeln!(&mut content, "                </div>")?;
            writeln!(&mut content, "            </article>")?;
        }
        
        writeln!(&mut content, "        </div>")?;
        writeln!(&mut content, "    </div>")?;
        
        self.write_html_footer(&mut content)?;
        
        let output_path = self.output_dir.join("categories").join("index.html");
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(output_path, content)?;
        
        Ok(())
    }
    
    /// 生成标签索引页面
    fn generate_tags_index(&self, tags: &HashMap<String, Vec<&Post>>) -> Result<()> {
        let mut content = String::with_capacity(4096);
        self.write_html_header(&mut content, "标签")?;
        
        writeln!(&mut content, "    <div class=\"tags-list\">")?;
        writeln!(&mut content, "        <h1>标签</h1>")?;
        writeln!(&mut content, "        <div class=\"tag-cloud\">")?;
        
        // 按标签名称排序
        let mut tag_names: Vec<&String> = tags.keys().collect();
        tag_names.sort();
        
        for tag_name in tag_names {
            let posts = &tags[tag_name];
            let font_size = 100.0 + (posts.len() as f32 * 10.0).min(100.0);
            
            writeln!(
                &mut content,
                "            <a href=\"/tags/{}\" style=\"font-size: {}%\">{} <span class=\"tag-count\">({})</span></a>",
                tag_name, font_size, tag_name, posts.len()
            )?;
        }
        
        writeln!(&mut content, "        </div>")?;
        
        // 获取所有文章并去重
        let mut all_posts = Vec::new();
        for (_, posts) in tags {
            for &post in posts {
                all_posts.push(post);
            }
        }
        
        // 去重
        all_posts.sort_by(|a, b| a.path.cmp(&b.path));
        all_posts.dedup_by(|a, b| a.path == b.path);
        
        // 按日期排序
        all_posts.sort_by(|a, b| b.date.cmp(&a.date));
        
        // 添加所有文章列表
        writeln!(&mut content, "        <div class=\"all-posts\">")?;
        writeln!(&mut content, "            <h2>全部标签文章</h2>")?;
        
        for post in &all_posts {
            writeln!(&mut content, "            <article class=\"post-item\">")?;
            writeln!(&mut content, "                <h3><a href=\"/{}\">{}</a></h3>", post.path, post.title)?;
            writeln!(&mut content, "                <div class=\"post-meta\">")?;
            writeln!(&mut content, "                    <time>{}</time>", post.date.format("%Y-%m-%d"))?;
            
            // 显示分类
            if !post.categories.is_empty() {
                write!(&mut content, " | 分类: ")?;
                for (i, category) in post.categories.iter().enumerate() {
                    if i > 0 {
                        write!(&mut content, ", ")?;
                    }
                    write!(&mut content, "<a href=\"/categories/{}\">{}</a>", category, category)?;
                }
            }
            
            // 显示标签
            if !post.tags.is_empty() {
                write!(&mut content, " | 标签: ")?;
                for (i, tag) in post.tags.iter().enumerate() {
                    if i > 0 {
                        write!(&mut content, ", ")?;
                    }
                    write!(&mut content, "<a href=\"/tags/{}\">{}</a>", tag, tag)?;
                }
            }
            
            writeln!(&mut content, "                </div>")?;
            writeln!(&mut content, "            </article>")?;
        }
        
        writeln!(&mut content, "        </div>")?;
        writeln!(&mut content, "    </div>")?;
        
        self.write_html_footer(&mut content)?;
        
        let output_path = self.output_dir.join("tags").join("index.html");
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(output_path, content)?;
        
        Ok(())
    }
    
    /// 生成归档索引页面
    fn generate_archives_index(&self, archives: &HashMap<(i32, u32), Vec<&Post>>) -> Result<()> {
        let mut content = String::with_capacity(4096);
        self.write_html_header(&mut content, "Archives")?;
        
        writeln!(&mut content, "    <div class=\"archives-list\">")?;
        writeln!(&mut content, "        <h1>Archives</h1>")?;
        
        // 按年月排序
        let mut archive_dates: Vec<(i32, u32)> = archives.keys().cloned().collect();
        archive_dates.sort_by(|a, b| b.cmp(a));
        
        let mut current_year = None;
        for (year, month) in archive_dates {
            if current_year != Some(year) {
                if current_year.is_some() {
                    writeln!(&mut content, "        </div>")?;
                }
                writeln!(&mut content, "        <div class=\"year-group\">")?;
                writeln!(&mut content, "            <h2>{}</h2>", year)?;
                current_year = Some(year);
            }
            
            let posts = &archives[&(year, month)];
            writeln!(&mut content, "            <div class=\"month-group\">")?;
            writeln!(&mut content, "                <h3><a href=\"/archives/{}/{:02}.html\">{}-{:02}</a> ({} posts)</h3>",
                    year, month, year, month, posts.len())?;
            writeln!(&mut content, "            </div>")?;
        }
        
        if current_year.is_some() {
            writeln!(&mut content, "        </div>")?;
        }
        
        writeln!(&mut content, "    </div>")?;
        
        self.write_html_footer(&mut content)?;
        
        let output_path = self.output_dir.join("archives").join("index.html");
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(output_path, content)?;
        
        Ok(())
    }
    
    /// 写入HTML头部
    fn write_html_header(&self, content: &mut String, title: &str) -> Result<()> {
        writeln!(content, "<!DOCTYPE html>")?;
        writeln!(content, "<html lang=\"{}\">", self.config.language.clone().unwrap_or_default())?;
        writeln!(content, "<head>")?;
        writeln!(content, "    <meta charset=\"UTF-8\">")?;
        writeln!(content, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(content, "    <title>{}</title>", title)?;
        writeln!(content, "    <link rel=\"stylesheet\" href=\"/css/style.css\">")?;
        writeln!(content, "</head>")?;
        writeln!(content, "<body>")?;
        writeln!(content, "    <header class=\"site-header\">")?;
        writeln!(content, "        <div class=\"container\">")?;
        writeln!(content, "            <h1><a href=\"/\">{}</a></h1>", self.config.title)?;
        if let Some(subtitle) = &self.config.subtitle {
            writeln!(content, "            <p class=\"site-description\">{}</p>", subtitle)?;
        }
        writeln!(content, "            <nav class=\"site-nav\">")?;
        writeln!(content, "                <a href=\"/\">Home</a>")?;
        writeln!(content, "                <a href=\"/archives/\">Archives</a>")?;
        writeln!(content, "                <a href=\"/categories/\">Categories</a>")?;
        writeln!(content, "                <a href=\"/tags/\">Tags</a>")?;
        writeln!(content, "            </nav>")?;
        writeln!(content, "        </div>")?;
        writeln!(content, "    </header>")?;
        writeln!(content, "    <main class=\"container\">")?;
        
        Ok(())
    }
    
    /// 写入HTML尾部
    fn write_html_footer(&self, content: &mut String) -> Result<()> {
        writeln!(content, "    </main>")?;
        writeln!(content, "    <footer class=\"site-footer\">")?;
        writeln!(content, "        <div class=\"container\">")?;
        writeln!(content, "            <p>&copy; {} {}", chrono::Utc::now().year(), self.config.title)?;
        if let Some(author) = &self.config.author {
            writeln!(content, " by {}", author)?;
        }
        writeln!(content, "            </p>")?;
        writeln!(content, "        </div>")?;
        writeln!(content, "    </footer>")?;
        writeln!(content, "</body>")?;
        writeln!(content, "</html>")?;
        
        Ok(())
    }
    
    /// 生成单个文章页面
    fn generate_post(&self, post: &Post) -> Result<()> {
        debug!("Generating post: {}", post.title);
        
        // 创建输出路径
        let post_path = self.output_dir.join(&post.path);
        fs::create_dir_all(post_path.parent().unwrap_or(&self.output_dir))?;
        
        // 使用Tera模板渲染文章页面
        let template_name = "post.html";
        let mut context = Context::new();
        
        // 添加页面数据
        context.insert("page", post);
        
        // 添加站点数据
        // 创建站点数据结构
        let mut site_data = serde_json::Map::new();
        
        // 添加配置信息 - 为了避免missing字段问题，我们手动构建一个完整的config对象
        let mut config_data = serde_json::Map::new();
        config_data.insert("title".to_string(), serde_json::Value::String(self.config.title.clone()));
        config_data.insert("subtitle".to_string(), serde_json::Value::String(self.config.subtitle.clone().unwrap_or_default()));
        config_data.insert("description".to_string(), serde_json::Value::String(self.config.description.clone().unwrap_or_default()));
        config_data.insert("author".to_string(), serde_json::Value::String(self.config.author.clone().unwrap_or_default()));
        
        // 语言字段处理，确保提供默认值
        let language = if self.config.language.clone().unwrap_or_default().is_empty() {
            "zh-CN".to_string()
        } else {
            self.config.language.clone().unwrap_or_default()
        };
        config_data.insert("language".to_string(), serde_json::Value::String(language));
        
        config_data.insert("timezone".to_string(), serde_json::Value::String(self.config.timezone.clone().unwrap_or_default()));
        config_data.insert("url".to_string(), serde_json::Value::String(self.config.url.clone().unwrap_or_default()));
        
        // root字段处理，确保提供默认值
        let root = if self.config.root.clone().unwrap_or_default().is_empty() {
            "/".to_string()
        } else {
            self.config.root.clone().unwrap_or_default()
        };
        config_data.insert("root".to_string(), serde_json::Value::String(root));
        
        // 默认的永久链接格式
        config_data.insert("permalink".to_string(), serde_json::Value::String(":year/:month/:day/:title/".to_string()));
        config_data.insert("theme".to_string(), serde_json::Value::String(self.config.theme.clone().unwrap_or_default()));
        
        // 添加各种目录配置
        config_data.insert("source_dir".to_string(), serde_json::Value::String("source".to_string()));
        config_data.insert("public_dir".to_string(), serde_json::Value::String("public".to_string()));
        config_data.insert("tag_dir".to_string(), serde_json::Value::String("tags".to_string()));
        config_data.insert("category_dir".to_string(), serde_json::Value::String("categories".to_string()));
        config_data.insert("archive_dir".to_string(), serde_json::Value::String("archives".to_string()));
        
        // 添加关键字和其他可选配置
        config_data.insert("keywords".to_string(), serde_json::Value::String("".to_string()));
        
        // 添加分页配置
        let per_page = self.config.per_page.unwrap_or(10);
        config_data.insert("per_page".to_string(), serde_json::Value::Number(serde_json::Number::from(per_page as i64)));
        
        config_data.insert("pagination_dir".to_string(), serde_json::Value::String("page".to_string()));
        
        // 将整个配置添加到站点数据中
        site_data.insert("config".to_string(), serde_json::Value::Object(config_data));
        
        // 添加其它站点信息（顶级）
        site_data.insert("title".to_string(), serde_json::Value::String(self.config.title.clone()));
        site_data.insert("url".to_string(), serde_json::Value::String(self.config.url.clone().unwrap_or_default()));
        site_data.insert("author".to_string(), serde_json::Value::String(self.config.author.clone().unwrap_or_default()));
        
        context.insert("site", &serde_json::Value::Object(site_data));
        
        // 添加当前时间函数
        let now = chrono::Utc::now();
        context.insert("now", &tera::Value::String(now.format("%Y-%m-%d %H:%M:%S").to_string()));
        
        // 添加插件数据
        let plugin_manager = &self.plugin_manager;
        match plugin_manager.get_all_plugins() {
            Ok(plugins) => {
                // 为特定插件添加标志
                let mut plugin_enabled = HashMap::new();
                for plugin in &plugins {
                    let name = plugin.name();
                    debug!("启用插件 {} 在文章模板中", name);
                    // 使用插件名作为键（统一使用中横线格式，与配置名保持一致）
                    let config_name = if name.contains('_') {
                        name.replace('_', "-")
                    } else {
                        name.to_string()
                    };
                    plugin_enabled.insert(config_name, true);
                }
                context.insert("plugins", &plugin_enabled);
            },
            Err(e) => {
                warn!("获取插件列表失败: {}", e);
            }
        }
        
        // 创建Tera实例
        let mut tera = Tera::default();
        
        // 从配置中获取主题目录路径
        let theme_name = &self.config.theme.clone().unwrap_or_default();
        let base_dir = self.output_dir.parent().unwrap_or(&self.output_dir);
        let theme_dir = base_dir.join("themes").join(theme_name);
        let layout_dir = theme_dir.join("layout");
        
        // 加载所有必要的模板
        debug!("加载主题模板，目录: {}", layout_dir.display());
        
        // 添加 layout.html 模板
        let layout_path = layout_dir.join("layout.html");
        if layout_path.exists() {
            let layout_content = fs::read_to_string(&layout_path)?;
            tera.add_raw_template("layout.html", &layout_content)?;
            debug!("加载模板: layout.html");
        } else {
            return Err(anyhow!("基础布局模板文件不存在: {}", layout_path.display()));
        }
        
        // 添加需要的模板文件
        let template_path = layout_dir.join(template_name);
        if template_path.exists() {
            let template_content = fs::read_to_string(&template_path)?;
            tera.add_raw_template(template_name, &template_content)?;
            debug!("加载模板: {}", template_name);
        } else {
            return Err(anyhow!("模板文件不存在: {}", template_path.display()));
        }
        
        // 注册基本的日期格式化和默认值函数
        self.register_basic_functions(&mut tera);
        
        // 从插件中动态注册模板函数
        self.plugin_manager.register_template_functions(&mut tera)?;
        
        // 渲染模板
        let rendered = tera.render(template_name, &context)?;
        
        // 写入文件
        fs::write(post_path, rendered)?;
        
        Ok(())
    }
    
    /// 注册基本的模板函数和过滤器
    fn register_basic_functions(&self, tera: &mut Tera) {
        // 注册日期格式化函数
        tera.register_function("date", |args: &HashMap<String, tera::Value>| {
            let value = match args.get("value") {
                Some(v) => v,
                None => return Err(tera::Error::msg("缺少必要的参数: value"))
            };
            
            let format = match args.get("format") {
                Some(f) => match f.as_str() {
                    Some(s) => s,
                    None => return Err(tera::Error::msg("format 必须是字符串"))
                },
                None => "%Y-%m-%d"
            };
            
            // 处理不同类型的日期值
            if let Some(date_str) = value.as_str() {
                // 尝试解析为RFC3339格式
                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date_str) {
                    return Ok(tera::Value::String(date.format(format).to_string()));
                }
                
                // 尝试解析为其他常见格式
                let formats = [
                    "%Y-%m-%d %H:%M:%S",
                    "%Y-%m-%d",
                    "%Y/%m/%d %H:%M:%S",
                    "%Y/%m/%d",
                ];
                
                for fmt in &formats {
                    if let Ok(date) = chrono::NaiveDateTime::parse_from_str(date_str, fmt) {
                        let date = chrono::Utc.from_utc_datetime(&date);
                        return Ok(tera::Value::String(date.format(format).to_string()));
                    }
                }
                
                // 返回原始字符串如果无法解析
                return Ok(tera::Value::String(date_str.to_string()));
            }
            
            // 处理数字类型（Unix时间戳）
            if let Some(timestamp) = value.as_i64() {
                if let Some(naive_dt) = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0) {
                    let date = chrono::Utc.from_utc_datetime(&naive_dt);
                    return Ok(tera::Value::String(date.format(format).to_string()));
                }
            }
            
            // 如果无法解析，返回错误
            Err(tera::Error::msg("无法将value解析为日期时间"))
        });
        
        // 注册默认函数
        tera.register_function("default", |args: &HashMap<String, tera::Value>| {
            let value = match args.get("value") {
                Some(v) => v,
                None => return Err(tera::Error::msg("缺少必要的参数: value"))
            };
            
            let default_value = match args.get("default") {
                Some(d) => d,
                None => return Err(tera::Error::msg("缺少必要的参数: default"))
            };
            
            if value.is_null() || (value.is_string() && value.as_str().unwrap_or("").is_empty()) {
                Ok(default_value.clone())
            } else {
                Ok(value.clone())
            }
        });
    }
    
    /// 生成单个索引页面
    fn generate_index_page(&self, posts: &[&Post], page_num: usize, total_pages: usize, output_file: &PathBuf) -> Result<()> {
        // 创建Tera上下文
        let mut context = Context::new();
        
        // 处理页面信息
        let prev_link = if page_num == 2 { 
            String::from("/") 
        } else { 
            format!("/page/{}/", page_num - 1) 
        };
        
        let page = json!({
            "posts": posts.iter().map(|post| {
            // 创建文章摘要
            let excerpt = post.excerpt.clone().unwrap_or_else(|| {
                    // 如果没有摘要，使用内容的前200个字符
                let content = &post.content;
                    if content.len() > 200 {
                        if let Some(idx) = content.char_indices().nth(200).map(|(i, _)| i) {
                            // 安全处理截断内容，确保HTML标签闭合
                            let truncated = &content[..idx];
                            // 简单规则：如果截断内容中有<code但没有</code>，添加</code></pre>
                            if truncated.contains("<code") && !truncated.contains("</code>") {
                                format!("{}...</code></pre>", truncated)
                            } else {
                                format!("{}...", truncated)
                            }
                        } else { 
                            content.clone() 
                        }
                } else {
                    content.clone()
                }
            });
            
                // 构建文章信息
                json!({
                    "title": post.title,
                    "path": post.path,
                    "date": post.date,
                    "categories": post.categories,
                    "tags": post.tags,
                    "excerpt": excerpt,
                    "content": post.content,
                    "permalink": post.permalink,
                })
            }).collect::<Vec<_>>(),
            "current": page_num,
            "total": total_pages,
            "prev": if page_num > 1 { Some(page_num - 1) } else { None },
            "next": if page_num < total_pages { Some(page_num + 1) } else { None },
            "prev_link": if page_num > 1 { Some(prev_link) } else { None },
            "next_link": if page_num < total_pages { Some(format!("/page/{}/", page_num + 1)) } else { None },
            "base": "/",
        });
        context.insert("page", &page);
        
        // 添加网站配置信息
        let site = json!({
            "config": {
                "title": self.config.title.clone(),
                "subtitle": self.config.subtitle.clone().unwrap_or_default(),
                "description": self.config.description.clone().unwrap_or_default(),
                "url": self.get_url(),
                "author": self.config.author.clone().unwrap_or_default(),
                "language": self.get_language(),
                "keywords": "", // 添加空的keywords字段
                "root": self.get_root(),
                "permalink": self.config.permalink.clone().unwrap_or_default(),
                "theme": self.config.theme.clone().unwrap_or_default()
            },
            "title": self.config.title.clone(),
            "url": self.get_url(),
            "author": self.config.author.clone().unwrap_or_default()
        });
        context.insert("site", &site);
        
        // 添加插件信息
        let plugin_manager = &self.plugin_manager;
        let mut plugins = HashMap::new();
        // 使用Arc和RwLock安全地访问插件
        if let Ok(plugin_lock) = plugin_manager.plugins.read() {
            for plugin_name in plugin_lock.keys() {
                plugins.insert(plugin_name.clone(), true);
            }
        }
        context.insert("plugins", &plugins);
        
        // 加载模板
        let theme_path = self.get_theme_path()?;
        let layout_dir = theme_path.join("layout");
        let template_path = layout_dir.join("index.html");

        if !template_path.exists() {
            return Err(anyhow!("找不到首页模板文件: {:?}", template_path));
        }

        // 创建Tera实例
        let mut tera = Tera::default();

        // 首先显式加载layout.html布局模板
        let layout_path = layout_dir.join("layout.html");
        if layout_path.exists() {
            tera.add_template_file(&layout_path, Some("layout.html"))?;
                } else {
            return Err(anyhow!("找不到布局模板文件: {:?}", layout_path));
        }

        // 然后加载index.html模板
        let template_name = "index.html";
        tera.add_template_file(&template_path, Some(template_name))?;

        // 添加模板目录中的所有其他模板
        if layout_dir.exists() {
            // 使用 WalkDir 遍历目录
            for entry in WalkDir::new(&layout_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "html") {
                    // 跳过已经手动加载的模板
                    if path == layout_path || path == template_path {
                        continue;
                    }
                    if let Ok(rel_path) = path.strip_prefix(&layout_dir) {
                        let name = rel_path.to_string_lossy();
                        tera.add_template_file(path, Some(name.as_ref()))?;
                    }
                }
            }
        }
        
        // 注册基本的日期格式化和默认值函数
        self.register_basic_functions(&mut tera);
        
        // 从插件中动态注册模板函数
        self.plugin_manager.register_template_functions(&mut tera)?;
        
        // 渲染模板
        let rendered = tera.render(template_name, &context)?;
        
        // 确保目标目录存在
        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 写入文件
        fs::write(output_file, rendered)?;
        
        Ok(())
    }

    // 获取主题目录路径的辅助方法
    fn get_theme_path(&self) -> Result<PathBuf> {
        let theme = self.config.theme.clone().unwrap_or_default();
        Ok(PathBuf::from("themes").join(theme))
    }

    // 获取语言的辅助方法
    fn get_language(&self) -> String {
        self.config.language.clone().unwrap_or_else(|| "zh-CN".to_string())
    }

    // 获取URL的辅助方法
    fn get_url(&self) -> String {
        self.config.url.clone().unwrap_or_else(|| "http://localhost".to_string())
    }

    // 获取根路径的辅助方法
    fn get_root(&self) -> String {
        self.config.root.clone().unwrap_or_else(|| "/".to_string())
    }

    /// 生成 RSS feed
    fn generate_rss_feed(&self, posts: &[Post]) -> Result<()> {
        let mut channel = Channel::default();
        channel.set_title(self.config.title.clone());
        channel.set_link(self.config.url.clone().unwrap_or_default());
        channel.set_description(self.config.description.clone().unwrap_or_default());
        channel.set_language(Some(self.config.language.clone().unwrap_or_default()));

        let mut posts = posts.to_vec();
        posts.sort_by(|a, b| b.date.cmp(&a.date));
        posts.truncate(20); // 默认显示最新的20篇文章

        for post in posts.iter().take(10) {
            let mut item = Item::default();
            item.set_title(post.title.clone());
            let url_base = self.config.url.clone().unwrap_or_default();
            item.set_link(format!("{}/{}", url_base.trim_end_matches('/'), post.path));
            item.set_guid(Guid {
                value: post.permalink.clone(),
                permalink: true,
            });
            item.set_pub_date(post.date.to_rfc2822());
            item.set_description(post.excerpt.clone().unwrap_or_default());
            
            channel.items.push(item);
        }

        let output_path = self.output_dir.join("rss.xml");
        let mut content = String::with_capacity(4096);
        write!(&mut content, "{}", channel.to_string())?;
        fs::write(output_path, content)?;

        Ok(())
    }
    
    /// 生成 Atom feed
    fn generate_atom_feed(&self, posts: &[Post]) -> Result<()> {
        let mut feed = Feed::default();
        feed.set_title(self.config.title.clone());
        feed.set_id(self.config.url.clone().unwrap_or_default());
        feed.set_updated(Utc::now());

        if let Some(subtitle) = &self.config.subtitle {
            feed.set_subtitle(Text::plain(subtitle.clone()));
        }

        feed.set_lang(Some(self.config.language.clone().unwrap_or_default()));

        let mut posts = posts.to_vec();
        posts.sort_by(|a, b| b.date.cmp(&a.date));
        posts.truncate(20); // 默认显示最新的20篇文章

        for post in posts {
            let mut entry = Entry::default();
            
            let url_base = self.config.url.clone().unwrap_or_default();
            let post_url = format!("{}/{}", url_base.trim_end_matches('/'), post.path);
            
            entry.set_id(post_url.clone());
            entry.set_title(Text::plain(post.title.clone()));
            
            let mut links = Vec::new();
            let mut link = atom_syndication::Link::default();
            link.set_href(post_url);
            link.set_rel("alternate".to_string());
            links.push(link);
            entry.set_links(links);
            
            // 将 Utc 时间转换为固定偏移时间
            let fixed_date = post.date.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
            entry.set_updated(fixed_date);
            
            // 发布时间同样需要转换
            if let Some(published) = Some(post.date) {
                let fixed_published = published.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
                entry.set_published(Some(fixed_published));
            }
            
            entry.set_summary(Text::plain(post.excerpt.clone().unwrap_or_default()));
            
            feed.entries.push(entry);
        }

        let output_path = self.output_dir.join("atom.xml");
        let mut content = String::with_capacity(4096);
        write!(&mut content, "{}", feed.to_string())?;
        fs::write(output_path, content)?;

        Ok(())
    }
    
    /// 生成搜索索引
    fn generate_search_index(&self, posts: &[Post]) -> Result<()> {
        // 检查是否启用搜索功能
        if !self.config.search.as_ref().map_or(true, |s| s.enable) {
            return Ok(());
        }
        
        // 创建搜索索引生成器
        let generator = SearchIndexGenerator::new(self.config.search.as_ref().map_or(false, |s| s.content));
        
        // 生成搜索索引
        generator.generate(posts, &self.output_dir)?;
        
        // 复制搜索页面和脚本
        self.generate_search_page()?;
        
        Ok(())
    }
    
    /// 生成搜索页面
    fn generate_search_page(&self) -> Result<()> {
        let mut content = String::with_capacity(4096);
        self.write_html_header(&mut content, "Search")?;

        writeln!(&mut content, "    <div class=\"search-container\">")?;
        writeln!(&mut content, "        <div class=\"search-box\">")?;
        writeln!(&mut content, "            <input type=\"text\" id=\"search-input\" placeholder=\"Search...\">")?;
        writeln!(&mut content, "        </div>")?;
        writeln!(&mut content, "        <div id=\"search-results\"></div>")?;
        writeln!(&mut content, "    </div>")?;

        writeln!(&mut content, "    <script>")?;
        writeln!(&mut content, "        const searchIndex = {{}};")?;
        writeln!(&mut content, "        document.getElementById('search-input').addEventListener('input', function(e) {{")?;
        writeln!(&mut content, "            const query = e.target.value.toLowerCase();")?;
        writeln!(&mut content, "            const results = Object.entries(searchIndex)")?;
        writeln!(&mut content, "                .filter(([_, post]) => post.title.toLowerCase().includes(query) || post.content.toLowerCase().includes(query))")?;
        writeln!(&mut content, "                .map(([_, post]) => `<div class=\"search-result\">")?;
        writeln!(&mut content, "                    <h3><a href=\"${{{{post.url}}}}\">${{{{post.title}}}}</a></h3>")?;
        writeln!(&mut content, "                    <p>${{{{post.excerpt}}}}</p>")?;
        writeln!(&mut content, "                    <div class=\"meta\">")?;
        writeln!(&mut content, "                        <span class=\"date\">${{{{post.date}}}}</span>")?;
        writeln!(&mut content, "                        ${{{{post.categories ? `<span class=\"categories\">Categories: ${{{{post.categories.join(', ')}}}}` : ''}}}}")?;
        writeln!(&mut content, "                        ${{{{post.tags ? `<span class=\"tags\">Tags: ${{{{post.tags.join(', ')}}}}` : ''}}}}")?;
        writeln!(&mut content, "                    </div>")?;
        writeln!(&mut content, "                </div>`)")?;
        writeln!(&mut content, "                .join('');")?;
        writeln!(&mut content, "            document.getElementById('search-results').innerHTML = results;")?;
        writeln!(&mut content, "        }});")?;
        writeln!(&mut content, "    </script>")?;

        self.write_html_footer(&mut content)?;

        Ok(())
    }
}

/// 辅助函数：统计字数（中英文混合）
fn count_words(content: &str) -> usize {
    let mut count = 0;
    let mut is_word = false;
    
    // 同时统计中英文
    for c in content.chars() {
        if c.is_ascii_alphanumeric() {
            if !is_word {
                is_word = true;
                count += 1;
            }
        } else if c.is_whitespace() || c.is_ascii_punctuation() {
            is_word = false;
        } else if c.is_alphabetic() {
            // 非ASCII字符（如中文、日文等）每个字符算一个字
            count += 1;
        }
    }
    
    count
}

/// 内部测试函数
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_mutability() -> Result<()> {
        let mut content = String::with_capacity(4096);
        
        // 测试字符串格式化
        writeln!(&mut content, "测试字符串: {}", "这是一个测试")?;
        
        // 测试复杂格式化（注意括号是转义的，不是参数）
        writeln!(&mut content, "复杂格式化: {{}} 和 {{}}")?;
        
        // 测试字符串长度是否合理
        assert!(content.len() > 0);
        assert!(content.capacity() >= 4096);
        
        Ok(())
    }
} 
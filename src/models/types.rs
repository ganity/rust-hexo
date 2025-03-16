use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 博客文章的基本结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    /// 文章标题
    pub title: String,
    /// 文章创建时间
    pub date: DateTime<Utc>,
    /// 文章更新时间
    pub updated: Option<DateTime<Utc>>,
    /// 是否允许评论
    pub comments: bool,
    /// 使用的布局
    pub layout: String,
    /// 文章内容（原始Markdown）
    pub content: String,
    /// 渲染后的HTML内容
    pub rendered_content: Option<String>,
    /// 源文件路径
    pub source: PathBuf,
    /// 输出URL路径
    pub path: String,
    /// 永久链接
    pub permalink: String,
    /// 文章摘要
    pub excerpt: Option<String>,
    /// 完整URL
    pub url: Option<String>,
    /// 文章分类
    pub categories: Vec<String>,
    /// 文章标签
    pub tags: Vec<String>,
    /// 自定义前置元数据
    pub front_matter: HashMap<String, serde_yaml::Value>,
}

/// 页面结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// 页面标题
    pub title: String,
    /// 页面创建时间
    pub date: DateTime<Utc>,
    /// 页面更新时间
    pub updated: Option<DateTime<Utc>>,
    /// 是否允许评论
    pub comments: bool,
    /// 使用的布局
    pub layout: String,
    /// 页面内容（原始Markdown）
    pub content: String,
    /// 渲染后的HTML内容
    pub rendered_content: Option<String>,
    /// 源文件路径
    pub source: PathBuf,
    /// 输出URL路径
    pub path: String,
    /// 永久链接
    pub permalink: String,
    /// 自定义前置元数据
    pub front_matter: HashMap<String, serde_yaml::Value>,
}

/// 分类结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// 分类名称
    pub name: String,
    /// 分类别名（用于URL）
    pub slug: String,
    /// 分类路径
    pub path: String,
    /// 父分类
    pub parent: Option<String>,
    /// 该分类下的文章数量
    pub post_count: usize,
}

/// 标签结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// 标签名称
    pub name: String,
    /// 标签别名（用于URL）
    pub slug: String,
    /// 标签路径
    pub path: String,
    /// 该标签下的文章数量
    pub post_count: usize,
}

/// 站点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// 站点标题
    pub title: String,
    /// 站点副标题
    pub subtitle: Option<String>,
    /// 站点描述
    pub description: Option<String>,
    /// 站点作者
    pub author: Option<String>,
    /// 站点语言
    pub language: String,
    /// 时区
    pub timezone: Option<String>,
    /// 网站URL
    pub url: String,
    /// 网站根目录
    pub root: String,
    /// 每页文章数
    pub per_page: usize,
    /// 主题
    pub theme: String,
    /// 部署配置
    pub deploy: Option<HashMap<String, serde_yaml::Value>>,
    /// 主题配置
    pub theme_config: Option<HashMap<String, serde_yaml::Value>>,
    /// 评论系统配置
    pub comments: Option<CommentsConfig>,
    /// 搜索配置
    pub search: Option<SearchConfig>,
    /// 其他配置
    #[serde(default)]
    pub extras: HashMap<String, serde_yaml::Value>,
}

/// 评论系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsConfig {
    /// 是否启用评论
    pub enable: bool,
    /// 评论系统类型
    pub system: String,
    /// Giscus 配置
    pub giscus: Option<GiscusConfig>,
    /// Disqus 配置
    pub disqus: Option<DisqusConfig>,
}

/// Giscus 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiscusConfig {
    pub repo_owner: String,
    pub repo: String,
    pub repo_id: String,
    pub category: String,
    pub category_id: String,
    pub mapping: String,
    pub strict: bool,
    pub reactions_enabled: bool,
    pub theme: String,
    pub lang: String,
}

/// Disqus 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisqusConfig {
    pub shortname: String,
}

/// 搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// 是否启用搜索
    pub enable: bool,
    /// 是否使用全文搜索
    pub full_content: bool,
}

impl Default for SiteConfig {
    fn default() -> Self {
        SiteConfig {
            title: "Rust Hexo Site".to_string(),
            subtitle: None,
            description: None,
            author: None,
            language: "en".to_string(),
            timezone: None,
            url: "http://localhost:4000".to_string(),
            root: "/".to_string(),
            per_page: 10,
            theme: "landscape".to_string(),
            deploy: None,
            theme_config: None,
            comments: None,
            search: None,
            extras: HashMap::new(),
        }
    }
}

/// 本地上下文，用于模板渲染
#[derive(Debug, Clone, Serialize)]
pub struct Locals {
    /// 当前页面信息
    pub page: HashMap<String, serde_yaml::Value>,
    /// 当前路径
    pub path: String,
    /// 完整URL
    pub url: String,
    /// 站点配置
    pub config: SiteConfig,
    /// 主题配置
    pub theme: HashMap<String, serde_yaml::Value>,
    /// 使用的布局
    pub layout: String,
    /// 站点信息
    pub site: SiteLocals,
}

/// 站点信息，用于模板渲染
#[derive(Debug, Clone, Serialize)]
pub struct SiteLocals {
    /// 所有文章
    pub posts: Vec<Post>,
    /// 所有页面
    pub pages: Vec<Page>,
    /// 所有分类
    pub categories: Vec<Category>,
    /// 所有标签
    pub tags: Vec<Tag>,
}

/// 渲染数据
#[derive(Debug, Clone)]
pub struct RenderData {
    /// 使用的引擎
    pub engine: Option<String>,
    /// 内容
    pub content: Option<String>,
    /// 源文件路径
    pub source: Option<PathBuf>,
    /// 是否首字母大写标题
    pub titlecase: bool,
}

/// 生成器返回结果
#[derive(Debug, Clone)]
pub struct GeneratorResult {
    /// 路径
    pub path: String,
    /// 数据
    pub data: HashMap<String, serde_yaml::Value>,
    /// 使用的布局
    pub layout: Option<Vec<String>>,
} 
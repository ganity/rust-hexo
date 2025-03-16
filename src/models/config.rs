use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub url: Option<String>,
    pub root: Option<String>,
    pub permalink: Option<String>,
    pub source_dir: Option<String>,
    pub public_dir: Option<String>,
    pub tag_dir: Option<String>,
    pub archive_dir: Option<String>,
    pub category_dir: Option<String>,
    pub code_dir: Option<String>,
    pub i18n_dir: Option<String>,
    pub skip_render: Option<Vec<String>>,
    pub new_post_name: Option<String>,
    pub default_layout: Option<String>,
    pub titlecase: Option<bool>,
    pub external_link: Option<bool>,
    pub filename_case: Option<i32>,
    pub render_drafts: Option<bool>,
    pub post_asset_folder: Option<bool>,
    pub relative_link: Option<bool>,
    pub future: Option<bool>,
    pub highlight: Option<HighlightConfig>,
    pub default_category: Option<String>,
    pub category_map: Option<HashMap<String, String>>,
    pub tag_map: Option<HashMap<String, String>>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    pub per_page: Option<i32>,
    pub pagination_dir: Option<String>,
    pub theme: Option<String>,
    pub theme_config: Option<serde_yaml::Value>,
    pub deploy: Option<DeployConfig>,
    pub markdown: Option<MarkdownConfig>,
    pub feed: Option<FeedConfig>,
    pub search: Option<SearchConfig>,
    pub plugins: Option<Vec<String>>,
    pub comments: Option<CommentsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightConfig {
    pub enable: bool,
    pub line_number: bool,
    pub auto_detect: bool,
    pub tab_replace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownConfig {
    pub render_html: bool,
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub type_: String,
    pub repo: String,
    pub branch: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfig {
    pub enable: bool,
    pub rss: bool,
    pub atom: bool,
    pub path: Option<String>,
    pub limit: usize,
    pub content_type: String,
    pub order_by: String,
    pub icon: Option<String>,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub enable: bool,
    pub path: String,
    pub field: String,
    pub content: bool,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsConfig {
    pub enable: bool,
    pub provider: String,
    pub repo_id: String,
    pub category_id: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "My Blog".to_string(),
            subtitle: None,
            description: None,
            author: None,
            language: Some("en".to_string()),
            timezone: None,
            url: None,
            root: Some("/".to_string()),
            permalink: None,
            source_dir: None,
            public_dir: None,
            tag_dir: None,
            archive_dir: None,
            category_dir: None,
            code_dir: None,
            i18n_dir: None,
            skip_render: None,
            new_post_name: None,
            default_layout: None,
            titlecase: None,
            external_link: None,
            filename_case: None,
            render_drafts: None,
            post_asset_folder: None,
            relative_link: None,
            future: None,
            highlight: None,
            default_category: None,
            category_map: None,
            tag_map: None,
            date_format: None,
            time_format: None,
            per_page: None,
            pagination_dir: None,
            theme: Some("default".to_string()),
            theme_config: None,
            deploy: None,
            markdown: None,
            feed: None,
            search: None,
            plugins: None,
            comments: None,
        }
    }
}

impl Config {
    /// 从文件加载配置
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    
    /// 加载配置的别名
    pub fn load(path: &Path) -> Result<Self> {
        Self::from_file(path)
    }
    
    /// 保存配置到文件
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(path, yaml)?;
        Ok(())
    }
    
    /// 保存配置的别名
    pub fn save(&self, path: &Path) -> Result<()> {
        self.save_to_file(path)
    }
} 
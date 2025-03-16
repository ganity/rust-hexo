use crate::models::Post;
use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;
use tracing::info;

/// 搜索索引项
#[derive(Debug, Serialize)]
pub struct SearchIndexItem {
    /// 文章标题
    pub title: String,
    /// 文章路径
    pub path: String,
    /// 文章内容（摘要或全文）
    pub content: String,
    /// 发布日期
    pub date: String,
    /// 分类
    pub categories: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
}

/// 搜索索引生成器
pub struct SearchIndexGenerator {
    /// 是否使用全文
    use_full_content: bool,
}

impl SearchIndexGenerator {
    /// 创建新的搜索索引生成器
    pub fn new(use_full_content: bool) -> Self {
        Self { use_full_content }
    }
    
    /// 生成搜索索引
    pub fn generate(&self, posts: &[Post], output_dir: &Path) -> Result<()> {
        info!("Generating search index...");
        
        let mut index = Vec::new();
        
        for post in posts {
            let content = if self.use_full_content {
                post.content.clone()
            } else {
                post.excerpt.clone().unwrap_or_else(|| {
                    // 如果没有摘要，使用内容的前150个字符
                    let content = &post.content;
                    if content.len() > 150 {
                        if let Some(idx) = content.char_indices().nth(150).map(|(i, _)| i) {
                            format!("{}...", &content[..idx])
                        } else {
                            content.clone()
                        }
                    } else {
                        content.clone()
                    }
                })
            };
            
            // 创建索引项
            let item = SearchIndexItem {
                title: post.title.clone(),
                path: post.path.clone(),
                content,
                date: post.date.format("%Y-%m-%d").to_string(),
                categories: post.categories.clone(),
                tags: post.tags.clone(),
            };
            
            index.push(item);
        }
        
        // 将索引写入文件
        let search_dir = output_dir.join("search");
        fs::create_dir_all(&search_dir)?;
        let index_json = serde_json::to_string(&index)?;
        fs::write(search_dir.join("search.json"), index_json)?;
        
        info!("Search index generated successfully");
        Ok(())
    }
} 
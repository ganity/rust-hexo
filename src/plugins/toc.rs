use super::{Plugin, PluginContext, PluginHook};
use anyhow::Result;
use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// TOC 插件配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TocConfig {
    /// 最大标题层级
    pub max_level: u8,
    /// 是否为标题添加锚点
    pub add_anchor: bool,
    /// 是否在文章开头自动添加目录
    pub auto_insert: bool,
    /// 目录占位符
    pub placeholder: String,
}

impl Default for TocConfig {
    fn default() -> Self {
        Self {
            max_level: 3,
            add_anchor: true,
            auto_insert: true,
            placeholder: "<!-- toc -->".to_string(),
        }
    }
}

/// TOC 插件
pub struct TocPlugin {
    name: String,
    version: String,
    description: String,
    author: String,
    config: TocConfig,
}

impl TocPlugin {
    pub fn new() -> Self {
        Self {
            name: "toc".to_string(),
            version: "0.1.0".to_string(),
            description: "自动生成文章目录".to_string(),
            author: "Rust-Hexo Team".to_string(),
            config: TocConfig::default(),
        }
    }
    
    /// 生成目录
    fn generate_toc(&self, content: &str) -> String {
        let mut headers = Vec::new();
        let mut current_level = 0;
        let parser = Parser::new(content);
        
        // 提取标题
        for event in parser {
            if let Event::Start(Tag::Heading(level)) = event {
                let level_num = level as u8;
                if level_num <= self.config.max_level {
                    current_level = level_num;
                }
            } else if let Event::Text(text) = event {
                if current_level > 0 && current_level <= self.config.max_level {
                    headers.push((current_level, text.to_string()));
                    current_level = 0;
                }
            }
        }
        
        // 生成目录 HTML
        let mut toc = String::from("<div class=\"toc\">\n<ul>\n");
        let mut current_level = 1;
        
        for (level, text) in headers {
            let id = slugify(&text);
            
            while current_level < level {
                toc.push_str("<ul>\n");
                current_level += 1;
            }
            while current_level > level {
                toc.push_str("</ul>\n");
                current_level -= 1;
            }
            
            toc.push_str(&format!(
                "<li><a href=\"#{}\">{}</a></li>\n",
                id, text
            ));
        }
        
        while current_level > 1 {
            toc.push_str("</ul>\n");
            current_level -= 1;
        }
        toc.push_str("</ul>\n</div>");
        
        toc
    }
    
    /// 为标题添加锚点
    fn add_anchors(&self, content: &str) -> String {
        let mut result = String::new();
        let mut current_level = 0;
        let parser = Parser::new(content);
        
        for event in parser {
            match event {
                Event::Start(Tag::Heading(level)) => {
                    current_level = level as u8;
                    result.push_str("<h");
                    result.push_str(&current_level.to_string());
                }
                Event::Text(text) if current_level > 0 => {
                    let id = slugify(&text);
                    result.push_str(&format!(" id=\"{}\">{}", id, text));
                    current_level = 0;
                }
                Event::End(Tag::Heading(_)) => {
                    result.push_str("</h");
                    result.push_str(&current_level.to_string());
                    result.push('>');
                }
                _ => {
                    // 其他内容保持不变
                    result.push_str(&event.to_string());
                }
            }
        }
        
        result
    }
    
    /// 处理文章内容
    fn process_content(&self, content: &str) -> String {
        let mut processed = content.to_string();
        
        // 生成目录
        let toc = self.generate_toc(content);
        
        // 如果配置了自动插入目录
        if self.config.auto_insert {
            if processed.contains(&self.config.placeholder) {
                // 替换占位符
                processed = processed.replace(&self.config.placeholder, &toc);
            } else {
                // 在文章开头插入目录
                processed = format!("{}\n\n{}", toc, processed);
            }
        }
        
        // 如果配置了添加锚点
        if self.config.add_anchor {
            processed = self.add_anchors(&processed);
        }
        
        processed
    }
}

impl Plugin for TocPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn author(&self) -> &str {
        &self.author
    }
    
    fn init(&mut self, context: &PluginContext) -> Result<()> {
        info!("初始化 TOC 插件");
        
        // 从配置中加载 TOC 设置
        if let Some(config) = context.config.get("toc") {
            if let Ok(toc_config) = serde_yaml::from_value(config.clone()) {
                self.config = toc_config;
                debug!("已加载 TOC 配置");
            }
        }
        
        Ok(())
    }
    
    fn execute_hook(&self, hook: PluginHook, _context: &PluginContext) -> Result<()> {
        match hook {
            PluginHook::BeforeGenerate => {
                info!("TOC 插件: 开始处理文章内容");
                // 在这里处理所有文章的内容
                // 实际实现中需要访问文章列表
            }
            _ => {}
        }
        Ok(())
    }
    
    fn cleanup(&self) -> Result<()> {
        info!("清理 TOC 插件资源");
        Ok(())
    }
}

/// 将文本转换为 URL 友好的 slug
fn slugify(text: &str) -> String {
    let re = Regex::new(r"[^\w\s-]").unwrap();
    let mut slug = re.replace_all(text, "").to_string();
    slug = slug.replace(' ', "-");
    slug.to_lowercase()
} 
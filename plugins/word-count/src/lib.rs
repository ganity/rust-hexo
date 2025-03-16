use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use semver::Version;
use tracing::{debug, info};
use serde_yaml::Value;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json;

/// 字数统计插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WordCountConfig {
    /// 是否显示字数
    show_word_count: bool,
    /// 是否显示阅读时间
    show_read_time: bool,
    /// 每分钟阅读字数
    words_per_minute: u32,
}

impl Default for WordCountConfig {
    fn default() -> Self {
        Self {
            show_word_count: true,
            show_read_time: true,
            words_per_minute: 200,
        }
    }
}

/// 字数统计插件
pub struct WordCountPlugin {
    name: String,
    version: String,
    description: String,
    author: String,
    config: WordCountConfig,
}

impl WordCountPlugin {
    pub fn new() -> Self {
        Self {
            name: "word-count".to_string(),
            version: "0.1.0".to_string(),
            description: "文章字数统计插件".to_string(),
            author: "Rust-Hexo Team".to_string(),
            config: WordCountConfig::default(),
        }
    }

    /// 统计文章字数
    fn count_words(&self, content: &str) -> usize {
        // 统计英文单词
        let english_words = content.split_whitespace().count();
        
        // 统计中文字数
        let chinese_chars = content.chars().filter(|c| c.is_chinese()).count();
        
        english_words + chinese_chars
    }

    /// 计算阅读时间（分钟）
    fn estimate_read_time(&self, word_count: usize) -> f32 {
        (word_count as f32 / self.config.words_per_minute as f32).ceil()
    }
    
    /// 字数统计函数，提供给模板系统
    fn word_count_function(&self, args: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> {
        let content = match args.get("content") {
            Some(val) => match val.as_str() {
                Some(s) => s,
                None => return Err(tera::Error::msg("无法将content转换为字符串")),
            },
            None => return Err(tera::Error::msg("缺少必要的参数: content")),
        };

        // 计算英文单词和中文字符
        let count = self.count_words(content);
        
        Ok(serde_json::Value::Number(serde_json::Number::from(count as i64)))
    }
    
    /// 估计阅读时间函数，提供给模板系统
    fn reading_time_function(&self, args: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> {
        let content = match args.get("content") {
            Some(val) => match val.as_str() {
                Some(s) => s,
                None => return Err(tera::Error::msg("无法将content转换为字符串")),
            },
            None => return Err(tera::Error::msg("缺少必要的参数: content")),
        };

        // 计算字数
        let count = self.count_words(content);
        
        // 阅读速度：每分钟200个单词/字
        let reading_time = self.estimate_read_time(count);
        
        // 确保阅读时间至少为1分钟
        let reading_time = if reading_time < 1.0 { 1.0 } else { reading_time };
        
        // 转换为整数
        Ok(serde_json::Value::Number(serde_json::Number::from(reading_time as i64)))
    }
}

impl Plugin for WordCountPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn init(&mut self, context: &PluginContext) -> Result<()> {
        info!("初始化字数统计插件");
        // 从配置中加载插件配置
        if let Some(config) = &context.config.plugins {
            if let Some(plugin_config) = config.iter().find(|c| c.as_str() == "word-count") {
                debug!("加载字数统计插件配置: {:?}", plugin_config);
                // TODO: 解析插件配置
            }
        }
        Ok(())
    }

    fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
        match hook {
            PluginHook::BeforePostRender => {
                info!("执行BeforePostRender钩子");
            }
            _ => {}
        }
        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        info!("清理字数统计插件");
        Ok(())
    }

    fn process_content(&self, content: &str, _content_type: ContentType) -> Result<String> {
        let word_count = self.count_words(content);
        let read_time = self.estimate_read_time(word_count);
        
        // 在内容末尾添加字数统计和阅读时间信息
        let mut result = content.to_string();
        if self.config.show_word_count || self.config.show_read_time {
            result.push_str("\n\n<div class=\"word-count-info\">");
            if self.config.show_word_count {
                result.push_str(&format!("<span class=\"word-count\">字数统计: {} 字</span>", word_count));
            }
            if self.config.show_read_time {
                if self.config.show_word_count {
                    result.push_str(" | ");
                }
                result.push_str(&format!("<span class=\"reading-time\">阅读时间: {} 分钟</span>", read_time));
            }
            result.push_str("</div>");
        }
        
        Ok(result)
    }
    
    fn get_template_functions(&self) -> HashMap<String, Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>> {
        let mut functions = HashMap::new();
        
        // 创建一个插件的引用用于在闭包中使用
        let plugin = Arc::new(self.clone());
        
        // 添加word_count函数
        let word_count_plugin = Arc::clone(&plugin);
        let word_count_fn = Box::new(move |args: &HashMap<String, serde_json::Value>| {
            word_count_plugin.word_count_function(args)
        }) as Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>;
        
        functions.insert("word_count".to_string(), word_count_fn);
        
        // 添加reading_time函数
        let reading_time_plugin = Arc::clone(&plugin);
        let reading_time_fn = Box::new(move |args: &HashMap<String, serde_json::Value>| {
            reading_time_plugin.reading_time_function(args)
        }) as Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>;
        
        functions.insert("reading_time".to_string(), reading_time_fn);
        
        functions
    }
}

// 为了能够在闭包中使用，需要实现Clone
impl Clone for WordCountPlugin {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            config: self.config.clone(),
        }
    }
}

/// 判断字符是否是中文
trait ChineseChar {
    fn is_chinese(&self) -> bool;
}

impl ChineseChar for char {
    fn is_chinese(&self) -> bool {
        matches!(self,
            '\u{4E00}'..='\u{9FFF}' | // CJK统一表意
            '\u{3400}'..='\u{4DBF}' | // CJK扩展A
            '\u{20000}'..='\u{2A6DF}' | // CJK扩展B
            '\u{2A700}'..='\u{2B73F}' | // CJK扩展C
            '\u{2B740}'..='\u{2B81F}' | // CJK扩展D
            '\u{2B820}'..='\u{2CEAF}' | // CJK扩展E
            '\u{2CEB0}'..='\u{2EBEF}' | // CJK扩展F
            '\u{30000}'..='\u{3134F}' // CJK扩展G
        )
    }
}

/// 创建插件实例
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(WordCountPlugin::new())
}
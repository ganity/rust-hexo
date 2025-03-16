use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};
use tracing::{debug, warn, info, error};
use crate::models::config::Config;
use chrono::{DateTime, Utc};
use pulldown_cmark::{html, Options, Parser};
use crate::plugins::PluginManager;

#[derive(Clone)]
pub struct ThemeRenderer {
    /// 主题目录
    pub theme_dir: PathBuf,
    /// 模板引擎
    pub tera: Tera,
    /// 主题配置
    pub config: Config,
    /// 插件管理器
    pub plugin_manager: Option<PluginManager>,
}

impl ThemeRenderer {
    /// 创建新的主题渲染器
    pub fn new(base_dir: &Path, config: Config) -> Result<Self> {
        let theme = config.theme.as_ref().unwrap_or(&"default".to_string()).clone();
        let theme_dir = base_dir.join("themes").join(&theme);
        
        if !theme_dir.exists() {
            return Err(anyhow!("主题目录不存在: {}", theme_dir.display()));
        }
        
        let mut tera = Tera::new(&format!("{}/**/*.html", theme_dir.join("layout").display()))?;
        
        // 注册过滤器和函数
        Self::register_filters(&mut tera);
        Self::register_functions(&mut tera);
        
        Ok(ThemeRenderer {
            theme_dir,
            tera,
            config,
            plugin_manager: None,
        })
    }
    
    /// 设置插件管理器
    pub fn set_plugin_manager(&mut self, plugin_manager: PluginManager) {
        self.plugin_manager = Some(plugin_manager);
    }
    
    /// 注册模板过滤器
    fn register_filters(tera: &mut Tera) {
        // 注册日期格式化过滤器
        tera.register_filter("date_format", Self::date_format_filter);
        // 注册Markdown过滤器
        tera.register_filter("markdown", Self::markdown_filter);
        // 其他过滤器...
    }
    
    /// 注册模板函数
    fn register_functions(tera: &mut Tera) {
        // 注册URL生成函数
        tera.register_function("url_for", Self::url_for_function);
        
        // 注册字数统计函数
        // tera.register_function("word_count", Self::word_count_function);
        
        // 注册阅读时间函数
        // tera.register_function("reading_time", Self::reading_time_function);
        
        // 其他函数...
    }
    
    /// 渲染模板
    pub fn render(&self, template: &str, context: &HashMap<String, serde_yaml::Value>) -> Result<String> {
        let mut tera_context = tera::Context::new();
        
        // 将YAML值转换为Tera值
        for (key, value) in context {
            tera_context.insert(key, &tera_yaml_to_value(value)?);
        }
        
        // 添加插件资源
        if let Some(ref plugin_manager) = self.plugin_manager {
            debug!("检查插件管理器...");
            let plugins = match plugin_manager.get_all_plugins() {
                Ok(p) => {
                    info!("获取到 {} 个插件", p.len());
                    p
                },
                Err(e) => {
                    warn!("获取插件列表失败: {}", e);
                    Vec::new()
                }
            };
            
            // 收集所有插件的资源
            let mut head_resources = Vec::new();
            let mut footer_resources = Vec::new();
            for plugin in &plugins {
                debug!("处理插件 {} 的资源", plugin.name());
                // 获取所有资源
                for resource in plugin.get_resources() {
                    match resource.1 {
                        crate::plugins::ResourceLocation::Head => head_resources.push(resource),
                        crate::plugins::ResourceLocation::Footer => footer_resources.push(resource),
                    }
                }
            }
            tera_context.insert("plugin_head_resources", &head_resources);
            tera_context.insert("plugin_footer_resources", &footer_resources);
            
            // 为特定插件添加标志
            let mut plugin_enabled = HashMap::new();
            for plugin in &plugins {
                let name = plugin.name();
                info!("启用插件 {} 在模板中", name);
                // 使用插件名作为键（统一使用中横线格式，与配置名保持一致）
                let config_name = if name.contains('_') {
                    name.replace('_', "-")
                } else {
                    name.to_string()
                };
                plugin_enabled.insert(config_name, true);
            }
            
            if !plugin_enabled.is_empty() {
                info!("向模板上下文添加 {} 个插件", plugin_enabled.len());
                tera_context.insert("plugins", &plugin_enabled);
            } else {
                warn!("没有插件被添加到模板上下文");
            }
        } else {
            warn!("没有可用的插件管理器");
        }
        
        // 渲染模板
        match self.tera.render(template, &tera_context) {
            Ok(result) => Ok(result),
            Err(e) => {
                error!("模板渲染失败: {}", e);
                Err(anyhow!(e))
            }
        }
    }
    
    /// 获取可用的布局列表
    pub fn available_layouts(&self) -> Vec<String> {
        self.tera.get_template_names().map(String::from).collect()
    }
    
    /// 检查布局是否存在
    pub fn has_layout(&self, layout: &str) -> bool {
        self.tera.get_template_names().any(|name| name == layout)
    }
    
    /// 获取主题资源目录
    pub fn source_dir(&self) -> PathBuf {
        self.theme_dir.join("source")
    }
    
    /// 从模板引擎中重新加载模板
    pub fn reload_templates(&mut self) -> Result<()> {
        debug!("Reloading theme templates...");
        self.tera.full_reload()?;
        Ok(())
    }

    pub fn render_template(&self, template_name: &str, context: &tera::Context) -> Result<String> {
        Ok(self.tera.render(template_name, context)?)
    }

    fn date_format_filter(value: &tera::Value, args: &std::collections::HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        if let Some(date) = value.as_str().and_then(|s| DateTime::parse_from_rfc3339(s).ok()) {
            let format = args.get("format")
                .and_then(|f| f.as_str())
                .unwrap_or("%Y-%m-%d");
            Ok(tera::Value::String(date.format(format).to_string()))
        } else {
            Ok(value.clone())
        }
    }

    fn markdown_filter(value: &tera::Value, _args: &std::collections::HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        if let Some(text) = value.as_str() {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_FOOTNOTES);
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TASKLISTS);
            // 不需要ENABLE_FENCED_CODE_BLOCKS，pulldown-cmark默认支持代码块
            // 添加其他有用选项
            options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
            options.insert(Options::ENABLE_SMART_PUNCTUATION);

            let parser = Parser::new_ext(text, options);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);

            // 处理代码块样式问题
            let html_output = if html_output.contains("<pre><code") {
                // 确保代码块有正确的语言标记
                html_output.replace("<pre><code", "<pre><code class=\"hljs\"")
            } else {
                html_output
            };

            Ok(tera::Value::String(html_output))
        } else {
            Ok(value.clone())
        }
    }

    fn url_for_function(_args: &std::collections::HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        // TODO: 实现URL生成功能
        Ok(tera::Value::String("/".to_string()))
    }

    /// 计算内容的字数
    fn word_count_function(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let content = match args.get("content") {
            Some(val) => match val.as_str() {
                Some(s) => s,
                None => return Err(tera::Error::msg("无法将content转换为字符串")),
            },
            None => return Err(tera::Error::msg("缺少必要的参数: content")),
        };

        // 计算英文单词和中文字符
        let english_words = content.split_whitespace().count();
        // 中文字符通常是Unicode中的CJK字符
        let chinese_chars = content.chars().filter(|c| {
            let cp = *c as u32;
            (0x4E00..=0x9FFF).contains(&cp) || // CJK统一汉字
            (0x3400..=0x4DBF).contains(&cp) || // CJK扩展A
            (0xF900..=0xFAFF).contains(&cp)    // CJK兼容汉字
        }).count();

        let total_count = english_words + chinese_chars;
        Ok(tera::Value::Number(serde_json::Number::from(total_count as i64)))
    }

    /// 估计内容的阅读时间（分钟）
    fn reading_time_function(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let content = match args.get("content") {
            Some(val) => match val.as_str() {
                Some(s) => s,
                None => return Err(tera::Error::msg("无法将content转换为字符串")),
            },
            None => return Err(tera::Error::msg("缺少必要的参数: content")),
        };

        // 计算字数
        let english_words = content.split_whitespace().count();
        let chinese_chars = content.chars().filter(|c| {
            let cp = *c as u32;
            (0x4E00..=0x9FFF).contains(&cp) || // CJK统一汉字
            (0x3400..=0x4DBF).contains(&cp) || // CJK扩展A
            (0xF900..=0xFAFF).contains(&cp)    // CJK兼容汉字
        }).count();

        let total_count = english_words + chinese_chars;
        
        // 阅读速度：每分钟200个单词/字
        let reading_time = (total_count as f64 / 200.0).ceil();
        
        // 确保阅读时间至少为1分钟
        let reading_time = if reading_time < 1.0 { 1.0 } else { reading_time };
        
        // 使用from_f64方法或转换为整数
        match serde_json::Number::from_f64(reading_time) {
            Some(num) => Ok(tera::Value::Number(num)),
            None => Ok(tera::Value::Number(serde_json::Number::from(reading_time as u64)))
        }
    }
}

/// 将YAML值转换为Tera值
fn tera_yaml_to_value(yaml: &serde_yaml::Value) -> Result<tera::Value> {
    Ok(match yaml {
        serde_yaml::Value::Null => tera::Value::Null,
        serde_yaml::Value::Bool(b) => tera::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                tera::Value::Number(serde_json::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                tera::Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| serde_json::Number::from(0)))
            } else {
                tera::Value::Null
            }
        },
        serde_yaml::Value::String(s) => tera::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            let values: Vec<tera::Value> = seq.iter()
                .map(tera_yaml_to_value)
                .collect::<Result<Vec<_>>>()?;
            tera::Value::Array(values)
        },
        serde_yaml::Value::Mapping(map) => {
            let mut object = serde_json::Map::new();
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    object.insert(key.clone(), tera_yaml_to_value(v)?);
                }
            }
            tera::Value::Object(object)
        },
        serde_yaml::Value::Tagged(tagged) => {
            // 处理带标签的值，直接使用内部值
            tera_yaml_to_value(&tagged.value)?
        },
    })
} 
# Rust-Hexo 插件开发指南

## 目录

1. [插件系统概述](#插件系统概述)
2. [开发环境准备](#开发环境准备)
3. [创建插件项目](#创建插件项目)
4. [实现Plugin接口](#实现plugin接口)
5. [插件生命周期](#插件生命周期)
6. [内容处理](#内容处理)
7. [钩子函数](#钩子函数)
8. [配置读取](#配置读取)
9. [模板函数](#模板函数)
10. [实例：字数统计插件](#实例字数统计插件)
11. [实例：代码高亮插件](#实例代码高亮插件)
12. [插件打包与发布](#插件打包与发布)
13. [调试技巧](#调试技巧)
14. [常见问题](#常见问题)

## 插件系统概述

Rust-Hexo的插件系统基于动态库加载机制，允许开发者通过实现标准的Plugin接口来扩展博客系统的功能。插件可以：

- 处理内容（如Markdown转HTML、代码高亮等）
- 响应系统事件（如文章渲染前后）
- 添加新的模板函数
- 自定义资源注入

插件在编译后作为动态库（`.so`、`.dll`或`.dylib`）加载到主程序中，实现了与主程序的解耦，使功能扩展更加灵活。

## 开发环境准备

开发Rust-Hexo插件前，请确保已安装：

- Rust工具链（rustc, cargo）最低版本1.70+
- Rust-Hexo本身，用于测试

## 创建插件项目

### 1. 创建插件的Cargo项目

```bash
cargo new --lib my-rust-hexo-plugin
cd my-rust-hexo-plugin
```

### 2. 配置Cargo.toml

```toml
[package]
name = "my-rust-hexo-plugin"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "一个示例Rust-Hexo插件"

[lib]
crate-type = ["cdylib"]  # 编译为动态库

[dependencies]
rust-hexo = { path = "/path/to/rust-hexo" }  # 本地开发时
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
tracing = "0.1"
```

### 3. 基本目录结构

```
my-rust-hexo-plugin/
├── Cargo.toml            # 项目配置文件
└── src/
    └── lib.rs            # 插件主代码
```

## 实现Plugin接口

所有Rust-Hexo插件都必须实现`Plugin` trait。以下是一个基本模板：

```rust
use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use anyhow::Result;
use tracing::info;

pub struct MyPlugin {
    name: String,
    version: String,
    description: String,
    // 插件配置和状态
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "我的第一个Rust-Hexo插件".to_string(),
            // 初始化其他字段
        }
    }
}

impl Plugin for MyPlugin {
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
        info!("初始化插件: {}", self.name);
        // 从配置中读取设置
        Ok(())
    }

    fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
        // 响应不同的系统事件
        match hook {
            PluginHook::BeforePostRender => {
                info!("文章渲染前");
            }
            _ => {} // 处理其他事件
        }
        Ok(())
    }

    fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
        // 处理内容
        Ok(content.to_string()) // 默认不做修改
    }

    fn cleanup(&self) -> Result<()> {
        info!("清理插件资源");
        Ok(())
    }
}

// 必须导出此函数，用于创建插件实例
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(MyPlugin::new())
}
```

## 插件生命周期

插件的生命周期包含以下阶段：

1. **加载**：系统启动时加载动态库并实例化插件
2. **初始化**：调用`init()`方法，读取配置
3. **运行时**：调用`execute_hook()`响应事件，调用`process_content()`处理内容
4. **清理**：系统关闭前调用`cleanup()`释放资源

完整的`Plugin` trait定义如下：

```rust
pub trait Plugin: Send + Sync {
    /// 获取插件名称
    fn name(&self) -> &str;
    
    /// 获取插件版本
    fn version(&self) -> &str;
    
    /// 获取插件描述
    fn description(&self) -> &str;
    
    /// 初始化插件
    fn init(&mut self, context: &PluginContext) -> Result<()>;
    
    /// 执行钩子
    fn execute_hook(&self, hook: &PluginHook) -> Result<()>;
    
    /// 处理内容
    fn process_content(&self, content: &str, content_type: ContentType) -> Result<String>;
    
    /// 获取资源 (可选实现)
    fn get_resources(&self) -> Vec<(String, ResourceLocation)> {
        Vec::new() // 默认实现返回空列表
    }
    
    /// 获取插件提供的模板函数 (可选实现)
    fn get_template_functions(&self) -> HashMap<String, Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>> {
        HashMap::new() // 默认实现返回空HashMap
    }
    
    /// 清理资源
    fn cleanup(&self) -> Result<()>;
}
```

## 内容处理

`process_content()` 方法允许插件处理内容，例如：

```rust
fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
    match content_type {
        ContentType::Markdown => {
            // 处理Markdown内容
            let processed = self.process_markdown(content);
            Ok(processed)
        }
        ContentType::HTML => {
            // 处理HTML内容
            let processed = self.process_html(content);
            Ok(processed)
        }
        _ => Ok(content.to_string()), // 其他类型默认不处理
    }
}

fn process_markdown(&self, content: &str) -> String {
    // 自定义Markdown处理逻辑
    content.replace("{{time}}", &chrono::Local::now().to_string())
}
```

内容类型通过`ContentType`枚举定义：

```rust
pub enum ContentType {
    Markdown,
    HTML,
    JSON,
    YAML,
    CSS,
    JavaScript,
    Plain,
}
```

## 钩子函数

`execute_hook()` 方法响应系统事件：

```rust
fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
    match hook {
        PluginHook::BeforePostRender => {
            // 在文章渲染前执行
        }
        PluginHook::AfterPostRender => {
            // 在文章渲染后执行
        }
        PluginHook::BeforeGenerate => {
            // 在站点生成前执行
        }
        PluginHook::AfterGenerate => {
            // 在站点生成后执行
        }
        _ => {}
    }
    Ok(())
}
```

Rust-Hexo支持的钩子包括：

- `Init`: 初始化
- `BeforeGenerate`: 生成前
- `AfterGenerate`: 生成后
- `BeforeDeploy`: 部署前
- `AfterDeploy`: 部署后
- `NewPost`: 新建文章
- `NewPage`: 新建页面
- `Clean`: 清理
- `ConfigChanged`: 配置变更
- `BeforePostRender`: 文章渲染前
- `AfterPostRender`: 文章渲染后
- `BeforePageRender`: 页面渲染前
- `AfterPageRender`: 页面渲染后
- 等更多钩子函数

## 配置读取

在`init()`方法中，可以从`PluginContext`读取配置：

```rust
fn init(&mut self, context: &PluginContext) -> Result<()> {
    // 检查插件是否启用
    if let Some(plugins) = &context.config.plugins {
        if plugins.contains(&"my-plugin".to_string()) {
            // 获取插件配置
            if let Some(config) = context.config.theme_config.as_ref() {
                if let Some(plugin_config) = config.get("my-plugin") {
                    // 反序列化插件配置
                    if let Ok(config) = serde_yaml::from_value(plugin_config.clone()) {
                        self.config = config;
                        tracing::debug!("已加载插件配置");
                    }
                }
            }
        }
    }
    Ok(())
}
```

对应的配置示例:

```yaml
# 在_config.yml中
plugins:
  - my-plugin

my-plugin:
  enabled: true
  option1: value1
  option2: value2
```

## 模板函数

插件可以为主题提供自定义的模板函数，在Tera模板中使用：

```rust
fn get_template_functions(&self) -> HashMap<String, Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>> {
    let mut functions = HashMap::new();
    
    // 创建一个插件的引用用于在闭包中使用
    let plugin = Arc::new(self.clone());
    
    // 添加自定义函数
    let plugin_clone = Arc::clone(&plugin);
    let my_function = Box::new(move |args: &HashMap<String, serde_json::Value>| {
        // 使用args参数执行操作并返回结果
        Ok(serde_json::Value::String("Hello from plugin!".to_string()))
    }) as Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>;
    
    functions.insert("my_function".to_string(), my_function);
    
    functions
}
```

在Tera模板中使用自定义函数：

```html
{{ my_function() }}
```

注意：如果要在闭包中使用插件实例，需要实现`Clone` trait：

```rust
impl Clone for MyPlugin {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            // 克隆其他字段
        }
    }
}
```

## 实例：字数统计插件

以下是一个完整的字数统计插件示例：

```rust
use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
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
    config: WordCountConfig,
}

impl WordCountPlugin {
    pub fn new() -> Self {
        Self {
            name: "word-count".to_string(),
            version: "0.1.0".to_string(),
            description: "文章字数统计插件".to_string(),
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
        if let Some(plugins) = &context.config.plugins {
            if plugins.contains(&"word-count".to_string()) {
                if let Some(config) = context.config.theme_config.as_ref() {
                    if let Some(word_count_config) = config.get("word_count") {
                        if let Ok(config) = serde_yaml::from_value(word_count_config.clone()) {
                            self.config = config;
                            debug!("已加载字数统计插件配置");
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
        match hook {
            PluginHook::BeforePostRender => {
                info!("处理文章字数统计");
            }
            _ => {}
        }
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
    
    fn cleanup(&self) -> Result<()> {
        info!("清理字数统计插件");
        Ok(())
    }
}

// 为了能够在闭包中使用，需要实现Clone
impl Clone for WordCountPlugin {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
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

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(WordCountPlugin::new())
}
```

## 实例：代码高亮插件

以下是代码高亮插件的简化示例：

```rust
use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use anyhow::Result;
use tracing::info;
use regex::Regex;

pub struct SyntaxHighlightPlugin {
    name: String,
    version: String,
    description: String,
}

impl SyntaxHighlightPlugin {
    pub fn new() -> Self {
        Self {
            name: "syntax-highlight".to_string(),
            version: "0.1.0".to_string(),
            description: "代码高亮插件".to_string(),
        }
    }
    
    /// 处理代码块
    fn process_code_blocks(&self, content: &str) -> String {
        let code_block_regex = Regex::new(r"```(\w*)\n([\s\S]*?)```").unwrap();
        
        let result = code_block_regex.replace_all(content, |caps: &regex::Captures| {
            let language = &caps[1];
            let code = &caps[2];
            
            // 生成语言标签和高亮代码
            let lang_class = if language.is_empty() { "text" } else { language };
            
            format!(
                r#"<div class="code-block">
<div class="code-language">{}</div>
<pre><code class="hljs {}">{}</code></pre>
</div>"#,
                lang_class, lang_class, self.escape_html(code)
            )
        });
        
        result.to_string()
    }
    
    /// 转义HTML特殊字符
    fn escape_html(&self, text: &str) -> String {
        text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#39;")
    }
}

impl Plugin for SyntaxHighlightPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn init(&mut self, _context: &PluginContext) -> Result<()> {
        info!("初始化代码高亮插件");
        Ok(())
    }

    fn execute_hook(&self, _hook: &PluginHook) -> Result<()> {
        Ok(())
    }

    fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
        match content_type {
            ContentType::Markdown => {
                // 处理Markdown中的代码块
                let processed = self.process_code_blocks(content);
                Ok(processed)
            }
            _ => Ok(content.to_string()), // 其他类型不处理
        }
    }

    fn cleanup(&self) -> Result<()> {
        info!("清理代码高亮插件");
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(SyntaxHighlightPlugin::new())
}
```

## 插件打包与发布

### 编译插件

```bash
cargo build --release
```

编译后的动态库位于`target/release/`目录下：
- Linux: `libmy_rust_hexo_plugin.so`
- macOS: `libmy_rust_hexo_plugin.dylib`
- Windows: `my_rust_hexo_plugin.dll`

### 安装插件

将编译好的动态库放入Rust-Hexo博客的plugins目录：

```
my-blog/
└── plugins/
    └── my-plugin/
        └── libmy_plugin.so  # 或 .dll 或 .dylib
```

### 启用插件

在博客的`_config.yml`文件中启用插件：

```yaml
plugins:
  - my-plugin

# 插件配置
my-plugin:
  option1: value1
  option2: value2
```

## 调试技巧

### 日志输出

使用`tracing`库进行日志输出：

```rust
use tracing::{debug, info, warn, error};

fn some_function() {
    info!("普通信息");
    debug!("调试信息");
    warn!("警告信息");
    error!("错误信息: {}", "具体错误");
}
```

### 错误处理

推荐使用`anyhow`库进行错误处理：

```rust
use anyhow::{Result, Context, anyhow};

fn function_that_might_fail() -> Result<()> {
    let config = std::fs::read_to_string("config.txt")
        .context("读取配置文件失败")?;
    
    if config.is_empty() {
        return Err(anyhow!("配置文件为空"));
    }
    
    Ok(())
}
```

### 测试插件

创建一个小型的测试博客，然后安装你的插件进行测试：

```bash
# 创建测试博客
rust-hexo init test-blog
cd test-blog

# 安装插件
mkdir -p plugins/my-plugin
cp /path/to/compiled/libmy_plugin.so plugins/my-plugin/

# 启用插件（编辑_config.yml）

# 生成站点并检查
rust-hexo generate
```

## 常见问题

### 1. 插件无法加载

如果插件无法加载，检查以下几点：

- 确认动态库路径正确
- 检查`create_plugin`函数是否正确导出
- 确认插件与Rust-Hexo版本兼容

### 2. 配置无法读取

如果插件配置无法读取，检查：

- 确认`_config.yml`中配置格式正确
- 配置结构体是否与YAML格式匹配
- 确认插件名称在`plugins`列表中

### 3. 模板函数无法使用

如果模板函数无法使用，检查：

- 函数是否正确注册
- 插件是否正确实现`get_template_functions`
- 主题模板中的函数名称是否正确

### 4. 处理多线程安全

如果插件需要处理共享状态，考虑使用：

```rust
use std::sync::{Arc, Mutex, RwLock};

struct MyPlugin {
    // ...
    shared_state: Arc<RwLock<HashMap<String, String>>>,
}
```

---

通过本指南，你应该能够开始开发自己的Rust-Hexo插件。插件系统的灵活性允许你实现各种功能，如内容处理、自定义模板函数、资源注入等。记得查阅Rust-Hexo的API文档获取更多细节，也可以参考内置插件的代码以获取灵感。

祝你开发愉快！如有问题，欢迎在GitHub仓库中提出。 
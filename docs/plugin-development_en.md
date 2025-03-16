# Rust-Hexo Plugin Development Guide

## Table of Contents

1. [Plugin System Overview](#plugin-system-overview)
2. [Development Environment](#development-environment)
3. [Creating a Plugin Project](#creating-a-plugin-project)
4. [Implementing the Plugin Interface](#implementing-the-plugin-interface)
5. [Plugin Lifecycle](#plugin-lifecycle)
6. [Content Processing](#content-processing)
7. [Hook Functions](#hook-functions)
8. [Configuration Reading](#configuration-reading)
9. [Template Functions](#template-functions)
10. [Example: Word Count Plugin](#example-word-count-plugin)
11. [Example: Syntax Highlighting Plugin](#example-syntax-highlighting-plugin)
12. [Packaging and Publishing](#packaging-and-publishing)
13. [Debugging Tips](#debugging-tips)
14. [Common Issues](#common-issues)

## Plugin System Overview

The Rust-Hexo plugin system is based on dynamic library loading, which allows developers to extend the blog system's functionality without modifying the core code. Plugins are shared libraries (.so, .dll, or .dylib files) that implement a standard Plugin interface. This provides a flexible way to add new functionality to Rust-Hexo.

## Development Environment

To develop Rust-Hexo plugins, you'll need:

- Rust toolchain (1.70+)
- Rust-Hexo for testing your plugin

## Creating a Plugin Project

Start by creating a new Cargo project:

```bash
cargo new my-plugin --lib
cd my-plugin
```

Configure your `Cargo.toml` file:

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
rust-hexo = { path = "/path/to/rust-hexo" }  # or use a git repository
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
```

## Implementing the Plugin Interface

Create a plugin by implementing the `Plugin` trait in your `lib.rs` file:

```rust
use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use anyhow::Result;
use tracing::info;

pub struct MyPlugin {
    name: String,
    version: String,
    description: String,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "My first Rust-Hexo plugin".to_string(),
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
        info!("Initializing my plugin");
        // Load configuration, initialize resources, etc.
        Ok(())
    }

    fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
        match hook {
            PluginHook::BeforePostRender => {
                info!("Processing BeforePostRender hook");
                // Execute actions before post rendering
            }
            _ => {}
        }
        Ok(())
    }

    fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
        // Process and transform content
        info!("Processing content of type: {:?}", content_type);
        Ok(content.to_string())  // Return unmodified content for now
    }

    fn cleanup(&self) -> Result<()> {
        info!("Cleaning up my plugin");
        // Clean up resources, close connections, etc.
        Ok(())
    }
}

// Export a function to create the plugin
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(MyPlugin::new()))
}
```

## Plugin Lifecycle

The plugin lifecycle consists of these stages:

1. **Loading**: Rust-Hexo loads your plugin library and calls `create_plugin()` to instantiate your plugin.
2. **Initialization**: The `init` method is called with a `PluginContext` object, which provides access to site configuration and content.
3. **Runtime**: During site generation, Rust-Hexo calls various plugin methods like `execute_hook` and `process_content`.
4. **Cleanup**: When Rust-Hexo finishes, it calls the `cleanup` method to free resources.

## Content Processing

The `process_content` method is where you can transform content before it's written to the output files:

```rust
fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
    match content_type {
        ContentType::Markdown => {
            // Process markdown content
            Ok(self.process_markdown(content))
        }
        ContentType::HTML => {
            // Process HTML content
            Ok(self.process_html(content))
        }
        _ => {
            // Return unmodified content for other types
            Ok(content.to_string())
        }
    }
}

fn process_markdown(&self, content: &str) -> String {
    // Transform markdown content
    content.replace("**bold**", "<strong>bold</strong>")
}

fn process_html(&self, content: &str) -> String {
    // Transform HTML content
    content.replace("<p>", "<p class=\"my-class\">")
}
```

## Hook Functions

The `execute_hook` method allows your plugin to respond to system events. Rust-Hexo supports these hooks:

- `BeforeInit`: Before system initialization
- `AfterInit`: After system initialization
- `BeforePostsLoad`: Before loading posts
- `AfterPostsLoad`: After loading posts
- `BeforePostRender`: Before rendering a post
- `AfterPostRender`: After rendering a post
- `BeforePageRender`: Before rendering a page
- `AfterPageRender`: After rendering a page
- `BeforeGenerate`: Before generating static files
- `AfterGenerate`: After generating static files
- `BeforeExit`: Before the system exits

Example:

```rust
fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
    match hook {
        PluginHook::BeforePostRender => {
            // Do something before each post is rendered
            info!("About to render a post");
        }
        PluginHook::AfterGenerate => {
            // Do something after all static files are generated
            info!("Site generation complete");
        }
        _ => {}
    }
    Ok(())
}
```

## Configuration Reading

During initialization, you can read plugin configuration from `PluginContext`:

```rust
fn init(&mut self, context: &PluginContext) -> Result<()> {
    // Check if this plugin is enabled in config
    if let Some(plugins) = &context.config.plugins {
        if plugins.contains(&"my-plugin".to_string()) {
            // Read plugin-specific config
            if let Some(config) = context.config.theme_config.as_ref() {
                if let Some(plugin_config) = config.get("my-plugin") {
                    // Parse plugin config
                    if let Ok(config) = serde_yaml::from_value(plugin_config.clone()) {
                        self.config = config;
                        info!("Plugin configuration loaded");
                    }
                }
            }
        }
    }
    Ok(())
}
```

## Template Functions

Plugins can provide custom functions for use in Tera templates:

```rust
use std::collections::HashMap;
use std::sync::Arc;

fn get_template_functions(&self) -> HashMap<String, Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>> {
    let mut functions = HashMap::new();
    
    // Create a reference to the plugin for use in the closure
    let plugin = Arc::new(self.clone());
    
    // Add a custom template function
    let plugin_clone = Arc::clone(&plugin);
    let function = Box::new(move |args: &HashMap<String, serde_json::Value>| {
        // Implement your function logic here
        let input = match args.get("input") {
            Some(val) => match val.as_str() {
                Some(s) => s,
                None => return Err(tera::Error::msg("Cannot convert input to string")),
            },
            None => return Err(tera::Error::msg("Missing required argument: input")),
        };
        
        // Process the input
        let result = format!("Processed: {}", input);
        
        Ok(serde_json::Value::String(result))
    }) as Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>;
    
    functions.insert("my_function".to_string(), function);
    
    functions
}

// For closures to use self, the plugin struct needs to implement Clone
impl Clone for MyPlugin {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            // Clone other fields as needed
        }
    }
}
```

## Example: Word Count Plugin

Here's a complete example of a word count plugin:

```rust
use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json;

/// Word count plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WordCountConfig {
    /// Whether to display word count
    show_word_count: bool,
    /// Whether to display reading time
    show_read_time: bool,
    /// Words read per minute
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
            description: "Word count plugin".to_string(),
            config: WordCountConfig::default(),
        }
    }
    
    /// Count words in content
    fn count_words(&self, content: &str) -> usize {
        // Count English words
        let english_words = content.split_whitespace().count();
        
        // Count Chinese characters
        let chinese_chars = content.chars().filter(|c| c.is_chinese()).count();
        
        english_words + chinese_chars
    }

    /// Estimate reading time in minutes
    fn estimate_read_time(&self, word_count: usize) -> f32 {
        (word_count as f32 / self.config.words_per_minute as f32).ceil()
    }
    
    /// Word count function for templates
    fn word_count_function(&self, args: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> {
        let content = match args.get("content") {
            Some(val) => match val.as_str() {
                Some(s) => s,
                None => return Err(tera::Error::msg("Cannot convert content to string")),
            },
            None => return Err(tera::Error::msg("Missing required argument: content")),
        };

        // Count words
        let count = self.count_words(content);
        
        Ok(serde_json::Value::Number(serde_json::Number::from(count as i64)))
    }
    
    /// Reading time function for templates
    fn reading_time_function(&self, args: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> {
        let content = match args.get("content") {
            Some(val) => match val.as_str() {
                Some(s) => s,
                None => return Err(tera::Error::msg("Cannot convert content to string")),
            },
            None => return Err(tera::Error::msg("Missing required argument: content")),
        };

        // Count words
        let count = self.count_words(content);
        
        // Calculate reading time
        let reading_time = self.estimate_read_time(count);
        
        // Ensure reading time is at least 1 minute
        let reading_time = if reading_time < 1.0 { 1.0 } else { reading_time };
        
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
        info!("Initializing word count plugin");
        // Load config from site configuration
        if let Some(plugins) = &context.config.plugins {
            if plugins.contains(&"word-count".to_string()) {
                if let Some(config) = context.config.theme_config.as_ref() {
                    if let Some(plugin_config) = config.get("word_count") {
                        if let Ok(config) = serde_yaml::from_value(plugin_config.clone()) {
                            self.config = config;
                            debug!("Word count plugin configuration loaded");
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
                info!("Adding word count to post");
            }
            _ => {}
        }
        Ok(())
    }

    fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
        if content_type != ContentType::HTML {
            return Ok(content.to_string());
        }
        
        let word_count = self.count_words(content);
        let read_time = self.estimate_read_time(word_count);
        
        // Add word count and reading time information to content
        let mut result = content.to_string();
        if self.config.show_word_count || self.config.show_read_time {
            result.push_str("\n\n<div class=\"word-count-info\">");
            if self.config.show_word_count {
                result.push_str(&format!("<span class=\"word-count\">Word count: {} words</span>", word_count));
            }
            if self.config.show_read_time {
                if self.config.show_word_count {
                    result.push_str(" | ");
                }
                result.push_str(&format!("<span class=\"reading-time\">Reading time: {} minutes</span>", read_time));
            }
            result.push_str("</div>");
        }
        
        Ok(result)
    }
    
    fn get_template_functions(&self) -> HashMap<String, Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>> {
        let mut functions = HashMap::new();
        
        // Create plugin reference for closures
        let plugin = Arc::new(self.clone());
        
        // Add word_count function
        let word_count_plugin = Arc::clone(&plugin);
        let word_count_fn = Box::new(move |args: &HashMap<String, serde_json::Value>| {
            word_count_plugin.word_count_function(args)
        }) as Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>;
        
        functions.insert("word_count".to_string(), word_count_fn);
        
        // Add reading_time function
        let reading_time_plugin = Arc::clone(&plugin);
        let reading_time_fn = Box::new(move |args: &HashMap<String, serde_json::Value>| {
            reading_time_plugin.reading_time_function(args)
        }) as Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>;
        
        functions.insert("reading_time".to_string(), reading_time_fn);
        
        functions
    }
    
    fn cleanup(&self) -> Result<()> {
        info!("Cleaning up word count plugin");
        Ok(())
    }
}

// For template functions to work, the plugin needs to implement Clone
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

/// Check if a character is Chinese
trait ChineseChar {
    fn is_chinese(&self) -> bool;
}

impl ChineseChar for char {
    fn is_chinese(&self) -> bool {
        matches!(self,
            '\u{4E00}'..='\u{9FFF}' | // CJK Unified Ideographs
            '\u{3400}'..='\u{4DBF}' | // CJK Unified Ideographs Extension A
            '\u{20000}'..='\u{2A6DF}' | // CJK Unified Ideographs Extension B
            '\u{2A700}'..='\u{2B73F}' | // CJK Unified Ideographs Extension C
            '\u{2B740}'..='\u{2B81F}' | // CJK Unified Ideographs Extension D
            '\u{2B820}'..='\u{2CEAF}' | // CJK Unified Ideographs Extension E
            '\u{2CEB0}'..='\u{2EBEF}' | // CJK Unified Ideographs Extension F
            '\u{30000}'..='\u{3134F}' // CJK Unified Ideographs Extension G
        )
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(WordCountPlugin::new()))
}
```

## Example: Syntax Highlighting Plugin

Here's a simplified example of a syntax highlighting plugin:

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
            description: "Syntax highlighting plugin".to_string(),
        }
    }
    
    fn process_code_blocks(&self, content: &str) -> String {
        let re = Regex::new(r"<pre><code class=\"language-([^\"]+)\">([^<]+)</code></pre>").unwrap();
        re.replace_all(content, |caps: &regex::Captures| {
            let language = &caps[1];
            let code = html_escape::decode_html_entities(&caps[2]).to_string();
            format!(
                r#"<pre><code class="language-{}">{}
</code></pre>"#,
                language, self.highlight_code(&code, language)
            )
        }).to_string()
    }
    
    fn highlight_code(&self, code: &str, language: &str) -> String {
        // In a real plugin, you would use a syntax highlighting library
        // This is just a simplified example
        format!("/* {} code */\n{}", language, code)
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
        info!("Initializing syntax highlighting plugin");
        Ok(())
    }

    fn execute_hook(&self, _hook: &PluginHook) -> Result<()> {
        Ok(())
    }

    fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
        if content_type == ContentType::HTML {
            Ok(self.process_code_blocks(content))
        } else {
            Ok(content.to_string())
        }
    }

    fn cleanup(&self) -> Result<()> {
        info!("Cleaning up syntax highlighting plugin");
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(SyntaxHighlightPlugin::new()))
}
```

## Packaging and Publishing

To compile your plugin:

```bash
cargo build --release
```

The compiled plugin will be in `target/release/` with a name like `libmy_plugin.so` (Linux), `libmy_plugin.dylib` (macOS), or `my_plugin.dll` (Windows).

To use your plugin in a Rust-Hexo blog:

1. Copy the dynamic library to the `plugins` directory of your blog
2. Add the plugin name to the `plugins` list in `_config.yml`
3. Add any plugin-specific configuration to `_config.yml`

## Debugging Tips

1. Use the `tracing` crate for logging:

```rust
use tracing::{debug, info, warn, error};

fn init(&mut self, context: &PluginContext) -> Result<()> {
    info!("Plugin initialization started");
    debug!("Debug information: {:?}", context.config);
    
    if something_wrong {
        warn!("Warning: Something might be wrong");
    }
    
    if serious_error {
        error!("Error: Something is definitely wrong");
    }
    
    Ok(())
}
```

2. Use `anyhow` for error handling:

```rust
use anyhow::{Result, anyhow, Context};

fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
    let processed = self.process_code(content)
        .context("Failed to process code blocks")?;
    
    if processed.is_empty() && !content.is_empty() {
        return Err(anyhow!("Processing resulted in empty content"));
    }
    
    Ok(processed)
}
```

## Common Issues

1. **Plugin not loading**: Ensure the dynamic library is in the correct location and the plugin name is listed in `_config.yml`.

2. **ABI compatibility issues**: Make sure your plugin is compiled with the same Rust version as Rust-Hexo.

3. **Segmentation faults**: These can occur if you have memory management issues or if your plugin crashes. Add error handling and avoid unsafe code when possible.

4. **Plugin initialization fails**: Check that your plugin's configuration section in `_config.yml` is properly formatted.

5. **Content not being processed**: Make sure you're handling the correct `ContentType` in your `process_content` method.

I hope this guide helps you develop plugins for Rust-Hexo. Happy coding!
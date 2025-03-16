use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use semver::Version;
use syntect::{
    highlighting::ThemeSet,
    html::{ClassedHTMLGenerator, ClassStyle},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use tracing::{debug, info};

/// 代码高亮插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyntaxHighlightConfig {
    /// 代码主题
    theme: String,
    /// 是否显示行号
    line_numbers: bool,
    /// 是否启用行高亮
    line_highlighting: bool,
    /// 是否启用代码复制按钮
    copy_button: bool,
    /// 是否显示语言标签
    show_language: bool,
    /// 默认语言（当未指定时）
    default_language: String,
}

impl Default for SyntaxHighlightConfig {
    fn default() -> Self {
        Self {
            theme: "Solarized (dark)".to_string(),
            line_numbers: true,
            line_highlighting: true,
            copy_button: true,
            show_language: true,
            default_language: "text".to_string(),
        }
    }
}

pub struct SyntaxHighlightPlugin {
    name: String,
    version: String,
    description: String,
    config: SyntaxHighlightConfig,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlightPlugin {
    pub fn new() -> Self {
        Self {
            name: "syntax-highlight".to_string(),
            version: "0.1.0".to_string(),
            description: "代码高亮插件，支持多种编程语言和主题".to_string(),
            config: SyntaxHighlightConfig::default(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    /// 高亮代码块
    fn highlight_code(&self, code: &str, language: Option<&str>) -> Result<String> {
        let syntax = match language {
            Some(lang) => self.syntax_set
                .find_syntax_by_token(lang)
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text()),
            None => self.syntax_set.find_syntax_plain_text(),
        };

        let theme = self.theme_set.themes
            .get(&self.config.theme)
            .unwrap_or_else(|| self.theme_set.themes.get("base16-ocean.dark").unwrap());

        let mut html = String::new();

        if self.config.show_language {
            if let Some(lang) = language {
                html.push_str(&format!(
                    "<div class=\"code-language\">{}</div>",
                    lang
                ));
            }
        }

        // 使用语言类名，但确保不会影响到非代码块内容
        let language_class = language.map_or("text".to_string(), |lang| lang.to_string());
        html.push_str(&format!("<pre><code class=\"hljs {}\">", language_class));
        
        if self.config.line_numbers {
            html.push_str("<table class=\"highlight-table\"><tbody>");
            let lines: Vec<&str> = code.lines().collect();
            let mut generator = ClassedHTMLGenerator::new_with_class_style(
                syntax,
                &self.syntax_set,
                ClassStyle::SpacedPrefixed { prefix: "highlight-" },
            );

            for (i, line) in lines.iter().enumerate() {
                html.push_str(&format!(
                    "<tr><td class=\"line-number\">{}</td><td class=\"line-content\">",
                    i + 1
                ));
                generator.parse_html_for_line_which_includes_newline(line)?;
                let line_html = generator.finalize();
                html.push_str(&line_html);
                html.push_str("</td></tr>");
                
                // 重新初始化生成器用于下一行
                generator = ClassedHTMLGenerator::new_with_class_style(
                    syntax,
                    &self.syntax_set,
                    ClassStyle::SpacedPrefixed { prefix: "highlight-" },
                );
            }
            html.push_str("</tbody></table>");
        } else {
            let mut generator = ClassedHTMLGenerator::new_with_class_style(
                syntax,
                &self.syntax_set,
                ClassStyle::SpacedPrefixed { prefix: "highlight-" },
            );
            for line in LinesWithEndings::from(code) {
                generator.parse_html_for_line_which_includes_newline(line)?;
            }
            html.push_str(&generator.finalize());
        }
        html.push_str("</code></pre>");

        if self.config.copy_button {
            html.push_str(
                r#"<button class="copy-button" onclick="copyCode(this)">复制</button>
                <script>
                function copyCode(button) {
                    const pre = button.previousElementSibling;
                    const code = pre.textContent;
                    navigator.clipboard.writeText(code).then(() => {
                        button.textContent = '已复制';
                        setTimeout(() => {
                            button.textContent = '复制';
                        }, 2000);
                    });
                }
                </script>"#
            );
        }

        Ok(html)
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

    fn init(&mut self, context: &PluginContext) -> Result<()> {
        info!("初始化代码高亮插件");
        
        // 从配置中加载设置
        if let Some(plugins) = &context.config.plugins {
            if plugins.contains(&"syntax-highlight".to_string()) {
                if let Some(config) = context.config.theme_config.as_ref() {
                    if let Some(highlight_config) = config.get("syntax-highlight") {
                        if let Ok(config) = serde_yaml::from_value(highlight_config.clone()) {
                            self.config = config;
                            debug!("已加载代码高亮配置");
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
        match hook {
            PluginHook::BeforePostRender | PluginHook::BeforePageRender => {
                info!("处理文章代码高亮");
            }
            _ => {}
        }
        Ok(())
    }

    fn process_content(&self, content: &str, _content_type: ContentType) -> Result<String> {
        let mut processed = String::new();
        let mut in_code_block = false;
        let mut current_language = None;
        let mut code_buffer = String::new();

        for line in content.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // 结束代码块
                    let highlighted = self.highlight_code(&code_buffer, current_language)?;
                    processed.push_str(&highlighted);
                    processed.push('\n');
                    code_buffer.clear();
                    in_code_block = false;
                    current_language = None;
                } else {
                    // 开始代码块
                    in_code_block = true;
                    current_language = line.trim_start_matches('`').trim().split_whitespace().next();
                    processed.push_str(&format!("<div class=\"code-block\">\n"));
                }
            } else if in_code_block {
                code_buffer.push_str(line);
                code_buffer.push('\n');
            } else {
                processed.push_str(line);
                processed.push('\n');
            }
        }
        
        // 处理未闭合的代码块
        if in_code_block {
            // 如果代码块未闭合，将其视为普通文本
            processed.push_str("<pre><code>");
            processed.push_str(&code_buffer.replace("<", "&lt;").replace(">", "&gt;"));
            processed.push_str("</code></pre>\n");
            processed.push_str("</div>\n");
        }

        Ok(processed)
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
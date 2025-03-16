use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use regex::Regex;

/// 数学公式渲染插件配置
#[derive(Debug, Serialize, Deserialize)]
struct MathConfig {
    /// 渲染引擎（katex/mathjax）
    engine: String,
    /// 是否启用行内公式
    inline: bool,
    /// 是否启用块级公式
    block: bool,
    /// KaTeX 配置
    katex: KatexConfig,
    /// MathJax 配置
    mathjax: MathJaxConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct KatexConfig {
    /// 是否启用宏
    macros: bool,
    /// 是否启用自动渲染
    auto_render: bool,
    /// 是否启用 mhchem 扩展
    mhchem: bool,
    /// 是否在错误时抛出
    throw_on_error: bool,
    /// 错误颜色
    error_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MathJaxConfig {
    /// MathJax CDN URL
    cdn_url: String,
    /// 是否启用 SVG 输出
    svg: bool,
    /// 是否启用 CommonHTML 输出
    common_html: bool,
    /// 是否启用 CHTML-preview 输出
    chtml_preview: bool,
}

impl Default for MathConfig {
    fn default() -> Self {
        Self {
            engine: "katex".to_string(),
            inline: true,
            block: true,
            katex: KatexConfig::default(),
            mathjax: MathJaxConfig::default(),
        }
    }
}

impl Default for KatexConfig {
    fn default() -> Self {
        Self {
            macros: true,
            auto_render: true,
            mhchem: true,
            throw_on_error: false,
            error_color: "#cc0000".to_string(),
        }
    }
}

impl Default for MathJaxConfig {
    fn default() -> Self {
        Self {
            cdn_url: "https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js".to_string(),
            svg: false,
            common_html: true,
            chtml_preview: false,
        }
    }
}

pub struct MathPlugin {
    name: String,
    version: String,
    description: String,
    config: MathConfig,
}

impl MathPlugin {
    pub fn new() -> Self {
        Self {
            name: "math".to_string(),
            version: "0.1.0".to_string(),
            description: "数学公式渲染插件，支持 KaTeX 和 MathJax".to_string(),
            config: MathConfig::default(),
        }
    }

    /// 处理行内公式
    fn process_inline_math(&self, content: &str) -> String {
        info!("处理行内公式");
        let inline_regex = Regex::new(r"\$([^\$]+)\$").unwrap();
        inline_regex.replace_all(content, |caps: &regex::Captures| {
            let formula = &caps[1];
            let result = match self.config.engine.as_str() {
                "katex" => format!(r#"<span class="math inline">\({}\)</span>"#, formula),
                "mathjax" => format!(r#"<span class="math inline">\({}\)</span>"#, formula),
                _ => caps[0].to_string(),
            };
            debug!("行内公式替换: {} -> {}", &caps[0], &result);
            result
        }).to_string()
    }

    /// 处理块级公式
    fn process_block_math(&self, content: &str) -> String {
        info!("处理块级公式");
        let block_regex = Regex::new(r"\$\$([\s\S]+?)\$\$").unwrap();
        block_regex.replace_all(content, |caps: &regex::Captures| {
            let formula = &caps[1];
            let result = match self.config.engine.as_str() {
                "katex" => format!(r#"<div class="math block">\[{}\]</div>"#, formula),
                "mathjax" => format!(r#"<div class="math block">\[{}\]</div>"#, formula),
                _ => caps[0].to_string(),
            };
            debug!("块级公式替换: {} -> {}", &caps[0], &result);
            result
        }).to_string()
    }

    /// 添加依赖
    fn add_dependencies(&self, content: &str) -> String {
        info!("添加数学公式渲染依赖脚本");
        let mut head_scripts = String::new();
        let mut body_scripts = String::new();

        match self.config.engine.as_str() {
            "katex" => {
                info!("使用KaTeX引擎");
                head_scripts.push_str(r#"
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css">
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js"></script>
"#);
                
                if self.config.katex.auto_render {
                    info!("启用KaTeX自动渲染");
                    head_scripts.push_str(r#"<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js"></script>"#);
                    body_scripts.push_str(&format!(r#"
<script>
document.addEventListener("DOMContentLoaded", function() {{
    renderMathInElement(document.body, {{
        delimiters: [
            {{left: "$$", right: "$$", display: true}},
            {{left: "$", right: "$", display: false}},
            {{left: "\\(", right: "\\)", display: false}},
            {{left: "\\[", right: "\\]", display: true}}
        ],
        throwOnError: {},
        errorColor: "{}",
        macros: {{}}
    }});
}});
</script>
"#, self.config.katex.throw_on_error, self.config.katex.error_color));
                }

                if self.config.katex.mhchem {
                    info!("启用KaTeX mhchem扩展");
                    head_scripts.push_str(r#"<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/mhchem.min.js"></script>"#);
                }
            }
            "mathjax" => {
                info!("使用MathJax引擎");
                head_scripts.push_str(&format!(r#"
<script>
MathJax = {{
    tex: {{
        inlineMath: [['$', '$'], ['\\(', '\\)']],
        displayMath: [['$$', '$$'], ['\\[', '\\]']],
        processEscapes: true,
        processEnvironments: true
    }},
    options: {{
        skipHtmlTags: ['script', 'noscript', 'style', 'textarea', 'pre']
    }}
}};
</script>
<script type="text/javascript" id="MathJax-script" async src="{}"></script>
"#, self.config.mathjax.cdn_url));
            }
            _ => {
                warn!("未知的数学公式渲染引擎: {}", self.config.engine);
            }
        }

        // 在 </head> 前插入头部脚本
        let has_head = content.contains("</head>");
        let content = if has_head {
            info!("在</head>标签前添加脚本");
            content.replace("</head>", &format!("{}</head>", head_scripts))
        } else {
            warn!("找不到</head>标签，无法添加头部脚本");
            content.to_string()
        };
        
        // 在 </body> 前插入主体脚本
        let has_body = content.contains("</body>");
        if has_body {
            info!("在</body>标签前添加脚本");
            content.replace("</body>", &format!("{}</body>", body_scripts))
        } else {
            warn!("找不到</body>标签，无法添加主体脚本");
            content.to_string()
        }
    }
}

impl Plugin for MathPlugin {
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
        info!("初始化数学公式渲染插件");
        
        // 从配置中加载设置
        if let Some(plugins) = &context.config.plugins {
            if plugins.contains(&"math".to_string()) {
                if let Some(config) = context.config.theme_config.as_ref() {
                    if let Some(math_config) = config.get("math") {
                        if let Ok(config) = serde_yaml::from_value(math_config.clone()) {
                            self.config = config;
                            debug!("已加载数学公式渲染配置");
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
                info!("处理数学公式");
            }
            _ => {}
        }
        Ok(())
    }

    fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
        info!("处理内容，类型: {:?}", content_type);
        
        // 只处理HTML内容
        if content_type != ContentType::HTML {
            debug!("非HTML内容，跳过处理");
            return Ok(content.to_string());
        }
        
        let mut processed = content.to_string();

        if self.config.inline {
            processed = self.process_inline_math(&processed);
        }

        if self.config.block {
            processed = self.process_block_math(&processed);
        }

        // 添加必要的脚本和样式
        processed = self.add_dependencies(&processed);

        Ok(processed)
    }

    fn cleanup(&self) -> Result<()> {
        info!("清理数学公式渲染插件");
        Ok(())
    }
}

// 创建插件实例的外部函数
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(MathPlugin::new()))
} 
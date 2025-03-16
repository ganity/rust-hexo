use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use semver::Version;
use tracing::{debug, info};

/// 评论系统类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CommentSystem {
    Giscus,
    Disqus,
}

/// Giscus 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GiscusConfig {
    /// GitHub 仓库
    repo: String,
    /// 仓库所有者
    repo_owner: String,
    /// 仓库 ID
    repo_id: String,
    /// 分类名称
    category: String,
    /// 分类 ID
    category_id: String,
    /// 映射方式
    mapping: String,
    /// 反应
    reactions_enabled: bool,
    /// 主题
    theme: String,
    /// 语言
    lang: String,
}

/// Disqus 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisqusConfig {
    /// Disqus shortname
    shortname: String,
    /// 语言
    lang: String,
}

/// 评论插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommentsConfig {
    /// 是否启用评论
    enabled: bool,
    /// 评论系统类型
    system: CommentSystem,
    /// Giscus 配置
    giscus: Option<GiscusConfig>,
    /// Disqus 配置
    disqus: Option<DisqusConfig>,
}

impl Default for CommentsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            system: CommentSystem::Giscus,
            giscus: None,
            disqus: None,
        }
    }
}

pub struct CommentsPlugin {
    name: String,
    version: String,
    description: String,
    config: CommentsConfig,
}

impl CommentsPlugin {
    pub fn new() -> Self {
        Self {
            name: "comments".to_string(),
            version: "0.1.0".to_string(),
            description: "评论系统插件，支持 Giscus 和 Disqus".to_string(),
            config: CommentsConfig::default(),
        }
    }

    /// 生成 Giscus 评论代码
    fn generate_giscus_html(&self, config: &GiscusConfig) -> String {
        format!(
            r#"<script src="https://giscus.app/client.js"
                data-repo="{}/{}"
                data-repo-id="{}"
                data-category="{}"
                data-category-id="{}"
                data-mapping="{}"
                data-reactions-enabled="{}"
                data-theme="{}"
                data-lang="{}"
                crossorigin="anonymous"
                async>
            </script>"#,
            config.repo_owner,
            config.repo,
            config.repo_id,
            config.category,
            config.category_id,
            config.mapping,
            config.reactions_enabled,
            config.theme,
            config.lang
        )
    }

    /// 生成 Disqus 评论代码
    fn generate_disqus_html(&self, config: &DisqusConfig) -> String {
        format!(
            r#"<div id="disqus_thread"></div>
            <script>
                var disqus_config = function () {{
                    this.language = "{}";
                }};
                (function() {{
                    var d = document, s = d.createElement('script');
                    s.src = 'https://{}.disqus.com/embed.js';
                    s.setAttribute('data-timestamp', +new Date());
                    (d.head || d.body).appendChild(s);
                }})();
            </script>"#,
            config.lang,
            config.shortname
        )
    }
}

impl Plugin for CommentsPlugin {
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
        info!("初始化评论插件");
        
        // 从配置中加载设置
        if let Some(plugins) = &context.config.plugins {
            if plugins.contains(&"comments".to_string()) {
                if let Some(config) = context.config.theme_config.as_ref() {
                    if let Some(comments_config) = config.get("comments") {
                        if let Ok(config) = serde_yaml::from_value(comments_config.clone()) {
                            self.config = config;
                            debug!("已加载评论插件配置");
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        match hook {
            PluginHook::BeforePostRender => {
                info!("处理文章评论");
                // TODO: 在这里处理评论系统的注入
            }
            _ => {}
        }
        Ok(())
    }

    fn process_content(&self, content: &str, _content_type: ContentType) -> Result<String> {
        if !self.config.enabled {
            return Ok(content.to_string());
        }

        // 根据配置的评论系统类型生成评论代码
        let comments_html = match &self.config.system {
            CommentSystem::Giscus => {
                if let Some(config) = &self.config.giscus {
                    Some(self.generate_giscus_html(config))
                } else {
                    None
                }
            }
            CommentSystem::Disqus => {
                if let Some(config) = &self.config.disqus {
                    Some(self.generate_disqus_html(config))
                } else {
                    None
                }
            }
        };

        // 将评论代码添加到内容末尾
        if let Some(html) = comments_html {
            Ok(format!("{}\n{}", content, html))
        } else {
            Ok(content.to_string())
        }
    }

    fn cleanup(&self) -> Result<()> {
        info!("清理评论插件");
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(CommentsPlugin::new())
} 
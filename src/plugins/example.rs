use super::{Plugin, PluginContext, PluginHook};
use anyhow::Result;
use tracing::info;

/// 示例插件
pub struct ExamplePlugin {
    name: String,
    version: String,
    description: String,
    author: String,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            name: "example".to_string(),
            version: "0.1.0".to_string(),
            description: "一个示例插件，用于展示插件系统的功能".to_string(),
            author: "Rust-Hexo Team".to_string(),
        }
    }
}

impl Plugin for ExamplePlugin {
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
        info!("初始化示例插件");
        info!("插件目录: {}", context.plugins_dir.display());
        Ok(())
    }
    
    fn execute_hook(&self, hook: PluginHook, _context: &PluginContext) -> Result<()> {
        match hook {
            PluginHook::Init => {
                info!("示例插件: 执行初始化钩子");
            }
            PluginHook::BeforeGenerate => {
                info!("示例插件: 生成之前的处理");
            }
            PluginHook::AfterGenerate => {
                info!("示例插件: 生成之后的处理");
            }
            PluginHook::BeforeDeploy => {
                info!("示例插件: 部署之前的处理");
            }
            PluginHook::AfterDeploy => {
                info!("示例插件: 部署之后的处理");
            }
            PluginHook::NewPost => {
                info!("示例插件: 新建文章时的处理");
            }
            PluginHook::NewPage => {
                info!("示例插件: 新建页面时的处理");
            }
            PluginHook::Clean => {
                info!("示例插件: 清理时的处理");
            }
        }
        Ok(())
    }
    
    fn cleanup(&self) -> Result<()> {
        info!("清理示例插件的资源");
        Ok(())
    }
} 
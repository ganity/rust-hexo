// 导入必要的库
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fs;
use anyhow::{anyhow, Result};
use libloading::{Library, Symbol};
use tracing::{info, warn, error, debug};
use serde::Serialize;
use tera;

// 重新导出子模块
mod error;
pub use error::*;

// 定义内容类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Markdown,
    HTML,
    JSON,
    YAML,
    CSS,
    JavaScript,
    Plain,
}

// 重新导出PluginHook
pub use error::PluginHook;

// 定义资源位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ResourceLocation {
    Head,
    Footer,
}

/// 插件上下文，提供给插件使用的环境信息
#[derive(Clone)]
pub struct PluginContext {
    /// 基础目录
    pub base_dir: PathBuf,
    /// 插件目录
    pub plugins_dir: PathBuf,
    /// 主题目录
    pub theme_dir: PathBuf,
    /// 基础URL
    pub base_url: String,
    /// 输出目录
    pub output_dir: PathBuf,
    /// 站点配置
    pub config: crate::models::config::Config,
    /// 所有文章
    pub posts: Vec<crate::models::Post>,
    /// 所有页面
    pub pages: Vec<crate::models::Page>,
    /// 所有分类
    pub categories: Vec<crate::models::Category>,
    /// 所有标签
    pub tags: Vec<crate::models::Tag>,
    /// 当前处理的文章
    pub current_post: Option<crate::models::Post>,
    /// 当前处理的页面
    pub current_page: Option<crate::models::Page>,
}

impl Default for PluginContext {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::new(),
            plugins_dir: PathBuf::new(),
            theme_dir: PathBuf::new(),
            base_url: String::from("/"),
            output_dir: PathBuf::new(),
            config: crate::models::config::Config::default(),
            posts: Vec::new(),
            pages: Vec::new(),
            categories: Vec::new(),
            tags: Vec::new(),
            current_post: None,
            current_page: None,
        }
    }
}

/// 插件特征，所有插件必须实现此特征
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
    
    /// 获取资源
    fn get_resources(&self) -> Vec<(String, ResourceLocation)> {
        Vec::new() // 默认实现返回空列表
    }
    
    /// 获取插件提供的模板函数
    fn get_template_functions(&self) -> HashMap<String, Box<dyn Fn(&HashMap<String, serde_json::Value>) -> Result<serde_json::Value, tera::Error> + Send + Sync>> {
        HashMap::new() // 默认实现返回空列表
    }
    
    /// 清理资源
    fn cleanup(&self) -> Result<()>;
}

/// 插件管理器，负责加载和管理插件
pub struct PluginManager {
    /// 基础目录
    pub base_dir: PathBuf,
    /// 插件目录
    pub plugins_dir: PathBuf,
    /// 插件上下文
    pub context: Arc<RwLock<PluginContext>>,
    /// 已加载的插件
    pub plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
    /// 已加载的库
    pub libraries: Arc<RwLock<Vec<Library>>>,
    /// 文件监视器
    pub watcher: Option<Box<dyn std::any::Any + Send>>,
    /// 是否正在监视
    pub is_watching: Arc<RwLock<bool>>,
    /// 是否已初始化
    pub initialized: bool,
}

impl Clone for PluginManager {
    fn clone(&self) -> Self {
        Self {
            base_dir: self.base_dir.clone(),
            plugins_dir: self.plugins_dir.clone(),
            context: self.context.clone(),
            plugins: self.plugins.clone(),
            libraries: self.libraries.clone(),
            watcher: None, // 不克隆监视器
            is_watching: self.is_watching.clone(),
            initialized: self.initialized,
        }
    }
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new(base_dir: PathBuf, context: PluginContext) -> Self {
        
        let plugins_dir = base_dir.join("plugins");
        info!("创建新的插件管理器，基础目录: {:?}", plugins_dir);
        Self {
            base_dir: base_dir.clone(),
            plugins_dir,
            context: Arc::new(RwLock::new(context)),
            plugins: Arc::new(RwLock::new(HashMap::new())),
            libraries: Arc::new(RwLock::new(Vec::new())),
            watcher: None,
            is_watching: Arc::new(RwLock::new(false)),
            initialized: false,
        }
    }
    
    /// 直接从动态链接库加载插件
    fn load_plugin_from_dylib(&mut self, lib_path: &PathBuf) -> Result<Box<dyn Plugin>> {
        info!("从动态链接库加载插件: {}", lib_path.display());

        // 尝试验证动态库文件
        if !lib_path.exists() {
            return Err(anyhow!(PluginError::LoadError {
                message: format!("插件文件不存在: {}", lib_path.display())
            }));
        }

        let metadata = match std::fs::metadata(lib_path) {
            Ok(m) => m,
            Err(e) => {
                return Err(anyhow!(PluginError::LoadError {
                    message: format!("无法读取插件文件元数据: {} - {}", lib_path.display(), e)
                }));
            }
        };

        if metadata.len() == 0 {
            return Err(anyhow!(PluginError::LoadError {
                message: format!("插件文件为空: {}", lib_path.display())
            }));
        }

        info!("验证插件文件 {} 成功，尝试加载库...", lib_path.display());

        // 使用 catch_unwind 来防止 panic 导致的程序崩溃
        let lib_result = std::panic::catch_unwind(|| {
            unsafe { 
                Library::new(lib_path)
            }
        });

        // 处理可能的 panic
        let lib = match lib_result {
            Ok(lib_result) => {
                match lib_result {
                    Ok(lib) => lib,
                    Err(e) => {
                        return Err(anyhow!(PluginError::LoadError {
                            message: format!("无法加载库 {}: {}", lib_path.display(), e.to_string()),
                        }));
                    }
                }
            },
            Err(_) => {
                return Err(anyhow!(PluginError::LoadError {
                    message: format!("加载库时发生严重错误，可能是ABI不兼容: {}", lib_path.display())
                }));
            }
        };

        info!("库文件 {} 已加载，尝试解析 create_plugin 符号...", lib_path.display());

        // 尝试获取创建插件的函数，使用更谨慎的方式
        let constructor: Symbol<unsafe fn() -> Box<dyn Plugin>> = match std::panic::catch_unwind(|| {
            unsafe {
                lib.get(b"create_plugin")
            }
        }) {
            Ok(symbol_result) => {
                match symbol_result {
                    Ok(symbol) => symbol,
                    Err(e) => {
                        return Err(anyhow!(PluginError::LoadError {
                            message: format!("找不到 create_plugin 函数: {}", e),
                        }));
                    }
                }
            },
            Err(_) => {
                return Err(anyhow!(PluginError::LoadError {
                    message: format!("解析 create_plugin 符号时发生严重错误: {}", lib_path.display())
                }));
            }
        };

        info!("成功解析 create_plugin 符号，正在创建插件实例...");

        // 尝试创建插件实例
        let plugin_result = std::panic::catch_unwind(|| {
            unsafe { constructor() }
        });

        let mut plugin = match plugin_result {
            Ok(plugin) => plugin,
            Err(_) => {
                return Err(anyhow!(PluginError::LoadError {
                    message: format!("调用 create_plugin 时发生严重错误: {}", lib_path.display())
                }));
            }
        };

        let context = self.context.read().unwrap();

        // 初始化插件
        match plugin.init(&context) {
            Ok(_) => {
                // 保存库引用
                self.libraries.write().unwrap().push(lib);
                Ok(plugin)
            },
            Err(e) => {
                error!("插件 {} 初始化失败: {}", plugin.name(), e);
                Err(anyhow!(PluginError::InitError {
                    plugin_name: plugin.name().to_string(),
                    message: e.to_string(),
                }))
            }
        }
    }
    
    /// 加载所有插件
    pub fn load_plugins(&mut self) -> Result<()> {
        // 确保插件目录存在
        info!("开始加载插件，目录: {}", self.plugins_dir.display());
        if !self.plugins_dir.exists() {
            info!("插件目录不存在，创建目录: {}", self.plugins_dir.display());
            std::fs::create_dir_all(&self.plugins_dir)?;
            return Ok(());
        }

        info!("开始加载插件，目录: {}", self.plugins_dir.display());
        let mut loaded_count = 0;
        let mut failed_count = 0;
        let mut failed_plugins = Vec::new();

        // 安全读取目录
        let entries = match std::fs::read_dir(&self.plugins_dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("读取插件目录失败: {} - {}", self.plugins_dir.display(), e);
                return Ok(());
            }
        };

        // 遍历每个条目
        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    warn!("读取目录条目失败: {}", e);
                    failed_count += 1;
                    continue;
                }
            };
            
            let path = entry.path();
            
            if path.is_file() {
                // 检查是否是动态链接库
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    let is_plugin_lib = (cfg!(target_os = "windows") && ext_str == "dll") ||
                                        (cfg!(target_os = "macos") && ext_str == "dylib") ||
                                        (cfg!(target_os = "linux") && ext_str == "so");
                    
                    if is_plugin_lib {
                        info!("尝试加载插件: {}", path.display());
                        
                        // 提取文件名，用于调试
                        let file_name = path.file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| String::from("unknown"));
                        
                        info!("插件文件名: {}", file_name);
                        
                        // 使用 std::panic::catch_unwind 防止整个进程崩溃
                        let load_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            self.load_plugin_from_dylib(&path)
                        }));
                        
                        match load_result {
                            Ok(plugin_result) => {
                                match plugin_result {
                            Ok(plugin) => {
                                let name = plugin.name().to_string();
                                info!("成功加载插件: {} v{} (文件: {})", name, plugin.version(), file_name);
                                
                                // 插件命名规范化 - 确保与配置文件中的名称匹配
                                let config_name = if name.contains('_') {
                                    name.replace('_', "-")
                                } else {
                                    name.clone()
                                };
                                
                                info!("插件配置名: {}", config_name);
                                self.plugins.write().unwrap().insert(config_name, plugin);
                                loaded_count += 1;
                            },
                            Err(e) => {
                                error!("加载插件失败: {} - {}", path.display(), e);
                                        failed_plugins.push(format!("{}: {}", file_name, e));
                                        failed_count += 1;
                                    }
                                }
                            },
                            Err(panic_err) => {
                                let panic_msg = match panic_err.downcast_ref::<&str>() {
                                    Some(s) => *s,
                                    None => match panic_err.downcast_ref::<String>() {
                                        Some(s) => s.as_str(),
                                        None => "未知错误（可能是内存访问或ABI不兼容）",
                                    },
                                };
                                
                                error!("加载插件时发生严重错误: {} - {}", path.display(), panic_msg);
                                failed_plugins.push(format!("{}: 严重错误 - {}", file_name, panic_msg));
                                failed_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // 打印已加载的插件列表
        let plugins = self.plugins.read().unwrap();
        if !plugins.is_empty() {
            info!("已加载的插件列表:");
            for (name, plugin) in plugins.iter() {
                info!("  - {} v{}", name, plugin.version());
            }
        }
        
        // 如果有失败的插件，输出详细信息
        if !failed_plugins.is_empty() {
            warn!("以下插件加载失败:");
            for failed in &failed_plugins {
                warn!("  - {}", failed);
            }
        }
        
        info!("插件加载完成 - 成功: {}, 失败: {}", loaded_count, failed_count);
        Ok(())
    }

    /// 初始化插件管理器
    pub fn init(&mut self) -> Result<()> {
        info!("初始化插件管理器...");
        
        // 加载所有插件
        self.load_plugins()?;
        self.initialized = true;
        Ok(())
    }
    
    /// 检查插件管理器是否已初始化
    pub fn is_initialized(&self) -> bool {
        // 检查内部状态标志和插件数量
        // if !self.initialized {
        //     return false;
        // }
        
        // // 检查是否已加载插件
        // let plugins = self.plugins.read().unwrap();
        // if plugins.is_empty() {
        //     return false;
        // }
        
        // true
        false
    }
    
    /// 设置插件上下文
    pub fn set_context(&mut self, context: PluginContext) {
        let mut ctx = self.context.write().unwrap();
        *ctx = context;
    }
    
    /// 获取所有插件
    pub fn get_all_plugins(&self) -> Result<Vec<Box<dyn Plugin>>> {
        let plugins = self.plugins.read().unwrap();
        let mut result = Vec::new();
        
        for (_, plugin) in plugins.iter() {
            // 使用克隆的插件实现，而不是引用
            let cloned: Box<dyn Plugin> = Box::new(ClonedPlugin { 
                name: plugin.name().to_string(),
                version: plugin.version().to_string(),
                description: plugin.description().to_string()
            });
            result.push(cloned);
        }
        
        Ok(result)
    }
    
    /// 处理内容
    pub fn process_content(&self, content: &str, content_type: ContentType) -> Result<String> {
        let plugins = self.plugins.read().unwrap();
        let mut processed = content.to_string();
        
        for (name, plugin) in plugins.iter() {
            match plugin.process_content(&processed, content_type) {
                Ok(new_content) => processed = new_content,
                Err(e) => {
                    warn!("插件 {} 处理内容失败: {}", name, e);
                    // 继续处理，不中断整个过程
                }
            }
        }
        
        Ok(processed)
    }
    
    /// 执行钩子
    pub fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
        let plugins = self.plugins.read().unwrap();
        let mut errors = Vec::new();
        
        for (name, plugin) in plugins.iter() {
            if let Err(e) = plugin.execute_hook(hook) {
                errors.push(format!("插件 {} 执行钩子 {:?} 失败: {}", name, hook, e));
            }
        }
        
        if !errors.is_empty() {
            return Err(anyhow!(PluginError::HookError {
                plugin_name: "multiple".to_string(),
                hook_name: format!("{:?}", hook),
                message: errors.join("; ")
            }));
        }
        
        Ok(())
    }
    
    /// 清理资源
    pub fn cleanup(&self) -> Result<()> {
        let plugins = self.plugins.read().unwrap();
        let mut errors = Vec::new();
        
        for (name, plugin) in plugins.iter() {
            if let Err(e) = plugin.cleanup() {
                errors.push(format!("插件 {} 清理失败: {}", name, e));
            }
        }
        
        if !errors.is_empty() {
            return Err(anyhow!(PluginError::CleanupError {
                message: errors.join("; ")
            }));
        }
        
        Ok(())
    }
    
    /// 注册到主题渲染器
    pub fn register_to_theme_renderer(&self, _renderer: &mut crate::theme::renderer::ThemeRenderer) -> Result<()> {
        Ok(())
    }
    
    /// 启动热重载
    pub fn start_hot_reload(&mut self) -> Result<()> {
        *self.is_watching.write().unwrap() = true;
        Ok(())
    }
    
    /// 停止热重载
    pub fn stop_hot_reload(&mut self) {
        *self.is_watching.write().unwrap() = false;
    }
    
    /// 注册所有插件提供的模板函数到Tera实例
    pub fn register_template_functions(&self, tera: &mut tera::Tera) -> Result<()> {
        let plugins = self.plugins.read().unwrap();
        
        for (name, plugin) in plugins.iter() {
            debug!("注册插件 {} 提供的模板函数", name);
            
            // 获取插件提供的模板函数
            let template_functions = plugin.get_template_functions();
            
            // 注册每个函数到Tera实例
            for (func_name, func) in template_functions {
                debug!("注册模板函数: {}", func_name);
                
                // 在闭包外部克隆 func_name，这样可以在闭包中使用克隆版本
                let func_name_clone = func_name.clone();
                
                // 创建一个闭包包装函数，将 serde_json::Value 转换为 tera::Value
                let wrapped_func = move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                    // 将 tera::Value 转换为 serde_json::Value
                    let converted_args: HashMap<String, serde_json::Value> = args.iter()
                        .map(|(k, v)| {
                            let json_value = match v {
                                tera::Value::Null => serde_json::Value::Null,
                                tera::Value::Bool(b) => serde_json::Value::Bool(*b),
                                tera::Value::Number(n) => serde_json::Value::Number(n.clone()),
                                tera::Value::String(s) => serde_json::Value::String(s.clone()),
                                tera::Value::Array(a) => {
                                    // 递归转换数组元素
                                    let json_array: Vec<serde_json::Value> = a.iter()
                                        .map(|item| match item {
                                            tera::Value::String(s) => serde_json::Value::String(s.clone()),
                                            tera::Value::Number(n) => serde_json::Value::Number(n.clone()),
                                            tera::Value::Bool(b) => serde_json::Value::Bool(*b),
                                            _ => serde_json::Value::Null,
                                        })
                                        .collect();
                                    serde_json::Value::Array(json_array)
                                },
                                tera::Value::Object(o) => {
                                    // 递归转换对象属性
                                    let mut json_object = serde_json::Map::new();
                                    for (key, val) in o {
                                        let json_val = match val {
                                            tera::Value::String(s) => serde_json::Value::String(s.clone()),
                                            tera::Value::Number(n) => serde_json::Value::Number(n.clone()),
                                            tera::Value::Bool(b) => serde_json::Value::Bool(*b),
                                            _ => serde_json::Value::Null,
                                        };
                                        json_object.insert(key.clone(), json_val);
                                    }
                                    serde_json::Value::Object(json_object)
                                },
                            };
                            (k.clone(), json_value)
                        })
                        .collect();
                    
                    // 调用插件提供的函数
                    match func(&converted_args) {
                        Ok(result) => {
                            // 将 serde_json::Value 转换回 tera::Value
                            match result {
                                serde_json::Value::Null => Ok(tera::Value::Null),
                                serde_json::Value::Bool(b) => Ok(tera::Value::Bool(b)),
                                serde_json::Value::Number(n) => Ok(tera::Value::Number(n)),
                                serde_json::Value::String(s) => Ok(tera::Value::String(s)),
                                serde_json::Value::Array(a) => {
                                    // 递归转换数组
                                    let tera_array: Vec<tera::Value> = a.iter()
                                        .map(|item| match item {
                                            serde_json::Value::String(s) => tera::Value::String(s.clone()),
                                            serde_json::Value::Number(n) => tera::Value::Number(n.clone()),
                                            serde_json::Value::Bool(b) => tera::Value::Bool(*b),
                                            _ => tera::Value::Null,
                                        })
                                        .collect();
                                    Ok(tera::Value::Array(tera_array))
                                },
                                serde_json::Value::Object(o) => {
                                    // 递归转换对象
                                    let mut tera_object = serde_json::Map::new();
                                    for (key, val) in o {
                                        let tera_val = match val {
                                            serde_json::Value::String(s) => tera::Value::String(s),
                                            serde_json::Value::Number(n) => tera::Value::Number(n),
                                            serde_json::Value::Bool(b) => tera::Value::Bool(b),
                                            _ => tera::Value::Null,
                                        };
                                        tera_object.insert(key, tera_val);
                                    }
                                    Ok(tera::Value::Object(tera_object))
                                },
                            }
                        },
                        Err(e) => Err(tera::Error::msg(format!("插件函数 '{}' 执行失败: {}", func_name_clone, e))),
                    }
                };
                
                // 注册函数到Tera
                tera.register_function(&func_name, Box::new(wrapped_func));
            }
        }
        
        Ok(())
    }
}

/// 插件克隆辅助结构体（只有基本信息）
#[derive(Clone)]
struct ClonedPlugin {
    name: String,
    version: String,
    description: String,
}

impl Plugin for ClonedPlugin {
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
        Ok(())
    }

    fn execute_hook(&self, _hook: &PluginHook) -> Result<()> {
        Ok(())
    }
    
    fn process_content(&self, content: &str, _content_type: ContentType) -> Result<String> {
        Ok(content.to_string())
    }
    
    fn cleanup(&self) -> Result<()> {
        Ok(())
    }
} 
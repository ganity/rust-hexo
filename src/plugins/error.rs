use std::fmt;
use std::error::Error;
use thiserror::Error;

/// 插件错误类型
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("版本错误: {message}")]
    VersionError {
        message: String,
    },
    
    #[error("加载插件失败: {message}")]
    LoadError {
        message: String,
    },
    
    #[error("初始化插件失败: {plugin_name} - {message}")]
    InitError {
        plugin_name: String,
        message: String,
    },
    
    #[error("执行钩子失败: 插件 {plugin_name} 在执行 {hook_name} 时出错: {message}")]
    HookError {
        plugin_name: String,
        hook_name: String,
        message: String,
    },
    
    #[error("清理插件失败: {message}")]
    CleanupError {
        message: String,
    },
    
    #[error("依赖错误: {message}")]
    DependencyError {
        message: String,
    },
    
    #[error("配置错误: {message}")]
    ConfigError {
        message: String,
    },
    
    #[error("函数注册错误: {message}")]
    FunctionRegistrationError {
        message: String,
    },
    
    #[error("过滤器注册错误: {message}")]
    FilterRegistrationError {
        message: String,
    },
    
    #[error("资源处理错误: {message}")]
    ResourceError {
        message: String,
    },
    
    #[error("内容处理错误: {plugin_name} - {message}")]
    ContentProcessingError {
        plugin_name: String,
        message: String,
    },
    
    #[error("其他错误: {0}")]
    Other(String),
}

/// 插件钩子类型
#[derive(Debug, Clone)]
pub enum PluginHook {
    /// 初始化
    Init,
    /// 生成前
    BeforeGenerate,
    /// 生成后
    AfterGenerate,
    /// 部署前
    BeforeDeploy,
    /// 部署后
    AfterDeploy,
    /// 新建文章
    NewPost,
    /// 新建页面
    NewPage,
    /// 清理
    Clean,
    /// 配置变更
    ConfigChanged,
    /// 文章渲染前
    BeforePostRender,
    /// 文章渲染后
    AfterPostRender,
    /// 页面渲染前
    BeforePageRender,
    /// 页面渲染后
    AfterPageRender,
    /// 路由生成前
    BeforeRouteGenerate,
    /// 路由生成后
    AfterRouteGenerate,
    /// 资源处理前
    BeforeAssetProcess,
    /// 资源处理后
    AfterAssetProcess,
    /// 服务器启动前
    BeforeServerStart,
    /// 服务器启动后
    AfterServerStart,
    /// 模板加载前
    BeforeTemplateLoad,
    /// 模板加载后
    AfterTemplateLoad,
} 
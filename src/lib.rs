pub mod core;
pub mod models;
pub mod plugins;
pub mod theme;
pub mod utils;

// Re-export commonly used types and traits
pub use crate::plugins::{Plugin, PluginContext, PluginHook};
pub use crate::models::{Post, Page, Category, Tag};
pub use crate::theme::renderer::ThemeRenderer; 
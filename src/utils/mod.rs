use chrono::{DateTime, Utc};
use std::path::Path;

/// 从文件名生成 URL 友好的别名
pub fn slugify(text: &str) -> String {
    slug::slugify(text)
}

/// 从日期和标题创建永久链接
pub fn create_permalink(
    date: &DateTime<Utc>,
    title: &str,
    pattern: &str,
) -> String {
    let slug = slugify(title);
    
    pattern
        .replace(":year", &date.format("%Y").to_string())
        .replace(":month", &date.format("%m").to_string())
        .replace(":day", &date.format("%d").to_string())
        .replace(":hour", &date.format("%H").to_string())
        .replace(":minute", &date.format("%M").to_string())
        .replace(":second", &date.format("%S").to_string())
        .replace(":title", &slug)
}

/// 检查文件是否为 Markdown 文件
pub fn is_markdown_file<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    if let Some(ext) = path.extension() {
        ext == "md" || ext == "markdown"
    } else {
        false
    }
}

/// 确保路径以斜杠结尾
pub fn ensure_trailing_slash(path: &str) -> String {
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}

/// 确保路径以斜杠开头
pub fn ensure_leading_slash(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    }
}

/// 计算两个日期之间的相对时间描述
pub fn relative_time_from_now(date: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*date);
    
    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 30 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_days() < 365 {
        format!("{} months ago", duration.num_days() / 30)
    } else {
        format!("{} years ago", duration.num_days() / 365)
    }
}

pub mod markdown; 
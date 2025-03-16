use anyhow::Result;
use pulldown_cmark::{html, Options, Parser};

/// 将Markdown渲染为HTML
pub fn render(markdown: &str) -> Result<String> {
    // 创建Markdown解析选项
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    
    // 解析Markdown
    let parser = Parser::new_ext(markdown, options);
    
    // 将解析结果渲染为HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    
    Ok(html_output)
} 
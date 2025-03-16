use rust_hexo::plugins::{Plugin, PluginContext, PluginHook, ContentType};
use rust_hexo::Post;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json;
use semver::Version;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, info};
use std::sync::{Arc, RwLock};

/// 搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchConfig {
    /// 是否启用搜索功能
    enabled: bool,
    /// 是否使用全文搜索
    full_content: bool,
    /// 搜索结果的最大数量
    limit: usize,
    /// 搜索结果的排序方式（date/-date/title/-title）
    order_by: String,
    /// 搜索字段权重
    weights: SearchWeights,
    /// 是否启用实时搜索
    live_search: bool,
    /// 搜索索引路径
    index_path: String,
    /// 搜索结果预览长度
    preview_length: usize,
    /// 搜索结果高亮
    highlight: bool,
}

/// 搜索字段权重
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchWeights {
    /// 标题权重
    title: f32,
    /// 内容权重
    content: f32,
    /// 标签权重
    tags: f32,
    /// 分类权重
    categories: f32,
}

/// 搜索索引项
#[derive(Debug, Serialize, Deserialize)]
struct SearchIndexItem {
    /// 文章标题
    title: String,
    /// 文章路径
    path: String,
    /// 文章内容
    content: String,
    /// 发布日期
    date: String,
    /// 分类
    categories: Vec<String>,
    /// 标签
    tags: Vec<String>,
    /// 预览内容
    preview: String,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            full_content: true,
            limit: 10,
            order_by: "date".to_string(),
            weights: SearchWeights {
                title: 2.0,
                content: 1.0,
                tags: 1.5,
                categories: 1.2,
            },
            live_search: true,
            index_path: "/search/search.json".to_string(),
            preview_length: 200,
            highlight: true,
        }
    }
}

pub struct SearchPlugin {
    name: String,
    version: String,
    description: String,
    config: SearchConfig,
    output_dir: PathBuf,
    context: Arc<RwLock<PluginContext>>,
}

impl SearchPlugin {
    pub fn new() -> Self {
        Self {
            name: "search".to_string(),
            version: "0.1.0".to_string(),
            description: "Search plugin for Rust-Hexo".to_string(),
            config: SearchConfig::default(),
            output_dir: PathBuf::new(),
            context: Arc::new(RwLock::new(PluginContext::default())),
        }
    }

    /// 生成搜索索引
    fn generate_search_index(&self, posts: &[Post]) -> Result<()> {
        let mut index = Vec::new();

        for post in posts {
            let content = if self.config.full_content {
                post.content.clone()
            } else {
                post.excerpt.clone().unwrap_or_else(|| {
                    let content = &post.content;
                    if content.len() > self.config.preview_length {
                        format!("{}...", &content[..self.config.preview_length])
                    } else {
                        content.clone()
                    }
                })
            };

            let preview = if content.chars().count() > self.config.preview_length {
                content.chars()
                    .take(self.config.preview_length)
                    .collect::<String>() + "..."
            } else {
                content.clone()
            };

            let item = SearchIndexItem {
                title: post.title.clone(),
                path: post.path.clone(),
                content,
                date: post.date.format("%Y-%m-%d").to_string(),
                categories: post.categories.clone(),
                tags: post.tags.clone(),
                preview,
            };

            index.push(item);
        }

        // 根据配置的排序方式排序
        match self.config.order_by.as_str() {
            "date" => index.sort_by(|a, b| b.date.cmp(&a.date)),
            "-date" => index.sort_by(|a, b| a.date.cmp(&b.date)),
            "title" => index.sort_by(|a, b| a.title.cmp(&b.title)),
            "-title" => index.sort_by(|a, b| b.title.cmp(&a.title)),
            _ => {}
        }

        // 限制结果数量
        if index.len() > self.config.limit {
            index.truncate(self.config.limit);
        }

        // 将索引写入文件
        let index_path = self.output_dir.join("search").join("search.json");
        if let Some(parent) = index_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let index_json = serde_json::to_string_pretty(&index)?;
        std::fs::write(index_path, index_json)?;

        Ok(())
    }

    /// 生成搜索页面
    fn generate_search_page(&self) -> Result<()> {
        let search_html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>站内搜索</title>
    <link rel="stylesheet" href="/css/style.css">
    <style>
        .search-container {{ max-width: 800px; margin: 50px auto; padding: 20px; }}
        #search-input {{ width: 100%; padding: 10px; font-size: 16px; margin-bottom: 20px; border: 1px solid #ddd; border-radius: 4px; }}
        #search-results {{ list-style: none; padding: 0; }}
        .search-result {{ margin-bottom: 20px; padding: 15px; border: 1px solid #eee; border-radius: 4px; }}
        .search-result h3 {{ margin: 0 0 10px 0; }}
        .search-result p {{ margin: 0; color: #666; }}
        .highlight {{ background-color: yellow; padding: 2px; }}
    </style>
</head>
<body>
    <div class="search-container">
        <input type="text" id="search-input" placeholder="输入关键词搜索...">
        <ul id="search-results"></ul>
    </div>
    <script>
        const searchConfig = {{
            indexPath: '{}',
            weights: {{ title: 10, content: 5, tags: 8, categories: 8 }},
            highlight: true,
            liveSearch: true
        }};

        let searchIndex = null;
        let searchTimeout = null;

        async function loadSearchIndex() {{
            try {{
                const response = await fetch(searchConfig.indexPath);
                if (!response.ok) throw new Error(`Failed to load search index: ${{response.status}}`);
                searchIndex = await response.json();
                console.log('Loaded', searchIndex.length, 'records');
            }} catch (error) {{
                console.error('Error loading search index:', error);
                throw error;
            }}
        }}

        function filterAndScoreResults(query) {{
            if (!Array.isArray(searchIndex)) {{
                console.error('Search index is not an array:', searchIndex);
                return [];
            }}

            const results = [];
            const terms = query.toLowerCase().split(/\s+/);

            try {{
                for (const item of searchIndex) {{
                    let score = 0;
                    const titleLower = (item.title || '').toLowerCase();
                    const contentLower = (item.content || '').toLowerCase();
                    const tags = (item.tags || []).map(t => t.toLowerCase());
                    const categories = (item.categories || []).map(c => c.toLowerCase());

                    for (const term of terms) {{
                        // 标题匹配
                        if (titleLower.includes(term)) {{
                            score += searchConfig.weights.title;
                        }}

                        // 内容匹配
                        if (contentLower.includes(term)) {{
                            score += searchConfig.weights.content;
                        }}

                        // 标签匹配
                        if (tags.some(tag => tag.includes(term))) {{
                            score += searchConfig.weights.tags;
                        }}

                        // 分类匹配
                        if (categories.some(cat => cat.includes(term))) {{
                            score += searchConfig.weights.categories;
                        }}
                    }}

                    if (score > 0) {{
                        results.push({{ ...item, score }});
                    }}
                }}
            }} catch (error) {{
                console.error('Error processing search:', error);
            }}

            return results.sort((a, b) => b.score - a.score);
        }}

        function highlightText(text, query) {{
            if (!text || !query) return text || '';
            const terms = query.toLowerCase().split(/\s+/);
            let highlighted = text;

            for (const term of terms) {{
                const regex = new RegExp(`(${{term}})`, 'gi');
                highlighted = highlighted.replace(regex, '<mark>$1</mark>');
            }}

            return highlighted;
        }}

        function displayResults(results, query) {{
            const container = document.getElementById('search-results');
            container.innerHTML = '';

            if (results.length === 0) {{
                container.innerHTML = '<div class="no-results">未找到相关文章</div>';
                return;
            }}

            const html = results.map(item => {{
                try {{
                    let preview = item.preview || '';
                    if (searchConfig.highlight) {{
                        preview = highlightText(preview, query);
                    }}

                    return `
                        <div class="search-result">
                            <h3 class="result-title">
                                <a href="/${{item.path}}">${{searchConfig.highlight ? highlightText(item.title, query) : item.title}}</a>
                            </h3>
                            <div class="result-meta">
                                <span class="date">${{item.date}}</span>
                                ${{item.categories && item.categories.length ? 
                                    `<span class="categories">分类：${{item.categories.join(', ')}}</span>` : ''}}
                                ${{item.tags && item.tags.length ? 
                                    `<span class="tags">标签：${{item.tags.join(', ')}}</span>` : ''}}
                            </div>
                            <div class="result-preview">${{preview}}</div>
                        </div>`;
                }} catch (error) {{
                    console.error('Error displaying result:', error);
                    return '<div class="search-error">显示搜索结果时发生错误</div>';
                }}
            }}).join('');

            container.innerHTML = html;
        }}

        async function search() {{
            const query = document.getElementById('search-input').value.trim();
            if (!query) {{
                document.getElementById('search-results').innerHTML = '';
                return;
            }}

            try {{
                if (!searchIndex) await loadSearchIndex();
                const results = filterAndScoreResults(query);
                console.log('Found', results.length, 'results');
                displayResults(results, query);
            }} catch (error) {{
                console.error('Search error:', error);
                document.getElementById('search-results').innerHTML = 
                    '<div class="search-error">搜索时发生错误，请稍后重试</div>';
            }}
        }}

        // 页面加载时自动执行搜索（如果URL中有查询参数）
        window.addEventListener('DOMContentLoaded', () => {{
            const urlParams = new URLSearchParams(window.location.search);
            const query = urlParams.get('q');
            if (query) {{
                document.getElementById('search-input').value = query;
                search();
            }}
        }});

        // 根据配置决定是使用实时搜索还是回车搜索
        if (searchConfig.liveSearch) {{
            document.getElementById('search-input').addEventListener('input', () => {{
                clearTimeout(searchTimeout);
                searchTimeout = setTimeout(search, 300);
            }});
        }} else {{
            document.getElementById('search-input').addEventListener('keypress', e => {{
                if (e.key === 'Enter') search();
            }});
        }}

        // 初始加载搜索索引
        loadSearchIndex().catch(error => {{
            console.error('Failed to load search index:', error);
        }});
    </script>
</body>
</html>"#,
            self.config.index_path
        );

        let search_page_path = self.output_dir.join("search/index.html");
        std::fs::create_dir_all(search_page_path.parent().unwrap())?;
        std::fs::write(search_page_path, search_html)?;

        Ok(())
    }

    /// 生成搜索脚本
    fn generate_search_script(&self) -> Result<()> {
        let search_js = r#"
async function search() {
    const input = document.getElementById('search-input');
    const query = input.value.toLowerCase();
    const resultsContainer = document.getElementById('search-results');

    if (!query) {
        resultsContainer.innerHTML = '';
        return;
    }

    try {
        console.log('开始搜索，搜索路径:', searchConfig.indexPath);
        const response = await fetch(searchConfig.indexPath);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const index = await response.json();
        console.log('成功获取搜索索引:', index.length, '条记录');
        
        const results = filterAndScoreResults(index, query);
        console.log('搜索结果:', results.length, '条记录');
        displayResults(results, query);
    } catch (error) {
        console.error('搜索出错:', error);
        resultsContainer.innerHTML = `<div class="search-error">搜索出错: ${error.message}</div>`;
    }
}

function filterAndScoreResults(index, query) {
    if (!Array.isArray(index)) {
        console.error('搜索索引格式错误:', index);
        return [];
    }
    
    return index
        .map(item => {
        let score = 0;
            
            try {
                const titleLower = (item.title || '').toLowerCase();
                const contentLower = (item.content || '').toLowerCase();
                const tags = Array.isArray(item.tags) ? item.tags : [];
                const categories = Array.isArray(item.categories) ? item.categories : [];
                
                // 标题匹配
                if (titleLower.includes(query)) {
            score += searchConfig.weights.title;
        }
                
                // 内容匹配
                if (contentLower.includes(query)) {
            score += searchConfig.weights.content;
        }
                
                // 标签匹配
                if (tags.some(tag => tag.toLowerCase().includes(query))) {
            score += searchConfig.weights.tags;
        }
                
                // 分类匹配
                if (categories.some(cat => cat.toLowerCase().includes(query))) {
            score += searchConfig.weights.categories;
        }
            } catch (error) {
                console.error('处理搜索项时出错:', error, item);
            }
            
        return { ...item, score };
        })
        .filter(item => item.score > 0)
        .sort((a, b) => b.score - a.score);
}

function displayResults(results, query) {
    const resultsContainer = document.getElementById('search-results');
    
    if (results.length === 0) {
        resultsContainer.innerHTML = '<div class="no-results">未找到相关文章</div>';
        return;
    }

    const html = results.map(item => {
        try {
            let preview = item.preview || '';
        if (searchConfig.highlight) {
            preview = highlightText(preview, query);
        }

        return `
            <div class="search-result">
                    <h3 class="result-title">
                        <a href="/${item.path}">${searchConfig.highlight ? highlightText(item.title, query) : item.title}</a>
                    </h3>
                    <div class="result-meta">
                    <span class="date">${item.date}</span>
                        ${item.categories && item.categories.length ? `<span class="categories">分类：${item.categories.join(', ')}</span>` : ''}
                        ${item.tags && item.tags.length ? `<span class="tags">标签：${item.tags.join(', ')}</span>` : ''}
                </div>
                    <div class="result-preview">${preview}</div>
                </div>`;
        } catch (error) {
            console.error('Error displaying result:', error);
            return '<div class="search-error">显示搜索结果时发生错误</div>';
        }
    }).join('');

    resultsContainer.innerHTML = html;
}

if (searchConfig.liveSearch) {
    document.getElementById('search-input').addEventListener('input', () => {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(search, 300);
    });
} else {
    document.getElementById('search-input').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            search();
        }
    });
}

// 初始加载搜索索引
loadSearchIndex().catch(error => {
    console.error('Failed to load search index:', error);
});
"#;

        let search_js_path = self.output_dir.join("search/search.js");
        std::fs::create_dir_all(search_js_path.parent().unwrap())?;
        std::fs::write(search_js_path, search_js)?;

        Ok(())
    }
}

impl Plugin for SearchPlugin {
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
        self.output_dir = context.output_dir.clone();
        let mut new_context = PluginContext::default();
        new_context.output_dir = context.output_dir.clone();
        new_context.base_dir = context.base_dir.clone();
        new_context.plugins_dir = context.plugins_dir.clone();
        new_context.theme_dir = context.theme_dir.clone();
        new_context.base_url = context.base_url.clone();
        new_context.config = context.config.clone();
        new_context.posts = context.posts.clone();
        new_context.pages = context.pages.clone();
        new_context.categories = context.categories.clone();
        new_context.tags = context.tags.clone();
        new_context.current_post = context.current_post.clone();
        new_context.current_page = context.current_page.clone();
        self.context = Arc::new(RwLock::new(new_context));
        Ok(())
    }

    fn execute_hook(&self, hook: &PluginHook) -> Result<()> {
        match hook {
            PluginHook::AfterGenerate => {
                let context = self.context.read().unwrap();
                    self.generate_search_index(&context.posts)?;
                    self.generate_search_page()?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn process_content(&self, content: &str, _content_type: ContentType) -> Result<String> {
        Ok(content.to_string())
    }

    fn cleanup(&self) -> Result<()> {
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(SearchPlugin::new()))
} 
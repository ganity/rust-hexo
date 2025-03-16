# Rust-Hexo

A static blog generator inspired by Hexo, written in Rust.

## Features

- **Speed & Efficiency**: Built with Rust for high performance and small memory footprint
- **Markdown Support**: First-class support for Markdown content
- **Flexible Theming**: Customizable theme system based on Tera templates
- **Plugin System**: Extendable functionality through a dynamic plugin system
- **Built-in Search**: Fast full-text search capabilities
- **Syntax Highlighting**: Code syntax highlighting support
- **Math Expressions**: LaTeX math rendering support via KaTeX or MathJax
- **Comments Integration**: Support for comment systems like Giscus and Disqus

## Installation

### Prerequisites

- Rust (1.70 or newer)

### Installation from Source

```bash
# Clone the repository
git clone https://github.com/your-username/rust-hexo.git
cd rust-hexo

# Build the project
cargo build --release

# The executable will be in target/release/rust-hexo
```

## Quick Start

### Create a New Blog

```bash
# Create a new blog
./rust-hexo init my-blog

# Change to the blog directory
cd my-blog
```

### Create a New Post

```bash
# Create a new post
./rust-hexo new "My First Post"
```

### Generate Static Files

```bash
# Generate static files
./rust-hexo generate
```

### Start Local Server

```bash
# Start a local server
./rust-hexo server
```

## Documentation

- [User Guide](docs/user-guide_en.md)
- [Theme Development](docs/theme-development_en.md)
- [Plugin Development](docs/plugin-development_en.md)
- [API Reference](docs/api-reference_en.md)

## Plugin System

Rust-Hexo features a powerful plugin system that allows you to extend functionality. Plugins are implemented as dynamic libraries (.so, .dll, or .dylib files).

### Creating Plugins

You can create plugins to add new features to your blog:

1. Create a new Rust crate with the crate-type set to "cdylib"
2. Implement the `Plugin` trait
3. Export a `create_plugin` function
4. Compile and place the resulting dynamic library in your blog's plugins directory

For detailed instructions, see the [Plugin Development Guide](docs/plugin-development_en.md).

### Built-in Plugins

Rust-Hexo comes with several built-in plugins:

- **Word Count**: Counts words in posts and estimates reading time
- **Syntax Highlighting**: Provides code syntax highlighting
- **Math**: Renders mathematical expressions using KaTeX or MathJax
- **Search**: Enables full-text search functionality
- **Comments**: Integrates with comment systems like Giscus and Disqus

## Themes

Rust-Hexo supports customizable themes, allowing you to change the appearance of your blog. Themes are based on the Tera templating engine.

### Default Theme

Rust-Hexo includes a default theme that provides a clean, modern design. You can use this theme as-is or as a starting point for your own custom theme.

### Custom Themes

To create a custom theme:

1. Create a new directory in the `themes` folder
2. Add the necessary template files (layout.html, index.html, post.html, etc.)
3. Add styles and assets
4. Update your `_config.yml` to use your new theme

For detailed instructions, see the [Theme Development Guide](docs/theme-development_en.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgements

- Inspired by [Hexo](https://hexo.io/)
- Built with [Rust](https://www.rust-lang.org/)
- Uses [Tera](https://tera.netlify.app/) for templating 
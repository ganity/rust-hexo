use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use tracing::error;
use tracing_subscriber::fmt;

mod cli;
mod core;
mod models;
mod theme;
mod plugins;
mod extensions;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    fmt()
        .with_target(false)
        .init();
    
    // 解析命令行参数
    let cli = cli::Cli::parse();
    
    // 打印欢迎信息
    println!("{}", "
 ____           _     _   _                
|  _ \\ _   _ __| |_  | | | | _____  _____  
| |_) | | | / _` __| | |_| |/ _ \\ \\/ / _ \\ 
|  _ <| |_| \\__ \\ |_  |  _  |  __/>  < (_) |
|_| \\_\\\\__,_|___/\\__| |_| |_|\\___/_/\\_\\___/ 
                                           
    ".bright_cyan());
    
    println!("{} {}", "Rust-Hexo".bright_cyan(), env!("CARGO_PKG_VERSION").bright_green());
    println!("{}", "A static blog generator inspired by Hexo".bright_white());
    println!();
    
    // 执行命令
    if let Err(e) = cli::execute(cli).await {
        error!("Error: {}", e);
        
        // 打印错误链
        let mut source = e.source();
        while let Some(e) = source {
            error!("Caused by: {}", e);
            source = e.source();
        }
        
        std::process::exit(1);
    }
    
    Ok(())
}

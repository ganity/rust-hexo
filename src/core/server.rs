use anyhow::Result;
use axum::{
    http::StatusCode,
    routing::get_service,
    Router,
};
use std::path::PathBuf;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tracing::info;

/// HTTP 服务器
pub struct Server {
    /// 站点目录
    public_dir: PathBuf,
    /// 端口
    port: u16,
    /// 文件更改通知通道
    tx: broadcast::Sender<()>,
}

impl Server {
    /// 创建新的服务器
    pub fn new(public_dir: PathBuf, port: u16) -> Self {
        let (tx, _) = broadcast::channel(10);
        Self {
            public_dir,
            port,
            tx,
        }
    }
    
    /// 文件更改事件发送器
    pub fn get_sender(&self) -> broadcast::Sender<()> {
        self.tx.clone()
    }
    
    /// 启动服务器
    pub async fn start(self) -> Result<()> {
        let public_dir = self.public_dir.clone();
        
        // 创建提供静态文件的服务
        let serve_dir = get_service(ServeDir::new(public_dir))
            .handle_error(|_| async move {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            });
        
        // 创建路由
        let app = Router::new()
            .fallback_service(serve_dir);
        
        // 启动HTTP服务器
        let addr: SocketAddr = format!("0.0.0.0:{}", self.port).parse()?;
        info!("Server started at http://localhost:{}", self.port);
        
        // 修改：使用 axum 新的方式启动服务器
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
} 
//! Web 服务器配置示例
//!
//! 这个示例展示了如何使用 Lingo 配置一个完整的 Web 服务器，包括：
//! - 服务器基本配置（主机、端口、工作线程等）
//! - TLS/SSL 配置
//! - CORS 配置
//! - 日志配置
//! - 嵌套配置结构

use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use lingo::Config;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, net::SocketAddr};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Config, Serialize, Deserialize, Debug, Default)]
struct ServerConfig {
    /// 服务器基本配置
    server: HttpServerConfig,

    /// TLS 配置
    tls: TlsConfig,

    /// CORS 配置
    cors: CorsConfig,

    /// 日志配置
    logging: LoggingConfig,

    /// API 配置
    api: ApiConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct HttpServerConfig {
    /// 服务器监听地址
    host: String,

    /// 服务器监听端口
    port: u16,

    /// 工作线程数
    workers: usize,

    /// 请求超时时间（秒）
    timeout: u64,

    /// 最大请求体大小（MB）
    max_body_size: usize,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 4,
            timeout: 30,
            max_body_size: 16,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TlsConfig {
    /// 启用 TLS
    enabled: Option<bool>,

    /// TLS 证书文件路径
    cert_file: Option<String>,

    /// TLS 私钥文件路径
    key_file: Option<String>,

    /// TLS 版本
    min_version: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: Some(false),
            cert_file: None,
            key_file: None,
            min_version: "1.2".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CorsConfig {
    /// 启用 CORS
    enabled: bool,

    /// 允许的源
    allowed_origins: String,

    /// 允许的方法
    allowed_methods: String,

    /// 允许的头部
    allowed_headers: String,

    /// 预检请求缓存时间（秒）
    max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: "*".to_string(),
            allowed_methods: "GET,POST,PUT,DELETE,OPTIONS".to_string(),
            allowed_headers: "*".to_string(),
            max_age: 3600,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct LoggingConfig {
    /// 日志级别
    level: String,

    /// 日志格式
    format: String,

    /// 启用访问日志
    access_log: bool,

    /// 日志文件路径
    file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            access_log: true,
            file: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiConfig {
    /// API 版本
    version: String,

    /// API 前缀
    prefix: String,

    /// 启用 API 文档
    docs_enabled: bool,

    /// 速率限制（每分钟请求数）
    rate_limit: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            version: "v1".to_string(),
            prefix: "/api".to_string(),
            docs_enabled: true,
            rate_limit: 100,
        }
    }
}

// API 响应类型
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

#[derive(Serialize)]
struct ServerInfo {
    name: String,
    version: String,
    uptime: String,
    config_summary: ConfigSummary,
}

#[derive(Serialize)]
struct ConfigSummary {
    host: String,
    port: u16,
    tls_enabled: bool,
    cors_enabled: bool,
    log_level: String,
    api_version: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Lingo Web 服务器配置示例 ===");
    println!();

    // 加载配置
    println!("正在加载服务器配置...");
    let config = ServerConfig::new();

    // 初始化日志
    init_logging(&config.logging)?;

    info!("服务器配置加载完成");
    info!("配置详情: {:#?}", config);

    // 构建应用路由
    let app = create_app(&config).await?;

    // 启动服务器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let socket_addr: SocketAddr = addr.parse()?;

    info!("服务器启动中...");
    info!("监听地址: {}", addr);
    info!("TLS 启用: {}", config.tls.enabled.unwrap_or(false));
    info!("CORS 启用: {}", config.cors.enabled);
    info!("API 前缀: {}/{}", config.api.prefix, config.api.version);

    println!();
    println!("🚀 服务器已启动！");
    println!("📍 地址: http://{}", addr);
    println!("📖 API 文档: http://{}/api/v1/docs", addr);
    println!("ℹ️  服务器信息: http://{}/api/v1/info", addr);
    println!("🔧 配置信息: http://{}/api/v1/config", addr);
    println!();
    println!("按 Ctrl+C 停止服务器");

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(socket_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_logging(config: &LoggingConfig) -> Result<(), Box<dyn Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.level));

    match config.format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
    }

    Ok(())
}

async fn create_app(config: &ServerConfig) -> Result<Router, Box<dyn Error>> {
    let mut app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route(&format!("{}/{}/info", config.api.prefix, config.api.version), get(info_handler))
        .route(&format!("{}/{}/config", config.api.prefix, config.api.version), get(config_handler))
        .route(&format!("{}/{}/echo", config.api.prefix, config.api.version), post(echo_handler));

    if config.api.docs_enabled {
        app = app.route(&format!("{}/{}/docs", config.api.prefix, config.api.version), get(docs_handler));
    }

    // 添加中间件
    if config.logging.access_log {
        app = app.layer(TraceLayer::new_for_http());
    }

    if config.cors.enabled {
        app = app.layer(CorsLayer::permissive()); // 简化的 CORS 配置
    }

    Ok(app)
}

// 路由处理器
async fn root_handler() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse {
        success: true,
        data: Some("Welcome to Lingo Web Server Example!"),
        message: "Server is running".to_string(),
    })
}

async fn health_handler() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse {
        success: true,
        data: Some("healthy"),
        message: "Server is healthy".to_string(),
    })
}

async fn info_handler() -> Json<ApiResponse<ServerInfo>> {
    let config = ServerConfig::new(); // 在实际应用中，这应该从应用状态中获取

    let info = ServerInfo {
        name: "Lingo Web Server Example".to_string(),
        version: "0.1.0".to_string(),
        uptime: "N/A".to_string(), // 在实际应用中计算运行时间
        config_summary: ConfigSummary {
            host: config.server.host,
            port: config.server.port,
            tls_enabled: config.tls.enabled.unwrap_or(false),
            cors_enabled: config.cors.enabled,
            log_level: config.logging.level,
            api_version: config.api.version,
        },
    };

    Json(ApiResponse {
        success: true,
        data: Some(info),
        message: "Server information retrieved".to_string(),
    })
}

async fn config_handler() -> Json<ApiResponse<ServerConfig>> {
    let config = ServerConfig::new();

    Json(ApiResponse {
        success: true,
        data: Some(config),
        message: "Configuration retrieved".to_string(),
    })
}

async fn echo_handler(Json(payload): Json<serde_json::Value>) -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse {
        success: true,
        data: Some(payload),
        message: "Echo successful".to_string(),
    })
}

async fn docs_handler() -> Json<ApiResponse<HashMap<&'static str, &'static str>>> {
    let mut docs = HashMap::new();
    docs.insert("GET /", "根路径，返回欢迎信息");
    docs.insert("GET /health", "健康检查端点");
    docs.insert("GET /api/v1/info", "获取服务器信息");
    docs.insert("GET /api/v1/config", "获取服务器配置");
    docs.insert("POST /api/v1/echo", "回显请求体");
    docs.insert("GET /api/v1/docs", "API 文档（当前页面）");

    Json(ApiResponse {
        success: true,
        data: Some(docs),
        message: "API documentation".to_string(),
    })
}
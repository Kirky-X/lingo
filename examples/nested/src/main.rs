//! # Lingo 复杂嵌套配置示例
//!
//! 这个示例展示了如何使用 Lingo 处理复杂的嵌套配置结构，包括：
//! - 嵌套的配置结构体
//! - 数组和向量配置
//! - 可选字段和默认值
//! - 复杂数据类型的配置

use lingo::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

/// 数据库配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct DatabaseConfig {
    /// 数据库主机
    host: String,
    /// 数据库端口
    port: u16,
    /// 数据库名称
    database: String,
    /// 用户名
    username: String,
    /// 密码
    password: String,
    /// 连接池大小
    pool_size: u32,
    /// 连接超时（秒）
    timeout: u64,
    /// SSL配置
    ssl: Option<SslConfig>,
}

/// SSL配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct SslConfig {
    /// 是否启用SSL
    enabled: bool,
    /// 证书路径
    cert_path: Option<String>,
    /// 私钥路径
    key_path: Option<String>,
    /// CA证书路径
    ca_path: Option<String>,
}

/// Redis配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct RedisConfig {
    /// Redis主机
    host: String,
    /// Redis端口
    port: u16,
    /// 数据库索引
    db: u8,
    /// 密码
    password: Option<String>,
    /// 连接池大小
    pool_size: u32,
}

/// 服务器配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ServerConfig {
    /// 监听地址
    host: String,
    /// 监听端口
    port: u16,
    /// 工作线程数
    workers: u32,
    /// 请求超时（秒）
    timeout: u64,
    /// 最大请求体大小（字节）
    max_body_size: u64,
    /// TLS配置
    tls: Option<TlsConfig>,
}

/// TLS配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct TlsConfig {
    /// 是否启用TLS
    enabled: bool,
    /// 证书文件路径
    cert_file: String,
    /// 私钥文件路径
    key_file: String,
}

/// 日志配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct LogConfig {
    /// 日志级别
    level: String,
    /// 日志格式 (json, text)
    format: String,
    /// 日志输出目标
    targets: Vec<LogTarget>,
    /// 日志轮转配置
    rotation: Option<LogRotation>,
}

/// 日志输出目标
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct LogTarget {
    /// 目标类型 (console, file, syslog)
    target_type: String,
    /// 文件路径（仅当type为file时）
    path: Option<String>,
    /// 最小日志级别
    min_level: Option<String>,
}

/// 日志轮转配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct LogRotation {
    /// 最大文件大小（MB）
    max_size: u64,
    /// 保留文件数量
    max_files: u32,
    /// 是否压缩旧文件
    compress: bool,
}

/// 功能特性配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct FeatureConfig {
    /// 启用的功能列表
    enabled: Vec<String>,
    /// 功能特定配置
    settings: HashMap<String, serde_json::Value>,
}

/// 主应用配置
#[derive(Config, Serialize, Deserialize, Debug)]
struct AppConfig {
    /// 应用程序名称
    name: String,
    /// 应用程序版本
    version: String,
    /// 环境 (development, staging, production)
    environment: String,
    /// 调试模式
    debug: bool,
    
    /// 服务器配置
    server: ServerConfig,
    /// 数据库配置
    database: DatabaseConfig,
    /// Redis配置
    redis: Option<RedisConfig>,
    /// 日志配置
    logging: LogConfig,
    /// 功能特性配置
    features: FeatureConfig,
    
    /// 外部服务配置
    external_services: HashMap<String, ExternalServiceConfig>,
    /// 监控配置
    monitoring: Option<MonitoringConfig>,
}

/// 外部服务配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ExternalServiceConfig {
    /// 服务URL
    url: String,
    /// API密钥
    api_key: Option<String>,
    /// 超时时间（秒）
    timeout: u64,
    /// 重试次数
    retries: u32,
    /// 自定义头部
    headers: HashMap<String, String>,
}

/// 监控配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct MonitoringConfig {
    /// 是否启用监控
    enabled: bool,
    /// 监控端点
    endpoint: String,
    /// 采样率
    sample_rate: f64,
    /// 标签
    tags: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut external_services = HashMap::new();
        external_services.insert(
            "payment_service".to_string(),
            ExternalServiceConfig {
                url: "https://api.payment.example.com".to_string(),
                api_key: None,
                timeout: 30,
                retries: 3,
                headers: HashMap::new(),
            },
        );
        
        let mut feature_settings = HashMap::new();
        feature_settings.insert(
            "rate_limiting".to_string(),
            serde_json::json!({
                "requests_per_minute": 1000,
                "burst_size": 100
            }),
        );
        
        let mut monitoring_tags = HashMap::new();
        monitoring_tags.insert("service".to_string(), "nested-example".to_string());
        monitoring_tags.insert("version".to_string(), "0.1.0".to_string());
        
        Self {
            name: "Nested Configuration Example".to_string(),
            version: "0.1.0".to_string(),
            environment: "development".to_string(),
            debug: true,
            
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: 4,
                timeout: 30,
                max_body_size: 1024 * 1024, // 1MB
                tls: None,
            },
            
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "app_db".to_string(),
                username: "app_user".to_string(),
                password: "password".to_string(),
                pool_size: 10,
                timeout: 30,
                ssl: None,
            },
            
            redis: Some(RedisConfig {
                host: "localhost".to_string(),
                port: 6379,
                db: 0,
                password: None,
                pool_size: 5,
            }),
            
            logging: LogConfig {
                level: "info".to_string(),
                format: "text".to_string(),
                targets: vec![
                    LogTarget {
                        target_type: "console".to_string(),
                        path: None,
                        min_level: Some("debug".to_string()),
                    },
                    LogTarget {
                        target_type: "file".to_string(),
                        path: Some("./logs/app.log".to_string()),
                        min_level: Some("info".to_string()),
                    },
                ],
                rotation: Some(LogRotation {
                    max_size: 100, // 100MB
                    max_files: 10,
                    compress: true,
                }),
            },
            
            features: FeatureConfig {
                enabled: vec![
                    "authentication".to_string(),
                    "rate_limiting".to_string(),
                    "caching".to_string(),
                ],
                settings: feature_settings,
            },
            
            external_services,
            
            monitoring: Some(MonitoringConfig {
                enabled: true,
                endpoint: "http://localhost:9090/metrics".to_string(),
                sample_rate: 0.1,
                tags: monitoring_tags,
            }),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Lingo 复杂嵌套配置示例 ===\n");
    
    // 加载配置
    println!("正在加载复杂嵌套配置...");
    let config = AppConfig::new();
    
    // 显示基本信息
    println!("配置加载完成！\n");
    println!("应用信息:");
    println!("  名称: {}", config.name);
    println!("  版本: {}", config.version);
    println!("  环境: {}", config.environment);
    println!("  调试模式: {}\n", config.debug);
    
    // 显示服务器配置
    println!("服务器配置:");
    println!("  地址: {}:{}", config.server.host, config.server.port);
    println!("  工作线程: {}", config.server.workers);
    println!("  请求超时: {}秒", config.server.timeout);
    println!("  最大请求体: {}字节", config.server.max_body_size);
    if let Some(ref tls) = config.server.tls {
        println!("  TLS: 启用 (证书: {}, 私钥: {})", tls.cert_file, tls.key_file);
    } else {
        println!("  TLS: 未启用");
    }
    println!();
    
    // 显示数据库配置
    println!("数据库配置:");
    println!("  主机: {}:{}", config.database.host, config.database.port);
    println!("  数据库: {}", config.database.database);
    println!("  用户名: {}", config.database.username);
    println!("  连接池大小: {}", config.database.pool_size);
    println!("  连接超时: {}秒", config.database.timeout);
    if let Some(ref ssl) = config.database.ssl {
        println!("  SSL: 启用 (证书: {:?})", ssl.cert_path);
    } else {
        println!("  SSL: 未启用");
    }
    println!();
    
    // 显示Redis配置
    if let Some(ref redis) = config.redis {
        println!("Redis配置:");
        println!("  主机: {}:{}", redis.host, redis.port);
        println!("  数据库: {}", redis.db);
        println!("  连接池大小: {}", redis.pool_size);
        if redis.password.is_some() {
            println!("  密码: 已设置");
        } else {
            println!("  密码: 未设置");
        }
        println!();
    }
    
    // 显示日志配置
    println!("日志配置:");
    println!("  级别: {}", config.logging.level);
    println!("  格式: {}", config.logging.format);
    println!("  输出目标:");
    for target in &config.logging.targets {
        match target.target_type.as_str() {
            "console" => println!("    - 控制台 (最小级别: {:?})", target.min_level),
            "file" => println!("    - 文件: {:?} (最小级别: {:?})", target.path, target.min_level),
            _ => println!("    - {}: {:?}", target.target_type, target.path),
        }
    }
    if let Some(ref rotation) = config.logging.rotation {
        println!("  轮转: 最大{}MB, 保留{}个文件, 压缩: {}", 
                rotation.max_size, rotation.max_files, rotation.compress);
    }
    println!();
    
    // 显示功能特性
    println!("功能特性:");
    println!("  启用的功能: {:?}", config.features.enabled);
    println!("  特性配置:");
    for (key, value) in &config.features.settings {
        println!("    {}: {}", key, value);
    }
    println!();
    
    // 显示外部服务
    println!("外部服务:");
    for (name, service) in &config.external_services {
        println!("  {}:", name);
        println!("    URL: {}", service.url);
        println!("    超时: {}秒", service.timeout);
        println!("    重试: {}次", service.retries);
        if service.api_key.is_some() {
            println!("    API密钥: 已设置");
        }
    }
    println!();
    
    // 显示监控配置
    if let Some(ref monitoring) = config.monitoring {
        println!("监控配置:");
        println!("  启用: {}", monitoring.enabled);
        println!("  端点: {}", monitoring.endpoint);
        println!("  采样率: {}", monitoring.sample_rate);
        println!("  标签: {:?}", monitoring.tags);
        println!();
    }
    
    // 显示配置来源说明
    println!("配置来源优先级 (从低到高):");
    println!("  1. 默认值 (代码中定义的复杂嵌套结构)");
    println!("  2. 配置文件: config.toml (支持完整的嵌套结构)");
    println!("  3. 环境变量: NESTED_* (支持嵌套路径，如 NESTED_SERVER_HOST)");
    println!("  4. 命令行参数: --server-host, --database-port 等\n");
    
    // 显示使用示例
    println!("使用示例:");
    println!("  # 使用环境变量设置嵌套配置");
    println!("  export NESTED_SERVER_HOST=0.0.0.0");
    println!("  export NESTED_SERVER_PORT=3000");
    println!("  export NESTED_DATABASE_HOST=db.example.com");
    println!("  export NESTED_DATABASE_PORT=5432");
    println!("  export NESTED_LOGGING_LEVEL=debug\n");
    
    println!("  # 使用命令行参数");
    println!("  cargo run -- --server-host 0.0.0.0 --server-port 3000 --debug\n");
    
    // 生成配置文件模板
    println!("生成配置文件模板:");
    println!("=== config.toml ===");
    let toml_str = toml::to_string_pretty(&config)?;
    println!("{}", toml_str);
    println!("==================\n");
    
    println!("[SUCCESS] 复杂嵌套配置示例运行完成！");
    
    Ok(())
}
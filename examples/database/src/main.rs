//! 数据库配置示例
//!
//! 这个示例展示了如何使用 Lingo 配置数据库连接，包括：
//! - 多种数据库类型支持（PostgreSQL、MySQL、SQLite）
//! - 连接池配置
//! - 连接超时和重试配置
//! - 数据库迁移配置
//! - SSL/TLS 配置
//! - 读写分离配置

use lingo::Config;
use serde::{Deserialize, Serialize};
use std::{error::Error, time::Duration};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Config, Serialize, Deserialize, Debug, Default)]
struct DatabaseConfig {
    /// 主数据库配置
    primary: DatabaseConnectionConfig,

    /// 只读副本数据库配置（可选）
    replica: Option<ReplicaConfig>,

    /// 连接池配置
    pool: ConnectionPoolConfig,

    /// 迁移配置
    migration: MigrationConfig,

    /// SSL/TLS 配置
    ssl: SslConfig,

    /// 监控和日志配置
    monitoring: MonitoringConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct DatabaseConnectionConfig {
    /// 数据库类型
    db_type: String,

    /// 数据库主机
    host: String,

    /// 数据库端口
    port: u16,

    /// 数据库名称
    database: String,

    /// 用户名
    username: String,

    /// 密码
    password: Option<String>,

    /// 连接超时（秒）
    connect_timeout: u64,

    /// 查询超时（秒）
    query_timeout: u64,

    /// 应用程序名称（用于连接标识）
    application_name: String,
}

impl Default for DatabaseConnectionConfig {
    fn default() -> Self {
        Self {
            db_type: "postgres".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "myapp".to_string(),
            username: "postgres".to_string(),
            password: None,
            connect_timeout: 30,
            query_timeout: 60,
            application_name: "lingo_app".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ReplicaConfig {
    /// 启用读写分离
    enabled: Option<bool>,

    /// 只读副本主机
    host: Option<String>,

    /// 只读副本端口
    port: Option<u16>,

    /// 只读副本数据库名
    database: Option<String>,

    /// 只读副本用户名
    username: Option<String>,

    /// 只读副本密码
    password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConnectionPoolConfig {
    /// 最小连接数
    min_connections: u32,

    /// 最大连接数
    max_connections: u32,

    /// 连接空闲超时（秒）
    idle_timeout: u64,

    /// 连接最大生命周期（秒）
    max_lifetime: u64,

    /// 获取连接超时（秒）
    acquire_timeout: u64,

    /// 连接测试查询
    test_query: String,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            idle_timeout: 600,
            max_lifetime: 3600,
            acquire_timeout: 30,
            test_query: "SELECT 1".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MigrationConfig {
    /// 启用自动迁移
    auto_migrate: bool,

    /// 迁移文件目录
    migrations_dir: String,

    /// 迁移表名
    migrations_table: String,

    /// 迁移超时（秒）
    migration_timeout: u64,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            auto_migrate: true,
            migrations_dir: "./migrations".to_string(),
            migrations_table: "_migrations".to_string(),
            migration_timeout: 300,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SslConfig {
    /// 启用 SSL
    enabled: Option<bool>,

    /// SSL 模式
    mode: String,

    /// CA 证书文件路径
    ca_cert_file: Option<String>,

    /// 客户端证书文件路径
    client_cert_file: Option<String>,

    /// 客户端私钥文件路径
    client_key_file: Option<String>,
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            enabled: None,
            mode: "prefer".to_string(),
            ca_cert_file: None,
            client_cert_file: None,
            client_key_file: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MonitoringConfig {
    /// 启用查询日志
    log_queries: bool,

    /// 慢查询阈值（毫秒）
    slow_query_threshold: u64,

    /// 启用连接池监控
    monitor_pool: bool,

    /// 监控间隔（秒）
    monitor_interval: u64,

    /// 启用性能指标
    enable_metrics: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            log_queries: false,
            slow_query_threshold: 1000,
            monitor_pool: true,
            monitor_interval: 60,
            enable_metrics: false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    println!("=== Lingo 数据库配置示例 ===");
    println!();

    // 加载配置
    info!("正在加载数据库配置...");
    let config = DatabaseConfig::new();

    info!("数据库配置加载完成");
    info!("配置详情: {:#?}", config);

    // 验证配置
    validate_config(&config)?;

    // 构建数据库连接字符串
    let connection_url = build_connection_url(&config.primary)?;
    info!("数据库连接 URL: {}", mask_password(&connection_url));

    // 模拟数据库连接测试
    test_database_connection(&config).await?;

    // 显示配置摘要
    display_config_summary(&config);

    // 模拟应用程序运行
    simulate_application_lifecycle(&config).await?;

    Ok(())
}

fn validate_config(config: &DatabaseConfig) -> Result<(), Box<dyn Error>> {
    info!("验证数据库配置...");

    // 验证数据库类型
    match config.primary.db_type.as_str() {
        "postgres" | "mysql" | "sqlite" => {}
        _ => {
            return Err(format!("不支持的数据库类型: {}", config.primary.db_type).into());
        }
    }

    // 验证连接池配置
    if config.pool.min_connections > config.pool.max_connections {
        return Err("最小连接数不能大于最大连接数".into());
    }

    // 验证超时配置
    if config.primary.connect_timeout == 0 {
        warn!("连接超时设置为 0，可能导致连接永久阻塞");
    }

    // 验证 SSL 配置
    if let Some(true) = config.ssl.enabled {
        if config.ssl.mode == "disable" {
            warn!("SSL 已启用但模式设置为 'disable'，这可能不是预期的配置");
        }
    }

    info!("配置验证通过");
    Ok(())
}

fn build_connection_url(config: &DatabaseConnectionConfig) -> Result<String, Box<dyn Error>> {
    let password = config.password.as_deref().unwrap_or("");

    let url = match config.db_type.as_str() {
        "postgres" => {
            format!(
                "postgresql://{}:{}@{}:{}/{}?application_name={}",
                config.username,
                password,
                config.host,
                config.port,
                config.database,
                config.application_name
            )
        }
        "mysql" => {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                config.username,
                password,
                config.host,
                config.port,
                config.database
            )
        }
        "sqlite" => {
            format!("sqlite://{}", config.database)
        }
        _ => return Err(format!("不支持的数据库类型: {}", config.db_type).into()),
    };

    Ok(url)
}

fn mask_password(url: &str) -> String {
    // 简单的密码掩码，实际应用中应该使用更安全的方法
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}

async fn test_database_connection(config: &DatabaseConfig) -> Result<(), Box<dyn Error>> {
    info!("测试数据库连接...");

    // 这里只是模拟连接测试，实际应用中会真正连接数据库
    tokio::time::sleep(Duration::from_millis(500)).await;

    match config.primary.db_type.as_str() {
        "postgres" => info!("PostgreSQL 连接测试成功"),
        "mysql" => info!("MySQL 连接测试成功"),
        "sqlite" => info!("SQLite 连接测试成功"),
        _ => error!("未知数据库类型: {}", config.primary.db_type),
    }

    // 测试只读副本连接（如果启用）
    if let Some(replica) = &config.replica {
        if replica.enabled.unwrap_or(false) {
            info!("测试只读副本连接...");
            tokio::time::sleep(Duration::from_millis(300)).await;
            info!("只读副本连接测试成功");
        }
    }

    Ok(())
}

fn display_config_summary(config: &DatabaseConfig) {
    println!();
    println!("📊 数据库配置摘要:");
    println!("  数据库类型: {}", config.primary.db_type);
    println!("  主机地址: {}:{}", config.primary.host, config.primary.port);
    println!("  数据库名: {}", config.primary.database);
    println!("  用户名: {}", config.primary.username);
    println!("  连接池: {}-{} 连接", config.pool.min_connections, config.pool.max_connections);
    println!("  SSL 启用: {}", config.ssl.enabled.unwrap_or(false));
    println!("  自动迁移: {}", config.migration.auto_migrate);

    if let Some(replica) = &config.replica {
        if replica.enabled.unwrap_or(false) {
            println!("  读写分离: 已启用");
            if let Some(host) = &replica.host {
                println!("  只读副本: {}", host);
            }
        }
    }

    println!("  查询日志: {}", config.monitoring.log_queries);
    println!("  慢查询阈值: {} ms", config.monitoring.slow_query_threshold);
    println!();
}

async fn simulate_application_lifecycle(config: &DatabaseConfig) -> Result<(), Box<dyn Error>> {
    info!("模拟应用程序生命周期...");

    // 模拟数据库迁移
    if config.migration.auto_migrate {
        info!("执行数据库迁移...");
        tokio::time::sleep(Duration::from_millis(1000)).await;
        info!("数据库迁移完成");
    }

    // 模拟连接池初始化
    info!("初始化连接池 ({}-{} 连接)...", 
          config.pool.min_connections, 
          config.pool.max_connections);
    tokio::time::sleep(Duration::from_millis(500)).await;
    info!("连接池初始化完成");

    // 模拟一些数据库操作
    info!("执行示例数据库操作...");

    // 模拟查询操作
    for i in 1..=3 {
        let query_time = 50 + i * 20;
        info!("执行查询 {} (耗时: {} ms)", i, query_time);

        if query_time > config.monitoring.slow_query_threshold {
            warn!("慢查询检测: 查询 {} 耗时 {} ms，超过阈值 {} ms", 
                  i, query_time, config.monitoring.slow_query_threshold);
        }

        tokio::time::sleep(Duration::from_millis(query_time)).await;
    }

    // 模拟连接池监控
    if config.monitoring.monitor_pool {
        info!("连接池状态: 活跃连接 3/{}, 空闲连接 {}", 
              config.pool.max_connections,
              config.pool.max_connections - 3);
    }

    info!("应用程序生命周期模拟完成");
    Ok(())
}
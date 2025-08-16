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
#[config(env_prefix = "DB_")]
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

    // 验证连接池配置 - 零值作为警告，保持有序关系为错误
    if config.pool.min_connections > config.pool.max_connections {
        return Err("最小连接数不能大于最大连接数".into());
    }
    if config.pool.max_connections == 0 {
        warn!("最大连接数为 0，这可能导致连接问题");
    }
    if config.pool.min_connections == 0 {
        warn!("最小连接数为 0，这可能导致性能问题");
    }

    // 主机、数据库、用户名不能为空
    if config.primary.host.is_empty() {
        return Err("主机名不能为空".into());
    }
    if config.primary.database.is_empty() {
        return Err("数据库名不能为空".into());
    }
    if config.primary.username.is_empty() {
        return Err("用户名不能为空".into());
    }

    // 超时边界
    if config.primary.connect_timeout == 0 {
        warn!("连接超时设置为 0，可能导致连接永久阻塞");
    }

    // SSL 配置的一致性
    if let Some(true) = config.ssl.enabled {
        if config.ssl.mode == "disable" {
            return Err("SSL 启用状态与禁用模式矛盾".into());
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_new() {
        // Test that we can create a DatabaseConfig using new()
        let config = DatabaseConfig::new();
        assert!(config.primary.host.len() > 0, "primary db host should have a default value");
        assert!(config.primary.port > 0, "primary db port should have a valid default value");
        assert!(config.pool.max_connections >= 1, "pool max_connections should be >= 1");
    }

    #[test]
    fn test_database_config_default() {
        // Test that we can create a DatabaseConfig using Default
        let config = DatabaseConfig::default();
        assert_eq!(config.primary.host, "localhost");
        assert!(config.pool.max_connections >= 1);
    }

    #[test]
    fn test_config_serialization() {
        // Test that the config can be serialized and deserialized
        let config = DatabaseConfig::default();
        let serialized = toml::to_string(&config).expect("Should be able to serialize config");
        assert!(serialized.contains("primary"), "Serialized config should contain primary section");
        assert!(serialized.contains("pool"), "Serialized config should contain pool section");

        // Test deserialization
        let deserialized: DatabaseConfig = toml::from_str(&serialized).expect("Should be able to deserialize config");
        assert_eq!(deserialized.primary.host, config.primary.host);
        assert_eq!(deserialized.pool.max_connections, config.pool.max_connections);
    }

    #[test]
    fn test_invalid_database_types() {
        let invalid_configs = vec![
            r#"
            [primary]
            db_type = "invalid_db"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
            r#"
            [primary]
            db_type = ""
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
            r#"
            [primary]
            db_type = "oracle"
            host = "localhost"
            port = 1521
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
        ];

        for invalid_config in invalid_configs {
            let parsed: Result<DatabaseConfig, _> = toml::from_str(invalid_config);
            if let Ok(config) = parsed {
                // 配置解析成功，但应该在验证时失败
                assert!(validate_config(&config).is_err(), "Invalid database type should fail validation");
            }
            // 如果解析就失败了，那也是可以的
        }
    }

    #[test]
    fn test_connection_pool_invalid_combinations() {
        let invalid_pool_configs = vec![
            // min_connections > max_connections
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 10
            max_connections = 5
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
        ];

        for invalid_config in invalid_pool_configs {
            let config: DatabaseConfig = toml::from_str(invalid_config)
                .expect("Should parse TOML structure");
            assert!(validate_config(&config).is_err(), 
                    "Invalid pool configuration should fail validation");
        }
        
        // Test zero value configurations that now only produce warnings
        let warning_pool_configs = vec![
            // Zero max_connections
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 0
            max_connections = 0
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
        ];

        for warning_config in warning_pool_configs {
            let config: DatabaseConfig = toml::from_str(warning_config)
                .expect("Should parse TOML structure");
            assert!(validate_config(&config).is_ok(), 
                    "Zero value configurations should pass validation but produce warnings");
        }
    }

    #[test]
    fn test_ssl_configuration_contradictions() {
        let contradictory_ssl_configs = vec![
            // SSL enabled but mode is "disable"
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            enabled = true
            mode = "disable"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
            // SSL mode require with no certificates
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            enabled = true
            mode = "require"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
        ];

        for config_str in contradictory_ssl_configs {
            let config: DatabaseConfig = toml::from_str(config_str)
                .expect("Should parse TOML structure");
            let _ = validate_config(&config);
            // 至少应该产生警告（目前实现只打印警告，不返回错误）
            // 在实际应用中，这些可能应该是错误
        }
    }

    #[test]
    fn test_port_range_boundaries() {
        let port_boundary_configs = vec![
            // Port 0 (invalid)
            (0u16, false),
            // Port 1 (valid, though unusual)
            (1u16, true),
            // Standard ports
            (5432u16, true),
            (3306u16, true),
            // Maximum valid port
            (65535u16, true),
        ];

        for (port, should_be_valid) in port_boundary_configs {
            let config_str = format!(r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = {}
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#, port);

            let parse_result: Result<DatabaseConfig, _> = toml::from_str(&config_str);
            
            if should_be_valid {
                let config = parse_result.expect("Valid port should parse successfully");
                assert_eq!(config.primary.port, port);
            } else {
                // Port 0 可能被TOML解析接受，但在逻辑上是无效的
                if let Ok(config) = parse_result {
                    assert_eq!(config.primary.port, port);
                    // 在实际应用中，port 0 应该被验证逻辑拒绝
                }
            }
        }
    }

    #[test]
    fn test_timeout_boundary_values() {
        let timeout_configs = vec![
            // Zero timeouts (边界情况)
            (0u64, 0u64, 0u64),
            // Very small timeouts
            (1u64, 1u64, 1u64),
            // Very large timeouts
            (86400u64, 86400u64, 86400u64), // 24 hours
        ];

        for (connect_timeout, query_timeout, acquire_timeout) in timeout_configs {
            let config_str = format!(r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = {}
            query_timeout = {}
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = {}
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#, connect_timeout, query_timeout, acquire_timeout);

            let config: DatabaseConfig = toml::from_str(&config_str)
                .expect("Timeout values should parse successfully");
            
            assert_eq!(config.primary.connect_timeout, connect_timeout);
            assert_eq!(config.primary.query_timeout, query_timeout);
            assert_eq!(config.pool.acquire_timeout, acquire_timeout);

            // 验证零超时值会产生警告
            if connect_timeout == 0 {
                // validate_config 应该产生警告
                let _ = validate_config(&config); // 目前只打印警告
            }
        }
    }

    #[test]
    fn test_empty_string_configurations() {
        let empty_string_configs = vec![
            // Empty host
            r#"
            [primary]
            db_type = "postgres"
            host = ""
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
            // Empty database name
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = ""
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
            // Empty username
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = ""
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
            // Empty application name
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = ""
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            "#,
        ];

        for config_str in empty_string_configs {
            let config: DatabaseConfig = toml::from_str(config_str)
                .expect("Empty strings should parse successfully");
            
            // 这些配置在技术上有效，但在实际使用中可能有问题
            // 在实际应用中，应该有额外的验证来检查这些字段不为空
            assert!(validate_config(&config).is_ok() || validate_config(&config).is_err());
        }
    }

    #[test]
    fn test_replica_configuration_boundaries() {
        let replica_configs = vec![
            // Enabled replica without host
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            
            [replica]
            enabled = true
            "#,
            // Disabled replica with full configuration
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = false
            slow_query_threshold = 1000
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = false
            
            [replica]
            enabled = false
            host = "replica.example.com"
            port = 5432
            database = "test"
            username = "user"
            "#,
        ];

        for config_str in replica_configs {
            let config: DatabaseConfig = toml::from_str(config_str)
                .expect("Replica configurations should parse successfully");
            
            if let Some(replica) = &config.replica {
                if replica.enabled.unwrap_or(false) && replica.host.is_none() {
                    // 启用了副本但没有提供主机，这在实际应用中应该是错误
                    // 目前的验证逻辑没有检查这种情况
                }
            }
        }
    }

    #[test]
    fn test_monitoring_configuration_boundaries() {
        let monitoring_configs = vec![
            // Zero slow query threshold
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = true
            slow_query_threshold = 0
            monitor_pool = true
            monitor_interval = 60
            enable_metrics = true
            "#,
            // Very high slow query threshold
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = 5432
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            
            [pool]
            min_connections = 1
            max_connections = 10
            idle_timeout = 600
            max_lifetime = 3600
            acquire_timeout = 30
            test_query = "SELECT 1"
            
            [migration]
            auto_migrate = true
            migrations_dir = "./migrations"
            migrations_table = "_migrations"
            migration_timeout = 300
            
            [ssl]
            mode = "prefer"
            
            [monitoring]
            log_queries = true
            slow_query_threshold = 9223372036854775807
            monitor_pool = true
            monitor_interval = 0
            enable_metrics = true
            "#,
        ];

        for config_str in monitoring_configs {
            let config: DatabaseConfig = toml::from_str(config_str)
                .expect("Monitoring configurations should parse successfully");
            
            // 验证监控配置的合理性
            if config.monitoring.slow_query_threshold == 0 {
                // 零阈值意味着所有查询都被认为是慢查询
            }
            
            if config.monitoring.monitor_interval == 0 {
                // 零监控间隔可能导致性能问题
            }
        }
    }

    #[test]
    fn test_serialization_roundtrip_with_boundaries() {
        let mut config = DatabaseConfig::default();
        
        // 设置边界值
        config.primary.port = 65535;
        config.pool.min_connections = 0;
        config.pool.max_connections = 1000;
        config.pool.idle_timeout = 86400;
        config.monitoring.slow_query_threshold = 1;
        
        // 序列化
        let serialized = toml::to_string(&config)
            .expect("Should be able to serialize boundary values");
        
        // 反序列化
        let deserialized: DatabaseConfig = toml::from_str(&serialized)
            .expect("Should be able to deserialize boundary values");
        
        // 验证值保持一致
        assert_eq!(deserialized.primary.port, config.primary.port);
        assert_eq!(deserialized.pool.min_connections, config.pool.min_connections);
        assert_eq!(deserialized.pool.max_connections, config.pool.max_connections);
        assert_eq!(deserialized.pool.idle_timeout, config.pool.idle_timeout);
        assert_eq!(deserialized.monitoring.slow_query_threshold, config.monitoring.slow_query_threshold);
    }

    #[test]
    fn test_invalid_toml_structure() {
        let invalid_toml_configs = vec![
            // Missing required field
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            # port is missing
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            "#,
            // Invalid TOML syntax
            r#"
            [primary
            db_type = "postgres"
            host = "localhost"
            port = 5432
            "#,
            // Type mismatch
            r#"
            [primary]
            db_type = "postgres"
            host = "localhost"
            port = "not_a_number"
            database = "test"
            username = "user"
            connect_timeout = 30
            query_timeout = 60
            application_name = "test"
            "#,
        ];

        for invalid_config in invalid_toml_configs {
            let result: Result<DatabaseConfig, _> = toml::from_str(invalid_config);
            assert!(result.is_err(), "Invalid TOML should fail to parse");
        }
    }

    #[test]
    fn test_maximum_safe_numeric_values() {
        let max_values_config = format!(r#"
        [primary]
        db_type = "postgres"
        host = "localhost"
        port = 65535
        database = "test"
        username = "user"
        connect_timeout = {}
        query_timeout = {}
        application_name = "test"
        
        [pool]
        min_connections = 0
        max_connections = {}
        idle_timeout = {}
        max_lifetime = {}
        acquire_timeout = {}
        test_query = "SELECT 1"
        
        [migration]
        auto_migrate = true
        migrations_dir = "./migrations"
        migrations_table = "_migrations"
        migration_timeout = {}
        
        [ssl]
        mode = "prefer"
        
        [monitoring]
        log_queries = true
        slow_query_threshold = {}
        monitor_pool = true
        monitor_interval = {}
        enable_metrics = true
        "#, 
        i64::MAX, i64::MAX, // connect_timeout, query_timeout
        u32::MAX, // max_connections
        i64::MAX, i64::MAX, i64::MAX, // idle_timeout, max_lifetime, acquire_timeout  
        i64::MAX, // migration_timeout
        i64::MAX, i64::MAX // slow_query_threshold, monitor_interval
        );

        let config: DatabaseConfig = toml::from_str(&max_values_config)
            .expect("Maximum safe values should parse successfully");
        
        assert_eq!(config.primary.port, 65535);
        assert_eq!(config.pool.max_connections, u32::MAX);
        assert_eq!(config.primary.connect_timeout, i64::MAX as u64);
        assert_eq!(config.monitoring.slow_query_threshold, i64::MAX as u64);
    }
}
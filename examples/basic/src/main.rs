//! 基本配置加载示例
//!
//! 这个示例展示了如何使用 Quantum Config 进行基本的配置管理，包括：
//! - 从配置文件加载配置
//! - 从环境变量加载配置
//! - 从命令行参数加载配置
//! - 配置优先级和覆盖

use quantum_config::Config;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Config, Serialize, Deserialize, Debug)]
#[config(env_prefix = "BASIC_")]
struct BasicConfig {
    /// 应用程序名称
    name: String,

    /// 服务器主机地址
    host: String,

    /// 服务器端口
    port: u16,

    /// 是否启用调试模式
    debug: Option<bool>,

    /// 日志级别
    log_level: String,

    /// 工作线程数
    workers: u32,
}

impl Default for BasicConfig {
    fn default() -> Self {
        Self {
            name: "Basic Example".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            debug: Some(false),
            log_level: "info".to_string(),
            workers: 4,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Quantum Config 基本配置加载示例 ===");
    println!();

    // 加载配置
    println!("正在加载配置...");
    let config = BasicConfig::new();

    // 显示加载的配置
    println!("配置加载完成！");
    println!();
    println!("当前配置:");
    println!("  应用程序名称: {}", config.name);
    println!("  服务器地址: {}:{}", config.host, config.port);
    println!("  调试模式: {:?}", config.debug.unwrap_or(false));
    println!("  日志级别: {}", config.log_level);
    println!("  工作线程数: {}", config.workers);
    println!();

    // 显示配置来源说明
    println!("配置来源优先级 (从低到高):");
    println!("  1. 默认值 (代码中定义)");
    println!("  2. 系统配置文件: /etc/basic_example/config.toml");
    println!("  3. 用户配置文件: ~/.config/basic_example/config.toml");
    println!("  4. 指定配置文件: --config <path>");
    println!("  5. 环境变量: BASIC_*");
    println!("  6. 命令行参数: --host, --port, --debug 等");
    println!();

    // 显示使用示例
    println!("使用示例:");
    println!("  # 使用环境变量");
    println!("  export BASIC_HOST=0.0.0.0");
    println!("  export BASIC_PORT=3000");
    println!("  export BASIC_DEBUG=true");
    println!();
    println!("  # 使用命令行参数");
    println!("  cargo run -- --host 0.0.0.0 --port 3000 --debug");
    println!();
    println!("  # 使用配置文件");
    println!("  cargo run -- --config ./config.toml");
    println!();

    // 生成配置文件模板
    println!("生成配置文件模板:");
    println!("=== config.toml ===");
    println!("# 应用程序名称");
    println!("name = \"{}\"", config.name);
    println!("# 服务器主机地址");
    println!("host = \"{}\"", config.host);
    println!("# 服务器端口");
    println!("port = {}", config.port);
    println!("# 启用调试模式");
    println!("debug = {}", config.debug.unwrap_or(false));
    println!("# 日志级别 (trace, debug, info, warn, error)");
    println!("log_level = \"{}\"", config.log_level);
    println!("# 工作线程数");
    println!("workers = {}", config.workers);
    println!("==================");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config_new() {
        // Test that we can create a BasicConfig using new()
        let config = BasicConfig::new();
        assert!(config.name.len() > 0, "name should have a default value");
        assert!(config.debug.is_some(), "debug field should be accessible");
        assert!(config.port > 0, "port should have a valid default value");
    }

    #[test]
    fn test_basic_config_default() {
        // Test that we can create a BasicConfig using Default
        let config = BasicConfig::default();
        assert_eq!(config.name, "Basic Example");
        assert_eq!(config.debug, Some(false));
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_config_serialization() {
        // Test that the config can be serialized and deserialized
        let config = BasicConfig::default();
        let serialized = toml::to_string(&config).expect("Should be able to serialize config");
        assert!(serialized.contains("name"), "Serialized config should contain name");
        assert!(serialized.contains("debug"), "Serialized config should contain debug");
        assert!(serialized.contains("port"), "Serialized config should contain port");

        // Test deserialization
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should be able to deserialize config");
        assert_eq!(deserialized.name, config.name);
        assert_eq!(deserialized.debug, config.debug);
        assert_eq!(deserialized.port, config.port);
    }

    // === 边界条件测试 ===

    #[test]
    fn test_empty_string_configurations() {
        // Test empty string configurations
        let mut config = BasicConfig::default();
        
        config.name = String::new();
        config.host = String::new();
        config.log_level = String::new();
        
        // Empty strings should be accepted
        assert!(config.name.is_empty(), "empty name should be accepted");
        assert!(config.host.is_empty(), "empty host should be accepted");
        assert!(config.log_level.is_empty(), "empty log_level should be accepted");
        
        // Should still be serializable with empty strings
        let serialized = toml::to_string(&config).expect("Should serialize config with empty strings");
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should deserialize config with empty strings");
        assert!(deserialized.name.is_empty());
        assert!(deserialized.host.is_empty());
        assert!(deserialized.log_level.is_empty());
    }

    #[test]
    fn test_zero_port_boundary() {
        // Test handling of zero port value
        let mut config = BasicConfig::default();
        config.port = 0;
        
        // Zero port should be detectable
        assert_eq!(config.port, 0, "port should accept zero value");
        
        // Should still be serializable with zero port
        let serialized = toml::to_string(&config).expect("Should serialize config with zero port");
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should deserialize config with zero port");
        assert_eq!(deserialized.port, 0);
    }

    #[test]
    fn test_max_port_boundary() {
        // Test handling of maximum port value
        let mut config = BasicConfig::default();
        config.port = u16::MAX; // 65535
        
        // Maximum port should be handled correctly
        assert_eq!(config.port, u16::MAX, "port should accept maximum value");
        
        // Should still be serializable with max port
        let serialized = toml::to_string(&config).expect("Should serialize config with max port");
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should deserialize config with max port");
        assert_eq!(deserialized.port, u16::MAX);
    }

    #[test]
    fn test_zero_workers_boundary() {
        // Test handling of zero workers value
        let mut config = BasicConfig::default();
        config.workers = 0;
        
        // Zero workers should be detectable
        assert_eq!(config.workers, 0, "workers should accept zero value");
        
        // Should still be serializable with zero workers
        let serialized = toml::to_string(&config).expect("Should serialize config with zero workers");
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should deserialize config with zero workers");
        assert_eq!(deserialized.workers, 0);
    }

    #[test]
    fn test_max_workers_boundary() {
        // Test handling of maximum workers value
        let mut config = BasicConfig::default();
        config.workers = u32::MAX;
        
        // Maximum workers should be handled correctly
        assert_eq!(config.workers, u32::MAX, "workers should accept maximum value");
        
        // Should still be serializable with max workers
        let serialized = toml::to_string(&config).expect("Should serialize config with max workers");
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should deserialize config with max workers");
        assert_eq!(deserialized.workers, u32::MAX);
    }

    #[test]
    fn test_none_debug_option() {
        // Test handling of None debug option
        let mut config = BasicConfig::default();
        config.debug = None;
        
        // None debug should be handled correctly
        assert!(config.debug.is_none(), "debug should accept None value");
        
        // Should still be serializable with None debug
        let serialized = toml::to_string(&config).expect("Should serialize config with None debug");
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should deserialize config with None debug");
        assert!(deserialized.debug.is_none());
    }

    #[test]
    fn test_serialization_roundtrip_with_boundaries() {
        // Test serialization roundtrip with all boundary values
        let mut config = BasicConfig::default();
        
        // Set various boundary values
        config.name = String::new();
        config.host = String::new();
        config.port = 0;
        config.debug = None;
        config.log_level = String::new();
        config.workers = 0;
        
        // Serialize and deserialize
        let serialized = toml::to_string(&config).expect("Should serialize boundary config");
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should deserialize boundary config");
        
        // Verify all boundary values preserved
        assert!(deserialized.name.is_empty());
        assert!(deserialized.host.is_empty());
        assert_eq!(deserialized.port, 0);
        assert!(deserialized.debug.is_none());
        assert!(deserialized.log_level.is_empty());
        assert_eq!(deserialized.workers, 0);
    }

    #[test]
    fn test_invalid_toml_handling() {
        // Test handling of invalid TOML strings
        let invalid_toml_cases = vec![
            "invalid toml",
            "name = invalid_string_without_quotes",
            "port = \"not_a_number\"",
            "workers = -1",
            "debug = invalid_boolean",
            "[unclosed_section",
            "",
        ];
        
        for invalid_toml in invalid_toml_cases {
            let result = toml::from_str::<BasicConfig>(invalid_toml);
            assert!(result.is_err(), "Should fail to parse invalid TOML: {}", invalid_toml);
        }
    }

    #[test]
    fn test_partial_toml_deserialization() {
        // Test that partial TOML deserialization with all required fields
        let partial_toml = r#"
name = "Test App"
host = "example.com"
port = 9000
log_level = "debug"
workers = 8
"#;
        
        let config: BasicConfig = toml::from_str(partial_toml).expect("Should parse partial TOML");
        
        // Specified values should be preserved
        assert_eq!(config.name, "Test App");
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 9000);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.workers, 8);
        
        // Optional field should be None when not specified in TOML
        assert_eq!(config.debug, None); // None when not specified
    }

    #[test]
    fn test_special_string_characters() {
        // Test handling of special characters in string fields
        let mut config = BasicConfig::default();
        
        config.name = "App with spaces & special chars: !@#$%^&*()".to_string();
        config.host = "测试主机.example.com".to_string(); // Unicode characters
        config.log_level = "🚀 debug".to_string(); // Emoji
        
        // Should handle special characters correctly
        assert!(config.name.contains("!@#$%^&*()"));
        assert!(config.host.contains("测试主机"));
        assert!(config.log_level.contains("🚀"));
        
        // Should still be serializable with special characters
        let serialized = toml::to_string(&config).expect("Should serialize config with special characters");
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should deserialize config with special characters");
        assert_eq!(deserialized.name, config.name);
        assert_eq!(deserialized.host, config.host);
        assert_eq!(deserialized.log_level, config.log_level);
    }

    #[test]
    fn test_very_long_strings() {
        // Test handling of very long string values
        let mut config = BasicConfig::default();
        
        let long_string = "x".repeat(10000); // 10KB string
        config.name = long_string.clone();
        config.host = long_string.clone();
        config.log_level = long_string.clone();
        
        // Should handle long strings correctly
        assert_eq!(config.name.len(), 10000);
        assert_eq!(config.host.len(), 10000);
        assert_eq!(config.log_level.len(), 10000);
        
        // Should still be serializable with long strings
        let serialized = toml::to_string(&config).expect("Should serialize config with long strings");
        let deserialized: BasicConfig = toml::from_str(&serialized).expect("Should deserialize config with long strings");
        assert_eq!(deserialized.name.len(), 10000);
        assert_eq!(deserialized.host.len(), 10000);
        assert_eq!(deserialized.log_level.len(), 10000);
    }
}
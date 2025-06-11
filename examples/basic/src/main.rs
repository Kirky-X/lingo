//! 基本配置加载示例
//!
//! 这个示例展示了如何使用 Lingo 进行基本的配置管理，包括：
//! - 从配置文件加载配置
//! - 从环境变量加载配置
//! - 从命令行参数加载配置
//! - 配置优先级和覆盖

use lingo::Config;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Config, Serialize, Deserialize, Debug)]
struct AppConfig {
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

impl Default for AppConfig {
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
    println!("=== Lingo 基本配置加载示例 ===");
    println!();

    // 加载配置
    println!("正在加载配置...");
    let config = AppConfig::new();

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
//! Lingo 配置提供者模块
//!
//! 此模块包含了不同配置源的数据提供者实现，用于与 figment 集成。
//! 每个提供者负责从特定的配置源（文件、环境变量、命令行参数）读取配置数据。

pub mod file_provider;
pub mod env_provider;
pub mod clap_provider;

pub use clap_provider::LingoClapProvider;
pub use env_provider::LingoEnvProvider;
// 重新导出主要类型
pub use file_provider::LingoFileProvider;

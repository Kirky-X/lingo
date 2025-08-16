//! 配置提供器模块
//!
//! 包含各种配置数据源的提供器实现。

pub mod clap_provider;
pub mod env_provider;
pub mod file_provider;
pub mod file_reader;

pub use clap_provider::QuantumConfigClapProvider;
pub use env_provider::QuantumConfigEnvProvider;
pub use file_provider::{QuantumConfigFileProvider, QuantumConfigFileProviderGeneric};
pub use file_reader::{FileReader, StandardFileReader};

// 向后兼容的类型别名（内部使用）
// 注意：这些类型别名仅用于内部兼容，不对外暴露

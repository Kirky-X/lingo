//! 配置提供器模块
//!
//! 包含各种配置数据源的提供器实现。

pub mod clap_provider;
pub mod env_provider;
pub mod file_provider;
pub mod file_reader;

pub use clap_provider::LingoClapProvider;
pub use env_provider::LingoEnvProvider;
pub use file_provider::{LingoFileProvider, LingoFileProviderGeneric};
pub use file_reader::{FileReader, StandardFileReader};

//! # Lingo
//!
//! A powerful and flexible configuration management library for Rust applications.
//!
//! Lingo provides a unified interface for loading configuration from multiple sources
//! including files, environment variables, and command-line arguments, with automatic
//! type conversion and validation.
//!
//! ## Features
//!
//! - **Multiple Configuration Sources**: Load from TOML, JSON, YAML files, environment variables, and CLI arguments
//! - **Hierarchical Configuration**: Support for nested configuration structures
//! - **Type Safety**: Automatic deserialization with compile-time type checking
//! - **Flexible Priority**: Configurable precedence for different configuration sources
//! - **Derive Macros**: Simple `#[derive(Config)]` for automatic configuration management
//! - **Async Support**: Optional async configuration loading
//! - **Template Generation**: Generate configuration file templates
//!
//! ## Quick Start
//!
//! ```ignore
//! use lingo::{Config, Deserialize, Serialize};
//!
//! #[derive(Config, Default, Deserialize, Serialize)]
//! struct AppConfig {
//!     host: String,
//!     port: u16,
//!     debug: bool,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = AppConfig::load()?;
//!     println!("Server running on {}:{}", config.host, config.port);
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod meta;
pub mod paths;
pub mod providers;

#[cfg(test)]
mod integration_tests;

// Re-export main types
pub use error::{ConfigDirType, LingoError};
pub use meta::{ClapAttrsMeta, FieldMeta, LingoAppMeta, StructMeta};
pub use paths::{add_specified_config_file, resolve_config_files, ConfigFilePath, ConfigFileType};

// 对外重导出 Serde 常用 traits
pub use serde::{Deserialize, Serialize};
// 兼容派生宏生成代码：在 crate 根下提供 `serde` 模块路径
pub use ::serde as serde;

// 新增：对外重导出 figment 与 clap 常用类型，供 derive 宏下游直接使用
pub use figment::Figment;
pub use clap::{Arg, ArgAction, ArgMatches, Command};
// 兼容派生宏生成代码：在 crate 根下提供 `toml` 模块路径
pub use ::toml as toml;

// 对外重导出 derive 宏
pub use lingo_derive::Config;

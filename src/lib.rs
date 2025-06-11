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
//! ```rust
//! use lingo::Config;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Config, Default, Deserialize, Serialize)]
//! struct AppConfig {
//!     host: String,
//!     port: u16,
//!     debug: bool,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = AppConfig::default();
//!     println!("Server running on {}:{}", config.host, config.port);
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod meta;
pub mod paths;
pub mod providers;

// Re-export main types
pub use error::{ConfigDirType, LingoError};
pub use meta::{ClapAttrsMeta, FieldMeta, LingoAppMeta, StructMeta};
pub use paths::{add_specified_config_file, resolve_config_files, ConfigFilePath, ConfigFileType};

// Re-export commonly used external types
pub use serde::{Deserialize, Serialize};

// Re-export the Config derive macro from lingo-derive
pub use lingo_derive::Config;

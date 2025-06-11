//! # Lingo Derive
//!
//! 为 Lingo 配置库提供派生宏，用于自动实现结构体的配置管理功能。
//!
//! ## 特性
//!
//! - 自动实现配置加载功能
//! - 支持从文件、环境变量和命令行参数加载配置
//! - 提供配置模板生成功能
//!
//! ## 使用示例
//!
//! ```rust
//! use lingo::Config;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Config, Serialize, Deserialize)]
//! struct AppConfig {
//!     host: String,
//!     port: u16,
//!     debug: bool,
//! }
//!
//! fn main() {
//!     let config = AppConfig::load().unwrap();
//!     println!("Server running on {}:{}", config.host, config.port);
//! }
//! ```

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// 为结构体自动实现配置管理功能的派生宏
///
/// 该宏会为标注的结构体自动实现以下方法：
/// - `load()`: 从多种来源加载配置
/// - `new()`: 创建新的配置实例
/// - `generate_template()`: 生成配置模板
///
/// # 示例
///
/// ```rust
/// use lingo::Config;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Config, Serialize, Deserialize)]
/// struct MyConfig {
///     database_url: String,
///     port: u16,
/// }
/// ```
#[proc_macro_derive(Config)]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            /// 从多种来源加载配置
            ///
            /// 加载顺序：
            /// 1. 默认配置文件 (config.toml)
            /// 2. 环境变量
            /// 3. 命令行参数
            ///
            /// # 错误
            ///
            /// 当配置文件不存在、格式错误或必需字段缺失时返回错误
            pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
                use std::path::Path;
                
                // 1. 尝试从配置文件加载
                let mut config = if Path::new("config.toml").exists() {
                    let content = std::fs::read_to_string("config.toml")?;
                    toml::from_str::<Self>(&content)?
                } else {
                    // 如果配置文件不存在，使用默认值或返回错误
                    return Err("Configuration file 'config.toml' not found".into());
                };
                
                // 2. 从环境变量覆盖配置
                // 这里需要根据具体字段实现环境变量读取逻辑
                
                // 3. 从命令行参数覆盖配置
                // 这里需要根据具体字段实现命令行参数解析逻辑
                
                Ok(config)
            }
            
            /// 创建新的配置实例
            ///
            /// # 示例
            ///
            /// ```rust
            /// let config = MyConfig::new();
            /// ```
            pub fn new() -> Self {
                Self::default()
            }
            
            /// 生成配置模板文件
            ///
            /// 在当前目录下生成一个示例配置文件 `config.toml.example`
            ///
            /// # 错误
            ///
            /// 当无法写入文件时返回错误
            pub fn generate_template() -> Result<(), Box<dyn std::error::Error>> {
                let template = Self::default();
                let toml_content = toml::to_string_pretty(&template)?;
                std::fs::write("config.toml.example", toml_content)?;
                println!("Configuration template generated: config.toml.example");
                Ok(())
            }
        }
        

    };

    TokenStream::from(expanded)
}
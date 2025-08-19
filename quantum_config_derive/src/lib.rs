//! # Quantum Config Derive
//!
//! 为 Quantum Config 配置库提供派生宏，用于自动实现结构体的配置管理功能。

//! ## 特性
//!
//! - 自动实现配置加载功能
//! - 支持从文件、环境变量和命令行参数加载配置
//! - 提供配置模板生成功能
//! - 支持自定义环境变量前缀

//! ## 使用示例
//!
//! ```ignore
//! use quantum_config::Config;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Config, Serialize, Deserialize, Default)]
//! #[config(env_prefix = "MYAPP_")]
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
use syn::{parse_macro_input, DeriveInput, Attribute, Meta};

/// 解析 #[config(...)] 属性中的 env_prefix 参数
fn parse_config_attributes(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("config") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens_str = meta_list.tokens.to_string();
                // 解析 env_prefix = "VALUE" 模式 (简化版本)
                if let Some(start) = tokens_str.find("env_prefix") {
                    let after_prefix = &tokens_str[start + "env_prefix".len()..];
                    if let Some(eq_pos) = after_prefix.find('=') {
                        let value_part = after_prefix[eq_pos + 1..].trim();
                        if value_part.starts_with('"') && value_part.ends_with('"') {
                            let prefix = value_part[1..value_part.len()-1].to_string();
                            return Some(prefix);
                        }
                    }
                }
            }
        }
    }
    None
}

/// 为结构体自动实现配置管理功能的派生宏
///
/// 该宏会为标注的结构体自动实现以下方法：
/// - `load()`: 从多种来源加载配置（文件 -> 环境变量 -> 命令行参数）
/// - `new()`: 创建新的配置实例
/// - `load_from_file()`: 从指定文件加载并反序列化
/// - `generate_template()`: 生成配置模板
///
/// 支持的属性：
/// - `#[config(env_prefix = "PREFIX_")]`: 自定义环境变量前缀
#[proc_macro_derive(Config, attributes(config))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // 解析自定义环境变量前缀
    let custom_env_prefix = parse_config_attributes(&input.attrs);

    // 使用 proc-macro-crate 动态解析依赖方对 `quantum_config` 的重命名
    let crate_ident = match proc_macro_crate::crate_name("quantum_config") {
        Ok(proc_macro_crate::FoundCrate::Itself) => quote! { crate },
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote! { #ident }
        }
        Err(_) => quote! { quantum_config }, // 回退：直接使用 quantum_config
    };

    // 生成环境变量前缀逻辑
    let env_prefix_expr = if let Some(prefix) = custom_env_prefix {
        quote! { Some(#prefix.to_string()) }
    } else {
        quote! { Some(format!("{}_", app_name.to_uppercase())) }
    };

    // 生成的实现：基于 quantum_config 暴露的公共 API 与 figment 进行合并
    let expanded = quote! {
        impl #name {
            /// 从多种来源加载配置
            ///
            /// 加载顺序（低 -> 高优先级覆盖）：
            /// 1. 文件（系统级、用户级、以及 --config 指定的文件）
            /// 2. 环境变量（可选使用前缀，默认使用结构体名大写并加下划线）
            /// 3. 命令行参数（clap 提供者）
            pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
                // 构造应用元数据（默认值）：
                // app_name 使用类型名，env_prefix 使用自定义或默认格式，行为版本与深度使用默认
                let cmd_name: &'static str = stringify!(#name);
                let app_name = cmd_name.to_string();
                let env_prefix = #env_prefix_expr;
                let app_meta = #crate_ident::QuantumConfigAppMeta { app_name, env_prefix, behavior_version: 1, max_parse_depth: 128 };

                // 解析候选配置文件路径（宽容处理目录缺失场景）
                let mut config_file_paths = match #crate_ident::resolve_config_files(&app_meta) {
                    Ok(v) => v,
                    Err(#crate_ident::QuantumConfigError::NoConfigFilesFoundInDir { .. }) |
                    Err(#crate_ident::QuantumConfigError::ConfigDirNotFound { .. }) => Vec::new(),
                    Err(e) => return Err(e.into()),
                };

                // 尝试从命令行解析 --config 以追加必选文件
                let clap_matches = #crate_ident::Command::new(cmd_name)
                    .arg(#crate_ident::Arg::new("config").long("config").short('c').num_args(1))
                    .arg(#crate_ident::Arg::new("config-dir").long("config-dir").num_args(1))
                    .arg(#crate_ident::Arg::new("log-level").long("log-level").num_args(1))
                    .arg(#crate_ident::Arg::new("verbose").long("verbose").short('v').action(#crate_ident::ArgAction::SetTrue))
                    .arg(#crate_ident::Arg::new("quiet").long("quiet").short('q').action(#crate_ident::ArgAction::SetTrue))
                    .arg(#crate_ident::Arg::new("output").long("output").short('o').num_args(1))
                    .arg(#crate_ident::Arg::new("format").long("format").num_args(1))
                    // Removed allow_external_subcommands(true) to prevent command injection
                    .get_matches_from(std::env::args());

                if let Some(cfg) = clap_matches.get_one::<String>("config") {
                    let path = std::path::PathBuf::from(cfg);
                    #crate_ident::add_specified_config_file(&mut config_file_paths, path)?;
                }

                let mut fig = #crate_ident::Figment::new();
                for cfg in config_file_paths {
                    let provider = #crate_ident::providers::QuantumConfigFileProvider::from_path(&cfg.path, cfg.is_required, app_meta.max_parse_depth)?;
                    fig = fig.merge(provider);
                }
                if let Some(prefix) = app_meta.env_prefix.clone() {
                    let env_provider = #crate_ident::providers::QuantumConfigEnvProvider::with_prefix(prefix);
                    fig = fig.merge(env_provider);
                }
                let clap_provider = #crate_ident::providers::clap_provider::with_common_mappings(clap_matches);
                fig = fig.merge(clap_provider);
                Ok(fig.extract()?)
            }

            /// 从多种来源加载配置（测试辅助：可注入命令行参数）
            pub fn load_with_args(args: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
                let cmd_name: &'static str = stringify!(#name);
                let app_name = cmd_name.to_string();
                let env_prefix = #env_prefix_expr;
                let app_meta = #crate_ident::QuantumConfigAppMeta { app_name, env_prefix, behavior_version: 1, max_parse_depth: 128 };

                let mut config_file_paths = match #crate_ident::resolve_config_files(&app_meta) {
                    Ok(v) => v,
                    Err(#crate_ident::QuantumConfigError::NoConfigFilesFoundInDir { .. }) |
                    Err(#crate_ident::QuantumConfigError::ConfigDirNotFound { .. }) => Vec::new(),
                    Err(e) => return Err(e.into()),
                };

                let clap_matches = #crate_ident::Command::new(cmd_name)
                    .arg(#crate_ident::Arg::new("config").long("config").short('c').num_args(1))
                    .arg(#crate_ident::Arg::new("config-dir").long("config-dir").num_args(1))
                    .arg(#crate_ident::Arg::new("log-level").long("log-level").num_args(1))
                    .arg(#crate_ident::Arg::new("verbose").long("verbose").short('v').action(#crate_ident::ArgAction::SetTrue))
                    .arg(#crate_ident::Arg::new("quiet").long("quiet").short('q').action(#crate_ident::ArgAction::SetTrue))
                    .arg(#crate_ident::Arg::new("output").long("output").short('o').num_args(1))
                    .arg(#crate_ident::Arg::new("format").long("format").num_args(1))
                    .allow_external_subcommands(true)
                    .try_get_matches_from(args)
                    .map_err(|e| #crate_ident::QuantumConfigError::Internal(format!("Failed to parse CLI args: {}", e)))?;

                if let Some(cfg) = clap_matches.get_one::<String>("config") {
                    let path = std::path::PathBuf::from(cfg);
                    #crate_ident::add_specified_config_file(&mut config_file_paths, path)?;
                }

                let mut fig = #crate_ident::Figment::new();
                for cfg in config_file_paths {
                    let provider = #crate_ident::providers::QuantumConfigFileProvider::from_path(&cfg.path, cfg.is_required, app_meta.max_parse_depth)?;
                    fig = fig.merge(provider);
                }
                if let Some(prefix) = app_meta.env_prefix.clone() {
                    let env_provider = #crate_ident::providers::QuantumConfigEnvProvider::with_prefix(prefix);
                    fig = fig.merge(env_provider);
                }
                let clap_provider = #crate_ident::providers::clap_provider::with_common_mappings(clap_matches);
                fig = fig.merge(clap_provider);
                Ok(fig.extract()?)
            }

            /// 创建新的配置实例（使用 Default），保持向后兼容
            pub fn new() -> Self { Self::default() }

            /// 从指定文件加载配置（仅文件，不合并其他来源），保持向后兼容
            pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
                let path = path.as_ref();
                let provider = #crate_ident::providers::QuantumConfigFileProvider::from_path(path, true, 128)?;
                let config: Self = #crate_ident::Figment::new().merge(provider).extract()?;
                Ok(config)
            }

            /// 生成配置模板文件
            pub fn generate_template() -> Result<(), Box<dyn std::error::Error>> {
                let template = Self::default();
                // 通过 serde 序列化为 TOML（依赖主库中的 toml 依赖）
                use #crate_ident::serde::Serialize;
                let toml_content = #crate_ident::toml::to_string_pretty(&template)?;
                std::fs::write("config.toml.example", toml_content)?;
                println!("Configuration template generated: config.toml.example");
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
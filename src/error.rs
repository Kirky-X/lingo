//! Quantum Config 错误处理模块
//!
//! 定义了 Quantum Config 库中所有可能的错误类型，提供统一的错误处理接口。

use std::path::{Path, PathBuf};
use thiserror::Error;

/// 过滤路径中的敏感信息，用于错误消息显示
#[allow(unused_variables)] // path_str is used in debug builds
fn sanitize_path_for_display(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    
    // 在生产环境中隐藏敏感路径信息
    #[cfg(not(debug_assertions))]
    {
        // 只显示文件名，隐藏完整路径
        if let Some(file_name) = path.file_name() {
            format!("<sanitized>/{}", file_name.to_string_lossy())
        } else {
            "<sanitized_path>".to_string()
        }
    }
    
    // 在调试环境中显示完整路径
    #[cfg(debug_assertions)]
    {
        path_str.to_string()
    }
}

/// 表示正在访问的配置目录类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDirType {
    /// 系统级配置目录
    System,
    /// 用户级配置目录
    User,
}

impl std::fmt::Display for ConfigDirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigDirType::System => write!(f, "system"),
            ConfigDirType::User => write!(f, "user"),
        }
    }
}

/// 配置模板格式枚举
/// 
/// 定义支持的配置文件模板格式类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateFormat {
    /// TOML 格式模板
    Toml,
    /// JSON 格式模板
    Json,
    /// INI 格式模板
    Ini,
}

impl TemplateFormat {
    /// 获取模板格式的文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            TemplateFormat::Toml => "toml",
            TemplateFormat::Json => "json",
            TemplateFormat::Ini => "ini",
        }
    }
    
    /// 获取模板格式的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TemplateFormat::Toml => "TOML",
            TemplateFormat::Json => "JSON",
            TemplateFormat::Ini => "INI",
        }
    }
}

/// Quantum Config 库中所有操作的统一错误类型
#[derive(Error, Debug)]
pub enum QuantumConfigError {
    /// I/O 错误，包含路径信息
    #[error("I/O error for path {}: {source}", sanitize_path_for_display(path))]
    Io {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },

    /// 文件读取错误
    #[error("Failed to read file {path}: {source}")]
    FileReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// 文件解析错误
    #[error("Failed to parse {format_name} file {}: {source_error}", sanitize_path_for_display(path))]
    FileParse {
        format_name: String,
        path: PathBuf,
        source_error: String,
    },

    /// Figment 配置提取错误
    #[error("Configuration extraction error: {0}")]
    Figment(#[from] Box<figment::Error>),

    /// 命令行参数解析错误
    #[error("Command line argument parsing error: {0}")]
    Clap(#[from] clap::Error),

    /// 缺少必需值错误
    #[error("A required value was missing for key: {key_path}")]
    MissingValue { key_path: String },

    /// 无效值错误
    #[error("Invalid value for key '{key_path}': {message}")]
    InvalidValue { key_path: String, message: String },

    /// 配置目录未找到错误
    #[error("Configuration directory for {dir_type} not found. Expected at: {}", 
        expected_path.as_ref().map(|p| sanitize_path_for_display(p)).unwrap_or_else(|| "<unknown>".to_string()))]
    ConfigDirNotFound {
        dir_type: ConfigDirType,
        expected_path: Option<PathBuf>,
    },

    /// 目录中未找到支持的配置文件
    #[error("No supported configuration files found in {dir_type} directory: {}", sanitize_path_for_display(path))]
    NoConfigFilesFoundInDir {
        dir_type: ConfigDirType,
        path: PathBuf,
    },

    /// 指定的配置文件未找到
    #[error("Specified configuration file not found: {}", sanitize_path_for_display(path))]
    SpecifiedFileNotFound { path: PathBuf },

    /// 不支持的配置文件格式
    #[error("Unsupported configuration file format for: {}", sanitize_path_for_display(path))]
    UnsupportedFormat { path: PathBuf },

    /// 模板生成错误
    #[error("Error generating {format:?} template: {reason}")]
    TemplateGeneration {
        format: TemplateFormat,
        reason: String,
    },

/// Internal Quantum Config error: {0}]
    #[error("Internal Quantum Config error: {0}")]
    Internal(String),

    /// 应用程序名称解析失败
    #[error("Failed to determine application name: {source_error}")]
    AppNameResolution { source_error: String },

    /// 安全违规错误
    #[error("Security violation: {message}")]
    SecurityViolation { message: String },

    /// 验证错误
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_config_dir_type_display() {
        assert_eq!(ConfigDirType::System.to_string(), "system");
        assert_eq!(ConfigDirType::User.to_string(), "user");
    }

    #[test]
    fn test_config_dir_type_equality() {
        assert_eq!(ConfigDirType::System, ConfigDirType::System);
        assert_eq!(ConfigDirType::User, ConfigDirType::User);
        assert_ne!(ConfigDirType::System, ConfigDirType::User);
    }

    #[test]
    fn test_io_error_display() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let path = PathBuf::from("/test/path");
        let quantum_config_error = QuantumConfigError::Io {
            source: io_error,
            path: path.clone(),
        };

        let error_msg = quantum_config_error.to_string();
        assert!(error_msg.contains("I/O error for path"));
        assert!(error_msg.contains("/test/path"));
        assert!(error_msg.contains("File not found"));
    }

    #[test]
    fn test_file_parse_error_display() {
        let error = QuantumConfigError::FileParse {
            format_name: "TOML".to_string(),
            path: PathBuf::from("/config/app.toml"),
            source_error: "Invalid syntax at line 5".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Failed to parse TOML file"));
        assert!(error_msg.contains("/config/app.toml"));
        assert!(error_msg.contains("Invalid syntax at line 5"));
    }

    #[test]
    fn test_missing_value_error_display() {
        let error = QuantumConfigError::MissingValue {
            key_path: "database.host".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("A required value was missing for key: database.host"));
    }

    #[test]
    fn test_invalid_value_error_display() {
        let error = QuantumConfigError::InvalidValue {
            key_path: "server.port".to_string(),
            message: "Port must be between 1 and 65535".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Invalid value for key 'server.port'"));
        assert!(error_msg.contains("Port must be between 1 and 65535"));
    }

    #[test]
    fn test_config_dir_not_found_error_display() {
        let error = QuantumConfigError::ConfigDirNotFound {
            dir_type: ConfigDirType::User,
            expected_path: Some(PathBuf::from("/home/user/.config")),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Configuration directory for user not found"));
        assert!(error_msg.contains("/home/user/.config"));
    }

    #[test]
    fn test_config_dir_not_found_error_no_path() {
        let error = QuantumConfigError::ConfigDirNotFound {
            dir_type: ConfigDirType::System,
            expected_path: None,
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Configuration directory for system not found"));
        assert!(error_msg.contains("<unknown>"));
    }

    #[test]
    fn test_no_config_files_found_error_display() {
        let error = QuantumConfigError::NoConfigFilesFoundInDir {
            dir_type: ConfigDirType::System,
            path: PathBuf::from("/etc/myapp"),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("No supported configuration files found in system directory"));
        assert!(error_msg.contains("/etc/myapp"));
    }

    #[test]
    fn test_specified_file_not_found_error_display() {
        let error = QuantumConfigError::SpecifiedFileNotFound {
            path: PathBuf::from("/custom/config.toml"),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Specified configuration file not found"));
        assert!(error_msg.contains("/custom/config.toml"));
    }

    #[test]
    fn test_unsupported_format_error_display() {
        let error = QuantumConfigError::UnsupportedFormat {
            path: PathBuf::from("/config/app.xml"),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Unsupported configuration file format for"));
        assert!(error_msg.contains("/config/app.xml"));
    }

    #[test]
    fn test_template_generation_error_display() {
        let error = QuantumConfigError::TemplateGeneration {
            format: TemplateFormat::Toml,
            reason: "Invalid field type".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Error generating"));
    }
}

// Backward compatibility alias
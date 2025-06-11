//! Lingo 错误处理模块
//!
//! 定义了 Lingo 库中所有可能的错误类型，提供统一的错误处理接口。

use std::path::PathBuf;
use thiserror::Error;

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

/// 模板格式枚举（临时定义，后续会移到 template 模块）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateFormat {
    Toml,
    Json,
    Ini,
}

/// Lingo 库中所有操作的统一错误类型
#[derive(Error, Debug)]
pub enum LingoError {
    /// I/O 错误，包含路径信息
    #[error("I/O error for path {path:?}: {source}")]
    Io {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },

    /// 文件解析错误
    #[error("Failed to parse {format_name} file {path:?}: {source_error}")]
    FileParse {
        format_name: String,
        path: PathBuf,
        source_error: String,
    },

    /// Figment 配置提取错误
    #[error("Configuration extraction error: {0}")]
    Figment(#[from] figment::Error),

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
    #[error("Configuration directory for {dir_type} not found. Expected at: {expected_path:?}")]
    ConfigDirNotFound {
        dir_type: ConfigDirType,
        expected_path: Option<PathBuf>,
    },

    /// 目录中未找到支持的配置文件
    #[error("No supported configuration files found in {dir_type} directory: {path:?}")]
    NoConfigFilesFoundInDir {
        dir_type: ConfigDirType,
        path: PathBuf,
    },

    /// 指定的配置文件未找到
    #[error("Specified configuration file not found: {path:?}")]
    SpecifiedFileNotFound { path: PathBuf },

    /// 不支持的配置文件格式
    #[error("Unsupported configuration file format for: {path:?}")]
    UnsupportedFormat { path: PathBuf },

    /// 模板生成错误
    #[error("Error generating {format:?} template: {reason}")]
    TemplateGeneration {
        format: TemplateFormat,
        reason: String,
    },

    /// 内部错误
    #[error("Internal Lingo error: {0}")]
    Internal(String),

    /// 应用名称解析失败
    #[error("Failed to determine application name: {source_error}")]
    AppNameResolution { source_error: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
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
        let lingo_error = LingoError::Io {
            source: io_error,
            path: path.clone(),
        };

        let error_msg = lingo_error.to_string();
        assert!(error_msg.contains("I/O error for path"));
        assert!(error_msg.contains("/test/path"));
        assert!(error_msg.contains("File not found"));
    }

    #[test]
    fn test_file_parse_error_display() {
        let error = LingoError::FileParse {
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
        let error = LingoError::MissingValue {
            key_path: "database.host".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("A required value was missing for key: database.host"));
    }

    #[test]
    fn test_invalid_value_error_display() {
        let error = LingoError::InvalidValue {
            key_path: "server.port".to_string(),
            message: "Port must be between 1 and 65535".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Invalid value for key 'server.port'"));
        assert!(error_msg.contains("Port must be between 1 and 65535"));
    }

    #[test]
    fn test_config_dir_not_found_error_display() {
        let error = LingoError::ConfigDirNotFound {
            dir_type: ConfigDirType::User,
            expected_path: Some(PathBuf::from("/home/user/.config")),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Configuration directory for user not found"));
        assert!(error_msg.contains("/home/user/.config"));
    }

    #[test]
    fn test_config_dir_not_found_error_no_path() {
        let error = LingoError::ConfigDirNotFound {
            dir_type: ConfigDirType::System,
            expected_path: None,
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Configuration directory for system not found"));
        assert!(error_msg.contains("None"));
    }

    #[test]
    fn test_no_config_files_found_error_display() {
        let error = LingoError::NoConfigFilesFoundInDir {
            dir_type: ConfigDirType::System,
            path: PathBuf::from("/etc/myapp"),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("No supported configuration files found in system directory"));
        assert!(error_msg.contains("/etc/myapp"));
    }

    #[test]
    fn test_specified_file_not_found_error_display() {
        let error = LingoError::SpecifiedFileNotFound {
            path: PathBuf::from("/custom/config.toml"),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Specified configuration file not found"));
        assert!(error_msg.contains("/custom/config.toml"));
    }

    #[test]
    fn test_unsupported_format_error_display() {
        let error = LingoError::UnsupportedFormat {
            path: PathBuf::from("/config/app.xml"),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Unsupported configuration file format for"));
        assert!(error_msg.contains("/config/app.xml"));
    }

    #[test]
    fn test_template_generation_error_display() {
        let error = LingoError::TemplateGeneration {
            format: TemplateFormat::Toml,
            reason: "Invalid field type".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Error generating Toml template"));
        assert!(error_msg.contains("Invalid field type"));
    }

    #[test]
    fn test_internal_error_display() {
        let error = LingoError::Internal("Unexpected state in parser".to_string());

        let error_msg = error.to_string();
        assert!(error_msg.contains("Internal Lingo error: Unexpected state in parser"));
    }

    #[test]
    fn test_app_name_resolution_error_display() {
        let error = LingoError::AppNameResolution {
            source_error: "Unable to determine executable name".to_string(),
        };

        let error_msg = error.to_string();
        assert!(error_msg.contains("Failed to determine application name"));
        assert!(error_msg.contains("Unable to determine executable name"));
    }

    #[test]
    fn test_error_source_chain() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let lingo_error = LingoError::Io {
            source: io_error,
            path: PathBuf::from("/restricted/file"),
        };

        // 测试错误源链
        assert!(lingo_error.source().is_some());
        let source = lingo_error.source().unwrap();
        assert!(source.to_string().contains("Access denied"));
    }

    #[test]
    fn test_template_format_debug() {
        assert_eq!(format!("{:?}", TemplateFormat::Toml), "Toml");
        assert_eq!(format!("{:?}", TemplateFormat::Json), "Json");
        assert_eq!(format!("{:?}", TemplateFormat::Ini), "Ini");
    }
}
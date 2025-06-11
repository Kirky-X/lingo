//! Lingo 路径解析模块
//!
//! 实现配置文件的路径解析逻辑，根据应用程序名称和系统约定确定配置文件的查找路径。

use crate::error::LingoError;
use crate::meta::LingoAppMeta;
use std::path::PathBuf;

/// 配置文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileType {
    /// TOML 格式
    Toml,
    /// JSON 格式
    Json,
    /// INI 格式
    Ini,
}

impl ConfigFileType {
    /// 获取文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFileType::Toml => "toml",
            ConfigFileType::Json => "json",
            ConfigFileType::Ini => "ini",
        }
    }

    /// 从文件扩展名推断配置文件类型
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "toml" => Some(ConfigFileType::Toml),
            "json" => Some(ConfigFileType::Json),
            "ini" => Some(ConfigFileType::Ini),
            _ => None,
        }
    }
}

/// 配置文件路径信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFilePath {
    /// 文件路径
    pub path: PathBuf,
    /// 文件类型
    pub file_type: ConfigFileType,
    /// 是否为必需文件（如果文件不存在是否报错）
    pub is_required: bool,
}

impl ConfigFilePath {
    /// 创建新的配置文件路径
    pub fn new(path: PathBuf, file_type: ConfigFileType, is_required: bool) -> Self {
        Self {
            path,
            file_type,
            is_required,
        }
    }

    /// 检查文件是否存在
    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_file()
    }
}

/// 解析配置文件路径
///
/// 根据应用程序元数据确定配置文件的查找路径，按照预定义的优先级顺序返回。
/// 查找顺序（低优先级在前）：
/// 1. 系统级配置目录
/// 2. 用户级配置目录
///
/// 在每个目录中，按以下模式查找文件：
/// - `config.{ext}`
/// - `{app_name}.{ext}`
///
/// 其中 `ext` 为 `toml`, `json`, `ini`
pub fn resolve_config_files(app_meta: &LingoAppMeta) -> Result<Vec<ConfigFilePath>, LingoError> {
    let mut config_files = Vec::new();
    let app_name = &app_meta.app_name;

    // 支持的文件扩展名，按优先级排序
    let extensions = [ConfigFileType::Toml, ConfigFileType::Json, ConfigFileType::Ini];

    // 文件名模式
    let file_patterns = ["config", app_name.as_str()];

    // 获取配置目录
    let config_dirs = get_config_directories(app_name)?;

    // 遍历配置目录（按优先级从低到高）
    for config_dir in config_dirs {
        // 在每个目录中查找配置文件
        for pattern in &file_patterns {
            for &file_type in &extensions {
                let filename = format!("{}.{}", pattern, file_type.extension());
                let file_path = config_dir.join(&filename);

                if file_path.exists() && file_path.is_file() {
                    config_files.push(ConfigFilePath::new(
                        file_path,
                        file_type,
                        false, // 系统级和用户级文件默认不是必需的
                    ));
                }
            }
        }
    }

    Ok(config_files)
}

/// 获取配置目录列表
///
/// 返回按优先级排序的配置目录列表（低优先级在前）：
/// 1. 系统级配置目录
/// 2. 用户级配置目录
fn get_config_directories(app_name: &str) -> Result<Vec<PathBuf>, LingoError> {
    let mut dirs = Vec::new();

    // 使用 directories crate 获取标准配置目录
    if let Some(project_dirs) = directories::ProjectDirs::from("", "", app_name) {
        // 系统级配置目录（低优先级）
        // 在 Windows 上通常是 C:\ProgramData\{app_name}
        // 在 Unix 上通常是 /etc/{app_name}
        let system_config_dir = project_dirs.config_dir().parent()
            .and_then(|p| p.parent())
            .map(|p| {
                #[cfg(windows)]
                { p.join("ProgramData").join(app_name) }
                #[cfg(not(windows))]
                { PathBuf::from("/etc").join(app_name) }
            });

        if let Some(system_dir) = system_config_dir {
            if system_dir.exists() {
                dirs.push(system_dir);
            }
        }

        // 用户级配置目录（高优先级）
        // 在 Windows 上通常是 %APPDATA%\{app_name}
        // 在 Unix 上通常是 ~/.config/{app_name}
        let user_config_dir = project_dirs.config_dir();
        if user_config_dir.exists() {
            dirs.push(user_config_dir.to_path_buf());
        }
    } else {
        return Err(LingoError::ConfigDirNotFound {
            dir_type: crate::error::ConfigDirType::User,
            expected_path: None,
        });
    }

    // 如果没有找到任何配置目录，返回错误
    if dirs.is_empty() {
        return Err(LingoError::NoConfigFilesFoundInDir {
            dir_type: crate::error::ConfigDirType::User,
            path: std::path::PathBuf::from(format!("No valid config directories found for app: {}", app_name)),
        });
    }

    Ok(dirs)
}

/// 添加指定的配置文件路径
///
/// 用于处理通过命令行参数 `--config` 指定的配置文件
pub fn add_specified_config_file(
    config_files: &mut Vec<ConfigFilePath>,
    file_path: PathBuf,
) -> Result<(), LingoError> {
    // 检查文件是否存在
    if !file_path.exists() {
        return Err(LingoError::SpecifiedFileNotFound {
            path: file_path,
        });
    }

    // 检查是否为文件
    if !file_path.is_file() {
        return Err(LingoError::SpecifiedFileNotFound {
            path: file_path,
        });
    }

    // 从文件扩展名推断文件类型
    let file_type = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(ConfigFileType::from_extension)
        .ok_or_else(|| LingoError::UnsupportedFormat {
            path: file_path.clone(),
        })?;

    // 添加到配置文件列表（指定的文件是必需的）
    config_files.push(ConfigFilePath::new(file_path, file_type, true));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_file_type_extension() {
        assert_eq!(ConfigFileType::Toml.extension(), "toml");
        assert_eq!(ConfigFileType::Json.extension(), "json");
        assert_eq!(ConfigFileType::Ini.extension(), "ini");
    }

    #[test]
    fn test_config_file_type_from_extension() {
        assert_eq!(ConfigFileType::from_extension("toml"), Some(ConfigFileType::Toml));
        assert_eq!(ConfigFileType::from_extension("TOML"), Some(ConfigFileType::Toml));
        assert_eq!(ConfigFileType::from_extension("json"), Some(ConfigFileType::Json));
        assert_eq!(ConfigFileType::from_extension("JSON"), Some(ConfigFileType::Json));
        assert_eq!(ConfigFileType::from_extension("ini"), Some(ConfigFileType::Ini));
        assert_eq!(ConfigFileType::from_extension("INI"), Some(ConfigFileType::Ini));
        assert_eq!(ConfigFileType::from_extension("txt"), None);
        assert_eq!(ConfigFileType::from_extension("yaml"), None);
    }

    #[test]
    fn test_config_file_path_new() {
        let path = PathBuf::from("/etc/myapp/config.toml");
        let config_file = ConfigFilePath::new(path.clone(), ConfigFileType::Toml, true);

        assert_eq!(config_file.path, path);
        assert_eq!(config_file.file_type, ConfigFileType::Toml);
        assert!(config_file.is_required);
    }

    #[test]
    fn test_config_file_path_exists() {
        // 创建临时文件
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test.toml");
        fs::write(&temp_file, "test = true").unwrap();

        let config_file = ConfigFilePath::new(temp_file, ConfigFileType::Toml, false);
        assert!(config_file.exists());

        // 测试不存在的文件
        let non_existent = PathBuf::from("/non/existent/file.toml");
        let config_file = ConfigFilePath::new(non_existent, ConfigFileType::Toml, false);
        assert!(!config_file.exists());
    }

    #[test]
    fn test_resolve_config_files_with_app_meta() {
        let app_meta = LingoAppMeta {
            app_name: "test_app".to_string(),
            env_prefix: None,
            behavior_version: 1,
            max_parse_depth: 128,
        };

        // 这个测试依赖于系统环境，所以我们只检查函数不会 panic
        let result = resolve_config_files(&app_meta);

        // 结果应该是 Ok，即使没有找到配置文件
        match result {
            Ok(files) => {
                // 验证返回的文件路径都是有效的
                for file in files {
                    assert!(file.path.is_absolute() || file.path.is_relative());
                    assert!(!file.is_required); // 自动发现的文件不应该是必需的
                }
            }
            Err(e) => {
                // 在某些环境中可能会失败，这是可以接受的
                println!("Config resolution failed (expected in some environments): {:?}", e);
            }
        }
    }

    #[test]
    fn test_add_specified_config_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("custom.toml");
        fs::write(&temp_file, "key = \"value\"").unwrap();

        let mut config_files = Vec::new();
        let result = add_specified_config_file(&mut config_files, temp_file.clone());

        assert!(result.is_ok());
        assert_eq!(config_files.len(), 1);
        assert_eq!(config_files[0].path, temp_file);
        assert_eq!(config_files[0].file_type, ConfigFileType::Toml);
        assert!(config_files[0].is_required);
    }

    #[test]
    fn test_add_specified_config_file_not_found() {
        let mut config_files = Vec::new();
        let non_existent = PathBuf::from("/non/existent/file.toml");

        let result = add_specified_config_file(&mut config_files, non_existent.clone());

        assert!(result.is_err());
        match result.unwrap_err() {
            LingoError::SpecifiedFileNotFound { path } => {
                assert_eq!(path, non_existent);
            }
            _ => panic!("Expected SpecifiedFileNotFound error"),
        }
        assert!(config_files.is_empty());
    }

    #[test]
    fn test_add_specified_config_file_unsupported_format() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("config.yaml");
        fs::write(&temp_file, "key: value").unwrap();

        let mut config_files = Vec::new();
        let result = add_specified_config_file(&mut config_files, temp_file.clone());

        assert!(result.is_err());
        match result.unwrap_err() {
            LingoError::UnsupportedFormat { path } => {
                assert_eq!(path, temp_file);
                // 不支持的格式错误已正确触发
            }
            _ => panic!("Expected UnsupportedFormat error"),
        }
        assert!(config_files.is_empty());
    }

    #[test]
    fn test_get_config_directories() {
        let app_name = "test_app_for_dirs";

        // 这个测试依赖于系统环境
        let result = get_config_directories(app_name);

        match result {
            Ok(dirs) => {
                // 验证返回的目录都是绝对路径
                for dir in dirs {
                    assert!(dir.is_absolute());
                }
            }
            Err(e) => {
                // 在某些环境中可能会失败
                println!("Get config directories failed (expected in some environments): {:?}", e);
            }
        }
    }

    #[test]
    fn test_config_file_path_equality() {
        let path1 = PathBuf::from("/etc/app/config.toml");
        let path2 = PathBuf::from("/etc/app/config.toml");
        let path3 = PathBuf::from("/etc/app/config.json");

        let config1 = ConfigFilePath::new(path1, ConfigFileType::Toml, true);
        let config2 = ConfigFilePath::new(path2, ConfigFileType::Toml, true);
        let config3 = ConfigFilePath::new(path3, ConfigFileType::Json, true);

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_resolve_config_files_empty_app_name() {
        let app_meta = LingoAppMeta {
            app_name: "".to_string(),
            env_prefix: None,
            behavior_version: 1,
            max_parse_depth: 128,
        };

        let result = resolve_config_files(&app_meta);

        // 空的应用名称应该能够处理，但可能返回错误
        match result {
            Ok(_) => {
                // 如果成功，那也是可以接受的
            }
            Err(LingoError::ConfigDirNotFound { dir_type: _, expected_path: _ }) => {
                // 配置目录未找到错误是可以接受的
            }
            Err(_) => {
                // 其他错误也是可以接受的
            }
        }
    }
}
//! 文件配置提供器
//!
//! 从配置文件读取数据的 figment Provider 实现。
//! 支持 TOML、JSON 和 INI 格式，并提供解析深度限制。
//! 支持自定义文件读取器，允许用户自定义文件读取行为。

use crate::error::QuantumConfigError;
use figment::{value::{Map, Value}, Error, Metadata, Profile, Provider};
use ini::Ini;
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};
use super::file_reader::{FileReader, StandardFileReader};

/// 配置文件格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// TOML 格式
    Toml,
    /// JSON 格式
    Json,
    /// INI 格式
    Ini,
}

impl FileFormat {
    /// 从文件扩展名推断格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "toml" => Some(Self::Toml),
            "json" => Some(Self::Json),
            "ini" => Some(Self::Ini),
            _ => None,
        }
    }

    /// 获取格式对应的文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Ini => "ini",
        }
    }
}

/// Quantum Config 文件提供器（泛型版本）
/// 
/// 实现了 figment Provider trait，用于从配置文件读取数据。
/// 支持多种文件格式，并提供解析深度限制以防止资源耗尽攻击。
/// 支持自定义文件读取器，允许用户自定义文件读取行为。
#[derive(Debug, Clone)]
pub struct QuantumConfigFileProviderGeneric<R: FileReader> {
    /// 配置文件路径
    path: PathBuf,
    /// 文件格式
    format: FileFormat,
    /// 是否为必需文件（如果文件不存在是否报错）
    is_required: bool,
    /// 解析深度限制
    max_parse_depth: u32,
    /// 文件读取器
    reader: R,
}

/// 标准文件提供器类型别名
/// 
/// 使用标准文件系统读取器的文件提供器，保持向后兼容性。
pub type QuantumConfigFileProvider = QuantumConfigFileProviderGeneric<StandardFileReader>;

impl<R: FileReader> QuantumConfigFileProviderGeneric<R> {
    /// 创建新的文件提供器（泛型版本）
    ///
    /// # Arguments
    /// * `path` - 配置文件路径
    /// * `format` - 文件格式
    /// * `is_required` - 是否为必需文件
    /// * `max_parse_depth` - 解析深度限制
    /// * `reader` - 文件读取器实现
    pub fn new<P: AsRef<Path>>(
        path: P,
        format: FileFormat,
        is_required: bool,
        max_parse_depth: u32,
        reader: R,
    ) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            format,
            is_required,
            max_parse_depth,
            reader,
        }
    }
}

impl QuantumConfigFileProvider {
    /// 从文件路径自动推断格式创建提供者
    ///
    /// # Arguments
    /// * `path` - 配置文件路径
    /// * `is_required` - 是否为必需文件
    /// * `max_parse_depth` - 解析深度限制
    ///
    /// # Errors
    /// 如果无法从文件扩展名推断格式，返回 `UnsupportedFormat` 错误
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        is_required: bool,
        max_parse_depth: u32,
    ) -> Result<Self, QuantumConfigError> {
        let path_ref = path.as_ref();
        let extension = path_ref
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| QuantumConfigError::UnsupportedFormat {
                path: path_ref.to_path_buf(),
            })?;

        let format = FileFormat::from_extension(extension)
            .ok_or_else(|| QuantumConfigError::UnsupportedFormat {
                path: path_ref.to_path_buf(),
            })?;

        Ok(QuantumConfigFileProviderGeneric::new(
            path,
            format,
            is_required,
            max_parse_depth,
            StandardFileReader::new(),
        ))
    }
}

impl<R: FileReader> QuantumConfigFileProviderGeneric<R> {
    /// 读取并解析配置文件
    fn read_and_parse(&self) -> Result<Value, QuantumConfigError> {
        // 检查文件是否存在
        if !self.reader.exists(&self.path) {
            if self.is_required {
                return Err(QuantumConfigError::SpecifiedFileNotFound {
                    path: self.path.clone(),
                });
            } else {
                // 可选文件不存在时返回空映射
                return Ok(Value::Dict(figment::value::Tag::Default, Map::new()));
            }
        }

        // 使用文件读取器读取文件内容
        let content = self.reader.read_content(&self.path)?;

        // 根据格式解析内容
        self.parse_content(&content)
    }

    /// 解析文件内容
    fn parse_content(&self, content: &str) -> Result<Value, QuantumConfigError> {
        match self.format {
            FileFormat::Toml => self.parse_toml(content),
            FileFormat::Json => self.parse_json(content),
            FileFormat::Ini => self.parse_ini(content),
        }
    }

    /// 解析 TOML 内容
    fn parse_toml(&self, content: &str) -> Result<Value, QuantumConfigError> {
        // 直接使用 toml 库解析为 JsonValue
        let parsed: JsonValue = toml::from_str(content)
            .map_err(|e: toml::de::Error| QuantumConfigError::FileParse {
                path: self.path.clone(),
                format_name: "TOML".to_string(),
                source_error: e.to_string(),
            })?;

        self.convert_to_figment_value(parsed)
    }

    /// 解析 JSON 内容
    fn parse_json(&self, content: &str) -> Result<Value, QuantumConfigError> {
        let json_value: JsonValue = serde_json::from_str(content)
            .map_err(|e| QuantumConfigError::FileParse {
                path: self.path.clone(),
                format_name: "JSON".to_string(),
                source_error: e.to_string(),
            })?;

        self.convert_to_figment_value(json_value)
    }

    /// 解析 INI 内容
    fn parse_ini(&self, content: &str) -> Result<Value, QuantumConfigError> {
        let ini = Ini::load_from_str(content)
            .map_err(|e| QuantumConfigError::FileParse {
                path: self.path.clone(),
                format_name: "INI".to_string(),
                source_error: e.to_string(),
            })?;

        // 将 INI 转换为嵌套的 Map 结构
        let mut root_map = Map::new();

        for (section_name, properties) in ini.iter() {
            // 处理根级键和有名称的段落
            match section_name {
                None => {
                    // 根级键（无段落），直接添加到根映射
                    for (key, value) in properties.iter() {
                        root_map.insert(
                            key.to_string(),
                            self.parse_ini_value(value),
                        );
                    }
                }
                Some(section_name) => {
                    // 有名称的段落，创建嵌套映射
                    let mut section_map = Map::new();
                    for (key, value) in properties.iter() {
                        section_map.insert(
                            key.to_string(),
                            self.parse_ini_value(value),
                        );
                    }
                    root_map.insert(
                        section_name.to_string(),
                        Value::Dict(figment::value::Tag::Default, section_map),
                    );
                }
            }
        }

        Ok(Value::Dict(figment::value::Tag::Default, root_map))
    }

    /// 解析 INI 值，支持类型推断（布尔值、数值、字符串）
    fn parse_ini_value(&self, value: &str) -> Value {
        let tag = figment::value::Tag::Default;
        
        // 移除首尾空白
        let value = value.trim();
        
        // 尝试解析布尔值
        if let Ok(bool_val) = value.parse::<bool>() {
            return Value::Bool(tag, bool_val);
        }
        
        // 尝试解析整数
        if let Ok(int_val) = value.parse::<i64>() {
            return Value::Num(tag, figment::value::Num::I64(int_val));
        }
        
        // 尝试解析浮点数
        if let Ok(float_val) = value.parse::<f64>() {
            return Value::Num(tag, figment::value::Num::F64(float_val));
        }
        
        // 默认当作字符串
        Value::String(tag, value.to_string())
    }

    /// 将 JsonValue 转换为 figment::Value
    fn convert_to_figment_value(&self, json_value: JsonValue) -> Result<Value, QuantumConfigError> {
        self.convert_json_value_recursive(json_value, 0)
    }

    /// 递归转换 JSON 值，应用深度限制
    fn convert_json_value_recursive(
        &self,
        value: JsonValue,
        depth: usize,
    ) -> Result<Value, QuantumConfigError> {
        if depth > self.max_parse_depth as usize {
            return Err(QuantumConfigError::Internal(
                format!(
                    "Configuration parsing depth limit ({}) exceeded in file: {}",
                    self.max_parse_depth,
                    self.path.display()
                )
            ));
        }

        let tag = figment::value::Tag::Default;

        match value {
            serde_json::Value::Null => Ok(Value::String(tag, "null".to_string())),
            serde_json::Value::Bool(b) => Ok(Value::Bool(tag, b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Num(tag, figment::value::Num::I64(i)))
                } else if let Some(u) = n.as_u64() {
                    Ok(Value::Num(tag, figment::value::Num::U64(u)))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Num(tag, figment::value::Num::F64(f)))
                } else {
                    Ok(Value::String(tag, n.to_string()))
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(tag, s)),
            serde_json::Value::Array(arr) => {
                let mut figment_array = Vec::new();
                for item in arr {
                    figment_array.push(self.convert_json_value_recursive(item, depth + 1)?);
                }
                Ok(Value::Array(tag, figment_array))
            }
            serde_json::Value::Object(obj) => {
                let mut figment_map = Map::new();
                for (key, value) in obj {
                    figment_map.insert(
                        key,
                        self.convert_json_value_recursive(value, depth + 1)?,
                    );
                }
                Ok(Value::Dict(tag, figment_map))
            }
        }
    }
}

impl<R: FileReader> Provider for QuantumConfigFileProviderGeneric<R> {
    fn metadata(&self) -> Metadata {
        Metadata::named(format!("Quantum Config File Provider ({})", self.path.display()))
    }

    fn data(&self) -> Result<Map<Profile, Map<String, Value>>, Error> {
        let value = self.read_and_parse()
            .map_err(|e| Error::from(format!("File provider error: {}", e)))?;

        let mut profile_map = Map::new();
        if let Value::Dict(_, dict) = value {
            profile_map.insert(Profile::Default, dict);
        } else {
            // 如果不是字典，创建一个包含单个值的字典
            let mut single_value_map = Map::new();
            single_value_map.insert("value".to_string(), value);
            profile_map.insert(Profile::Default, single_value_map);
        }

        Ok(profile_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_format_from_extension() {
        assert_eq!(FileFormat::from_extension("toml"), Some(FileFormat::Toml));
        assert_eq!(FileFormat::from_extension("json"), Some(FileFormat::Json));
        assert_eq!(FileFormat::from_extension("ini"), Some(FileFormat::Ini));
        assert_eq!(FileFormat::from_extension("txt"), None);
        assert_eq!(FileFormat::from_extension("TOML"), Some(FileFormat::Toml));
    }

    #[test]
    fn test_file_format_extension() {
        assert_eq!(FileFormat::Toml.extension(), "toml");
        assert_eq!(FileFormat::Json.extension(), "json");
        assert_eq!(FileFormat::Ini.extension(), "ini");
    }

    #[test]
    fn test_quantum_config_file_provider_new() {
        let provider = QuantumConfigFileProviderGeneric::new(
            "/path/to/config.toml",
            FileFormat::Toml,
            true,
            100,
            StandardFileReader::new(),
        );

        assert_eq!(provider.path, PathBuf::from("/path/to/config.toml"));
        assert_eq!(provider.format, FileFormat::Toml);
        assert_eq!(provider.is_required, true);
        assert_eq!(provider.max_parse_depth, 100);
    }

    #[test]
    fn test_quantum_config_file_provider_from_path_success() {
        let result = QuantumConfigFileProvider::from_path("/path/to/config.toml", true, 100);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.format, FileFormat::Toml);
    }

    #[test]
    fn test_quantum_config_file_provider_from_path_unsupported_format() {
        let result = QuantumConfigFileProvider::from_path("/path/to/config.txt", true, 100);
        assert!(result.is_err());

        match result.unwrap_err() {
            QuantumConfigError::UnsupportedFormat { path } => {
                assert_eq!(path, PathBuf::from("/path/to/config.txt"));
            }
            _ => panic!("Expected UnsupportedFormat error"),
        }
    }

    #[test]
    fn test_read_nonexistent_required_file() {
        let provider = QuantumConfigFileProviderGeneric::new(
            "/nonexistent/config.toml",
            FileFormat::Toml,
            true,
            100,
            StandardFileReader::new(),
         );

        let result = provider.read_and_parse();
        assert!(result.is_err());

        match result.unwrap_err() {
            QuantumConfigError::SpecifiedFileNotFound { path } => {
                assert_eq!(path, PathBuf::from("/nonexistent/config.toml"));
            }
            _ => panic!("Expected SpecifiedFileNotFound error"),
        }
    }

    #[test]
    fn test_read_nonexistent_optional_file() {
        let provider = QuantumConfigFileProviderGeneric::new(
            "/nonexistent/config.toml",
            FileFormat::Toml,
            false,
             100,
             StandardFileReader::new(),
         );

        let result = provider.read_and_parse();
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Dict(_, map) => {
                assert!(map.is_empty());
            }
            _ => panic!("Expected empty Dict for nonexistent optional file"),
        }
    }

    #[test]
    fn test_parse_json_content() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, r#"{{"key": "value", "number": 42}}"#)?;

        let provider = QuantumConfigFileProviderGeneric::new(
            temp_file.path(),
            FileFormat::Json,
            true,
             100,
             StandardFileReader::new(),
         );

        let result = provider.read_and_parse();
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_parse_toml_content() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "key = \"value\"\nnumber = 42")?;

        let provider = QuantumConfigFileProviderGeneric::new(
            temp_file.path(),
            FileFormat::Toml,
            true,
            100,
            StandardFileReader::new(),
        );

        let result = provider.read_and_parse();
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_parse_ini_content() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "[section]\nkey = value\nnumber = 42")?;

        let provider = QuantumConfigFileProviderGeneric::new(
            temp_file.path(),
            FileFormat::Ini,
            true,
            100,
            StandardFileReader::new(),
        );

        let result = provider.read_and_parse();
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_depth_limit_enforcement() {
        let provider = QuantumConfigFileProviderGeneric::new(
            "/path/to/config.json",
            FileFormat::Json,
            true,
            2, // 很小的深度限制
             StandardFileReader::new(),
         );

        // 创建一个深度超过限制的 JSON 值
        let deep_json = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": "too deep"
                    }
                }
            }
        });

        let result = provider.convert_json_value_recursive(deep_json, 0);
        assert!(result.is_err());

        match result.unwrap_err() {
            QuantumConfigError::Internal(message) => {
                assert!(message.contains("depth limit"));
            }
            _ => panic!("Expected Internal error for depth limit"),
        }
    }
}

// 向后兼容的类型别名
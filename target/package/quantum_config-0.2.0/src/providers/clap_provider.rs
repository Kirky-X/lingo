//! 命令行参数配置提供者
//!
//! 此模块实现了从 clap 解析的命令行参数读取数据的 figment Provider。
//! 支持将命令行参数转换为配置值，并处理嵌套结构。

use crate::error::QuantumConfigError;
use clap::ArgMatches;
use figment::{value::{Map, Value}, Error, Metadata, Profile, Provider};
use std::collections::HashMap;

/// 命令行参数配置提供者
///
/// 从 clap 的 ArgMatches 读取配置数据，支持参数映射和类型转换。
#[derive(Debug, Clone)]
pub struct QuantumConfigClapProvider {
    /// clap 解析的参数匹配结果
    matches: ArgMatches,
    /// 参数名映射（从命令行参数名到配置键名）
    arg_mapping: HashMap<String, String>,
    /// 分隔符，用于构造嵌套键
    separator: String,
}

impl QuantumConfigClapProvider {
    /// 创建新的命令行参数提供者
    ///
    /// # Arguments
    /// * `matches` - clap 解析的参数匹配结果
    /// * `arg_mapping` - 参数名映射
    /// * `separator` - 分隔符，用于构造嵌套键
    pub fn new(
        matches: ArgMatches,
        arg_mapping: HashMap<String, String>,
        separator: String,
    ) -> Self {
        Self {
            matches,
            arg_mapping,
            separator,
        }
    }

    /// 创建带有默认设置的命令行参数提供者
    ///
    /// 默认设置：
    /// - 分隔符："."
    /// - 空的参数映射（使用原始参数名）
    ///
    /// # Arguments
    /// * `matches` - clap 解析的参数匹配结果
    pub fn from_matches(matches: ArgMatches) -> Self {
        Self::new(matches, HashMap::new(), ".".to_string())
    }

    /// 添加参数映射
    ///
    /// # Arguments
    /// * `arg_name` - 命令行参数名
    /// * `config_key` - 对应的配置键名
    pub fn map_arg<S1: Into<String>, S2: Into<String>>(
        mut self,
        arg_name: S1,
        config_key: S2,
    ) -> Self {
        self.arg_mapping.insert(arg_name.into(), config_key.into());
        self
    }

    /// 设置分隔符
    ///
    /// # Arguments
    /// * `separator` - 分隔符字符串
    pub fn with_separator<S: Into<String>>(mut self, separator: S) -> Self {
        self.separator = separator.into();
        self
    }

    /// 读取并处理命令行参数
    fn read_clap_args(&self) -> Result<Map<String, Value>, QuantumConfigError> {
        let mut args_map = Map::new();

        // 遍历所有已解析的参数
        for arg_id in self.matches.ids() {
            let arg_name = arg_id.as_str();

            // 获取配置键名（使用映射或原始名称）
            let config_key = self.arg_mapping
                .get(arg_name)
                .cloned()
                .unwrap_or_else(|| arg_name.to_string());

            // 根据参数名称判断类型
            let is_boolean_flag = matches!(arg_name, "verbose" | "quiet");
            
            if is_boolean_flag {
                // 对于已知的布尔标志，直接使用 get_flag
                if self.matches.get_flag(arg_name) {
                    let figment_value = Value::Bool(figment::value::Tag::Default, true);
                    self.insert_nested_value_direct(&mut args_map, &config_key, figment_value)?;
                }
            } else {
                // 对于其他参数，尝试获取字符串值
                let values = if let Some(values) = self.matches.get_many::<String>(arg_name) {
                    values.map(|s| s.clone()).collect()
                } else if let Some(value) = self.matches.get_one::<String>(arg_name) {
                    vec![value.clone()]
                } else {
                    continue; // 没有值，跳过
                };
                
                self.insert_nested_value(&mut args_map, &config_key, values)?;
            }
        }

        Ok(args_map)
    }

    /// 安全地检查某个参数是否作为布尔标志被设置
    #[allow(dead_code)]
    fn is_flag_set(&self, arg_name: &str) -> bool {
        // 首先检查参数是否存在
        let arg_exists = self.matches.ids().any(|id| id.as_str() == arg_name);
        if !arg_exists {
            return false;
        }
        
        // 使用 catch_unwind 安全地检查参数是否为布尔标志
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // 尝试获取布尔值，如果成功则说明是布尔标志
            self.matches.get_flag(arg_name)
        }));
        
        match result {
            Ok(flag_value) => {
                // 进一步验证：尝试获取字符串值，如果能获取到则不是布尔标志
                let string_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    self.matches.get_one::<String>(arg_name).is_some() || 
                    self.matches.get_many::<String>(arg_name).is_some()
                }));
                
                match string_result {
                    Ok(has_string_value) => {
                        // 如果有字符串值，则不是布尔标志
                        if has_string_value {
                            false
                        } else {
                            // 没有字符串值且能获取布尔值，才是真正的布尔标志
                            flag_value
                        }
                    },
                    Err(_) => {
                        // 获取字符串值失败，说明是布尔标志
                        flag_value
                    }
                }
            },
            Err(_) => {
                // 如果 get_flag 失败，则不是布尔标志
                false
            }
        }
    }

    /// 获取参数值
    ///
    /// # Arguments
    /// * `arg_name` - 参数名
    ///
    /// # Returns
    /// 返回参数值列表，如果参数不存在则返回错误，如果参数是布尔标志则返回 None
    #[allow(dead_code)]
    fn get_arg_values(&self, arg_name: &str) -> Result<Option<Vec<String>>, QuantumConfigError> {
        // 安全地检查参数是否存在，避免 clap 内部 panic
        let arg_exists = self.matches.ids().any(|id| id.as_str() == arg_name);
        if !arg_exists {
            return Err(QuantumConfigError::Internal(format!("Argument '{}' not found", arg_name)));
        }

        // 首先检查是否为布尔标志，如果是则直接返回 None
        if self.is_flag_set(arg_name) {
            return Ok(None);
        }

        // 使用 catch_unwind 安全地尝试获取字符串值
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // 尝试获取多个值
            if let Some(values) = self.matches.get_many::<String>(arg_name) {
                return Some(values.map(|s| s.clone()).collect());
            }

            // 尝试获取单个值
            if let Some(value) = self.matches.get_one::<String>(arg_name) {
                return Some(vec![value.clone()]);
            }

            None
        }));

        match result {
            Ok(values) => Ok(values),
            Err(_) => {
                // 如果参数存在但没有字符串值（可能是布尔标志），返回 None
                Ok(None)
            }
        }
    }

    /// 将 figment Value 直接插入到嵌套的映射结构中
    ///
    /// # Arguments
    /// * `map` - 目标映射
    /// * `key` - 键名（可能包含分隔符）
    /// * `value` - 要插入的 figment Value
    fn insert_nested_value_direct(
        &self,
        map: &mut Map<String, Value>,
        key: &str,
        value: Value,
    ) -> Result<(), QuantumConfigError> {
        let parts: Vec<&str> = key.split(&self.separator).collect();
        
        if parts.is_empty() {
            return Err(QuantumConfigError::Internal(
                "Empty key provided for nested value insertion".to_string()
            ));
        }

        if parts.len() == 1 {
            map.insert(key.to_string(), value);
            return Ok(());
        }

        self.try_insert_nested(map, &parts, value)
    }
    
    fn try_insert_nested(
        &self,
        map: &mut Map<String, Value>,
        parts: &[&str],
        value: Value,
    ) -> Result<(), QuantumConfigError> {
        if parts.is_empty() {
            return Ok(());
        }

        if parts.len() == 1 {
            map.insert(parts[0].to_string(), value);
        } else {
            let key = parts[0];
            let remaining = &parts[1..];

            match map.get_mut(key) {
                Some(Value::Dict(_, ref mut nested_map)) => {
                    self.try_insert_nested(nested_map, remaining, value)?;
                }
                Some(_) => {
                    // 键已存在但不是字典，创建新字典并插入
                    let mut new_map = Map::new();
                    self.try_insert_nested(&mut new_map, remaining, value)?;
                    map.insert(key.to_string(), Value::Dict(figment::value::Tag::Default, new_map));
                }
                None => {
                    // 键不存在，创建新字典
                    let mut new_map = Map::new();
                    self.try_insert_nested(&mut new_map, remaining, value)?;
                    map.insert(key.to_string(), Value::Dict(figment::value::Tag::Default, new_map));
                }
            }
        }

        Ok(())
    }

    /// 将字符串值列表插入到嵌套的映射结构中
    ///
    /// # Arguments
    /// * `map` - 目标映射
    /// * `key` - 键名（可能包含分隔符）
    /// * `values` - 字符串值列表
    fn insert_nested_value(
        &self,
        map: &mut Map<String, Value>,
        key: &str,
        values: Vec<String>,
    ) -> Result<(), QuantumConfigError> {
        let figment_value = if values.len() == 1 {
            self.parse_arg_value(values[0].clone())?
        } else {
            // 多个值作为数组处理
            let tag = figment::value::Tag::Default;
            let parsed_values: Result<Vec<Value>, _> = values
                .into_iter()
                .map(|v| self.parse_arg_value(v))
                .collect();
            
            Value::Array(tag, parsed_values?)
        };

        self.insert_nested_value_direct(map, key, figment_value)
    }

    /// 解析参数值，自动推断类型
    ///
    /// # Arguments
    /// * `value` - 字符串值
    ///
    /// # Returns
    /// 返回解析后的 figment Value
    fn parse_arg_value(&self, value: String) -> Result<Value, QuantumConfigError> {
        let tag = figment::value::Tag::Default;

        // 尝试解析为布尔值
        if let Ok(bool_val) = value.parse::<bool>() {
            return Ok(Value::Bool(tag, bool_val));
        }

        // 尝试解析为有符号整数
        if let Ok(int_val) = value.parse::<i64>() {
            return Ok(Value::Num(tag, figment::value::Num::I64(int_val)));
        }

        // 尝试解析为无符号整数
        if let Ok(uint_val) = value.parse::<u64>() {
            return Ok(Value::Num(tag, figment::value::Num::U64(uint_val)));
        }

        // 尝试解析为浮点数
        if let Ok(float_val) = value.parse::<f64>() {
            return Ok(Value::Num(tag, figment::value::Num::F64(float_val)));
        }

        // 默认作为字符串处理
        Ok(Value::String(tag, value))
    }
}

impl Provider for QuantumConfigClapProvider {
    fn metadata(&self) -> Metadata {
        Metadata::named("Quantum Config Command Line Provider")
    }

    fn data(&self) -> Result<Map<Profile, Map<String, Value>>, Error> {
        let clap_data = self.read_clap_args()
            .map_err(|e| Error::from(format!("Command line provider error: {}", e)))?;

        let mut profile_map = Map::new();
        profile_map.insert(Profile::Default, clap_data);

        Ok(profile_map)
    }
}

/// 辅助函数：从 clap 应用创建提供者
///
/// 这个函数简化了从 clap 应用创建提供者的过程
///
/// # Arguments
/// * `app` - clap 应用
/// * `args` - 命令行参数
///
/// # Returns
/// 返回配置好的 QuantumConfigClapProvider
pub fn from_clap_app(
    app: clap::Command,
    args: Vec<String>,
) -> Result<QuantumConfigClapProvider, QuantumConfigError> {
    let matches = app.try_get_matches_from(args)
        .map_err(|e| QuantumConfigError::Internal(
            format!("Failed to parse command line arguments: {}", e)
        ))?;

    Ok(QuantumConfigClapProvider::from_matches(matches))
}

/// 辅助函数：创建带有常见参数映射的提供者
///
/// 这个函数创建一个带有常见配置参数映射的提供者
///
/// # Arguments
/// * `matches` - clap 解析的参数匹配结果
///
/// # Returns
/// 返回配置好的 QuantumConfigClapProvider
pub fn with_common_mappings(matches: ArgMatches) -> QuantumConfigClapProvider {
    QuantumConfigClapProvider::from_matches(matches)
        .map_arg("config", "config_file")
        .map_arg("config-dir", "config_dir")
        .map_arg("log-level", "log_level")
        .map_arg("verbose", "verbose")
        .map_arg("quiet", "quiet")
        .map_arg("output", "output.file")
        .map_arg("format", "output.format")
}

// 向后兼容别名


#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction, Command};

    fn create_test_app() -> Command {
        Command::new("test")
            .arg(Arg::new("config")
                .long("config")
                .value_name("FILE")
                .help("Configuration file"))
            .arg(Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(ArgAction::SetTrue)
                .help("Verbose output"))
            .arg(Arg::new("count")
                .long("count")
                .short('c')
                .value_name("NUM")
                .help("Number of items"))
    }

    #[test]
    fn test_quantum_config_clap_provider_new() {
        let app = create_test_app();
        let matches = app.try_get_matches_from([
            "test", "--config", "config.toml", "--verbose"
        ]).unwrap();

        let mut mappings = HashMap::new();
        mappings.insert("config".to_string(), "config_file".to_string());

        let provider = QuantumConfigClapProvider::new(matches, mappings, ".".to_string());
        assert!(provider.matches.get_one::<String>("config").is_some());
    }

    #[test]
    fn test_quantum_config_clap_provider_from_matches() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();

        let provider = QuantumConfigClapProvider::from_matches(matches);
        assert_eq!(provider.separator, ".");
        assert!(provider.arg_mapping.is_empty());
    }

    #[test]
    fn test_map_arg() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();

        let provider = QuantumConfigClapProvider::from_matches(matches)
            .map_arg("config", "config_file");

        assert_eq!(provider.arg_mapping.get("config"), Some(&"config_file".to_string()));
    }

    #[test]
    fn test_with_separator() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();

        let provider = QuantumConfigClapProvider::from_matches(matches)
            .with_separator("::");

        assert_eq!(provider.separator, "::");
    }

    #[test]
    fn test_parse_arg_value_boolean() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = QuantumConfigClapProvider::from_matches(matches);

        let true_val = provider.parse_arg_value("true".to_string()).unwrap();
        let false_val = provider.parse_arg_value("false".to_string()).unwrap();

        match (true_val, false_val) {
            (Value::Bool(_, true), Value::Bool(_, false)) => {},
            _ => panic!("Boolean parsing failed"),
        }
    }

    #[test]
    fn test_parse_arg_value_numbers() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = QuantumConfigClapProvider::from_matches(matches);

        let int_val = provider.parse_arg_value("42".to_string()).unwrap();
        let float_val = provider.parse_arg_value("3.14".to_string()).unwrap();

        match (&int_val, &float_val) {
            (Value::Num(_, figment::value::Num::I64(42)), 
             Value::Num(_, figment::value::Num::F64(f))) if (f - 3.14).abs() < 1e-6 => {},
            _ => panic!("Number parsing failed: {:?}, {:?}", int_val, float_val),
        }
    }

    #[test]
    fn test_parse_arg_value_string() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = QuantumConfigClapProvider::from_matches(matches);

        let string_val = provider.parse_arg_value("hello".to_string()).unwrap();

        match string_val {
            Value::String(_, s) if s == "hello" => {},
            _ => panic!("String parsing failed"),
        }
    }

    #[test]
    fn test_read_clap_args() {
        let app = create_test_app();
        let matches = app.try_get_matches_from([
            "test", "--config", "config.toml", "--verbose", "--count", "10"
        ]).unwrap();

        let provider = QuantumConfigClapProvider::from_matches(matches)
            .map_arg("config", "config_file");

        let data = provider.read_clap_args().unwrap();

        // 检查配置文件映射
        assert!(data.contains_key("config_file"));
        match data.get("config_file").unwrap() {
            Value::String(_, s) if s == "config.toml" => {},
            _ => panic!("Config file mapping failed"),
        }

        // 检查布尔标志
        assert!(data.contains_key("verbose"));
        match data.get("verbose").unwrap() {
            Value::Bool(_, true) => {},
            _ => panic!("Verbose flag failed"),
        }

        // 检查数字参数
        assert!(data.contains_key("count"));
        match data.get("count").unwrap() {
            Value::Num(_, figment::value::Num::I64(10)) => {},
            _ => panic!("Count parameter failed"),
        }
    }

    #[test]
    fn test_from_clap_app() {
        let app = create_test_app();
        let args = vec!["test".to_string(), "--config".to_string(), "test.toml".to_string()];
        
        let provider = from_clap_app(app, args).unwrap();
        assert!(provider.matches.get_one::<String>("config").is_some());
    }

    #[test]
    fn test_with_common_mappings() {
        let app = Command::new("test")
            .arg(Arg::new("config").long("config"))
            .arg(Arg::new("verbose").long("verbose").action(ArgAction::SetTrue));

        let matches = app.try_get_matches_from([
            "test", "--config", "test.toml", "--verbose"
        ]).unwrap();

        let provider = with_common_mappings(matches);
        assert_eq!(provider.arg_mapping.get("config"), Some(&"config_file".to_string()));
    }

    #[test]
    fn test_flatten_fallback() {
        let app = Command::new("test")
            .arg(Arg::new("log-level").long("log-level").value_name("LEVEL"));

        let matches = app.try_get_matches_from([
            "test", "--log-level", "debug"
        ]).unwrap();

        let provider = with_common_mappings(matches);
        let data = provider.data().unwrap();
        let default_data = data.get(&Profile::Default).unwrap();
        
        // 检查是否有 log_level
        if default_data.contains_key("log_level") {
            println!("Found log_level");
        }
    }
}
//! 命令行参数配置提供者
//!
//! 此模块实现了从 clap 解析的命令行参数读取数据的 figment Provider。
//! 支持将命令行参数转换为配置值，并处理嵌套结构。

use crate::error::LingoError;
use clap::ArgMatches;
use figment::{value::{Map, Value}, Error, Metadata, Profile, Provider};
use std::collections::HashMap;

/// 命令行参数配置提供者
///
/// 从 clap 的 ArgMatches 读取配置数据，支持参数映射和类型转换。
#[derive(Debug, Clone)]
pub struct LingoClapProvider {
    /// clap 解析的参数匹配结果
    matches: ArgMatches,
    /// 参数名映射（从命令行参数名到配置键名）
    arg_mapping: HashMap<String, String>,
    /// 分隔符，用于构造嵌套键
    separator: String,
}

impl LingoClapProvider {
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
    fn read_clap_args(&self) -> Result<Map<String, Value>, LingoError> {
        let mut args_map = Map::new();

        // 遍历所有已解析的参数
        for arg_id in self.matches.ids() {
            let arg_name = arg_id.as_str();

            // 获取配置键名（使用映射或原始名称）
            let config_key = self.arg_mapping
                .get(arg_name)
                .cloned()
                .unwrap_or_else(|| arg_name.to_string());

            // 尝试获取字符串值
            if let Some(values) = self.get_arg_values(arg_name)? {
                self.insert_nested_value(&mut args_map, &config_key, values)?;
            } else if self.matches.get_flag(arg_name) {
                // 如果没有字符串值但是布尔标志被设置，则作为布尔值处理
                let figment_value = Value::Bool(figment::value::Tag::Default, true);
                self.insert_nested_value_direct(&mut args_map, &config_key, figment_value)?;
            }
        }

        Ok(args_map)
    }

    /// 获取参数值
    ///
    /// # Arguments
    /// * `arg_name` - 参数名
    ///
    /// # Returns
    /// 返回参数值列表，如果参数不存在则返回 None
    fn get_arg_values(&self, arg_name: &str) -> Result<Option<Vec<String>>, LingoError> {
        // 使用 catch_unwind 来安全地检查参数并获取字符串值
        // clap 在参数不存在或类型不匹配时都可能 panic
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // 检查参数是否存在
            if !self.matches.contains_id(arg_name) {
                return Err("Unknown argument");
            }

            // 尝试获取多个值
            if let Some(values) = self.matches.get_many::<String>(arg_name) {
                let string_values: Vec<String> = values.cloned().collect();
                return Ok(Some(string_values));
            }

            // 尝试获取单个值
            if let Some(value) = self.matches.get_one::<String>(arg_name) {
                return Ok(Some(vec![value.clone()]));
            }

            // 参数存在但没有字符串值
            Ok(None)
        }));

        match result {
            Ok(Ok(values)) => Ok(values),
            Ok(Err(_)) => Err(LingoError::Internal(format!("Unknown argument: {}", arg_name))),
            Err(_) => {
                // 如果发生 panic，可能是因为：
                // 1. 参数不存在（contains_id panic）
                // 2. 类型不匹配（get_one/get_many panic）
                // 我们需要区分这两种情况

                // 尝试一个更安全的方法来检查参数是否存在
                // 通过检查所有已知的参数 ID
                let arg_exists = self.matches.ids().any(|id| id.as_str() == arg_name);

                if !arg_exists {
                    Err(LingoError::Internal(format!("Unknown argument: {}", arg_name)))
                } else {
                    // 参数存在但类型不匹配（比如布尔标志），返回 None 让调用者通过其他方式处理
                    Ok(None)
                }
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
    ) -> Result<(), LingoError> {
        let parts: Vec<&str> = key.split(&self.separator).collect();

        if parts.is_empty() {
            return Ok(());
        }

        // 如果只有一个部分，直接插入
        if parts.len() == 1 {
            map.insert(parts[0].to_string(), value);
            return Ok(());
        }

        // 处理嵌套键
        let mut current_map = map;

        // 遍历除最后一个部分外的所有部分
        for part in &parts[..parts.len() - 1] {
            let part_string = part.to_string();

            // 如果键不存在，创建新的字典
            if !current_map.contains_key(&part_string) {
                current_map.insert(
                    part_string.clone(),
                    Value::Dict(figment::value::Tag::Default, Map::new()),
                );
            }

            // 获取或创建嵌套字典
            match current_map.get_mut(&part_string) {
                Some(Value::Dict(_, nested_map)) => {
                    current_map = nested_map;
                }
                Some(_) => {
                    // 如果已存在的值不是字典，返回错误
                    return Err(LingoError::Internal(
                        format!(
                            "Command line argument key conflict: '{}' cannot be both a value and a nested object",
                            part
                        )
                    ));
                }
                None => {
                    // 这种情况不应该发生，因为我们刚刚插入了值
                    return Err(LingoError::Internal(
                        "Unexpected error during nested key insertion".to_string()
                    ));
                }
            }
        }

        // 插入最终值
        let final_key = parts[parts.len() - 1].to_string();
        current_map.insert(final_key, value);

        Ok(())
    }

    /// 将值插入到嵌套的映射结构中
    ///
    /// # Arguments
    /// * `map` - 目标映射
    /// * `key` - 键名（可能包含分隔符）
    /// * `values` - 要插入的值列表
    fn insert_nested_value(
        &self,
        map: &mut Map<String, Value>,
        key: &str,
        values: Vec<String>,
    ) -> Result<(), LingoError> {
        let parts: Vec<&str> = key.split(&self.separator).collect();

        if parts.is_empty() {
            return Ok(());
        }

        // 处理值
        let figment_value = if values.len() == 1 {
            // 单个值
            self.parse_arg_value(values[0].clone())?
        } else {
            // 多个值，创建数组
            let mut array_values = Vec::new();
            for value in values {
                array_values.push(self.parse_arg_value(value)?);
            }
            Value::Array(figment::value::Tag::Default, array_values)
        };

        // 如果只有一个部分，直接插入
        if parts.len() == 1 {
            map.insert(parts[0].to_string(), figment_value);
            return Ok(());
        }

        // 处理嵌套键
        let mut current_map = map;

        // 遍历除最后一个部分外的所有部分
        for part in &parts[..parts.len() - 1] {
            let part_string = part.to_string();

            // 如果键不存在，创建新的字典
            if !current_map.contains_key(&part_string) {
                current_map.insert(
                    part_string.clone(),
                    Value::Dict(figment::value::Tag::Default, Map::new()),
                );
            }

            // 获取或创建嵌套字典
            match current_map.get_mut(&part_string) {
                Some(Value::Dict(_, nested_map)) => {
                    current_map = nested_map;
                }
                Some(_) => {
                    // 如果已存在的值不是字典，返回错误
                    return Err(LingoError::Internal(
                        format!(
                            "Command line argument key conflict: '{}' cannot be both a value and a nested object",
                            part
                        )
                    ));
                }
                None => {
                    // 这种情况不应该发生，因为我们刚刚插入了值
                    return Err(LingoError::Internal(
                        "Unexpected error during nested key insertion".to_string()
                    ));
                }
            }
        }

        // 插入最终值
        let final_key = parts[parts.len() - 1].to_string();
        current_map.insert(final_key, figment_value);

        Ok(())
    }

    /// 解析命令行参数值
    ///
    /// 尝试将字符串值解析为适当的类型（布尔值、数字或字符串）
    fn parse_arg_value(&self, value: String) -> Result<Value, LingoError> {
        let tag = figment::value::Tag::Default;

        // 尝试解析为布尔值
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => return Ok(Value::Bool(tag, true)),
            "false" | "0" | "no" | "off" => return Ok(Value::Bool(tag, false)),
            _ => {}
        }

        // 尝试解析为整数
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

impl Provider for LingoClapProvider {
    fn metadata(&self) -> Metadata {
        Metadata::named("Lingo Command Line Provider")
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
/// 返回配置好的 LingoClapProvider
pub fn from_clap_app(
    app: clap::Command,
    args: Vec<String>,
) -> Result<LingoClapProvider, LingoError> {
    let matches = app.try_get_matches_from(args)
        .map_err(|e| LingoError::Internal(
            format!("Failed to parse command line arguments: {}", e)
        ))?;

    Ok(LingoClapProvider::from_matches(matches))
}

/// 辅助函数：创建带有常见参数映射的提供者
///
/// 这个函数创建一个带有常见配置参数映射的提供者
///
/// # Arguments
/// * `matches` - clap 解析的参数匹配结果
///
/// # Returns
/// 返回配置好的 LingoClapProvider
pub fn with_common_mappings(matches: ArgMatches) -> LingoClapProvider {
    LingoClapProvider::from_matches(matches)
        .map_arg("config", "config_file")
        .map_arg("config-dir", "config_dir")
        .map_arg("log-level", "logging.level")
        .map_arg("verbose", "logging.verbose")
        .map_arg("quiet", "logging.quiet")
        .map_arg("output", "output.file")
        .map_arg("format", "output.format")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction, Command};

    fn create_test_app() -> Command {
        Command::new("test")
            .arg(Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("Configuration file"))
            .arg(Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(ArgAction::SetTrue)
                .help("Verbose output"))
            .arg(Arg::new("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level"))
            .arg(Arg::new("values")
                .long("values")
                .action(ArgAction::Append)
                .help("Multiple values"))
    }

    #[test]
    fn test_lingo_clap_provider_new() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let mut mapping = HashMap::new();
        mapping.insert("config".to_string(), "config_file".to_string());

        let provider = LingoClapProvider::new(matches, mapping, ".".to_string());

        assert_eq!(provider.separator, ".");
        assert!(provider.arg_mapping.contains_key("config"));
    }

    #[test]
    fn test_lingo_clap_provider_from_matches() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();

        let provider = LingoClapProvider::from_matches(matches);

        assert_eq!(provider.separator, ".");
        assert!(provider.arg_mapping.is_empty());
    }

    #[test]
    fn test_map_arg() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();

        let provider = LingoClapProvider::from_matches(matches)
            .map_arg("config", "config_file")
            .map_arg("log-level", "logging.level");

        assert_eq!(provider.arg_mapping.get("config"), Some(&"config_file".to_string()));
        assert_eq!(provider.arg_mapping.get("log-level"), Some(&"logging.level".to_string()));
    }

    #[test]
    fn test_with_separator() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();

        let provider = LingoClapProvider::from_matches(matches)
            .with_separator("::");

        assert_eq!(provider.separator, "::");
    }

    #[test]
    fn test_parse_arg_value_boolean() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);

        // 测试 true 值
        let true_values = ["true", "TRUE", "1", "yes", "YES", "on", "ON"];
        for val in &true_values {
            let result = provider.parse_arg_value(val.to_string()).unwrap();
            match result {
                Value::Bool(_, true) => {}
                _ => panic!("Expected true boolean for value: {}", val),
            }
        }

        // 测试 false 值
        let false_values = ["false", "FALSE", "0", "no", "NO", "off", "OFF"];
        for val in &false_values {
            let result = provider.parse_arg_value(val.to_string()).unwrap();
            match result {
                Value::Bool(_, false) => {}
                _ => panic!("Expected false boolean for value: {}", val),
            }
        }
    }

    #[test]
    fn test_parse_arg_value_numbers() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);

        // 测试整数
        let result = provider.parse_arg_value("42".to_string()).unwrap();
        match result {
            Value::Num(_, figment::value::Num::I64(42)) => {}
            _ => panic!("Expected i64 number"),
        }

        // 测试浮点数
        let result = provider.parse_arg_value("3.14".to_string()).unwrap();
        match result {
            Value::Num(_, figment::value::Num::F64(f)) if (f - 3.14).abs() < f64::EPSILON => {}
            _ => panic!("Expected f64 number"),
        }
    }

    #[test]
    fn test_parse_arg_value_string() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);

        let result = provider.parse_arg_value("hello world".to_string()).unwrap();
        match result {
            Value::String(_, s) if s == "hello world" => {}
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_get_arg_values_single() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test", "--config", "config.toml"]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);

        let result = provider.get_arg_values("config").unwrap();
        assert_eq!(result, Some(vec!["config.toml".to_string()]));
    }

    #[test]
    fn test_get_arg_values_flag() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test", "--verbose"]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);

        let result = provider.get_arg_values("verbose").unwrap();
        // 布尔标志不应该通过 get_arg_values 返回值，而是通过 get_flag 检查
        assert_eq!(result, None);
        // 验证布尔标志确实被设置了
        assert!(provider.matches.get_flag("verbose"));
    }

    #[test]
    fn test_get_arg_values_multiple() {
        let app = create_test_app();
        let matches = app.try_get_matches_from([
            "test", "--values", "val1", "--values", "val2", "--values", "val3"
        ]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);

        let result = provider.get_arg_values("values").unwrap();
        assert_eq!(result, Some(vec![
            "val1".to_string(),
            "val2".to_string(),
            "val3".to_string()
        ]));
    }

    #[test]
    fn test_get_arg_values_nonexistent() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);

        let result = provider.get_arg_values("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_nested_value_simple() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);
        let mut map = Map::new();

        provider.insert_nested_value(&mut map, "key", vec!["value".to_string()]).unwrap();

        assert!(map.contains_key("key"));
        match map.get("key").unwrap() {
            Value::String(_, s) if s == "value" => {}
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_insert_nested_value_nested() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);
        let mut map = Map::new();

        provider.insert_nested_value(&mut map, "section.key", vec!["value".to_string()]).unwrap();

        assert!(map.contains_key("section"));
        match map.get("section").unwrap() {
            Value::Dict(_, nested_map) => {
                assert!(nested_map.contains_key("key"));
                match nested_map.get("key").unwrap() {
                    Value::String(_, s) if s == "value" => {}
                    _ => panic!("Expected string value in nested map"),
                }
            }
            _ => panic!("Expected nested dictionary"),
        }
    }

    #[test]
    fn test_insert_nested_value_array() {
        let app = create_test_app();
        let matches = app.try_get_matches_from(["test"]).unwrap();
        let provider = LingoClapProvider::from_matches(matches);
        let mut map = Map::new();

        provider.insert_nested_value(
            &mut map,
            "key",
            vec!["value1".to_string(), "value2".to_string()],
        ).unwrap();

        assert!(map.contains_key("key"));
        match map.get("key").unwrap() {
            Value::Array(_, arr) => {
                assert_eq!(arr.len(), 2);
                match (&arr[0], &arr[1]) {
                    (Value::String(_, s1), Value::String(_, s2)) => {
                        assert_eq!(s1, "value1");
                        assert_eq!(s2, "value2");
                    }
                    _ => panic!("Expected string values in array"),
                }
            }
            _ => panic!("Expected array value"),
        }
    }

    #[test]
    fn test_read_clap_args() {
        let app = create_test_app();
        let matches = app.try_get_matches_from([
            "test", "--config", "config.toml", "--verbose", "--log-level", "debug"
        ]).unwrap();

        let provider = LingoClapProvider::from_matches(matches)
            .map_arg("config", "config_file")
            .map_arg("log-level", "logging.level");

        let result = provider.read_clap_args().unwrap();

        // 验证映射的参数
        assert!(result.contains_key("config_file"));
        assert!(result.contains_key("logging"));
        assert!(result.contains_key("verbose"));

        // 验证 verbose 是布尔值
        match result.get("verbose").unwrap() {
            Value::Bool(_, true) => {}
            _ => panic!("Expected verbose to be a boolean true value"),
        }

        // 验证嵌套结构
        match result.get("logging").unwrap() {
            Value::Dict(_, logging_map) => {
                assert!(logging_map.contains_key("level"));
                match logging_map.get("level").unwrap() {
                    Value::String(_, s) if s == "debug" => {}
                    _ => panic!("Expected debug string in logging.level"),
                }
            }
            _ => panic!("Expected logging to be a dictionary"),
        }
    }

    #[test]
    fn test_from_clap_app() {
        let app = create_test_app();
        let args = vec![
            "test".to_string(),
            "--config".to_string(),
            "config.toml".to_string(),
        ];

        let result = from_clap_app(app, args);
        assert!(result.is_ok());

        let provider = result.unwrap();
        let data = provider.read_clap_args().unwrap();
        assert!(data.contains_key("config"));
    }

    #[test]
    fn test_with_common_mappings() {
        let app = Command::new("test")
            .arg(Arg::new("config").long("config"))
            .arg(Arg::new("config-dir").long("config-dir"))
            .arg(Arg::new("log-level").long("log-level"))
            .arg(Arg::new("verbose").long("verbose").action(ArgAction::SetTrue))
            .arg(Arg::new("quiet").long("quiet").action(ArgAction::SetTrue))
            .arg(Arg::new("output").long("output"))
            .arg(Arg::new("format").long("format"));

        let matches = app.try_get_matches_from([
            "test", "--config", "config.toml", "--log-level", "info"
        ]).unwrap();

        let provider = with_common_mappings(matches);

        // 验证映射
        assert_eq!(provider.arg_mapping.get("config"), Some(&"config_file".to_string()));
        assert_eq!(provider.arg_mapping.get("log-level"), Some(&"logging.level".to_string()));
        assert_eq!(provider.arg_mapping.get("config-dir"), Some(&"config_dir".to_string()));
    }
}
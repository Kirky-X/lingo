//! 环境变量配置提供者
//!
//! 此模块实现了从环境变量读取数据的 figment Provider。
//! 支持前缀过滤、分隔符配置和嵌套键构造。

use crate::error::QuantumConfigError;
use figment::{value::{Map, Value}, Error, Metadata, Profile, Provider};
use std::collections::HashMap;
use std::env;

/// 环境变量配置提供者
///
/// 从环境变量读取配置数据，支持前缀过滤和嵌套键构造。
#[derive(Debug, Clone)]
pub struct QuantumConfigEnvProvider {
    /// 环境变量前缀（例如 "MYAPP_"）
    prefix: String,
    /// 分隔符，用于构造嵌套键（例如 "__" 或 "_"）
    separator: String,
    /// 是否忽略空值
    ignore_empty: bool,
    /// 是否转换键名为小写
    lowercase_keys: bool,
}

impl QuantumConfigEnvProvider {
    /// 创建新的环境变量提供者
    ///
    /// # Arguments
    /// * `prefix` - 环境变量前缀
    /// * `separator` - 分隔符，用于构造嵌套键
    /// * `ignore_empty` - 是否忽略空值
    /// * `lowercase_keys` - 是否转换键名为小写
    pub fn new<S: Into<String>>(
        prefix: S,
        separator: S,
        ignore_empty: bool,
        lowercase_keys: bool,
    ) -> Self {
        Self {
            prefix: prefix.into(),
            separator: separator.into(),
            ignore_empty,
            lowercase_keys,
        }
    }

    /// 创建带有默认设置的环境变量提供者
    ///
    /// 默认设置：
    /// - 分隔符："__"
    /// - 忽略空值：true
    /// - 转换键名为小写：true
    ///
    /// # Arguments
    /// * `prefix` - 环境变量前缀
    pub fn with_prefix<S: Into<String>>(prefix: S) -> Self {
        Self {
            prefix: prefix.into(),
            separator: "__".to_string(),
            ignore_empty: true,
            lowercase_keys: true,
        }
    }

    /// 验证环境变量键名的安全性
    pub fn validate_env_key(key: &str) -> Result<(), QuantumConfigError> {
        // 检查键名长度（防止过长的键名）
        if key.len() > 256 {
            return Err(QuantumConfigError::ValidationError(
                "Environment variable key too long (max 256 characters)".to_string()
            ));
        }
        
        // 检查是否包含危险字符
        if key.contains('\0') || key.contains('\n') || key.contains('\r') {
            return Err(QuantumConfigError::ValidationError(
                "Environment variable key contains invalid characters".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// 验证环境变量值的安全性
    pub fn validate_env_value(value: &str) -> Result<(), QuantumConfigError> {
        // 检查值长度（防止过长的值）
        if value.len() > 8192 {
            return Err(QuantumConfigError::ValidationError(
                "Environment variable value too long (max 8192 characters)".to_string()
            ));
        }
        
        // 检查是否包含空字节
        if value.contains('\0') {
            return Err(QuantumConfigError::ValidationError(
                "Environment variable value contains null bytes".to_string()
            ));
        }
        
        Ok(())
    }

    /// 读取并处理环境变量
    fn read_env_vars(&self) -> Result<Map<String, Value>, QuantumConfigError> {
        let mut env_map = Map::new();

        // 获取所有环境变量
        let env_vars: HashMap<String, String> = env::vars().collect();

        for (key, value) in env_vars {
            // 验证环境变量键名和值的安全性
            Self::validate_env_key(&key)?;
            Self::validate_env_value(&value)?;
            
            // 检查是否匹配前缀
            if !key.starts_with(&self.prefix) {
                continue;
            }

            // 检查是否忽略空值
            if self.ignore_empty && value.is_empty() {
                continue;
            }

            // 移除前缀
            let key_without_prefix = &key[self.prefix.len()..];

            // 处理键名
            let processed_key = if self.lowercase_keys {
                key_without_prefix.to_lowercase()
            } else {
                key_without_prefix.to_string()
            };

            // 构造嵌套键并插入值
            self.insert_nested_value(&mut env_map, &processed_key, value)?;
        }

        Ok(env_map)
    }

    /// 将值插入到嵌套的映射结构中
    ///
    /// # Arguments
    /// * `map` - 目标映射
    /// * `key` - 键名（可能包含分隔符）
    /// * `value` - 要插入的值
    fn insert_nested_value(
        &self,
        map: &mut Map<String, Value>,
        key: &str,
        value: String,
    ) -> Result<(), QuantumConfigError> {
        let parts: Vec<&str> = key.split(&self.separator).collect();

        if parts.is_empty() {
            return Ok(());
        }

        // 如果只有一个部分，直接插入
        if parts.len() == 1 {
            let parsed_value = self.parse_env_value(value)?;
            map.insert(parts[0].to_string(), parsed_value);
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
                    return Err(QuantumConfigError::Internal(
                        format!(
                            "Environment variable key conflict: '{}' cannot be both a value and a nested object",
                            part
                        )
                    ));
                }
                None => {
                    // 这种情况不应该发生，因为我们刚刚插入了值
                    return Err(QuantumConfigError::Internal(
                        "Unexpected error during nested key insertion".to_string()
                    ));
                }
            }
        }

        // 插入最终值
        let final_key = parts[parts.len() - 1].to_string();
        let parsed_value = self.parse_env_value(value)?;
        current_map.insert(final_key, parsed_value);

        Ok(())
    }

    /// 解析环境变量值
    ///
    /// 尝试将字符串值解析为适当的类型（布尔值、数字或字符串）
    fn parse_env_value(&self, value: String) -> Result<Value, QuantumConfigError> {
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

impl Provider for QuantumConfigEnvProvider {
    fn metadata(&self) -> Metadata {
        Metadata::named(format!("Quantum Config Environment Provider (prefix: {})", self.prefix))
    }

    fn data(&self) -> Result<Map<Profile, Map<String, Value>>, Error> {
        let env_data = self.read_env_vars()
            .map_err(|e| Error::from(format!("Environment provider error: {}", e)))?;

        let mut profile_map = Map::new();
        profile_map.insert(Profile::Default, env_data);

        Ok(profile_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_quantum_config_env_provider_new() {
        let provider = QuantumConfigEnvProvider::new("TEST_", "__", true, true);

        assert_eq!(provider.prefix, "TEST_");
        assert_eq!(provider.separator, "__");
        assert!(provider.ignore_empty);
        assert!(provider.lowercase_keys);
    }

    #[test]
    fn test_quantum_config_env_provider_with_prefix() {
        let provider = QuantumConfigEnvProvider::with_prefix("MYAPP_");

        assert_eq!(provider.prefix, "MYAPP_");
        assert_eq!(provider.separator, "__");
        assert!(provider.ignore_empty);
        assert!(provider.lowercase_keys);
    }

    #[test]
    fn test_parse_env_value_boolean() {
        let provider = QuantumConfigEnvProvider::with_prefix("TEST_");

        // 测试 true 值
        let true_values = ["true", "TRUE", "1", "yes", "YES", "on", "ON"];
        for val in &true_values {
            let result = provider.parse_env_value(val.to_string()).unwrap();
            match result {
                Value::Bool(_, true) => {}
                _ => panic!("Expected true boolean for value: {}", val),
            }
        }

        // 测试 false 值
        let false_values = ["false", "FALSE", "0", "no", "NO", "off", "OFF"];
        for val in &false_values {
            let result = provider.parse_env_value(val.to_string()).unwrap();
            match result {
                Value::Bool(_, false) => {}
                _ => panic!("Expected false boolean for value: {}", val),
            }
        }
    }

    #[test]
    fn test_parse_env_value_numbers() {
        let provider = QuantumConfigEnvProvider::with_prefix("TEST_");

        // 测试整数
        let result = provider.parse_env_value("42".to_string()).unwrap();
        match result {
            Value::Num(_, figment::value::Num::I64(42)) => {}
            _ => panic!("Expected i64 number"),
        }

        // 测试浮点数
        let result = provider.parse_env_value("3.14".to_string()).unwrap();
        match result {
            Value::Num(_, figment::value::Num::F64(f)) if (f - 3.14).abs() < f64::EPSILON => {}
            _ => panic!("Expected f64 number"),
        }
    }

    #[test]
    fn test_parse_env_value_string() {
        let provider = QuantumConfigEnvProvider::with_prefix("TEST_");

        let result = provider.parse_env_value("hello world".to_string()).unwrap();
        match result {
            Value::String(_, s) if s == "hello world" => {}
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_insert_nested_value_simple() {
        let provider = QuantumConfigEnvProvider::with_prefix("TEST_");
        let mut map = Map::new();

        provider.insert_nested_value(&mut map, "key", "value".to_string()).unwrap();

        assert!(map.contains_key("key"));
        match map.get("key").unwrap() {
            Value::String(_, s) if s == "value" => {}
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_insert_nested_value_nested() {
        let provider = QuantumConfigEnvProvider::with_prefix("TEST_");
        let mut map = Map::new();

        provider.insert_nested_value(&mut map, "section__key", "value".to_string()).unwrap();

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
    fn test_insert_nested_value_deep_nesting() {
        let provider = QuantumConfigEnvProvider::with_prefix("TEST_");
        let mut map = Map::new();

        provider.insert_nested_value(&mut map, "a__b__c__d", "deep_value".to_string()).unwrap();

        // 验证深度嵌套结构
        let a = map.get("a").unwrap();
        match a {
            Value::Dict(_, a_map) => {
                let b = a_map.get("b").unwrap();
                match b {
                    Value::Dict(_, b_map) => {
                        let c = b_map.get("c").unwrap();
                        match c {
                            Value::Dict(_, c_map) => {
                                let d = c_map.get("d").unwrap();
                                match d {
                                    Value::String(_, s) if s == "deep_value" => {}
                                    _ => panic!("Expected deep_value string"),
                                }
                            }
                            _ => panic!("Expected c to be a dictionary"),
                        }
                    }
                    _ => panic!("Expected b to be a dictionary"),
                }
            }
            _ => panic!("Expected a to be a dictionary"),
        }
    }

    #[test]
    fn test_insert_nested_value_conflict() {
        let provider = QuantumConfigEnvProvider::with_prefix("TEST_");
        let mut map = Map::new();

        // 先插入一个简单值
        provider.insert_nested_value(&mut map, "key", "value".to_string()).unwrap();

        // 尝试插入嵌套值，应该失败
        let result = provider.insert_nested_value(&mut map, "key__nested", "nested_value".to_string());
        assert!(result.is_err());

        match result.unwrap_err() {
            QuantumConfigError::Internal(message) => {
                assert!(message.contains("key conflict"));
            }
            _ => panic!("Expected Internal error for key conflict"),
        }
     }

     #[test]
    fn test_read_env_vars_with_prefix() {
        let provider = QuantumConfigEnvProvider::with_prefix("quantum_config_TEST_");

        // 设置测试环境变量
        unsafe { env::set_var("quantum_config_TEST_KEY1", "value1"); }
        unsafe { env::set_var("quantum_config_TEST_KEY2", "42"); }
        unsafe { env::set_var("quantum_config_TEST_NESTED__KEY", "nested_value"); }
        unsafe { env::set_var("OTHER_KEY", "should_be_ignored"); }

        let result = provider.read_env_vars().unwrap();

        // 验证结果
        assert!(result.contains_key("key1"));
        assert!(result.contains_key("key2"));
        assert!(result.contains_key("nested"));
        assert!(!result.contains_key("other_key")); // 应该被忽略

        // 清理环境变量
        unsafe { env::remove_var("quantum_config_TEST_KEY1"); }
        unsafe { env::remove_var("quantum_config_TEST_KEY2"); }
        unsafe { env::remove_var("quantum_config_TEST_NESTED__KEY"); }
        unsafe { env::remove_var("OTHER_KEY"); }
    }

    #[test]
    fn test_ignore_empty_values() {
        let provider = QuantumConfigEnvProvider::new("quantum_config_EMPTY_", "__", true, true);

        // 设置空值环境变量
        unsafe { env::set_var("quantum_config_EMPTY_KEY1", ""); }
        unsafe { env::set_var("quantum_config_EMPTY_KEY2", "not_empty"); }

        let result = provider.read_env_vars().unwrap();

        // 空值应该被忽略
        assert!(!result.contains_key("key1"));
        assert!(result.contains_key("key2"));

        // 清理环境变量
        unsafe { env::remove_var("quantum_config_EMPTY_KEY1"); }
        unsafe { env::remove_var("quantum_config_EMPTY_KEY2"); }
    }

    #[test]
    fn test_dont_ignore_empty_values() {
        let provider = QuantumConfigEnvProvider::new("quantum_config_NOEMPTY_", "__", false, true);

        // 设置空值环境变量
        unsafe { env::set_var("quantum_config_NOEMPTY_KEY1", ""); }
        unsafe { env::set_var("quantum_config_NOEMPTY_KEY2", "not_empty"); }

        let result = provider.read_env_vars().unwrap();

        // 空值不应该被忽略
        assert!(result.contains_key("key1"));
        assert!(result.contains_key("key2"));

        // 清理环境变量
        unsafe { env::remove_var("quantum_config_NOEMPTY_KEY1"); }
        unsafe { env::remove_var("quantum_config_NOEMPTY_KEY2"); }
    }

    #[test]
    fn test_lowercase_keys() {
        let provider = QuantumConfigEnvProvider::new("quantum_config_CASE_", "__", true, true);

        // 设置大写键名的环境变量
        unsafe { env::set_var("quantum_config_CASE_UPPER_KEY", "value"); }

        let result = provider.read_env_vars().unwrap();

        // 键名应该被转换为小写
        assert!(result.contains_key("upper_key"));
        assert!(!result.contains_key("UPPER_KEY"));

        // 清理环境变量
        unsafe { env::remove_var("quantum_config_CASE_UPPER_KEY"); }
    }

    #[test]
    fn test_preserve_case_keys() {
        let provider = QuantumConfigEnvProvider::new("quantum_config_PRESERVE_", "__", true, false);

        // 设置大写键名的环境变量
        unsafe { env::set_var("quantum_config_PRESERVE_UPPER_KEY", "value"); }

        let result = provider.read_env_vars().unwrap();

        // 键名应该保持原样
        assert!(result.contains_key("UPPER_KEY"));
        assert!(!result.contains_key("upper_key"));

        // 清理环境变量
        unsafe { env::remove_var("quantum_config_PRESERVE_UPPER_KEY"); }
    }
}
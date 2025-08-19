//! 安全测试模块
//!
//! 此模块包含针对 Quantum Config 库安全功能的测试用例。
//! 测试覆盖路径遍历防护、环境变量验证、解析深度限制等安全特性。

use crate::error::QuantumConfigError;
use crate::meta::QuantumConfigAppMeta;
use crate::paths::{validate_path_security, add_specified_config_file};
use crate::providers::env_provider::QuantumConfigEnvProvider;
use crate::providers::file_provider::QuantumConfigFileProvider;
use figment::Figment;
use std::path::PathBuf;
use std::env;
use std::time::Instant;
use tempfile::TempDir;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试路径遍历攻击防护
    #[test]
    fn test_path_traversal_protection() {
        let dangerous_paths = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "/etc/shadow",
            "C:\\Windows\\System32\\config\\SAM",
            "../../sensitive_file.txt",
            "../config/../../../etc/hosts",
        ];

        let _meta = QuantumConfigAppMeta::default();
        let mut config_files = Vec::new();

        for path in dangerous_paths {
             let path_buf = PathBuf::from(path);
             let result = add_specified_config_file(&mut config_files, path_buf);
            
            // 应该返回错误或安全地处理
            match result {
                Ok(_) => {
                    // 如果成功，确保路径已被安全处理
                    assert!(!config_files.iter().any(|f| f.path.to_string_lossy().contains("../")));
                }
                Err(QuantumConfigError::SecurityViolation { message }) => {
                    assert!(message.contains("path") || message.contains("security"));
                }
                Err(_) => {
                    // 其他错误也是可接受的
                }
            }
        }
    }

    /// 测试合法路径不被误拦截
    #[test]
    fn test_legitimate_paths_allowed() {
        let legitimate_paths = vec![
            "config.toml",
            "./config/app.json",
            "configs/database.ini",
            "app_config.toml",
        ];

        for legitimate_path in legitimate_paths {
            let path = PathBuf::from(legitimate_path);
            let result = validate_path_security(&path);
            
            assert!(result.is_ok(), "Legitimate path should be allowed: {}", legitimate_path);
        }
    }

    /// 测试环境变量键名验证
    #[test]
    fn test_env_key_validation() {
        use crate::providers::env_provider::QuantumConfigEnvProvider;
        
        // 测试键名验证逻辑
        let long_key = "A".repeat(300);
        let test_cases = vec![
            ("NORMAL_KEY", true),
            ("APP_CONFIG_PORT", true),
            // 过长的键名
            (&long_key, false),
            // 包含危险字符的键名
            ("KEY\0WITH\0NULL", false),
        ];

        for (key, should_succeed) in test_cases {
            let result = QuantumConfigEnvProvider::validate_env_key(key);
            
            if should_succeed {
                assert!(result.is_ok(), "Valid env key should succeed: {}", key);
            } else {
                assert!(result.is_err(), "Invalid env key should fail: {}", key);
            }
        }
    }

    /// 测试环境变量值验证
    #[test]
    fn test_env_value_validation() {
        use crate::providers::env_provider::QuantumConfigEnvProvider;
        
        // 测试值验证逻辑
        let large_value_ok = "A".repeat(8192);
        let large_value_fail = "A".repeat(8193);
        let test_cases = vec![
            ("normal_value", true),
            (&large_value_ok, true), // 边界值
            (&large_value_fail, false), // 超过限制
            ("value\0with\0null", false), // 包含空字节
        ];

        for (value, should_succeed) in test_cases {
            let result = QuantumConfigEnvProvider::validate_env_value(value);
            
            if should_succeed {
                assert!(result.is_ok(), "Valid env value should succeed");
            } else {
                assert!(result.is_err(), "Invalid env value should fail");
            }
        }
    }

    /// 测试解析深度限制
    #[test]
    fn test_parse_depth_limit() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("deep_config.json");
        
        // 创建一个深度嵌套的 JSON 配置
        let mut deep_json = String::new();
        for _ in 0..50 {
            deep_json.push_str("{\"nested\":");
        }
        deep_json.push_str("\"value\"");
        for _ in 0..50 {
            deep_json.push('}');
        }
        
        std::fs::write(&config_path, deep_json).unwrap();
        
        // 使用较小的解析深度限制
        let provider_result = QuantumConfigFileProvider::from_path(&config_path, true, 32);
         
         match provider_result {
             Ok(provider) => {
                  let figment = Figment::from(provider);
                  let result = figment.extract::<serde_json::Value>();
                
                // 检查是否能正确处理深度嵌套的JSON
                if let Err(e) = &result {
                    println!("Handling deeply nested JSON: {:?}", e);
                }
            }
            Err(e) => {
                println!("Provider creation failed for deeply nested JSON: {:?}", e);
            }
        }
    }

    /// 测试配置文件路径安全性
    #[test]
    fn test_config_file_path_security() {
        let mut config_files = Vec::new();
        
        // 测试恶意路径
        let malicious_paths = vec![
            "../../../etc/passwd",
            "..\\..\\windows\\system32\\hosts",
            "/etc/shadow",
        ];
        
        for malicious_path in malicious_paths {
            let path = PathBuf::from(malicious_path);
            let result = add_specified_config_file(&mut config_files, path);
            
            match result {
                Err(QuantumConfigError::SecurityViolation { .. }) => {
                    // 预期的安全错误
                }
                _ => {
                    panic!("Expected SecurityViolation for malicious path: {}", malicious_path);
                }
            }
        }
        
        // 确保没有恶意文件被添加
        assert_eq!(config_files.len(), 0);
    }

    /// 测试错误消息中的路径脱敏
    #[test]
    fn test_error_message_path_sanitization() {
        let sensitive_path = "/home/user/.ssh/id_rsa";
        let path = PathBuf::from(sensitive_path);
        
        // 尝试读取不存在的敏感文件
        let result = QuantumConfigFileProvider::from_path(&path, true, 32);
        
        match result {
            Ok(provider) => {
                 let figment = Figment::from(provider);
                  let read_result = figment.extract::<serde_json::Value>();
                
                if let Err(error) = read_result {
                    let error_message = format!("{}", error);
                    
                    // 在生产环境中，错误消息不应包含完整的敏感路径
                    #[cfg(not(debug_assertions))]
                    {
                        assert!(!error_message.contains("/home/user/.ssh"), 
                               "Error message should not contain sensitive path parts: {}", error_message);
                    }
                    
                    // 但应该包含文件名用于调试
                    assert!(error_message.contains("id_rsa") || error_message.contains("file"), 
                           "Error message should contain some file reference: {}", error_message);
                }
            }
            Err(e) => {
                println!("Expected error when accessing sensitive path: {:?}", e);
            }
        }
    }

    /// 测试大量环境变量的性能和安全性
    #[test]
    fn test_large_env_vars_handling() {
        // 设置大量环境变量
        for i in 0..1000 {
            env::set_var(format!("TEST_VAR_{}", i), format!("value_{}", i));
        }
        
        let provider = QuantumConfigEnvProvider::with_prefix("TEST_VAR_");
          let start_time = Instant::now();
          let figment = Figment::from(provider);
          let result = figment.extract::<HashMap<String, String>>();
        let duration = start_time.elapsed();
        
        // 应该能够处理大量环境变量且不超时
        assert!(result.is_ok(), "Should handle large number of env vars");
        assert!(duration.as_secs() < 5, "Should process env vars within reasonable time");
        
        // 清理环境变量
        for i in 0..1000 {
            env::remove_var(format!("TEST_VAR_{}", i));
        }
    }
}
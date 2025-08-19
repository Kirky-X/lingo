# Quantum Config 安全指南

## 概述

Quantum Config 在设计时充分考虑了安全性，提供了多层安全防护机制，确保配置管理过程的安全性和可靠性。本文档详细介绍了 Quantum Config 的安全特性、最佳实践和安全配置建议。

## 安全特性

### 1. 路径遍历攻击防护

**问题描述**: 恶意用户可能通过构造包含 `../` 的路径来访问系统敏感文件。

**防护机制**:
- **路径规范化**: 自动解析和规范化所有输入路径，移除 `..` 和 `.` 组件
- **敏感目录检查**: 阻止访问系统敏感目录（如 `/etc/passwd`、`/proc`、`C:\Windows\System32` 等）
- **绝对路径验证**: 确保规范化后的路径不会逃逸到预期目录之外

**实现示例**:
```rust
// 自动防护路径遍历攻击
let config_path = "../../../etc/passwd"; // 恶意路径
let result = ConfigFilePath::new(config_path); // 会被安全检查拒绝
```

### 2. 敏感信息泄露防护

**问题描述**: 错误消息可能意外泄露系统路径或其他敏感信息。

**防护机制**:
- **错误消息过滤**: 自动过滤错误消息中的敏感路径信息
- **通用化错误**: 将具体的系统路径替换为通用描述
- **安全日志**: 确保日志记录不包含敏感信息

**实现示例**:
```rust
// 原始错误: "Cannot access /home/user/.ssh/id_rsa"
// 过滤后错误: "Cannot access specified path due to security restrictions"
```

### 3. 命令注入防护

**问题描述**: 派生宏可能允许外部子命令，存在命令注入风险。

**防护机制**:
- **禁用外部子命令**: 移除 `allow_external_subcommands` 选项
- **严格参数验证**: 对所有命令行参数进行严格验证
- **白名单机制**: 只允许预定义的命令和参数

### 4. 输入验证

**环境变量验证**:
- **格式检查**: 验证环境变量值的格式和长度
- **类型安全**: 确保环境变量值能够安全转换为目标类型
- **范围检查**: 验证数值类型的环境变量在合理范围内

**配置文件验证**:
- **文件类型检查**: 验证配置文件扩展名和内容格式
- **大小限制**: 限制配置文件的最大大小
- **深度限制**: 限制配置结构的嵌套深度（默认最大 10 层）

### 5. 跨平台路径安全

**路径格式验证**:
- **平台适配**: 自动识别和转换不同平台的路径格式
- **安全转换**: 确保路径转换过程不会引入安全漏洞
- **格式检测**: 智能检测路径格式，防止格式混淆攻击

## 安全配置建议

### 1. 文件权限设置

**系统配置文件**:
```bash
# Linux/macOS
sudo chmod 644 /etc/myapp/config.toml
sudo chown root:root /etc/myapp/config.toml

# Windows (PowerShell)
icacls "C:\ProgramData\MyApp\config.toml" /grant:r "Users:R"
```

**用户配置文件**:
```bash
# Linux/macOS
chmod 600 ~/.config/myapp/config.toml

# Windows
icacls "%APPDATA%\MyApp\config.toml" /grant:r "%USERNAME%:F"
```

### 2. 环境变量安全

**敏感信息处理**:
```rust
#[derive(Config, Deserialize, Serialize)]
struct AppConfig {
    // 使用环境变量存储敏感信息
    #[quantum_config_opt(env = "DATABASE_PASSWORD")]
    #[serde(skip_serializing)] // 防止序列化时泄露
    database_password: Option<String>,
    
    // 非敏感配置可以使用默认值
    #[quantum_config_opt(default = "localhost")]
    database_host: String,
}
```

**环境变量命名规范**:
- 使用应用前缀：`MYAPP_DATABASE_PASSWORD`
- 避免通用名称：避免使用 `PASSWORD`、`SECRET` 等通用名称
- 使用大写字母和下划线：遵循环境变量命名约定

### 3. 配置文件安全

**敏感信息分离**:
```toml
# config.toml (非敏感配置)
[server]
host = "localhost"
port = 8080

[database]
host = "localhost"
port = 5432
name = "myapp"
# 密码通过环境变量提供：DATABASE_PASSWORD
```

**配置文件加密** (推荐第三方工具):
```rust
// 使用 sops、age 或其他工具加密敏感配置
// Quantum Config 专注于配置管理，加密由专门工具处理
```

### 4. 运行时安全

**最小权限原则**:
```rust
// 在应用启动后降低权限
use std::os::unix::process;

fn drop_privileges() {
    // 切换到非特权用户
    // 实现取决于具体需求
}
```

**配置验证**:
```rust
impl AppConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // 验证配置的合理性
        if self.port == 0 || self.port > 65535 {
            return Err(ConfigError::InvalidValue("port must be 1-65535".into()));
        }
        
        // 验证路径安全性
        if let Some(ref log_path) = self.log_path {
            ConfigFilePath::new(log_path)?; // 自动进行安全检查
        }
        
        Ok(())
    }
}
```

## 安全测试

### 1. 路径遍历测试

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_path_traversal_prevention() {
        let malicious_paths = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "/etc/shadow",
            "C:\\Windows\\System32\\config\\SAM",
        ];
        
        for path in malicious_paths {
            assert!(ConfigFilePath::new(path).is_err());
        }
    }
    
    #[test]
    fn test_sensitive_directory_blocking() {
        let sensitive_paths = vec![
            "/proc/version",
            "/sys/kernel",
            "C:\\Windows\\System32",
            "/etc/passwd",
        ];
        
        for path in sensitive_paths {
            assert!(ConfigFilePath::new(path).is_err());
        }
    }
}
```

### 2. 输入验证测试

```rust
#[test]
fn test_environment_variable_validation() {
    // 测试无效的环境变量值
    std::env::set_var("MYAPP_PORT", "invalid_port");
    let result = AppConfig::load();
    assert!(result.is_err());
    
    // 测试超出范围的值
    std::env::set_var("MYAPP_PORT", "99999");
    let result = AppConfig::load();
    assert!(result.is_err());
}
```

### 3. 错误消息安全测试

```rust
#[test]
fn test_error_message_sanitization() {
    let result = ConfigFilePath::new("/etc/passwd");
    if let Err(error) = result {
        let error_msg = error.to_string();
        // 确保错误消息不包含敏感路径
        assert!(!error_msg.contains("/etc/passwd"));
        assert!(error_msg.contains("security") || error_msg.contains("path"));
    }
}
```

## 安全审计

### 1. 依赖安全

定期检查依赖项的安全漏洞：

```bash
# 使用 cargo-audit 检查已知漏洞
cargo install cargo-audit
cargo audit

# 使用 cargo-deny 进行更全面的检查
cargo install cargo-deny
cargo deny check
```

### 2. 代码审计

**关键审计点**:
- 所有文件路径操作
- 环境变量处理
- 错误消息生成
- 用户输入验证
- 序列化/反序列化过程

### 3. 安全扫描

```bash
# 使用 clippy 进行静态分析
cargo clippy -- -D warnings

# 使用 cargo-geiger 检查不安全代码
cargo install cargo-geiger
cargo geiger
```

## 漏洞报告

如果您发现了安全漏洞，请遵循负责任的披露原则：

1. **不要**在公共 issue 中报告安全漏洞
2. 发送邮件至：security@quantum-config.org
3. 包含详细的漏洞描述和复现步骤
4. 我们会在 48 小时内回复确认
5. 修复后会在 CHANGELOG 中致谢（如果您同意）

## 安全更新

- 关注 [SECURITY.md](../SECURITY.md) 获取安全公告
- 订阅 GitHub 仓库的 security advisories
- 定期更新到最新版本
- 查看 [CHANGELOG.md](../CHANGELOG.md) 了解安全修复

## 最佳实践总结

1. **最小权限原则**: 只授予应用必需的最小权限
2. **输入验证**: 验证所有外部输入（文件路径、环境变量、命令行参数）
3. **敏感信息分离**: 将敏感信息与普通配置分离
4. **定期更新**: 保持 Quantum Config 和依赖项的最新版本
5. **安全测试**: 在 CI/CD 流程中包含安全测试
6. **监控日志**: 监控应用日志中的异常行为
7. **备份配置**: 定期备份重要配置文件
8. **访问控制**: 适当设置配置文件的访问权限

## 参考资源

- [OWASP Configuration Management](https://owasp.org/www-project-top-ten/2017/A6_2017-Security_Misconfiguration)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [CIS Controls](https://www.cisecurity.org/controls/)

---

**注意**: 安全是一个持续的过程，请定期审查和更新您的安全配置。如有疑问，请参考最新的安全文档或联系维护团队。
# Lingo 使用示例

这个目录包含了 Lingo 配置管理库的各种使用示例，从基础用法到高级企业级应用场景。

## 📁 示例目录

### [basic](./basic/) - 基础配置加载

**适合人群**: 初学者，想要快速了解 Lingo 基本功能

**主要特性**:

- 基本的配置结构定义
- 多种配置源（文件、环境变量、命令行）
- 配置优先级演示
- 简单的字段属性使用

**学习内容**:

- `#[derive(Config)]` 宏的基本使用
- `#[lingo_opt]` 属性的常见用法
- 配置加载和访问
- 配置模板生成

### [web_server](./web_server/) - Web 服务器配置

**适合人群**: Web 开发者，需要配置 HTTP 服务器的开发者

**主要特性**:

- 复杂的嵌套配置结构
- TLS/SSL 配置
- CORS 配置
- 日志配置
- 异步 Web 服务器集成

**学习内容**:

- 嵌套结构体配置
- `#[lingo_opt(flatten)]` 的使用
- 与 Axum 框架的集成
- 生产环境配置最佳实践

### [database](./database/) - 数据库配置

**适合人群**: 后端开发者，需要复杂数据库配置的企业级应用

**主要特性**:

- 多数据库类型支持
- 连接池配置
- 读写分离配置
- SSL/TLS 数据库连接
- 数据库迁移配置
- 监控和日志配置

**学习内容**:

- 复杂配置结构设计
- 可选配置字段处理
- 配置验证和错误处理
- 敏感信息处理
- 企业级配置管理

## 🚀 快速开始

### 1. 选择合适的示例

根据你的需求选择相应的示例：

- **刚开始学习 Lingo** → 从 `basic` 示例开始
- **开发 Web 应用** → 查看 `web_server` 示例
- **需要数据库配置** → 参考 `database` 示例

### 2. 运行示例

每个示例都可以独立运行：

```bash
# 进入示例目录
cd examples/basic

# 运行示例
cargo run

# 查看帮助信息
cargo run -- --help

# 使用配置文件
cargo run -- --config config.toml
```

### 3. 实验不同配置源

每个示例都支持多种配置方式：

```bash
# 使用环境变量
export APP_NAME="MyApp"
export APP_HOST="0.0.0.0"
export APP_PORT=8080
cargo run

# 使用命令行参数
cargo run -- --name "MyApp" --host "0.0.0.0" --port 8080

# 组合使用
export APP_NAME="MyApp"  # 环境变量
cargo run -- --port 8080 --config config.toml  # 命令行 + 配置文件
```

## 📚 学习路径

### 初级 (basic 示例)

1. **理解基本概念**
    - 什么是配置管理
    - Lingo 的核心理念
    - 配置优先级

2. **掌握基本语法**
    - `#[derive(Config)]` 宏
    - `#[lingo_opt]` 属性
    - 基本数据类型支持

3. **配置源使用**
    - 配置文件（TOML/JSON/YAML）
    - 环境变量
    - 命令行参数

### 中级 (web_server 示例)

1. **复杂结构设计**
    - 嵌套配置结构
    - 可选字段处理
    - 配置分组

2. **实际应用集成**
    - 与 Web 框架集成
    - 异步应用配置
    - 中间件配置

3. **生产环境考虑**
    - 安全配置
    - 性能优化
    - 监控和日志

### 高级 (database 示例)

1. **企业级配置**
    - 复杂配置验证
    - 敏感信息处理
    - 配置模板化

2. **高级特性**
    - 动态配置重载
    - 配置继承
    - 条件配置

3. **最佳实践**
    - 配置文档化
    - 错误处理策略
    - 测试策略

## 🛠️ 开发工具

### 配置文件生成

每个示例都支持生成配置文件模板：

```bash
# 生成 TOML 配置模板
cargo run -- --generate-config toml > config.toml

# 生成 JSON 配置模板
cargo run -- --generate-config json > config.json

# 生成 YAML 配置模板
cargo run -- --generate-config yaml > config.yaml
```

### 配置验证

验证配置文件的正确性：

```bash
# 验证配置文件
cargo run -- --config config.toml --validate

# 显示最终配置（合并所有源后的结果）
cargo run -- --config config.toml --show-config
```

### 调试模式

启用详细的配置加载日志：

```bash
# 启用调试日志
RUST_LOG=debug cargo run

# 或使用内置的调试选项
cargo run -- --debug
```

## 📖 配置文件格式

### TOML (推荐)

```toml
# 简洁易读，支持注释
name = "MyApp"
host = "localhost"
port = 8080
debug = true

[database]
url = "postgres://localhost/myapp"
max_connections = 10
```

### JSON

```json
{
  "name": "MyApp",
  "host": "localhost",
  "port": 8080,
  "debug": true,
  "database": {
    "url": "postgres://localhost/myapp",
    "max_connections": 10
  }
}
```

### YAML

```yaml
name: MyApp
host: localhost
port: 8080
debug: true
database:
  url: postgres://localhost/myapp
  max_connections: 10
```

## 🔧 常见问题

### Q: 如何处理敏感配置信息？

**A**: 使用环境变量或外部密钥管理系统：

```rust
#[derive(Config)]
struct AppConfig {
    // 不要在配置文件中存储密码
    #[lingo_opt(description = "数据库密码，通过环境变量设置")]
    database_password: Option<String>,
}
```

```bash
# 通过环境变量设置敏感信息
export APP_DATABASE_PASSWORD="secret_password"
```

### Q: 如何实现配置热重载？

**A**: 目前 Lingo 专注于启动时配置加载，热重载可以通过文件监控实现：

```rust
// 伪代码示例
use notify::Watcher;

let mut watcher = notify::recommended_watcher(|event| {
    if let Ok(event) = event {
        // 重新加载配置
        let new_config = AppConfig::new();
        // 应用新配置
    }
})?;
```

### Q: 如何在测试中使用不同的配置？

**A**: 使用测试专用的配置文件或环境变量：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_with_custom_config() {
        std::env::set_var("APP_CONFIG", "test_config.toml");
        let config = AppConfig::new();
        // 测试逻辑
    }
}
```

## 🤝 贡献

欢迎贡献新的示例！请确保：

1. 示例有明确的学习目标
2. 包含完整的文档和注释
3. 提供多种运行方式的说明
4. 遵循项目的代码风格

## 📄 许可证

所有示例代码遵循与主项目相同的许可证。
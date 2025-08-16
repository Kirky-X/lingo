# 复杂嵌套配置示例

这个示例展示了如何使用 QuantumConfig 处理复杂的嵌套配置结构，包括多层嵌套、数组配置、可选字段和复杂数据类型。

## 功能特性

- **多层嵌套结构**: 服务器、数据库、Redis、日志等配置的深度嵌套
- **数组配置**: 日志输出目标、启用功能列表等
- **可选字段**: TLS配置、SSL配置、Redis配置等可选组件
- **复杂数据类型**: HashMap、Vec、自定义结构体等
- **类型安全**: 所有配置项都有明确的类型定义
- **默认值**: 为所有配置项提供合理的默认值

## 配置结构

### 主要配置组件

1. **应用基本信息**
   - 应用名称、版本、环境、调试模式

2. **服务器配置** (`ServerConfig`)
   - 监听地址、端口、工作线程数
   - 请求超时、最大请求体大小
   - 可选的TLS配置

3. **数据库配置** (`DatabaseConfig`)
   - 连接信息（主机、端口、数据库名）
   - 认证信息（用户名、密码）
   - 连接池配置
   - 可选的SSL配置

4. **Redis配置** (`RedisConfig`) - 可选
   - 连接信息和认证
   - 连接池配置

5. **日志配置** (`LogConfig`)
   - 日志级别和格式
   - 多个输出目标（控制台、文件等）
   - 日志轮转配置

6. **功能特性配置** (`FeatureConfig`)
   - 启用的功能列表
   - 功能特定的设置（使用HashMap存储）

7. **外部服务配置**
   - 多个外部服务的配置
   - API密钥、超时、重试等设置
   - 自定义HTTP头部

8. **监控配置** (`MonitoringConfig`) - 可选
   - 监控端点和采样率
   - 自定义标签

## 运行示例

```bash
# 基本运行
cd examples/nested
cargo run

# 使用自定义配置文件
cargo run -- --config ./config.toml

# 使用环境变量覆盖配置
export NESTED_SERVER_HOST=0.0.0.0
export NESTED_SERVER_PORT=3000
export NESTED_DATABASE_HOST=db.example.com
export NESTED_LOGGING_LEVEL=debug
cargo run

# 使用命令行参数
cargo run -- --server-host 0.0.0.0 --server-port 3000 --debug
```

## 配置优先级

配置加载遵循以下优先级（从低到高）：

1. **默认值** - 代码中定义的复杂嵌套结构
2. **配置文件** - `config.toml`（支持完整的嵌套结构）
3. **环境变量** - `NESTED_*`（支持嵌套路径，如 `NESTED_SERVER_HOST`）
4. **命令行参数** - `--server-host`, `--database-port` 等

## 环境变量示例

```bash
# 服务器配置
export NESTED_SERVER_HOST=0.0.0.0
export NESTED_SERVER_PORT=8080
export NESTED_SERVER_WORKERS=8

# 数据库配置
export NESTED_DATABASE_HOST=localhost
export NESTED_DATABASE_PORT=5432
export NESTED_DATABASE_DATABASE=myapp
export NESTED_DATABASE_USERNAME=user
export NESTED_DATABASE_PASSWORD=secret
export NESTED_DATABASE_POOL_SIZE=20

# 日志配置
export NESTED_LOGGING_LEVEL=debug
export NESTED_LOGGING_FORMAT=json

# 功能开关
export NESTED_FEATURES_ENABLED='["authentication", "rate_limiting"]'

# 监控配置
export NESTED_MONITORING_ENABLED=true
export NESTED_MONITORING_ENDPOINT=http://prometheus:9090/metrics
```

## 配置文件示例

查看 `config.toml` 文件了解完整的配置文件格式。该文件展示了：

- 如何定义嵌套的配置段
- 如何配置数组和列表
- 如何设置可选配置项
- 如何使用表格数组（`[[logging.targets]]`）
- 如何配置复杂的嵌套映射

## 学习要点

1. **结构体嵌套**: 如何设计和组织复杂的配置结构
2. **可选配置**: 使用 `Option<T>` 处理可选的配置组件
3. **集合类型**: 使用 `Vec<T>` 和 `HashMap<K, V>` 处理列表和映射
4. **类型安全**: 利用 Rust 的类型系统确保配置的正确性
5. **默认值策略**: 为复杂结构提供合理的默认配置
6. **配置验证**: 通过类型系统和 serde 进行配置验证

这个示例展示了 QuantumConfig 在处理企业级应用复杂配置需求时的强大能力。
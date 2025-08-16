# Web 服务器配置示例

这个示例展示了如何使用 QuantumConfig 配置一个完整的 Web 服务器应用程序，包括复杂的嵌套配置结构和实际的 HTTP 服务器实现。

## 功能特性

- **完整的 Web 服务器配置**：主机、端口、工作线程、超时等
- **TLS/SSL 配置**：证书文件、私钥、TLS 版本
- **CORS 配置**：跨域资源共享设置
- **日志配置**：日志级别、格式、访问日志
- **API 配置**：版本、前缀、文档、速率限制
- **嵌套配置结构**：使用 `flatten` 属性组织配置
- **实际的 HTTP 服务器**：基于 Axum 框架

## 运行示例

```bash
cd examples/web_server
cargo run
```

服务器启动后，你可以访问以下端点：

- `http://localhost:3000/` - 根路径
- `http://localhost:3000/health` - 健康检查
- `http://localhost:3000/api/v1/info` - 服务器信息
- `http://localhost:3000/api/v1/config` - 配置信息
- `http://localhost:3000/api/v1/docs` - API 文档
- `http://localhost:3000/api/v1/echo` - 回显端点 (POST)

## 配置示例

### 1. 使用默认配置

```bash
cargo run
```

### 2. 使用环境变量

```bash
export WEB_SERVER_HOST="127.0.0.1"
export WEB_SERVER_PORT="8080"
export WEB_TLS_ENABLED="true"
export WEB_CORS_ENABLED="false"
export WEB_LOGGING_LEVEL="debug"
export WEB_API_VERSION="v2"
cargo run
```

### 3. 使用命令行参数

```bash
cargo run -- --host 127.0.0.1 --port 8080 --tls-enabled --logging-level debug
```

### 4. 使用配置文件

```bash
cargo run -- --config ./config.toml
```

### 5. 生产环境配置示例

创建 `production.toml`：

```toml
[server]
host = "0.0.0.0"
port = 443
workers = 8
timeout = 60
max_body_size = 32

[tls]
enabled = true
cert_file = "/etc/ssl/certs/server.crt"
key_file = "/etc/ssl/private/server.key"
min_version = "1.3"

[cors]
enabled = true
allowed_origins = "https://example.com,https://app.example.com"
allowed_methods = "GET,POST,PUT,DELETE"
allowed_headers = "Content-Type,Authorization"
max_age = 86400

[logging]
level = "warn"
format = "json"
access_log = true
file = "/var/log/web_server.log"

[api]
version = "v1"
prefix = "/api"
docs_enabled = false
rate_limit = 1000
```

然后运行：

```bash
cargo run -- --config ./production.toml
```

## 配置结构说明

### ServerConfig (根配置)

- `server`: HTTP 服务器基本配置
- `tls`: TLS/SSL 安全配置
- `cors`: 跨域资源共享配置
- `logging`: 日志系统配置
- `api`: API 相关配置

### 嵌套配置的优势

1. **逻辑分组**：相关配置项组织在一起
2. **命名空间**：避免配置项名称冲突
3. **可维护性**：大型配置更容易管理
4. **复用性**：配置结构可以在不同项目间复用

## 环境变量映射

由于使用了 `flatten` 属性，环境变量名会自动生成：

- `WEB_SERVER_HOST` → `server.host`
- `WEB_SERVER_PORT` → `server.port`
- `WEB_TLS_ENABLED` → `tls.enabled`
- `WEB_CORS_ENABLED` → `cors.enabled`
- `WEB_LOGGING_LEVEL` → `logging.level`
- `WEB_API_VERSION` → `api.version`

## 测试 API 端点

### 获取服务器信息

```bash
curl http://localhost:3000/api/v1/info
```

### 获取配置信息

```bash
curl http://localhost:3000/api/v1/config
```

### 测试回显端点

```bash
curl -X POST http://localhost:3000/api/v1/echo \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello, QuantumConfig!"}'
```

### 健康检查

```bash
curl http://localhost:3000/health
```

## 学习要点

1. **嵌套配置结构**：使用 `#[quantum_config_opt(flatten)]` 组织复杂配置
2. **实际应用集成**：将配置系统集成到真实的 Web 应用中
3. **配置验证**：通过类型系统确保配置的正确性
4. **运行时配置访问**：在应用程序中访问和使用配置
5. **配置的可观测性**：通过 API 端点暴露配置信息
6. **环境特定配置**：开发、测试、生产环境的不同配置策略
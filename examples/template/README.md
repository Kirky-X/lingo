# QuantumConfig 配置模板生成示例

这个示例展示了如何使用 QuantumConfig 的配置模板生成功能，自动生成各种格式的配置文件和部署模板。

## 功能特性

- **配置文件模板生成**: 自动生成带注释的 TOML 配置文件
- **环境变量模板**: 生成 `.env` 文件模板
- **Docker 部署配置**: 生成 Docker Compose 配置文件
- **Kubernetes 部署配置**: 生成 K8s 部署 YAML 文件
- **详细文档**: 每个配置项都包含详细说明
- **生产就绪**: 生成的模板适用于生产环境

## 配置结构

```rust
#[derive(Config)]
struct AppConfig {
    // 应用基本信息
    app_name: String,
    app_version: String,
    environment: String,
    debug: bool,
    host: String,
    port: u16,
    
    // 复杂嵌套配置
    database: DatabaseConfig,
    redis: RedisConfig,
    logging: LoggingConfig,
    security: SecurityConfig,
    monitoring: MonitoringConfig,
}
```

## 运行示例

### 生成所有模板

```bash
# 生成所有配置模板
cargo run --example template -- --generate-all

# 指定输出目录
cargo run --example template -- --generate-all --output-dir ./my-templates
```

### 生成特定模板

```bash
# 只生成配置文件模板
cargo run --example template -- --generate-config

# 只生成环境变量模板
cargo run --example template -- --generate-env

# 只生成 Docker Compose 配置
cargo run --example template -- --generate-docker

# 只生成 Kubernetes 配置
cargo run --example template -- --generate-k8s
```

### 使用自定义配置

```bash
# 从配置文件生成模板
cargo run --example template -- --generate-all --config custom_config.toml
```

## 生成的文件

运行模板生成器后，会在输出目录中创建以下文件：

```
generated/
├── config.toml              # 完整的配置文件模板
├── .env.example             # 环境变量模板
├── docker-compose.yml       # Docker Compose 配置
└── k8s-deployment.yaml      # Kubernetes 部署配置
```

## 配置文件模板 (config.toml)

生成的配置文件包含：

- **应用基本配置**: 名称、版本、环境、端口等
- **数据库配置**: PostgreSQL 连接配置
- **Redis 配置**: 缓存和会话存储配置
- **日志配置**: 日志级别、格式、输出目标
- **安全配置**: JWT、CORS、限流、加密设置
- **监控配置**: 指标收集、健康检查、分布式追踪、告警

每个配置项都包含详细的注释说明。

## 环境变量模板 (.env.example)

生成的环境变量文件包含：

```bash
# 应用基本配置
APP_NAME="Template Example App"
APP_VERSION="0.2.0"
ENVIRONMENT="production"
DEBUG=false
HOST="0.0.0.0"
PORT=8080

# 数据库配置
DATABASE_HOST="localhost"
DATABASE_PORT=5432
DATABASE_DATABASE="myapp"
DATABASE_USERNAME="postgres"
DATABASE_PASSWORD="your-secure-password"

# Redis 配置
REDIS_URL="redis://localhost:6379"

# 安全配置
SECURITY_JWT_SECRET="your-jwt-secret-key"

# 监控配置
MONITORING_ENABLED=true
```

## Docker Compose 配置

生成的 Docker Compose 文件包含：

- **应用服务**: 主应用程序容器
- **PostgreSQL**: 数据库服务
- **Redis**: 缓存服务
- **Prometheus**: 监控指标收集
- **Grafana**: 监控仪表板
- **Jaeger**: 分布式追踪

所有服务都配置了健康检查、重启策略和网络连接。

## Kubernetes 配置

生成的 K8s 配置包含：

- **Namespace**: 应用命名空间
- **ConfigMap**: 配置文件映射
- **Secret**: 敏感信息存储
- **Deployment**: 应用部署配置
- **Service**: 服务暴露配置
- **Ingress**: 外部访问配置
- **HPA**: 水平自动扩缩容

## 配置优先级

1. **默认值** - 代码中定义的默认配置
2. **配置文件** - `config.toml` 文件中的设置
3. **环境变量** - 系统环境变量
4. **命令行参数** - 运行时传入的参数

## 环境变量命名规则

环境变量使用以下命名规则：

- 顶级字段: `FIELD_NAME`
- 嵌套字段: `PARENT_CHILD_FIELD`
- 数组索引: `PARENT_CHILD_0_FIELD`

例如：
```bash
# 顶级字段
APP_NAME="My App"
PORT=8080

# 嵌套字段
DATABASE_HOST="localhost"
DATABASE_PORT=5432
SECURITY_JWT_SECRET="secret"

# 深度嵌套
SECURITY_RATE_LIMIT_ENABLED=true
MONITORING_TRACING_ENABLED=false
```

## 生产环境部署

### 使用 Docker Compose

```bash
# 复制并修改环境变量
cp .env.example .env
vim .env

# 启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f app
```

### 使用 Kubernetes

```bash
# 应用配置
kubectl apply -f k8s-deployment.yaml

# 查看部署状态
kubectl get pods -n template-example-app

# 查看服务
kubectl get services -n template-example-app

# 查看日志
kubectl logs -f deployment/template-example-app-deployment -n template-example-app
```

## 自定义模板

你可以基于生成的模板进行自定义：

1. **修改配置结构**: 在代码中添加或修改配置字段
2. **更新默认值**: 修改 `Default` 实现中的默认配置
3. **添加验证**: 在配置加载后添加验证逻辑
4. **扩展模板**: 添加新的模板生成功能

## 学习要点

1. **模板生成**: 了解如何自动生成配置模板
2. **配置文档**: 学习如何为配置添加详细文档
3. **部署配置**: 掌握 Docker 和 K8s 部署配置
4. **环境变量**: 理解环境变量的命名和使用规则
5. **生产就绪**: 学习生产环境的配置最佳实践
6. **配置管理**: 掌握复杂应用的配置管理策略

## 扩展建议

- 添加配置验证和约束
- 实现配置加密和解密
- 添加配置版本管理
- 集成配置中心（如 etcd、Consul）
- 添加配置变更审计
- 实现配置热重载机制
- 添加配置性能监控
- 支持多环境配置管理
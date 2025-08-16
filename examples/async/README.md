# QuantumConfig 异步配置示例

这个示例展示了如何在异步环境中使用 QuantumConfig 进行配置管理，包括配置热重载、响应式配置更新和异步服务协调。

## 功能特性

- **异步配置加载**: 在异步环境中无阻塞地加载配置
- **配置热重载**: 运行时动态重新加载配置文件
- **响应式更新**: 配置变更时自动更新所有相关服务
- **并发服务管理**: 同时管理多个异步服务
- **配置广播**: 使用 broadcast channel 分发配置变更
- **异步任务协调**: 协调多个长时间运行的异步任务

## Configuration Validation

This async example implements configuration validation to ensure all service configurations are consistent and sensible:

### Validation Features

**Warning-level validations**:
- Zero connection pool configurations (`max_connections = 0`)
- Very short timeout values (request timeout < 100ms)
- Empty or overly permissive cache configurations
- Disabled metrics collection in production
- Scheduler disabled when required

The validation approach uses warnings rather than errors for most issues to allow the application to start with sub-optimal but functional configurations, providing clear guidance for production deployment.

Example validation output:
```
Warning: max_connections is 0, which may cause connection issues
Warning: request_timeout of 50ms is very short and may cause timeouts
Warning: metrics collection is disabled - consider enabling for production monitoring
```

## 配置结构

```rust
#[derive(Config)]
struct AsyncAppConfig {
    // 应用基本信息
    name: String,
    version: String,
    debug: bool,
    port: u16,
    max_connections: usize,
    request_timeout: u64,
    
    // 子配置模块
    http_client: HttpClientConfig,
    scheduler: SchedulerConfig,
    cache: CacheConfig,
    metrics: MetricsConfig,
    
    // 热重载设置
    config_reload_interval: u64,
    hot_reload: bool,
}
```

## 运行示例

```bash
# 基本运行
cargo run --example async

# 使用环境变量
ASYNC_DEBUG=false ASYNC_PORT=9000 cargo run --example async

# 使用命令行参数
cargo run --example async -- --port 9000 --debug false

# 指定配置文件
cargo run --example async -- --config custom_config.toml
```

## 配置优先级

1. **默认值** - 代码中定义的默认配置
2. **配置文件** - `config.toml` 文件中的设置
3. **环境变量** - `ASYNC_*` 前缀的环境变量
4. **命令行参数** - 运行时传入的参数

## 环境变量示例

```bash
# 应用基本配置
export ASYNC_NAME="My Async App"
export ASYNC_VERSION="1.0.0"
export ASYNC_DEBUG=true
export ASYNC_PORT=8080
export ASYNC_MAX_CONNECTIONS=2000
export ASYNC_REQUEST_TIMEOUT=60

# HTTP客户端配置
export ASYNC_HTTP_CLIENT_BASE_URL="https://api.production.com"
export ASYNC_HTTP_CLIENT_TIMEOUT=45
export ASYNC_HTTP_CLIENT_MAX_RETRIES=5
export ASYNC_HTTP_CLIENT_USER_AGENT="ProductionApp/1.0.0"
export ASYNC_HTTP_CLIENT_COMPRESSION=true

# 调度器配置
export ASYNC_SCHEDULER_ENABLED=true
export ASYNC_SCHEDULER_WORKER_THREADS=8
export ASYNC_SCHEDULER_QUEUE_SIZE=2000
export ASYNC_SCHEDULER_TASK_TIMEOUT=600

# 缓存配置
export ASYNC_CACHE_ENABLED=true
export ASYNC_CACHE_CACHE_TYPE="redis"
export ASYNC_CACHE_DEFAULT_TTL=7200
export ASYNC_CACHE_MAX_ENTRIES=50000

# 指标配置
export ASYNC_METRICS_ENABLED=true
export ASYNC_METRICS_ENDPOINT="http://prometheus:9090/metrics"
export ASYNC_METRICS_COLLECTION_INTERVAL=5
export ASYNC_METRICS_BATCH_SIZE=200

# 热重载配置
export ASYNC_CONFIG_RELOAD_INTERVAL=10
export ASYNC_HOT_RELOAD=true
```

## 配置文件示例

```toml
# config.toml
name = "Production Async App"
version = "1.0.0"
debug = false
port = 8080
max_connections = 2000
request_timeout = 60
config_reload_interval = 10
hot_reload = true

[http_client]
base_url = "https://api.production.com"
timeout = 45
max_retries = 5
retry_interval = 2000
user_agent = "ProductionApp/1.0.0"
compression = true

[scheduler]
enabled = true
worker_threads = 8
queue_size = 2000
task_timeout = 600
cleanup_interval = 120

[cache]
enabled = true
cache_type = "redis"
default_ttl = 7200
max_entries = 50000
cleanup_interval = 600

[metrics]
enabled = true
endpoint = "http://prometheus:9090/metrics"
collection_interval = 5
batch_size = 200
buffer_size = 2000
```

## 异步特性说明

### 1. 配置热重载

```rust
// 配置管理器会定期检查配置变更
async fn start_hot_reload(&self) {
    let mut interval = interval(reload_interval);
    loop {
        interval.tick().await;
        // 检查并重新加载配置
        if let Ok(new_config) = self.reload_config().await {
            self.update_config(new_config).await;
        }
    }
}
```

### 2. 响应式配置更新

```rust
// 使用 broadcast channel 分发配置变更
let mut config_rx = config_manager.subscribe();
while let Ok(new_config) = config_rx.recv().await {
    // 更新所有服务的配置
    update_all_services(new_config).await;
}
```

### 3. 并发服务管理

```rust
// 同时启动多个异步服务
let tasks = vec![
    start_http_client(),
    start_scheduler(),
    start_cache(),
    start_metrics(),
];
join_all(tasks).await;
```

## 学习要点

1. **异步配置加载**: 了解如何在异步环境中使用 QuantumConfig
2. **配置热重载**: 实现运行时配置更新机制
3. **响应式架构**: 使用 broadcast channel 实现配置变更通知
4. **服务协调**: 管理多个异步服务的生命周期
5. **错误处理**: 异步环境中的错误处理最佳实践
6. **资源管理**: 使用 Arc 和 RwLock 进行并发访问控制

## 扩展建议

- 添加配置验证逻辑
- 实现配置回滚机制
- 集成外部配置源（如 etcd、Consul）
- 添加配置变更审计日志
- 实现配置加密和安全存储
- 添加配置性能监控
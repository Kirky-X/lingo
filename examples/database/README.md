# 数据库配置示例

这个示例展示了如何使用 Lingo 进行复杂的数据库配置管理，包括连接池、SSL、读写分离、迁移等高级功能。

## 功能特性

- 🗄️ **多数据库支持**: PostgreSQL、MySQL、SQLite
- 🏊 **连接池管理**: 可配置的连接池参数
- 🔒 **SSL/TLS 支持**: 安全连接配置
- 📖 **读写分离**: 支持只读副本配置
- 🔄 **数据库迁移**: 自动迁移管理
- 📊 **监控和日志**: 查询日志、慢查询检测、连接池监控
- ⚡ **异步支持**: 基于 Tokio 的异步操作

## 运行示例

### 1. 使用默认配置

```bash
cd examples/database
cargo run
```

### 2. 使用配置文件

```bash
# 使用提供的配置文件
cargo run -- --config config.toml

# 使用自定义配置文件
cargo run -- --config /path/to/your/database.toml
```

### 3. 使用环境变量

```bash
# 设置数据库连接信息
export DB_DB_TYPE=postgres
export DB_HOST=localhost
export DB_PORT=5432
export DB_DATABASE=myapp
export DB_USERNAME=postgres
export DB_PASSWORD=secret123

# 设置连接池配置
export DB_POOL_MIN_CONNECTIONS=5
export DB_POOL_MAX_CONNECTIONS=50

# 启用 SSL
export DB_SSL_ENABLED=true
export DB_SSL_MODE=require

# 启用读写分离
export DB_REPLICA_ENABLED=true
export DB_REPLICA_HOST=replica.localhost
export DB_REPLICA_USERNAME=readonly
export DB_REPLICA_PASSWORD=readonly123

# 启用监控
export DB_MONITORING_LOG_QUERIES=true
export DB_MONITORING_SLOW_QUERY_THRESHOLD=500

cargo run
```

### 4. 使用命令行参数

```bash
# 基本数据库配置
cargo run -- \
  --db-type postgres \
  --host localhost \
  --port 5432 \
  --database myapp \
  --username postgres \
  --password secret123

# 连接池配置
cargo run -- \
  --pool-min-connections 5 \
  --pool-max-connections 50 \
  --pool-idle-timeout 300

# SSL 配置
cargo run -- \
  --ssl-enabled true \
  --ssl-mode require \
  --ssl-ca-cert-file /path/to/ca.crt

# 监控配置
cargo run -- \
  --monitoring-log-queries true \
  --monitoring-slow-query-threshold 500 \
  --monitoring-monitor-pool true
```

## 配置优先级

配置加载优先级（从低到高）：

1. **默认值** - 代码中定义的默认配置
2. **系统配置文件** - `/etc/database_app/config.toml`
3. **用户配置文件** - `~/.config/database_app/config.toml`
4. **指定配置文件** - `--config` 参数指定的文件
5. **环境变量** - `DB_` 前缀的环境变量
6. **命令行参数** - 直接传递的参数

## 生产环境配置示例

### PostgreSQL 生产配置

```toml
# production.toml
db_type = "postgres"
host = "prod-db.company.com"
port = 5432
database = "production_app"
username = "app_user"
password = "${DB_PASSWORD}"  # 从环境变量获取
connect_timeout = 10
query_timeout = 30
application_name = "myapp_prod"

[replica]
enabled = true
host = "prod-db-replica.company.com"
port = 5432
database = "production_app"
username = "readonly_user"
password = "${DB_REPLICA_PASSWORD}"

[pool]
min_connections = 10
max_connections = 100
idle_timeout = 300
max_lifetime = 1800
acquire_timeout = 10

[ssl]
enabled = true
mode = "require"
ca_cert_file = "/etc/ssl/certs/ca.crt"
client_cert_file = "/etc/ssl/certs/client.crt"
client_key_file = "/etc/ssl/private/client.key"

[monitoring]
log_queries = false
slow_query_threshold = 1000
monitor_pool = true
monitor_interval = 30
enable_metrics = true
```

### MySQL 配置示例

```toml
db_type = "mysql"
host = "mysql.company.com"
port = 3306
database = "myapp"
username = "app_user"
password = "secure_password"

[pool]
min_connections = 5
max_connections = 50
test_query = "SELECT 1"

[ssl]
enabled = true
mode = "required"
```

### SQLite 配置示例

```toml
db_type = "sqlite"
database = "/var/lib/myapp/database.db"

[pool]
min_connections = 1
max_connections = 1  # SQLite 通常使用单连接

[migration]
auto_migrate = true
migrations_dir = "./migrations"
```

## 配置结构说明

### 主数据库配置 (`DatabaseConnectionConfig`)

- `db_type`: 数据库类型（postgres/mysql/sqlite）
- `host`: 数据库主机地址
- `port`: 数据库端口
- `database`: 数据库名称
- `username`: 用户名
- `password`: 密码
- `connect_timeout`: 连接超时时间
- `query_timeout`: 查询超时时间
- `application_name`: 应用程序标识

### 只读副本配置 (`ReplicaConfig`)

- `enabled`: 是否启用读写分离
- `host`: 只读副本主机
- `port`: 只读副本端口
- `database`: 只读副本数据库名
- `username`: 只读副本用户名
- `password`: 只读副本密码

### 连接池配置 (`ConnectionPoolConfig`)

- `min_connections`: 最小连接数
- `max_connections`: 最大连接数
- `idle_timeout`: 连接空闲超时
- `max_lifetime`: 连接最大生命周期
- `acquire_timeout`: 获取连接超时
- `test_query`: 连接测试查询

### 迁移配置 (`MigrationConfig`)

- `auto_migrate`: 是否启用自动迁移
- `migrations_dir`: 迁移文件目录
- `migrations_table`: 迁移记录表名
- `migration_timeout`: 迁移超时时间

### SSL 配置 (`SslConfig`)

- `enabled`: 是否启用 SSL
- `mode`: SSL 模式
- `ca_cert_file`: CA 证书文件路径
- `client_cert_file`: 客户端证书文件路径
- `client_key_file`: 客户端私钥文件路径

### 监控配置 (`MonitoringConfig`)

- `log_queries`: 是否记录查询日志
- `slow_query_threshold`: 慢查询阈值（毫秒）
- `monitor_pool`: 是否监控连接池
- `monitor_interval`: 监控间隔
- `enable_metrics`: 是否启用性能指标

## 环境变量映射

所有配置项都可以通过环境变量设置，使用 `DB_` 前缀：

```bash
# 主数据库配置
DB_DB_TYPE=postgres
DB_HOST=localhost
DB_PORT=5432
DB_DATABASE=myapp
DB_USERNAME=postgres
DB_PASSWORD=secret

# 连接池配置
DB_POOL_MIN_CONNECTIONS=5
DB_POOL_MAX_CONNECTIONS=50

# SSL 配置
DB_SSL_ENABLED=true
DB_SSL_MODE=require

# 只读副本配置
DB_REPLICA_ENABLED=true
DB_REPLICA_HOST=replica.localhost

# 监控配置
DB_MONITORING_LOG_QUERIES=true
DB_MONITORING_SLOW_QUERY_THRESHOLD=500
```

## 学习要点

### 1. 复杂嵌套结构

```rust
#[derive(Config, Serialize, Deserialize, Debug, Default)]
struct DatabaseConfig {
    #[lingo_opt(flatten)]
    primary: DatabaseConnectionConfig,
    
    #[lingo_opt(flatten)]
    replica: Option<ReplicaConfig>,
    
    #[lingo_opt(flatten)]
    pool: ConnectionPoolConfig,
}
```

### 2. 可选配置字段

```rust
struct ReplicaConfig {
    enabled: Option<bool>,
    host: Option<String>,
    password: Option<String>,
}
```

### 3. 配置验证

```rust
fn validate_config(config: &DatabaseConfig) -> Result<(), Box<dyn Error>> {
    if config.pool.min_connections > config.pool.max_connections {
        return Err("最小连接数不能大于最大连接数".into());
    }
    Ok(())
}
```

### 4. 敏感信息处理

```rust
fn mask_password(url: &str) -> String {
    // 在日志中隐藏密码
    // ...
}
```

### 5. 异步配置应用

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = DatabaseConfig::new();
    test_database_connection(&config).await?;
    Ok(())
}
```

这个示例展示了 Lingo 在复杂企业级应用中的配置管理能力，特别适合需要精细化数据库配置的场景。
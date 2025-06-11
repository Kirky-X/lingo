# Lingo (玲珑) 🎯

一个强大且灵活的 Rust 配置管理库，让配置加载变得简单而优雅。

[![Crates.io](https://img.shields.io/crates/v/lingo.svg)](https://crates.io/crates/lingo)
[![Documentation](https://docs.rs/lingo/badge.svg)](https://docs.rs/lingo)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://github.com/Kirky-X/lingo/actions/workflows/rust.yml/badge.svg)](https://github.com/Kirky-X/lingo/actions/workflows/rust.yml)

**[English](README_EN.md)** | **[更新日志](CHANGELOG.md)** | **[文档](https://docs.rs/lingo)**

## 🌟 特性

- **多源配置加载** - 支持 TOML、JSON、INI 文件、环境变量和命令行参数
- **智能优先级** - 自动按优先级合并配置：系统文件 < 用户文件 < 指定文件 < 环境变量 < 命令行参数
- **过程宏驱动** - 通过 `#[derive(LingoLoader)]` 和 `#[lingo_opt(...)]` 属性简化配置定义
- **类型安全** - 完全的编译时类型检查，避免运行时配置错误
- **深度集成 Clap** - 自动生成命令行参数解析，包括帮助信息和版本信息
- **嵌套结构体** - 支持任意深度的嵌套配置结构
- **配置模板生成** - 自动生成带注释的配置文件模板
- **异步支持** - 提供同步和异步两种加载方式
- **跨平台** - 支持 Linux、macOS 和 Windows

## 🚀 快速开始

### 添加依赖

```toml
[dependencies]
lingo = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
```

### 基本用法

```rust
use lingo::Config;
use serde::{Deserialize, Serialize};

#[derive(LingoLoader, Serialize, Deserialize, Debug, Default)]
#[lingo(app_name = "myapp", env_prefix = "MYAPP")]
struct AppConfig {
    #[lingo_opt(description = "服务器主机地址", default = "\"localhost\".to_string()")]
    host: String,
    
    #[lingo_opt(description = "服务器端口", default = "8080")]
    port: u16,
    
    #[lingo_opt(description = "是否启用调试模式", name_clap_long = "debug")]
    debug_mode: Option<bool>,
    
    #[lingo_opt(flatten)]
    database: DatabaseConfig,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct DatabaseConfig {
    #[lingo_opt(description = "数据库URL")]
    url: Option<String>,
    
    #[lingo_opt(description = "最大连接数", default = "10")]
    max_connections: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let config = AppConfig::load()?;
    
    println!("服务器将在 {}:{} 启动", config.host, config.port);
    println!("调试模式: {:?}", config.debug_mode);
    
    Ok(())
}
```

### 配置文件示例

**config.toml**
```toml
# 服务器主机地址
host = "0.0.0.0"
# 服务器端口
port = 3000

[database]
# 数据库URL
url = "postgresql://localhost/myapp"
# 最大连接数
max_connections = 20
```

### 环境变量

```bash
export MYAPP_HOST="0.0.0.0"
export MYAPP_PORT="3000"
export MYAPP_DATABASE_URL="postgresql://localhost/myapp"
```

### 命令行参数

```bash
./myapp --host 0.0.0.0 --port 3000 --debug --database-url postgresql://localhost/myapp
```

## 📖 详细文档

### 配置加载优先级

Lingo 按以下优先级加载和合并配置（后者覆盖前者）：

1. **系统配置文件** - `/etc/{app_name}/config.{toml,json,ini}`
2. **用户配置文件** - `~/.config/{app_name}/config.{toml,json,ini}`
3. **指定配置文件** - 通过 `--config` 参数指定
4. **环境变量** - 使用 `{ENV_PREFIX}_` 前缀
5. **命令行参数** - 最高优先级

### 字段属性详解

#### `#[lingo_opt(...)]` 属性

- `description = "描述"` - 字段描述，用于生成帮助信息和配置模板
- `default = "表达式"` - 默认值表达式
- `name_config = "名称"` - 配置文件中的键名
- `name_env = "名称"` - 环境变量名
- `name_clap_long = "名称"` - 长命令行选项名
- `name_clap_short = 'c'` - 短命令行选项
- `flatten` - 展平嵌套结构体
- `skip` - 跳过此字段
- `clap(...)` - 传递给 clap 的额外属性

#### `#[lingo(...)]` 结构体属性

- `app_name = "名称"` - 应用程序名称，用于配置文件路径
- `env_prefix = "前缀"` - 环境变量前缀
- `version = 1` - 宏行为版本（用于向后兼容）
- `max_parse_depth = 128` - 最大解析深度

### 异步支持

启用 `async` 特性后，可以使用异步加载：

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load_async().await?;
    // ...
    Ok(())
}
```

### 配置模板生成

```rust
// 生成 TOML 配置模板
let template = AppConfig::generate_config_template(lingo_core::paths::ConfigFileType::Toml)?;
println!("{}", template);
```

### 错误处理

Lingo 提供详细的错误信息：

```rust
use lingo_core::error::LingoError;

match AppConfig::load() {
    Ok(config) => println!("配置加载成功: {:?}", config),
    Err(LingoError::FileParse { format_name, path, source_error }) => {
        eprintln!("配置文件解析错误: {} 文件 {:?} - {}", format_name, path, source_error);
    }
    Err(LingoError::Io { source, path }) => {
        eprintln!("IO 错误: {:?} - {}", path, source);
    }
    Err(e) => eprintln!("其他错误: {}", e),
}
```

## 🔧 高级用法

### 自定义配置文件路径

```rust
use lingo_core::paths::{ConfigFilePath, ConfigFileType};

let custom_paths = vec![
    ConfigFilePath {
        path: "/custom/path/config.toml".into(),
        file_type: ConfigFileType::Toml,
        is_required: false,
    }
];

let config = AppConfig::load_with_custom_paths(&custom_paths)?;
```

### 与现有 Clap 应用集成

```rust
use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    #[command(flatten)]
    config_args: AppConfigClapArgs,  // 由 LingoLoader 生成
    
    #[arg(long)]
    verbose: bool,
}

let cli = CliArgs::parse();
let config = AppConfig::from_clap_matches(&cli.config_args.into())?;
```

## 🎯 示例项目

查看 [`examples/`](./examples/) 目录获取更多完整示例：

- [`basic/`](./examples/basic/) - 基本配置加载
- [`web_server/`](./examples/web_server/) - Web 服务器配置
- [`database/`](./examples/database/) - 数据库连接配置
- [`nested/`](./examples/nested/) - 复杂嵌套配置
- [`async/`](./examples/async/) - 异步配置加载
- [`custom_paths/`](./examples/custom_paths/) - 自定义配置路径

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

### 开发环境设置

```bash
git clone https://github.com/Kirky-X/lingo.git
cd lingo
cargo test
cargo doc --open
```

## 📄 许可证

本项目采用 Apache-2.0 许可证。详情请参阅 [LICENSE](LICENSE) 文件。

## 👨‍💻 作者

- Kirky.X <Kirky-X@outlook.com>

## 🙏 致谢

- [figment](https://github.com/SergioBenitez/Figment) - 强大的配置库基础
- [clap](https://github.com/clap-rs/clap) - 优秀的命令行参数解析
- [serde](https://github.com/serde-rs/serde) - Rust 序列化框架

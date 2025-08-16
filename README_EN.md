# Quantum Config

A powerful and flexible Rust configuration management library that makes configuration loading simple and elegant.

[![Rust](https://github.com/Kirky-X/quantum_config/actions/workflows/rust.yml/badge.svg)](https://github.com/Kirky-X/quantum_config/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/quantum_config.svg)](https://crates.io/crates/quantum_config)
[![Docs.rs](https://docs.rs/quantum_config/badge.svg)](https://docs.rs/quantum_config)

## 📋 Scope

What Quantum Config focuses on:
- Configuration loading, parsing, and type conversion
- Merging multiple sources (files, environment variables, CLI)
- Configuration validation and error handling
- Developer tooling (template generation, help docs)

What Quantum Config does not include:
- Web server or HTTP implementation
- Database connections or ORM functionality
- Infrastructure components like caches or message queues
- Business logic or application frameworks

About the examples/ directory: The examples demonstrate using Quantum Config in different scenarios (web service, database app, async program). These applications themselves are beyond Quantum Config's core library scope.

**[中文](README.md)** | **[Changelog](CHANGELOG.md)** | **[Documentation](https://docs.rs/quantum_config)**

## 🌟 Features

- **Multi-source Configuration Loading** - Support for TOML, JSON, INI files, environment variables, and command-line arguments
- **Smart Priority System** - Automatic configuration merging by priority: system files < user files < specified files < environment variables < command-line arguments
- **Procedural Macro Driven** - Simplify configuration definition with `#[derive(Config)]`, `#[config(...)]` and `#[quantum_config_opt(...)]` attributes
- **Type Safety** - Complete compile-time type checking to avoid runtime configuration errors
- **Deep Clap Integration** - Automatic command-line argument parsing with help and version information
- **Nested Structures** - Support for arbitrarily deep nested configuration structures
- **Template Generation** - Automatic generation of commented configuration file templates
- **Async Support** - Both synchronous and asynchronous loading methods
- **Cross-platform** - Support for Linux, macOS, and Windows

## 🚀 Quick Start

### Add Dependencies

```toml
[dependencies]
quantum_config = "0.2.0"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Usage

```rust
use quantum_config::Config;
use serde::{Deserialize, Serialize};

#[derive(Config, Serialize, Deserialize, Debug, Default)]
#[config(env_prefix = "MYAPP_")]
struct AppConfig {
    #[quantum_config_opt(description = "Server host address", default = "\"localhost\".to_string()")]
    host: String,
    
    #[quantum_config_opt(description = "Server port", default = "8080")]
    port: u16,
    
    #[quantum_config_opt(description = "Enable debug mode", name_clap_long = "debug")]
    debug_mode: Option<bool>,
    
    #[quantum_config_opt(flatten)]
    database: DatabaseConfig,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct DatabaseConfig {
    #[quantum_config_opt(description = "Database URL")]
    url: Option<String>,
    
    #[quantum_config_opt(description = "Maximum connections", default = "10")]
    max_connections: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::load()?;
    
    println!("Server will start on {}:{}", config.host, config.port);
    println!("Debug mode: {:?}", config.debug_mode);
    
    Ok(())
}
```

### Configuration File Example

**config.toml**
```toml
# Server host address
host = "0.0.0.0"
# Server port
port = 3000

[database]
# Database URL
url = "postgresql://localhost/myapp"
# Maximum connections
max_connections = 20
```

### Environment Variables

```bash
export MYAPP_HOST="0.0.0.0"
export MYAPP_PORT="3000"
export MYAPP_DATABASE_URL="postgresql://localhost/myapp"
```

### Command Line Arguments

```bash
./myapp --host 0.0.0.0 --port 3000 --debug --database-url postgresql://localhost/myapp
```

## 📖 Detailed Documentation

### Configuration Loading Priority

Quantum Config loads and merges configuration in the following priority order (later sources override earlier ones):

1. **System configuration files** - `/etc/{app_name}/config.{toml,json,ini}`
2. **User configuration files** - `~/.config/{app_name}/config.{toml,json,ini}`
3. **Specified configuration files** - Via `--config` parameter
4. **Environment variables** - Using `{ENV_PREFIX}_` prefix
5. **Command line arguments** - Highest priority

### Field Attributes Reference

#### `#[quantum_config_opt(...)]` Attributes

- `description = "description"` - Field description for help text and config templates
- `default = "expression"` - Default value expression
- `name_config = "name"` - Key name in configuration files
- `name_env = "name"` - Environment variable name
- `name_clap_long = "name"` - Long command-line option name
- `name_clap_short = 'c'` - Short command-line option
- `flatten` - Flatten nested structures
- `skip` - Skip this field
- `clap(...)` - Additional attributes passed to clap

#### `#[config(...)]` Struct Attributes

- `env_prefix = "prefix"` - Environment variable prefix, e.g. `"MYAPP_"`

### Async Support

With the `async` feature enabled, you can use asynchronous loading:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load_async().await?;
    // ...
    Ok(())
}
```

### Configuration Template Generation

```rust
// Generate TOML configuration template file on disk
AppConfig::generate_template()?;
```

### Error Handling

Quantum Config provides detailed error information:

```rust
use quantum_config::QuantumConfigError;

match AppConfig::load() {
    Ok(config) => println!("Configuration loaded successfully: {:?}", config),
    Err(QuantumConfigError::FileParse { format_name, path, source_error }) => {
        eprintln!("Configuration file parse error: {} file {:?} - {}", format_name, path, source_error);
    }
    Err(QuantumConfigError::Io { source, path }) => {
        eprintln!("IO error: {:?} - {}", path, source);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## 🔧 Advanced Usage

### Custom Configuration File Paths

```rust
use quantum_config::{ConfigFilePath, ConfigFileType};

let custom_paths = vec![
    ConfigFilePath {
        path: "/custom/path/config.toml".into(),
        file_type: ConfigFileType::Toml,
        is_required: false,
    }
];

let config = AppConfig::load_with_custom_paths(&custom_paths)?;
```



## 🎯 Example Projects

Check out the [`examples/`](./examples/) directory for more complete examples:

- [`basic/`](./examples/basic/) - Basic configuration loading
- [`web_server/`](./examples/web_server/) - Web server configuration
- [`database/`](./examples/database/) - Database connection configuration
- [`nested/`](./examples/nested/) - Complex nested configuration
- [`async/`](./examples/async/) - Asynchronous configuration loading
- [`custom_paths/`](./examples/custom_paths/) - Custom configuration paths

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Environment Setup

```bash
git clone https://github.com/Kirky-X/quantum_config.git
cd quantum_config
cargo test
cargo doc --open
```

## 📄 License

This project is licensed under the Apache-2.0 license. See [LICENSE](LICENSE) for details.

## 👨‍💻 Author

**Kirky.X** - [Kirky-X@outlook.com](mailto:Kirky-X@outlook.com)

## 🙏 Acknowledgments

- [figment](https://github.com/SergioBenitez/Figment) - Powerful configuration library foundation
- [clap](https://github.com/clap-rs/clap) - Excellent command-line argument parsing
- [serde](https://github.com/serde-rs/serde) - Rust serialization framework

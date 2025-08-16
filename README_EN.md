# Quantum Config

A powerful and flexible Rust configuration management library that makes configuration loading simple and elegant.

[![Rust](https://github.com/Kirky-X/quantum_config/actions/workflows/rust.yml/badge.svg)](https://github.com/Kirky-X/quantum_config/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/quantum_config.svg)](https://crates.io/crates/quantum_config)
[![Docs.rs](https://docs.rs/quantum_config/badge.svg)](https://docs.rs/quantum_config)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

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
- **Error Handling** - Comprehensive configuration management error types
- **Path Resolution** - Automatic discovery of system and user configuration directories
- **Async Support** - Both synchronous and asynchronous loading methods
- **Cross-platform** - Support for Linux, macOS, and Windows

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
    Err(QuantumConfigError::Figment(figment_error)) => {
        eprintln!("Configuration extraction error: {}", figment_error);
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

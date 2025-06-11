# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure
- Core configuration management functionality
- Support for multiple configuration sources (files, environment variables, command line)
- Intelligent priority system for configuration loading
- Procedural macro-driven configuration derivation
- Type-safe configuration handling
- Deep integration with Clap for command-line argument parsing
- Support for nested structures
- Configuration template generation
- Asynchronous support
- Cross-platform compatibility

### Changed
- Updated dependency versions in Cargo.toml
- Enhanced error handling and reporting
- Improved documentation and examples

### Fixed
- Compilation issues with updated dependencies
- Type safety improvements
- Memory safety enhancements

## [0.1.0] - 2025-06-11

### Added
- Initial release of lingo configuration management library
- Basic configuration loading from TOML files
- Environment variable support
- Command-line argument integration
- Derive macro for automatic configuration struct generation
- Configuration validation
- Error handling with detailed error messages
- Documentation and usage examples

### Features
- **Multi-source Configuration Loading**: Load from files, environment variables, and command-line arguments
- **Intelligent Priority System**: Automatic precedence handling across configuration sources
- **Procedural Macro Driven**: Derive-based configuration with minimal boilerplate
- **Type Safety**: Compile-time guarantees for configuration correctness
- **Deep Clap Integration**: Seamless command-line argument parsing
- **Nested Structures**: Support for complex configuration hierarchies
- **Template Generation**: Automatic configuration file template creation
- **Async Support**: Full compatibility with async/await patterns
- **Cross-platform**: Works on Windows, macOS, and Linux

### Dependencies
- figment = "0.10.19"
- clap = { version = "4.5.21", features = ["derive"] }
- thiserror = "2.0.3"
- log = "0.4.22"
- tracing = "0.1.41"
- tokio = { version = "1.41.1", features = ["full"] }
- directories = "5.0.1"
- lingo-derive = { path = "./lingo-derive", version = "0.1.0" }

### Examples
- Basic configuration example
- Database configuration with connection pooling
- Web server configuration with HTTP settings

### Documentation
- Comprehensive README with usage examples
- API documentation
- Configuration guide
- Best practices documentation

---

## Release Notes

### Version 0.1.0
This is the initial release of the lingo configuration management library. It provides a robust, type-safe, and flexible solution for managing application configuration in Rust projects.

**Key Highlights:**
- Zero-configuration setup with sensible defaults
- Automatic configuration merging from multiple sources
- Compile-time validation of configuration structures
- Rich error reporting for configuration issues
- Extensive documentation and examples

**Getting Started:**
```toml
[dependencies]
lingo = "0.1.0"
```

**Basic Usage:**
```rust
use lingo::Config;

#[derive(Config)]
struct AppConfig {
    name: String,
    port: u16,
    debug: bool,
}

fn main() {
    let config = AppConfig::load().unwrap();
    println!("Starting {} on port {}", config.name, config.port);
}
```

For more examples and detailed documentation, please refer to the README.md file and the examples directory.
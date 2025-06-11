# Changelog

All notable changes to this project will be documented in this file.

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

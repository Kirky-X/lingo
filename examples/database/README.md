# Database Example

This example demonstrates how to use quantum_config for database configuration management.

## Overview

The database example shows:
- Connection pool configuration
- SSL settings management
- Environment-specific configurations
- Type-safe database configuration

## Features

- **Connection Pool Management**: Configure min/max connections with automatic validation
- **SSL Configuration**: Flexible SSL settings with certificate management
- **Multi-Database Support**: Support for PostgreSQL, MySQL, and SQLite
- **Environment Variables**: Override configuration with environment variables

## Configuration Validation

This example includes comprehensive configuration validation to ensure production-ready deployments:

### Validation Categories

**Error-level validations** (prevent application startup):
- Database type must be valid (`postgresql`, `mysql`, or `sqlite`)
- Host and database name cannot be empty
- Username cannot be empty for non-SQLite databases
- `min_connections` cannot exceed `max_connections`
- SSL contradictions: requiring SSL but providing invalid certificates

**Warning-level validations** (allow startup with warnings):
- Zero connection pool sizes (`min_connections = 0` or `max_connections = 0`)
- Very short connection timeouts (< 1 second)
- Disabled SSL in production environments

### Validation Examples

```rust
// This will produce a warning but allow startup
DatabaseConfig {
    min_connections: 0,     // Warning: may impact performance
    max_connections: 10,
    // ... other fields
}

// This will cause startup failure
DatabaseConfig {
    min_connections: 15,    // Error: violates pool constraints
    max_connections: 10,
    // ... other fields
}
```

### Configuration Validation in Practice

The validation framework helps catch common configuration issues:

1. **Development Safety**: Warns about configurations that work but aren't optimal
2. **Production Readiness**: Prevents deployment with invalid configurations
3. **Operational Visibility**: Clear error messages for troubleshooting

Run the example to see validation in action:

```bash
cargo run --bin database
```

The application will validate the configuration on startup and display any warnings or errors before proceeding.

## Running the Example

```bash
cargo run --bin database
```

## Configuration Structure

```rust
#[derive(Config, Debug)]
struct DatabaseConfig {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl_mode: String,
    min_connections: u32,
    max_connections: u32,
    connection_timeout: u64,
}
```
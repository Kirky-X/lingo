# Security Guide for Quantum Config

## Overview

This document outlines the security measures implemented in Quantum Config and provides guidelines for secure usage.

## Security Features

### 1. Path Traversal Protection

**Issue**: Malicious configuration file paths could access sensitive system files.

**Solution**: Implemented `validate_path_security` function that:
- Normalizes file paths to prevent directory traversal
- Blocks dangerous patterns like `../` and `..\`
- Prevents access to sensitive system directories
- Validates path components for security

**Location**: `src/paths.rs`

**Usage**: Automatically applied to all user-specified configuration file paths.

### 2. Information Disclosure Prevention

**Issue**: Error messages could leak sensitive file system information.

**Solution**: Implemented `sanitize_path_for_display` function that:
- Hides full file paths in production environments
- Shows only filenames to users
- Preserves full paths in debug mode for development
- Applies to all error messages containing file paths

**Location**: `src/error.rs`

**Configuration**: Controlled by `cfg(debug_assertions)` flag.

### 3. Environment Variable Validation

**Issue**: Malicious environment variables could inject harmful data.

**Solution**: Implemented validation functions that:
- Limit environment variable key and value lengths
- Block dangerous characters and patterns
- Prevent control character injection
- Validate against known malicious patterns

**Location**: `src/providers/env_provider.rs`

**Limits**:
- Key length: max 256 characters
- Value length: max 8192 characters
- Blocked characters: control chars, null bytes

### 4. Parse Depth Limiting

**Issue**: Deeply nested configurations could cause stack overflow attacks.

**Solution**: Reduced default parse depth limit:
- Previous limit: 128 levels
- New limit: 32 levels
- Configurable via `QuantumConfigAppMeta::max_parse_depth`

**Location**: `src/meta.rs`, `src/paths.rs`, `src/providers/file_provider.rs`

### 5. Command Injection Prevention

**Issue**: Derive macro could enable external command execution.

**Solution**: Removed `allow_external_subcommands` attribute to prevent:
- Arbitrary command execution
- Shell injection attacks
- Privilege escalation

**Location**: `quantum_config_derive/src/lib.rs`

### 6. Path Conversion Security

**Issue**: Exposing path conversion utilities could enable path manipulation attacks.

**Solution**: Internalized path conversion functionality:
- Removed public `PathConverter` and `PathFormat` exports from library API
- Removed public path conversion methods from `ConfigFilePath` struct
- Path conversion now happens automatically within the library
- Users cannot directly manipulate path conversion logic
- Prevents potential bypass of security validations

**Location**: `src/lib.rs`, `src/paths.rs`, `src/path_conversion.rs`

**Security Benefits**:
- Reduces attack surface by limiting public API
- Prevents users from bypassing internal path validation
- Ensures consistent path handling across all operations
- Eliminates potential for path manipulation vulnerabilities

## Security Testing

Comprehensive security tests are implemented in `src/security_tests.rs`:

- Path traversal attack prevention
- Environment variable injection protection
- Parse depth limit enforcement
- Error message sanitization
- Path conversion security validation
- Performance under malicious input

Additional path conversion tests in `src/paths.rs`:

- Internal path conversion functionality
- Cross-platform path handling
- Path normalization security

## Security Best Practices

### For Library Users

1. **File Permissions**: Ensure configuration files have appropriate permissions (600 or 644).
2. **Environment Variables**: Validate environment variables in your application before using them.
3. **Error Handling**: Don't log full error messages in production to prevent information leakage.
4. **Input Validation**: Always validate configuration values after loading.
5. **Principle of Least Privilege**: Run applications with minimal required permissions.

### For Developers

1. **Security Reviews**: All security-related changes must undergo code review.
2. **Testing**: Run security tests regularly: `cargo test security_tests`.
3. **Dependencies**: Keep dependencies updated and audit regularly: `cargo audit`.
4. **Static Analysis**: Use tools like `cargo clippy` for security lints.
5. **Documentation**: Document any security implications of new features.

## Vulnerability Reporting

If you discover a security vulnerability:

1. **Do not** create a public issue
2. Contact the maintainers privately
3. Provide detailed reproduction steps
4. Allow reasonable time for fixes before disclosure

## Security Checklist

Before releasing:

- [ ] All security tests pass
- [ ] Dependencies audited with `cargo audit`
- [ ] Static analysis with `cargo clippy`
- [ ] Security documentation updated
- [ ] No hardcoded secrets or credentials
- [ ] Error messages don't leak sensitive information
- [ ] Input validation covers all user inputs
- [ ] File permissions are restrictive
- [ ] Path conversion functionality is internal only
- [ ] No public APIs expose path manipulation utilities
- [ ] Cross-platform path handling is secure

## Compliance

This library implements security measures aligned with:

- OWASP Top 10 security risks
- CWE (Common Weakness Enumeration) guidelines
- Rust security best practices
- Supply chain security recommendations

## Updates

This security guide is updated with each release. Check the changelog for security-related changes.

---

**Last Updated**: January 2025
**Version**: 0.2.1
**Security Enhancements**: Path conversion internalization, enhanced API security
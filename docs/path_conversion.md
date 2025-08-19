# 路径转换功能 (Path Conversion)

## 概述

Quantum Config 提供了强大的跨平台路径转换功能，允许您在 Unix 和 Windows 路径格式之间进行无缝转换。这个功能特别适用于需要在不同操作系统之间共享配置文件的应用程序。

## 核心特性

- **跨平台兼容性**: 支持 Unix (`/`) 和 Windows (`\`) 路径分隔符
- **自动格式检测**: 智能识别路径格式
- **统一内部格式**: 内部统一使用 Unix 格式，对外自动转换
- **类型安全**: 为 `Path`、`PathBuf`、`str` 和 `String` 提供扩展方法
- **工具函数**: 提供便捷的工具函数用于批量转换

## 使用方法

### 1. 基本路径转换

```rust
use quantum_config::PathConverter;

// 字符串路径转换
let unix_path = "/home/user/config.toml";
let windows_path = unix_path.to_windows_format()?; // "\\home\\user\\config.toml"
let native_path = unix_path.to_native_format()?;   // 根据当前平台返回适当格式

// Windows 路径转换为 Unix 格式
let windows_path = "C:\\Users\\user\\config.toml";
let unix_path = windows_path.to_unix_format()?;    // "/C/Users/user/config.toml"

// PathBuf 转换
use std::path::PathBuf;
let path_buf = PathBuf::from("data/logs/app.log");
let unix_format = path_buf.to_unix_format()?;
let windows_format = path_buf.to_windows_format()?;
```

### 2. 工具函数

```rust
use quantum_config::path_conversion::utils;

// 格式检测
let format = utils::detect_format("/unix/path");     // PathFormat::Unix
let format = utils::detect_format("C:\\windows");    // PathFormat::Windows

// 格式验证
let is_unix = utils::is_unix_format("/unix/path");      // true
let is_windows = utils::is_windows_format("C:\\win");   // true

// 批量转换
let unix_path = utils::to_unix("C:\\Users\\user")?;     // "/C/Users/user"
let windows_path = utils::to_windows("/home/user")?;   // "\\home\\user"
let native_path = utils::to_native("any/path")?;       // 根据平台转换

// 平台特定规范化
let normalized = utils::normalize_for_platform("test/path", true)?;  // Windows 格式
let normalized = utils::normalize_for_platform("test/path", false)?; // Unix 格式
```

### 3. ConfigFilePath 集成

```rust
use quantum_config::ConfigFilePath;

let config_path = ConfigFilePath::new("/etc/app/config.toml")?;

// 转换为不同格式
let unix_path = config_path.to_unix_path()?;
let windows_path = config_path.to_windows_path()?;
let native_path = config_path.to_native_path()?;

// 平台规范化
let normalized = config_path.normalize_for_platform()?;
```

### 4. 配置文件中的路径处理

```rust
use quantum_config::{Config, PathConverter};
use serde::{Deserialize, Serialize};

#[derive(Config, Deserialize, Serialize)]
struct AppConfig {
    log_path: String,
    data_dirs: Vec<String>,
}

let mut config = AppConfig {
    log_path: "C:\\logs\\app.log".to_string(),
    data_dirs: vec![
        "/var/data".to_string(),
        "C:\\ProgramData\\App".to_string(),
    ],
};

// 统一转换为 Unix 格式进行内部处理
config.log_path = config.log_path.to_unix_format()?;
for path in &mut config.data_dirs {
    *path = path.to_unix_format()?;
}

// 使用时转换回原生格式
let native_log_path = config.log_path.to_native_format()?;
```

## 路径格式

### Unix 格式
- 使用正斜杠 (`/`) 作为路径分隔符
- 绝对路径以 `/` 开头
- 示例: `/home/user/config.toml`, `./relative/path`

### Windows 格式
- 使用反斜杠 (`\`) 作为路径分隔符
- 绝对路径通常以驱动器字母开头 (如 `C:`)
- 示例: `C:\\Users\\user\\config.toml`, `.\\relative\\path`

### 格式检测规则

1. **Windows 格式**: 包含驱动器字母 (`C:`) 或使用反斜杠分隔符
2. **Unix 格式**: 使用正斜杠分隔符且不包含驱动器字母
3. **未知格式**: 单个文件名或无法明确识别的格式

## 安全考虑

路径转换功能与 Quantum Config 的安全机制完全集成：

- **路径遍历防护**: 转换后的路径仍会进行安全验证
- **敏感目录检查**: 支持跨平台的敏感目录识别
- **路径规范化**: 自动处理 `..` 和 `.` 等特殊路径组件

## 性能优化

- **零拷贝优化**: 当路径已经是目标格式时，避免不必要的转换
- **缓存机制**: 内部使用高效的字符串操作
- **惰性转换**: 只在需要时进行格式转换

## 错误处理

路径转换可能返回以下错误：

- `PathConversionError::InvalidPath`: 无效的路径格式
- `PathConversionError::UnsupportedOperation`: 不支持的转换操作
- `PathConversionError::SystemError`: 系统级错误

```rust
use quantum_config::PathConverter;

match "invalid\0path".to_unix_format() {
    Ok(converted) => println!("Converted: {}", converted),
    Err(e) => eprintln!("Conversion failed: {}", e),
}
```

## 示例项目

完整的使用示例请参考 `examples/path_conversion` 目录，其中包含了各种路径转换场景的演示代码。

运行示例：

```bash
cd examples/path_conversion
cargo run
```

## 最佳实践

1. **内部统一格式**: 在应用程序内部统一使用 Unix 格式存储路径
2. **边界转换**: 在与操作系统交互时转换为原生格式
3. **配置文件**: 配置文件中可以使用任意格式，程序自动处理转换
4. **错误处理**: 始终处理路径转换可能产生的错误
5. **测试覆盖**: 确保在不同平台上测试路径转换功能

## API 参考

### PathConverter Trait

```rust
pub trait PathConverter {
    type Error;
    
    fn to_unix_format(&self) -> Result<String, Self::Error>;
    fn to_windows_format(&self) -> Result<String, Self::Error>;
    fn to_native_format(&self) -> Result<PathBuf, Self::Error>;
    fn normalize_for_platform(&self) -> Result<String, Self::Error>;
}
```

### 工具函数

```rust
pub mod utils {
    pub fn detect_format(path: &str) -> PathFormat;
    pub fn is_unix_format(path: &str) -> bool;
    pub fn is_windows_format(path: &str) -> bool;
    pub fn to_unix(path: &str) -> Result<String, PathConversionError>;
    pub fn to_windows(path: &str) -> Result<String, PathConversionError>;
    pub fn to_native(path: &str) -> Result<PathBuf, PathConversionError>;
    pub fn normalize_for_platform(path: &str, is_windows: bool) -> Result<String, PathConversionError>;
}
```

### PathFormat 枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathFormat {
    Unix,
    Windows,
    Unknown,
}
```
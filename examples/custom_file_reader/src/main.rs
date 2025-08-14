use lingo::providers::{FileReader, LingoFileProviderGeneric, file_provider::FileFormat};
use lingo::LingoError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 自定义的内存文件读取器
/// 这个实现展示了如何创建一个完全自定义的文件读取策略
#[derive(Debug, Clone)]
pub struct MemoryFileReader {
    /// 内存中的文件存储，使用 Arc<Mutex<>> 来支持多线程访问
    files: Arc<Mutex<HashMap<String, String>>>,
}

impl MemoryFileReader {
    /// 创建一个新的内存文件读取器
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 向内存文件系统添加文件
    pub fn add_file<P: AsRef<Path>>(&self, path: P, content: String) -> Result<(), LingoError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let mut files = self.files.lock().map_err(|_| {
            LingoError::Io {
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to acquire lock on memory files",
                ),
                path: std::path::PathBuf::new(),
            }
        })?;
        files.insert(path_str, content);
        Ok(())
    }

    /// 从内存文件系统移除文件
    pub fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LingoError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let mut files = self.files.lock().map_err(|_| {
            LingoError::Io {
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to acquire lock on memory files",
                ),
                path: std::path::PathBuf::new(),
            }
        })?;
        files.remove(&path_str);
        Ok(())
    }

    /// 列出所有文件
    pub fn list_files(&self) -> Result<Vec<String>, LingoError> {
        let files = self.files.lock().map_err(|_| {
            LingoError::Io {
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to acquire lock on memory files",
                ),
                path: std::path::PathBuf::new(),
            }
        })?;
        Ok(files.keys().cloned().collect())
    }
}

/// 实现 FileReader trait
impl FileReader for MemoryFileReader {
    fn read_content(&self, path: &Path) -> Result<String, LingoError> {
        let path_str = path.to_string_lossy().to_string();
        let files = self.files.lock().map_err(|_| {
            LingoError::Io {
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to acquire lock on memory files",
                ),
                path: path.to_path_buf(),
            }
        })?;
        
        files.get(&path_str).cloned().ok_or_else(|| {
            LingoError::Io {
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File not found in memory: {}", path_str),
                ),
                path: path.to_path_buf(),
            }
        })
    }

    fn exists(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_string();
        if let Ok(files) = self.files.lock() {
            files.contains_key(&path_str)
        } else {
            false
        }
    }
}

/// 示例配置结构体
#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    app: AppInfo,
    database: DatabaseConfig,
    features: Features,
    cache: CacheConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppInfo {
    name: String,
    version: String,
    debug: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Features {
    enabled: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheConfig {
    ttl: u64,
    max_size: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Lingo Custom File Reader Example");
    println!("=====================================\n");

    // 创建内存文件读取器
    let memory_reader = MemoryFileReader::new();
    
    // 模拟配置文件内容
    let config_content = r#"[app]
name = "Memory Config Example"
version = "2.0.0"
debug = false

[database]
host = "memory-db.example.com"
port = 3306
username = "memory_user"
password = "secure_password"
database = "memory_db"

[features]
enabled = ["memory_cache", "fast_lookup", "compression"]

[cache]
ttl = 7200
max_size = 2048
"#;

    // 添加配置文件到内存文件系统
    memory_reader.add_file("config.toml", config_content.to_string())?;
    
    // 添加一些额外的配置文件
    memory_reader.add_file("database.toml", r#"
[connection]
pool_size = 10
timeout = 30
"#.to_string())?;
    
    memory_reader.add_file("logging.json", r#"{
  "level": "info",
  "format": "json",
  "output": "stdout"
}"#.to_string())?;

    println!("📁 Files in memory file system:");
    for file in memory_reader.list_files()? {
        println!("   - {}", file);
    }
    println!();

    // 创建使用自定义文件读取器的 Lingo 提供者
    let provider = LingoFileProviderGeneric::new(
        std::path::Path::new("config.toml"),
        FileFormat::Toml,
        true, // is_required
        10,   // max_parse_depth
        memory_reader.clone(),
    );

    println!("🔧 Loading configuration using custom memory file reader...");
    
    // 使用figment读取并解析配置
    use figment::Figment;
    let config: AppConfig = Figment::new()
        .merge(provider)
        .extract()?;
    
    println!("✅ Configuration loaded successfully!");
    println!("📋 Configuration details:");
    println!("   App Name: {}", config.app.name);
    println!("   App Version: {}", config.app.version);
    println!("   Debug Mode: {}", config.app.debug);
    println!("   Database Host: {}", config.database.host);
    println!("   Database Port: {}", config.database.port);
    println!("   Enabled Features: {:?}", config.features.enabled);
    println!("   Cache TTL: {} seconds", config.cache.ttl);
    println!("   Cache Max Size: {} MB", config.cache.max_size);
    println!();

    // 演示文件存在性检查
    println!("🔍 File existence checks:");
    println!("   config.toml exists: {}", memory_reader.exists(std::path::Path::new("config.toml")));
    println!("   database.toml exists: {}", memory_reader.exists(std::path::Path::new("database.toml")));
    println!("   nonexistent.toml exists: {}", memory_reader.exists(std::path::Path::new("nonexistent.toml")));
    println!();

    // 演示错误处理
    println!("❌ Testing error handling:");
    let nonexistent_provider = LingoFileProviderGeneric::new(
        std::path::Path::new("nonexistent.toml"),
        FileFormat::Toml,
        true, // is_required
        10,   // max_parse_depth
        memory_reader.clone(),
    );
    
    match Figment::new().merge(nonexistent_provider).extract::<AppConfig>() {
        Ok(_) => println!("   Unexpected success!"),
        Err(e) => println!("   Expected error: {}", e),
    }
    println!();

    // 演示动态文件管理
    println!("🔄 Dynamic file management:");
    memory_reader.add_file("runtime.toml", r#"
[runtime]
mode = "production"
workers = 4
"#.to_string())?;
    
    println!("   Added runtime.toml");
    println!("   runtime.toml exists: {}", memory_reader.exists(std::path::Path::new("runtime.toml")));
    
    memory_reader.remove_file("runtime.toml")?;
    println!("   Removed runtime.toml");
    println!("   runtime.toml exists: {}", memory_reader.exists(std::path::Path::new("runtime.toml")));
    println!();

    println!("🎉 Custom File Reader example completed successfully!");
    println!("\n💡 Key takeaways:");
    println!("   - Custom FileReader implementations provide flexibility");
    println!("   - Memory-based readers are perfect for testing");
    println!("   - Error handling works seamlessly with custom readers");
    println!("   - Dynamic file management enables runtime configuration");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingo::providers::LingoFileProviderGeneric;
    use lingo::providers::file_provider::FileFormat;

    #[test]
    fn test_memory_file_reader_basic_operations() {
        let reader = MemoryFileReader::new();
        
        // Test file doesn't exist initially
        assert!(!reader.exists(std::path::Path::new("test.txt")));
        
        // Add file
        reader.add_file("test.txt", "Hello, World!".to_string()).unwrap();
        assert!(reader.exists(std::path::Path::new("test.txt")));
        
        // Read file
        let content = reader.read_content(std::path::Path::new("test.txt")).unwrap();
        assert_eq!(content, "Hello, World!");
        
        // Remove file
        reader.remove_file("test.txt").unwrap();
        assert!(!reader.exists(std::path::Path::new("test.txt")));
    }

    #[test]
    fn test_memory_file_reader_with_lingo_provider() {
        let reader = MemoryFileReader::new();
        
        // Add a TOML config file
        let config_content = r#"
[app]
name = "Test App"
version = "1.0.0"
"#;
        reader.add_file("config.toml", config_content.to_string()).unwrap();
        
        // Create provider with custom reader
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("config.toml"),
            FileFormat::Toml,
            true, // is_required
            10,   // max_parse_depth
            reader,
        );
        
        // Test parsing
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct AppConfig {
            name: String,
            version: String,
        }
        
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct TestConfig {
            app: AppConfig,
        }
        
        use figment::Figment;
        let config: TestConfig = Figment::new().merge(provider).extract().unwrap();
        assert_eq!(config.app.name, "Test App");
        assert_eq!(config.app.version, "1.0.0");
    }

    #[test]
    fn test_memory_file_reader_error_handling() {
        let reader = MemoryFileReader::new();
        
        // Test reading non-existent file
        let result = reader.read_content(std::path::Path::new("nonexistent.txt"));
        assert!(result.is_err());
        
        // Test with provider
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("nonexistent.toml"),
            FileFormat::Toml,
            true, // is_required
            10,   // max_parse_depth
            reader,
        );
        
        #[derive(serde::Deserialize)]
        struct TestConfig {
            name: String,
        }
        
        use figment::Figment;
        let result = Figment::new().merge(provider).extract::<TestConfig>();
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_file_reader_list_files() {
        let reader = MemoryFileReader::new();
        
        // Initially empty
        let files = reader.list_files().unwrap();
        assert!(files.is_empty());
        
        // Add some files
        reader.add_file("file1.txt", "content1".to_string()).unwrap();
        reader.add_file("file2.txt", "content2".to_string()).unwrap();
        
        let files = reader.list_files().unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file1.txt".to_string()));
        assert!(files.contains(&"file2.txt".to_string()));
    }
}
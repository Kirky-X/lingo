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
    use figment::Figment;

    #[test]
    fn test_memory_file_reader_basic_ops() {
        let reader = MemoryFileReader::new();
        assert!(!reader.exists(std::path::Path::new("a.txt")));

        // MemoryFileReader implements FileReader with add/remove operations via add_file/remove_file
        reader.add_file("a.txt", "hello".to_string()).unwrap();
        assert!(reader.exists(std::path::Path::new("a.txt")));

        let content = reader.read_content(std::path::Path::new("a.txt")).unwrap();
        assert_eq!(content, "hello");

        reader.remove_file("a.txt").unwrap();
        assert!(!reader.exists(std::path::Path::new("a.txt")));
    }

    #[test]
    fn test_config_extract_from_memory_reader() {
        let memory_reader = MemoryFileReader::new();
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("config.toml"),
            FileFormat::Toml,
            true,
            10,
            memory_reader.clone(),
        );

        // write a minimal config to memory
        memory_reader
            .add_file(
                "config.toml",
                r#"
                [app]
                name = "MyApp"
                version = "1.0.0"
                debug = false

                [database]
                host = "localhost"
                port = 5432
                username = "user"
                password = "pass"
                database = "mydb"

                [features]
                enabled = ["a", "b"]

                [cache]
                ttl = 60
                max_size = 1024
                "#.to_string(),
            )
            .unwrap();

        let config: AppConfig = Figment::new()
            .merge(provider)
            .extract()
            .expect("Should be able to extract AppConfig from memory reader");

        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
    }

    #[test]
    fn test_memory_reader_concurrent_safety() {
        use std::thread;
        use std::sync::Arc;

        let reader = Arc::new(MemoryFileReader::new());
        let reader1 = Arc::clone(&reader);
        let reader2 = Arc::clone(&reader);

        // 并发写入测试
        let handle1 = thread::spawn(move || {
            for i in 0..50 {
                reader1.add_file(format!("file_{}.txt", i), format!("content_{}", i)).unwrap();
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 50..100 {
                reader2.add_file(format!("file_{}.txt", i), format!("content_{}", i)).unwrap();
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        // 验证所有文件都成功写入
        for i in 0..100 {
            assert!(reader.exists(std::path::Path::new(&format!("file_{}.txt", i))));
        }

        // 验证文件数量
        let files = reader.list_files().unwrap();
        assert_eq!(files.len(), 100);
    }

    #[test]
    fn test_memory_reader_file_not_found_error() {
        let reader = MemoryFileReader::new();
        
        // 读取不存在的文件应该返回错误
        let result = reader.read_content(std::path::Path::new("nonexistent.toml"));
        assert!(result.is_err());
        
        match result.unwrap_err() {
            LingoError::Io { source, path } => {
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
                assert_eq!(path, std::path::PathBuf::from("nonexistent.toml"));
            },
            _ => panic!("Expected LingoError::Io with NotFound error kind"),
        }
    }

    #[test]
    fn test_memory_reader_empty_file_handling() {
        let reader = MemoryFileReader::new();
        
        // 添加空文件
        reader.add_file("empty.toml", "".to_string()).unwrap();
        assert!(reader.exists(std::path::Path::new("empty.toml")));
        
        let content = reader.read_content(std::path::Path::new("empty.toml")).unwrap();
        assert_eq!(content, "");
        
        // 尝试解析空配置文件应该失败
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("empty.toml"),
            FileFormat::Toml,
            true,
            10,
            reader.clone(),
        );
        
        let result = Figment::new().merge(provider).extract::<AppConfig>();
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_reader_invalid_toml_handling() {
        let reader = MemoryFileReader::new();
        
        // 添加无效的 TOML 文件
        let invalid_toml = r#"
        [app
        name = "Invalid TOML"
        debug = 
        "#;
        
        reader.add_file("invalid.toml", invalid_toml.to_string()).unwrap();
        
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("invalid.toml"),
            FileFormat::Toml,
            true,
            10,
            reader.clone(),
        );
        
        let result = Figment::new().merge(provider).extract::<AppConfig>();
        assert!(result.is_err(), "Invalid TOML should fail to parse");
    }

    #[test]
    fn test_memory_reader_partial_config_missing_sections() {
        let reader = MemoryFileReader::new();
        
        // 只包含部分必需字段的配置
        let partial_config = r#"
        [app]
        name = "Partial App"
        version = "1.0.0"
        debug = false
        "#;
        
        reader.add_file("partial.toml", partial_config.to_string()).unwrap();
        
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("partial.toml"),
            FileFormat::Toml,
            true,
            10,
            reader.clone(),
        );
        
        let result = Figment::new().merge(provider).extract::<AppConfig>();
        assert!(result.is_err(), "Partial config missing required sections should fail");
    }

    #[test]
    fn test_memory_reader_boundary_values() {
        let reader = MemoryFileReader::new();
        
        // 使用边界值的配置
        let boundary_config = r#"
        [app]
        name = ""
        version = ""
        debug = true

        [database]
        host = ""
        port = 0
        username = ""
        password = ""
        database = ""

        [features]
        enabled = []

        [cache]
        ttl = 0
        max_size = 0
        "#;
        
        reader.add_file("boundary.toml", boundary_config.to_string()).unwrap();
        
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("boundary.toml"),
            FileFormat::Toml,
            true,
            10,
            reader.clone(),
        );
        
        let config: AppConfig = Figment::new()
            .merge(provider)
            .extract()
            .expect("Boundary values should be parseable");
        
        assert_eq!(config.app.name, "");
        assert_eq!(config.database.port, 0);
        assert_eq!(config.features.enabled.len(), 0);
        assert_eq!(config.cache.ttl, 0);
        assert_eq!(config.cache.max_size, 0);
    }

    #[test]
    fn test_memory_reader_max_values() {
        let reader = MemoryFileReader::new();
        
        // 使用最大合法值的配置
        let max_config = r#"
        [app]
        name = "Max Values Test"
        version = "999.999.999"
        debug = true

        [database]
        host = "very-long-hostname-that-is-still-valid.example.com"
        port = 65535
        username = "very_long_username_that_might_be_used_in_some_systems"
        password = "very_long_password_with_special_chars_!@#$%^&*()_+-=[]{}|;':\",./<>?"
        database = "very_long_database_name_that_exceeds_normal_expectations"

        [features]
        enabled = ["feature1", "feature2", "feature3", "feature4", "feature5"]

        [cache]
        ttl = 9223372036854775807
        max_size = 9223372036854775807
        "#;
        
        reader.add_file("max.toml", max_config.to_string()).unwrap();
        
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("max.toml"),
            FileFormat::Toml,
            true,
            10,
            reader.clone(),
        );
        
        let config: AppConfig = Figment::new()
            .merge(provider)
            .extract()
            .expect("Max values should be parseable");
        
        assert_eq!(config.database.port, 65535);
        assert_eq!(config.cache.ttl, i64::MAX as u64);
        assert_eq!(config.cache.max_size, i64::MAX as u64);
        assert_eq!(config.features.enabled.len(), 5);
    }

    #[test]
    fn test_memory_reader_unicode_content() {
        let reader = MemoryFileReader::new();
        
        // 包含Unicode字符的配置
        let unicode_config = r#"
        [app]
        name = "测试应用程序 🚀"
        version = "1.0.0-α"
        debug = false

        [database]
        host = "数据库.example.com"
        port = 3306
        username = "用户名"
        password = "密码123"
        database = "数据库名称"

        [features]
        enabled = ["功能1", "功能2", "测试功能"]

        [cache]
        ttl = 3600
        max_size = 1024
        "#;
        
        reader.add_file("unicode.toml", unicode_config.to_string()).unwrap();
        
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("unicode.toml"),
            FileFormat::Toml,
            true,
            10,
            reader.clone(),
        );
        
        let config: AppConfig = Figment::new()
            .merge(provider)
            .extract()
            .expect("Unicode content should be parseable");
        
        assert_eq!(config.app.name, "测试应用程序 🚀");
        assert_eq!(config.database.host, "数据库.example.com");
        assert!(config.features.enabled.contains(&"功能1".to_string()));
    }

    #[test]
    fn test_memory_reader_dynamic_file_operations() {
        let reader = MemoryFileReader::new();
        
        // 测试动态添加、修改、删除文件
        let initial_config = r#"
        [app]
        name = "Initial Config"
        version = "1.0.0"
        debug = false

        [database]
        host = "localhost"
        port = 5432
        username = "user"
        password = "pass"
        database = "mydb"

        [features]
        enabled = ["initial"]

        [cache]
        ttl = 60
        max_size = 1024
        "#;
        
        reader.add_file("dynamic.toml", initial_config.to_string()).unwrap();
        
        // 验证初始配置
        let provider = LingoFileProviderGeneric::new(
            std::path::Path::new("dynamic.toml"),
            FileFormat::Toml,
            true,
            10,
            reader.clone(),
        );
        
        let config: AppConfig = Figment::new()
            .merge(provider)
            .extract()
            .expect("Initial config should be parseable");
        
        assert_eq!(config.app.name, "Initial Config");
        
        // 修改配置文件
        let updated_config = r#"
        [app]
        name = "Updated Config"
        version = "2.0.0"
        debug = true

        [database]
        host = "updated-host"
        port = 5433
        username = "new_user"
        password = "new_pass"
        database = "new_db"

        [features]
        enabled = ["updated", "new_feature"]

        [cache]
        ttl = 120
        max_size = 2048
        "#;
        
        reader.add_file("dynamic.toml", updated_config.to_string()).unwrap();
        
        // 验证更新后的配置
        let updated_provider = LingoFileProviderGeneric::new(
            std::path::Path::new("dynamic.toml"),
            FileFormat::Toml,
            true,
            10,
            reader.clone(),
        );
        
        let updated_config: AppConfig = Figment::new()
            .merge(updated_provider)
            .extract()
            .expect("Updated config should be parseable");
        
        assert_eq!(updated_config.app.name, "Updated Config");
        assert_eq!(updated_config.app.version, "2.0.0");
        assert_eq!(updated_config.database.host, "updated-host");
        
        // 删除配置文件
        reader.remove_file("dynamic.toml").unwrap();
        assert!(!reader.exists(std::path::Path::new("dynamic.toml")));
        
        // 验证删除后的文件不存在
        let result = reader.read_content(std::path::Path::new("dynamic.toml"));
        assert!(result.is_err());
    }
}
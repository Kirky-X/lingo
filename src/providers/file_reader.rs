//! 文件读取器抽象层
//!
//! 提供文件读取的通用trait，允许用户自定义文件读取行为。

use std::path::Path;
use crate::error::LingoError;

/// 文件读取器trait
/// 
/// 定义了读取文件内容的基本能力，允许用户自定义文件读取行为。
/// 例如：从网络读取、从内存读取、从加密文件读取等。
pub trait FileReader: Send + Sync {
    /// 读取指定路径的文件内容
    /// 
    /// # 参数
    /// 
    /// * `path` - 文件路径
    /// 
    /// # 返回值
    /// 
    /// 返回文件内容字符串，如果读取失败则返回错误
    fn read_content(&self, path: &Path) -> Result<String, LingoError>;
    
    /// 检查文件是否存在
    /// 
    /// # 参数
    /// 
    /// * `path` - 文件路径
    /// 
    /// # 返回值
    /// 
    /// 如果文件存在返回true，否则返回false
    fn exists(&self, path: &Path) -> bool;
}

/// 标准文件系统读取器
/// 
/// 使用标准的文件系统API读取文件内容。
/// 这是默认的文件读取实现。
#[derive(Debug, Clone, Default)]
pub struct StandardFileReader;

impl StandardFileReader {
    /// 创建新的标准文件读取器实例
    pub fn new() -> Self {
        Self
    }
}

impl FileReader for StandardFileReader {
    fn read_content(&self, path: &Path) -> Result<String, LingoError> {
        std::fs::read_to_string(path)
            .map_err(|e| LingoError::FileReadError {
                path: path.to_string_lossy().to_string(),
                source: e,
            })
    }
    
    fn exists(&self, path: &Path) -> bool {
        path.exists() && path.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_standard_file_reader_read_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let content = "Hello, World!";
        
        fs::write(&file_path, content).unwrap();
        
        let reader = StandardFileReader::new();
        let result = reader.read_content(&file_path).unwrap();
        
        assert_eq!(result, content);
    }
    
    #[test]
    fn test_standard_file_reader_read_nonexistent_file() {
        let reader = StandardFileReader::new();
        let result = reader.read_content(Path::new("nonexistent.txt"));
        
        assert!(result.is_err());
        match result.unwrap_err() {
            LingoError::FileReadError { path, .. } => {
                assert_eq!(path, "nonexistent.txt");
            }
            _ => panic!("Expected FileReadError"),
        }
    }
    
    #[test]
    fn test_standard_file_reader_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        
        let reader = StandardFileReader::new();
        
        // 文件不存在
        assert!(!reader.exists(&file_path));
        
        // 创建文件
        fs::write(&file_path, "test").unwrap();
        
        // 文件存在
        assert!(reader.exists(&file_path));
    }
    
    #[test]
    fn test_standard_file_reader_exists_directory() {
        let dir = tempdir().unwrap();
        let reader = StandardFileReader::new();
        
        // 目录不应该被认为是文件
        assert!(!reader.exists(dir.path()));
    }
    
    // 测试自定义FileReader实现
    #[derive(Debug)]
    struct MockFileReader {
        content: String,
        should_exist: bool,
    }
    
    impl MockFileReader {
        fn new(content: String, should_exist: bool) -> Self {
            Self { content, should_exist }
        }
    }
    
    impl FileReader for MockFileReader {
        fn read_content(&self, _path: &Path) -> Result<String, LingoError> {
            if self.should_exist {
                Ok(self.content.clone())
            } else {
                Err(LingoError::FileReadError {
                    path: "mock_file.txt".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::NotFound, "Mock file not found"),
                })
            }
        }
        
        fn exists(&self, _path: &Path) -> bool {
            self.should_exist
        }
    }
    
    #[test]
    fn test_mock_file_reader() {
        let content = "Mock content";
        let reader = MockFileReader::new(content.to_string(), true);
        
        assert!(reader.exists(Path::new("any_path")));
        assert_eq!(reader.read_content(Path::new("any_path")).unwrap(), content);
        
        let reader = MockFileReader::new(String::new(), false);
        assert!(!reader.exists(Path::new("any_path")));
        assert!(reader.read_content(Path::new("any_path")).is_err());
    }
}
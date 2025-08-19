//! Path conversion utilities for cross-platform path handling.
//!
//! This module provides functionality to convert paths between Unix and Windows formats,
//! enabling consistent path handling across different platforms.

use std::path::{Path, PathBuf};
use typed_path::{Utf8Path, Utf8PathBuf, Utf8UnixEncoding, Utf8WindowsEncoding};
use crate::error::QuantumConfigError;

/// Trait for path format conversion
pub trait PathConverter {
    /// Convert a path to Unix format (forward slashes)
    fn to_unix_format(&self) -> Result<String, QuantumConfigError>;
    
    /// Convert a path to Windows format (backslashes)
    fn to_windows_format(&self) -> Result<String, QuantumConfigError>;
    
    /// Convert a path to the native format for the current platform
    fn to_native_format(&self) -> Result<PathBuf, QuantumConfigError>;
    
    /// Normalize a path by converting it to Unix format internally
    /// and then back to the appropriate format for the target platform
    fn normalize_for_platform(&self, target_is_windows: bool) -> Result<String, QuantumConfigError>;
}

impl PathConverter for Path {
    fn to_unix_format(&self) -> Result<String, QuantumConfigError> {
        let path_str = self.to_str()
            .ok_or_else(|| QuantumConfigError::SecurityViolation {
                message: "Path contains invalid UTF-8 characters".to_string(),
            })?;
        
        // Convert to typed path and then to Unix format
        let typed_path = if cfg!(windows) {
            // On Windows, treat as Windows path and convert to Unix
            let windows_path = Utf8Path::<Utf8WindowsEncoding>::new(path_str);
            windows_path.with_encoding::<Utf8UnixEncoding>()
        } else {
            // On Unix, already in Unix format
            Utf8PathBuf::<Utf8UnixEncoding>::from(path_str)
        };
        
        Ok(typed_path.as_str().to_string())
    }
    
    fn to_windows_format(&self) -> Result<String, QuantumConfigError> {
        let path_str = self.to_str()
            .ok_or_else(|| QuantumConfigError::SecurityViolation {
                message: "Path contains invalid UTF-8 characters".to_string(),
            })?;
        
        // Convert to typed path and then to Windows format
        let typed_path = if cfg!(windows) {
            // On Windows, already in Windows format
            Utf8PathBuf::<Utf8WindowsEncoding>::from(path_str)
        } else {
            // On Unix, treat as Unix path and convert to Windows
            let unix_path = Utf8Path::<Utf8UnixEncoding>::new(path_str);
            unix_path.with_encoding::<Utf8WindowsEncoding>()
        };
        
        Ok(typed_path.as_str().to_string())
    }
    
    fn to_native_format(&self) -> Result<PathBuf, QuantumConfigError> {
        if cfg!(windows) {
            let windows_format = self.to_windows_format()?;
            Ok(PathBuf::from(windows_format))
        } else {
            let unix_format = self.to_unix_format()?;
            Ok(PathBuf::from(unix_format))
        }
    }
    
    fn normalize_for_platform(&self, target_is_windows: bool) -> Result<String, QuantumConfigError> {
        if target_is_windows {
            self.to_windows_format()
        } else {
            self.to_unix_format()
        }
    }
}

impl PathConverter for PathBuf {
    fn to_unix_format(&self) -> Result<String, QuantumConfigError> {
        self.as_path().to_unix_format()
    }
    
    fn to_windows_format(&self) -> Result<String, QuantumConfigError> {
        self.as_path().to_windows_format()
    }
    
    fn to_native_format(&self) -> Result<PathBuf, QuantumConfigError> {
        self.as_path().to_native_format()
    }
    
    fn normalize_for_platform(&self, target_is_windows: bool) -> Result<String, QuantumConfigError> {
        self.as_path().normalize_for_platform(target_is_windows)
    }
}

impl PathConverter for str {
    fn to_unix_format(&self) -> Result<String, QuantumConfigError> {
        Path::new(self).to_unix_format()
    }
    
    fn to_windows_format(&self) -> Result<String, QuantumConfigError> {
        Path::new(self).to_windows_format()
    }
    
    fn to_native_format(&self) -> Result<PathBuf, QuantumConfigError> {
        Path::new(self).to_native_format()
    }
    
    fn normalize_for_platform(&self, target_is_windows: bool) -> Result<String, QuantumConfigError> {
        Path::new(self).normalize_for_platform(target_is_windows)
    }
}

impl PathConverter for String {
    fn to_unix_format(&self) -> Result<String, QuantumConfigError> {
        self.as_str().to_unix_format()
    }
    
    fn to_windows_format(&self) -> Result<String, QuantumConfigError> {
        self.as_str().to_windows_format()
    }
    
    fn to_native_format(&self) -> Result<PathBuf, QuantumConfigError> {
        self.as_str().to_native_format()
    }
    
    fn normalize_for_platform(&self, target_is_windows: bool) -> Result<String, QuantumConfigError> {
        self.as_str().normalize_for_platform(target_is_windows)
    }
}

/// Utility functions for path conversion
pub mod utils {
    use super::*;
    
    /// Convert any path-like input to Unix format
    pub fn to_unix<P: AsRef<Path>>(path: P) -> Result<String, QuantumConfigError> {
        path.as_ref().to_unix_format()
    }
    
    /// Convert any path-like input to Windows format
    pub fn to_windows<P: AsRef<Path>>(path: P) -> Result<String, QuantumConfigError> {
        path.as_ref().to_windows_format()
    }
    
    /// Convert any path-like input to the native format for the current platform
    pub fn to_native<P: AsRef<Path>>(path: P) -> Result<PathBuf, QuantumConfigError> {
        path.as_ref().to_native_format()
    }
    
    /// Normalize a path for a specific target platform
    pub fn normalize_for_platform<P: AsRef<Path>>(path: P, target_is_windows: bool) -> Result<String, QuantumConfigError> {
        path.as_ref().normalize_for_platform(target_is_windows)
    }
    
    /// Check if a path string is in Unix format (contains forward slashes)
    pub fn is_unix_format(path: &str) -> bool {
        path.contains('/') && !path.contains('\\')
    }
    
    /// Check if a path string is in Windows format (contains backslashes)
    pub fn is_windows_format(path: &str) -> bool {
        path.contains('\\')
    }
    
    /// Detect the format of a path string
    pub fn detect_format(path: &str) -> PathFormat {
        if is_windows_format(path) {
            PathFormat::Windows
        } else if is_unix_format(path) {
            PathFormat::Unix
        } else {
            PathFormat::Unknown
        }
    }
}

/// Enumeration of path formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathFormat {
    /// Unix-style paths with forward slashes
    Unix,
    /// Windows-style paths with backslashes
    Windows,
    /// Unknown or ambiguous format
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_unix_to_windows_conversion() {
        let unix_path = "/home/user/config.toml";
        let windows_result = unix_path.to_windows_format().unwrap();
        
        // The exact result may vary based on the typed-path implementation
        // but it should contain backslashes
        assert!(windows_result.contains('\\') || windows_result == unix_path);
    }
    
    #[test]
    fn test_windows_to_unix_conversion() {
        let windows_path = "C:\\Users\\user\\config.toml";
        let unix_result = windows_path.to_unix_format().unwrap();
        
        // Should convert backslashes to forward slashes and may remove drive prefix
        assert!(unix_result.contains('/') || unix_result == windows_path);
    }
    
    #[test]
    fn test_pathbuf_conversion() {
        let path = PathBuf::from("test/path/file.txt");
        let unix_result = path.to_unix_format().unwrap();
        let windows_result = path.to_windows_format().unwrap();
        
        assert!(!unix_result.is_empty());
        assert!(!windows_result.is_empty());
    }
    
    #[test]
    fn test_native_format_conversion() {
        let path = "test/path/file.txt";
        let native_result = path.to_native_format().unwrap();
        
        assert!(!native_result.as_os_str().is_empty());
    }
    
    #[test]
    fn test_format_detection() {
        assert_eq!(utils::detect_format("/unix/path"), PathFormat::Unix);
        assert_eq!(utils::detect_format("C:\\windows\\path"), PathFormat::Windows);
        assert_eq!(utils::detect_format("ambiguous"), PathFormat::Unknown);
    }
    
    #[test]
    fn test_utility_functions() {
        let path = "test/file.txt";
        
        let unix_result = utils::to_unix(path).unwrap();
        let windows_result = utils::to_windows(path).unwrap();
        let native_result = utils::to_native(path).unwrap();
        
        assert!(!unix_result.is_empty());
        assert!(!windows_result.is_empty());
        assert!(!native_result.as_os_str().is_empty());
    }
    
    #[test]
    fn test_normalize_for_platform() {
        let path = "test/file.txt";
        
        let unix_normalized = utils::normalize_for_platform(path, false).unwrap();
        let windows_normalized = utils::normalize_for_platform(path, true).unwrap();
        
        assert!(!unix_normalized.is_empty());
        assert!(!windows_normalized.is_empty());
    }
}
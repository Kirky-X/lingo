//! Path Conversion Example
//!
//! This example demonstrates how to use the path conversion functionality
//! in Quantum Config to handle cross-platform path formats.

use quantum_config::{
    Config, Deserialize, Serialize,
    path_conversion::{PathConverter, utils},
};
use std::path::PathBuf;

#[derive(Config, Default, Deserialize, Serialize, Debug)]
struct AppConfig {
    /// Application name
    pub app_name: String,
    /// Configuration file paths (can be in any format)
    pub config_paths: Vec<String>,
    /// Log file path
    pub log_path: String,
    /// Data directory path
    pub data_dir: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Quantum Config Path Conversion Example ===");
    
    // Example paths in different formats
    let unix_path = "/home/user/config/app.toml";
    let windows_path = "C:\\Users\\user\\config\\app.toml";
    let relative_path = "./config/app.toml";
    
    println!("\n1. Path Format Detection:");
    println!("Unix path: {} -> {:?}", unix_path, utils::detect_format(unix_path));
    println!("Windows path: {} -> {:?}", windows_path, utils::detect_format(windows_path));
    println!("Relative path: {} -> {:?}", relative_path, utils::detect_format(relative_path));
    
    println!("\n2. Path Conversion Examples:");
    
    // Convert Unix path to different formats
    println!("\nUnix path conversions:");
    println!("Original: {}", unix_path);
    println!("To Unix: {}", unix_path.to_unix_format()?);
    println!("To Windows: {}", unix_path.to_windows_format()?);
    println!("To Native: {:?}", unix_path.to_native_format()?);
    
    // Convert Windows path to different formats
    println!("\nWindows path conversions:");
    println!("Original: {}", windows_path);
    println!("To Unix: {}", windows_path.to_unix_format()?);
    println!("To Windows: {}", windows_path.to_windows_format()?);
    println!("To Native: {:?}", windows_path.to_native_format()?);
    
    // Convert relative path to different formats
    println!("\nRelative path conversions:");
    println!("Original: {}", relative_path);
    println!("To Unix: {}", relative_path.to_unix_format()?);
    println!("To Windows: {}", relative_path.to_windows_format()?);
    println!("To Native: {:?}", relative_path.to_native_format()?);
    
    println!("\n3. Utility Functions:");
    
    // Using utility functions
    let test_path = "test/config/file.toml";
    println!("\nUsing utility functions with: {}", test_path);
    println!("utils::to_unix: {}", utils::to_unix(test_path)?);
    println!("utils::to_windows: {}", utils::to_windows(test_path)?);
    println!("utils::to_native: {:?}", utils::to_native(test_path)?);
    
    // Platform-specific normalization
    println!("\nPlatform normalization:");
    println!("For Unix: {}", utils::normalize_for_platform(test_path, false)?);
    println!("For Windows: {}", utils::normalize_for_platform(test_path, true)?);
    
    println!("\n4. Working with PathBuf:");
    
    let path_buf = PathBuf::from("data/logs/app.log");
    println!("PathBuf: {:?}", path_buf);
    println!("To Unix: {}", path_buf.to_unix_format()?);
    println!("To Windows: {}", path_buf.to_windows_format()?);
    println!("To Native: {:?}", path_buf.to_native_format()?);
    
    println!("\n5. Configuration with Path Conversion:");
    
    // Create a sample configuration
    let mut config = AppConfig {
        app_name: "path_conversion_example".to_string(),
        config_paths: vec![
            "/etc/myapp/config.toml".to_string(),
            "C:\\ProgramData\\MyApp\\config.toml".to_string(),
            "./local_config.toml".to_string(),
        ],
        log_path: "/var/log/myapp.log".to_string(),
        data_dir: "C:\\Users\\user\\AppData\\Local\\MyApp".to_string(),
    };
    
    println!("Original configuration:");
    println!("{:#?}", config);
    
    // Convert all paths to Unix format for internal processing
    println!("\nConverting paths to Unix format:");
    let mut unix_config_paths = Vec::new();
    for (i, path) in config.config_paths.iter().enumerate() {
        let unix_path = path.to_unix_format()?;
        println!("Config path {}: {} -> {}", i + 1, path, unix_path);
        unix_config_paths.push(unix_path);
    }
    config.config_paths = unix_config_paths;
    
    config.log_path = config.log_path.to_unix_format()?;
    config.data_dir = config.data_dir.to_unix_format()?;
    
    println!("\nConfiguration with Unix paths:");
    println!("{:#?}", config);
    
    // Convert back to native format for actual use
    println!("\nConverting back to native format:");
    for (i, path) in config.config_paths.iter().enumerate() {
        let native_path = path.to_native_format()?;
        println!("Config path {}: {} -> {:?}", i + 1, path, native_path);
    }
    
    let native_log_path = config.log_path.to_native_format()?;
    let native_data_dir = config.data_dir.to_native_format()?;
    
    println!("Log path: {} -> {:?}", config.log_path, native_log_path);
    println!("Data dir: {} -> {:?}", config.data_dir, native_data_dir);
    
    println!("\n6. Format Detection and Validation:");
    
    let test_paths = vec![
        "/unix/style/path",
        "C:\\windows\\style\\path",
        "relative/path",
        "just_filename",
        "./current/dir/file",
        "../parent/dir/file",
    ];
    
    for path in test_paths {
        let format = utils::detect_format(path);
        let is_unix = utils::is_unix_format(path);
        let is_windows = utils::is_windows_format(path);
        
        println!("Path: {} | Format: {:?} | Unix: {} | Windows: {}", 
                 path, format, is_unix, is_windows);
    }
    
    println!("\n=== Path Conversion Example Complete ===");
    
    Ok(())
}
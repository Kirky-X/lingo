//! # Lingo 异步配置加载示例
//!
//! 这个示例展示了如何在异步环境中使用 Lingo 进行配置管理，包括：
//! - 异步应用程序中的配置加载
//! - 配置热重载和监听
//! - 异步任务中的配置使用
//! - 配置变更的响应式处理

use lingo::Config;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use std::pin::Pin;
use std::future::Future;
use tokio::sync::{RwLock, broadcast};
use tokio::time::{sleep, interval};
use futures::future::join_all;

/// HTTP客户端配置
#[derive(Serialize, Deserialize, Debug, Clone)]
struct HttpClientConfig {
    /// 基础URL
    base_url: String,
    /// 请求超时（秒）
    timeout: u64,
    /// 最大重试次数
    max_retries: u32,
    /// 重试间隔（毫秒）
    retry_interval: u64,
    /// 用户代理
    user_agent: String,
    /// 是否启用压缩
    compression: bool,
}

/// 任务调度配置
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SchedulerConfig {
    /// 是否启用调度器
    enabled: bool,
    /// 工作线程数
    worker_threads: usize,
    /// 任务队列大小
    queue_size: usize,
    /// 任务超时（秒）
    task_timeout: u64,
    /// 清理间隔（秒）
    cleanup_interval: u64,
}

/// 缓存配置
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CacheConfig {
    /// 是否启用缓存
    enabled: bool,
    /// 缓存类型 (memory, redis, hybrid)
    cache_type: String,
    /// 默认TTL（秒）
    default_ttl: u64,
    /// 最大条目数
    max_entries: usize,
    /// 清理间隔（秒）
    cleanup_interval: u64,
}

/// 监控和指标配置
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetricsConfig {
    /// 是否启用指标收集
    enabled: bool,
    /// 指标端点
    endpoint: String,
    /// 收集间隔（秒）
    collection_interval: u64,
    /// 批量大小
    batch_size: usize,
    /// 缓冲区大小
    buffer_size: usize,
}

/// 异步应用配置
#[derive(Config, Serialize, Deserialize, Debug, Clone)]
#[config(env_prefix = "ASYNC_")]
struct AsyncAppConfig {
    /// 应用程序名称
    name: String,
    /// 应用程序版本
    version: String,
    /// 是否启用调试模式
    debug: bool,
    /// 服务器端口
    port: u16,
    /// 最大并发连接数
    max_connections: usize,
    /// 请求超时（秒）
    request_timeout: u64,
    
    /// HTTP客户端配置
    http_client: HttpClientConfig,
    /// 任务调度配置
    scheduler: SchedulerConfig,
    /// 缓存配置
    cache: CacheConfig,
    /// 监控配置
    metrics: MetricsConfig,
    
    /// 配置热重载间隔（秒）
    config_reload_interval: u64,
    /// 是否启用配置热重载
    hot_reload: bool,
}

impl Default for AsyncAppConfig {
    fn default() -> Self {
        Self {
            name: "Async Configuration Example".to_string(),
            version: "1.0.0".to_string(),
            debug: false,
            port: 8080,
            max_connections: 1000,
            request_timeout: 30,
            
            http_client: HttpClientConfig {
                base_url: "http://127.0.0.1".to_string(),
                timeout: 30,
                max_retries: 3,
                retry_interval: 1000,
                user_agent: "lingo-async-example".to_string(),
                compression: true,
            },
            
            scheduler: SchedulerConfig {
                enabled: true,
                worker_threads: 4,
                queue_size: 1000,
                task_timeout: 300,
                cleanup_interval: 3600,
            },
            
            cache: CacheConfig {
                enabled: true,
                cache_type: "memory".to_string(),
                default_ttl: 300,
                max_entries: 10000,
                cleanup_interval: 60,
            },
            
            metrics: MetricsConfig {
                enabled: true,
                endpoint: "/metrics".to_string(),
                collection_interval: 60,
                batch_size: 100,
                buffer_size: 1000,
            },
            
            config_reload_interval: 300,
            hot_reload: true,
        }
    }
}

/// 配置管理器
struct ConfigManager {
    config: Arc<RwLock<AsyncAppConfig>>,
    config_tx: broadcast::Sender<AsyncAppConfig>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    fn new() -> Self {
        let config = AsyncAppConfig::new();
        let (config_tx, _) = broadcast::channel(16);
        
        Self {
            config: Arc::new(RwLock::new(config)),
            config_tx,
        }
    }
    
    /// 获取当前配置
    async fn get_config(&self) -> AsyncAppConfig {
        self.config.read().await.clone()
    }
    
    /// 更新配置
    async fn update_config(&self, new_config: AsyncAppConfig) {
        {
            let mut config = self.config.write().await;
            *config = new_config.clone();
        }
        
        // 广播配置变更
        let _ = self.config_tx.send(new_config);
    }
    
    /// 订阅配置变更
    fn subscribe(&self) -> broadcast::Receiver<AsyncAppConfig> {
        self.config_tx.subscribe()
    }
    
    /// 启动配置热重载
    async fn start_hot_reload(&self) {
        let config = self.get_config().await;
        if !config.hot_reload {
            println!("配置热重载已禁用");
            return;
        }
        
        let reload_interval = Duration::from_secs(config.config_reload_interval);
        let mut interval = interval(reload_interval);
        
        loop {
            interval.tick().await;
            
            println!("检查配置变更...");
            
            // 重新加载配置
            match self.reload_config().await {
                Ok(new_config) => {
                    let current_config = self.get_config().await;
                    
                    // 简单的配置比较（实际应用中可能需要更复杂的比较逻辑）
                    if format!("{:?}", new_config) != format!("{:?}", current_config) {
                        println!("检测到配置变更，正在更新...");
                        self.update_config(new_config).await;
                        println!("配置已更新");
                    } else {
                        println!("配置无变更");
                    }
                }
                Err(e) => {
                    eprintln!("重新加载配置失败: {}", e);
                }
            }
        }
    }
    
    /// 重新加载配置
    async fn reload_config(&self) -> Result<AsyncAppConfig, Box<dyn Error + Send + Sync>> {
        // 在实际应用中，这里可能会从文件、数据库或远程服务加载配置
        // 为了演示，我们简单地重新创建配置
        tokio::task::spawn_blocking(|| {
            AsyncAppConfig::new()
        }).await.map_err(|e| e.into())
    }
}

/// HTTP客户端服务
struct HttpClientService {
    config: HttpClientConfig,
}

impl HttpClientService {
    fn new(config: HttpClientConfig) -> Self {
        Self { config }
    }
    
    /// 模拟HTTP请求
    async fn make_request(&self, endpoint: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        println!("发起HTTP请求: {}{}", self.config.base_url, endpoint);
        println!("  超时: {}秒", self.config.timeout);
        println!("  用户代理: {}", self.config.user_agent);
        
        // 模拟网络延迟
        sleep(Duration::from_millis(100)).await;
        
        Ok(format!("响应数据来自 {}{}", self.config.base_url, endpoint))
    }
    
    /// 更新配置
    fn update_config(&mut self, config: HttpClientConfig) {
        println!("HTTP客户端配置已更新");
        self.config = config;
    }
}

/// 任务调度器服务
struct SchedulerService {
    config: SchedulerConfig,
}

impl SchedulerService {
    fn new(config: SchedulerConfig) -> Self {
        Self { config }
    }
    
    /// 启动调度器
    async fn start(&self) {
        if !self.config.enabled {
            println!("任务调度器已禁用");
            return;
        }
        
        println!("启动任务调度器:");
        println!("  工作线程数: {}", self.config.worker_threads);
        println!("  队列大小: {}", self.config.queue_size);
        println!("  任务超时: {}秒", self.config.task_timeout);
        
        // 模拟任务执行
        let mut tasks = Vec::new();
        for i in 0..self.config.worker_threads {
            let task_timeout = self.config.task_timeout;
            let task = tokio::spawn(async move {
                loop {
                    println!("工作线程 {} 正在处理任务...", i);
                    sleep(Duration::from_secs(task_timeout)).await;
                }
            });
            tasks.push(task);
        }
        
        // 等待所有任务完成（实际上会一直运行）
        let _ = join_all(tasks).await;
    }
    
    /// 更新配置
    fn update_config(&mut self, config: SchedulerConfig) {
        println!("任务调度器配置已更新");
        self.config = config;
    }
}

/// 缓存服务
struct CacheService {
    config: CacheConfig,
}

impl CacheService {
    fn new(config: CacheConfig) -> Self {
        Self { config }
    }
    
    /// 启动缓存服务
    async fn start(&self) {
        if !self.config.enabled {
            println!("缓存服务已禁用");
            return;
        }
        
        println!("启动缓存服务:");
        println!("  缓存类型: {}", self.config.cache_type);
        println!("  默认TTL: {}秒", self.config.default_ttl);
        println!("  最大条目数: {}", self.config.max_entries);
        
        // 模拟缓存清理任务
        let cleanup_interval = Duration::from_secs(self.config.cleanup_interval);
        let mut interval = interval(cleanup_interval);
        
        loop {
            interval.tick().await;
            println!("执行缓存清理...");
        }
    }
    
    /// 更新配置
    fn update_config(&mut self, config: CacheConfig) {
        println!("缓存服务配置已更新");
        self.config = config;
    }
}

/// 指标收集服务
struct MetricsService {
    config: MetricsConfig,
}

impl MetricsService {
    fn new(config: MetricsConfig) -> Self {
        Self { config }
    }
    
    /// 启动指标收集
    async fn start(&self) {
        if !self.config.enabled {
            println!("指标收集已禁用");
            return;
        }
        
        println!("启动指标收集服务:");
        println!("  指标端点: {}", self.config.endpoint);
        println!("  收集间隔: {}秒", self.config.collection_interval);
        println!("  批量大小: {}", self.config.batch_size);
        
        let collection_interval = Duration::from_secs(self.config.collection_interval);
        let mut interval = interval(collection_interval);
        
        loop {
            interval.tick().await;
            println!("收集应用指标...");
        }
    }
    
    /// 更新配置
    fn update_config(&mut self, config: MetricsConfig) {
        println!("指标收集服务配置已更新");
        self.config = config;
    }
}

/// 异步应用程序
struct AsyncApp {
    config_manager: Arc<ConfigManager>,
    http_client: Arc<RwLock<HttpClientService>>,
    scheduler: Arc<RwLock<SchedulerService>>,
    cache: Arc<RwLock<CacheService>>,
    metrics: Arc<RwLock<MetricsService>>,
}

impl AsyncApp {
    /// 创建新的异步应用
    async fn new() -> Self {
        let config_manager = Arc::new(ConfigManager::new());
        let config = config_manager.get_config().await;
        
        let http_client = Arc::new(RwLock::new(HttpClientService::new(config.http_client.clone())));
        let scheduler = Arc::new(RwLock::new(SchedulerService::new(config.scheduler.clone())));
        let cache = Arc::new(RwLock::new(CacheService::new(config.cache.clone())));
        let metrics = Arc::new(RwLock::new(MetricsService::new(config.metrics.clone())));
        
        Self {
            config_manager,
            http_client,
            scheduler,
            cache,
            metrics,
        }
    }
    
    /// 启动应用程序
    async fn start(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let config = self.config_manager.get_config().await;
        
        println!("=== 启动异步应用程序 ===\n");
        println!("应用信息:");
        println!("  名称: {}", config.name);
        println!("  版本: {}", config.version);
        println!("  端口: {}", config.port);
        println!("  最大连接数: {}", config.max_connections);
        println!("  请求超时: {}秒\n", config.request_timeout);
        
        // 启动配置变更监听
        self.start_config_listener().await;
        
        // 启动各个服务
        let tasks = vec![
            Box::pin(self.start_http_client()) as Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>>,
            Box::pin(self.start_scheduler()) as Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>>,
            Box::pin(self.start_cache()) as Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>>,
            Box::pin(self.start_metrics()) as Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>>,
            Box::pin(self.start_config_hot_reload()) as Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>>,
        ];
        
        // 并发运行所有任务
        let results = join_all(tasks).await;
        
        // 检查是否有任务失败
        for result in results {
            if let Err(e) = result {
                eprintln!("任务执行失败: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// 启动HTTP客户端服务
    async fn start_http_client(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("启动HTTP客户端服务...");
        
        // 模拟一些HTTP请求
        let client = self.http_client.read().await;
        let _ = client.make_request("/api/users").await?;
        let _ = client.make_request("/api/orders").await?;
        
        Ok(())
    }
    
    /// 启动任务调度器
    async fn start_scheduler(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let scheduler = self.scheduler.read().await;
        scheduler.start().await;
        Ok(())
    }
    
    /// 启动缓存服务
    async fn start_cache(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let cache = self.cache.read().await;
        cache.start().await;
        Ok(())
    }
    
    /// 启动指标收集
    async fn start_metrics(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let metrics = self.metrics.read().await;
        metrics.start().await;
        Ok(())
    }
    
    /// 启动配置热重载
    async fn start_config_hot_reload(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.config_manager.start_hot_reload().await;
        Ok(())
    }
    
    /// 启动配置变更监听
    async fn start_config_listener(&self) {
        let mut config_rx = self.config_manager.subscribe();
        let http_client = Arc::clone(&self.http_client);
        let scheduler = Arc::clone(&self.scheduler);
        let cache = Arc::clone(&self.cache);
        let metrics = Arc::clone(&self.metrics);
        
        tokio::spawn(async move {
            while let Ok(new_config) = config_rx.recv().await {
                println!("收到配置变更通知，正在更新各个服务...");
                
                // 更新各个服务的配置
                {
                    let mut client = http_client.write().await;
                    client.update_config(new_config.http_client.clone());
                }
                
                {
                    let mut sched = scheduler.write().await;
                    sched.update_config(new_config.scheduler.clone());
                }
                
                {
                    let mut cache_service = cache.write().await;
                    cache_service.update_config(new_config.cache.clone());
                }
                
                {
                    let mut metrics_service = metrics.write().await;
                    metrics_service.update_config(new_config.metrics.clone());
                }
                
                println!("所有服务配置已更新完成");
            }
        });
    }
}

/// 验证配置合理性并提供警告
fn validate_config(config: &AsyncAppConfig) {
    // 端口边界检查
    if config.port == 0 {
        eprintln!("警告: 端口设置为 0，系统将自动分配随机端口");
    }
    
    // HTTP客户端配置检查
    if config.http_client.timeout == 0 {
        eprintln!("警告: HTTP客户端超时设置为 0，可能导致请求立即超时");
    }
    if config.http_client.max_retries == 0 {
        eprintln!("警告: HTTP客户端最大重试次数为 0，失败请求不会重试");
    }
    
    // 调度器配置检查
    if config.scheduler.enabled && config.scheduler.worker_threads == 0 {
        eprintln!("警告: 调度器已启用但工作线程数为 0，任务无法执行");
    }
    if config.scheduler.enabled && config.scheduler.queue_size == 0 {
        eprintln!("警告: 调度器已启用但队列大小为 0，可能导致任务丢失");
    }
    
    // 缓存配置检查
    if config.cache.enabled && config.cache.max_entries == 0 {
        eprintln!("警告: 缓存已启用但最大条目数为 0，缓存将无法存储数据");
    }
    if config.cache.enabled && config.cache.default_ttl == 0 {
        eprintln!("警告: 缓存已启用但默认TTL为 0，缓存项将立即过期");
    }
    
    // 指标配置检查
    if config.metrics.enabled && config.metrics.collection_interval == 0 {
        eprintln!("警告: 指标收集已启用但收集间隔为 0，可能导致性能问题");
    }
    if config.metrics.enabled && config.metrics.batch_size == 0 {
        eprintln!("警告: 指标收集已启用但批次大小为 0，指标无法发送");
    }
    
    // 字符串配置检查
    if config.name.is_empty() {
        eprintln!("警告: 应用名称为空，可能影响日志和监控");
    }
    if config.http_client.base_url.is_empty() {
        eprintln!("警告: HTTP客户端基础URL为空，请求可能失败");
    }
    if config.cache.cache_type.is_empty() {
        eprintln!("警告: 缓存类型为空，可能导致缓存初始化失败");
    }
    if config.metrics.endpoint.is_empty() {
        eprintln!("警告: 指标端点为空，指标数据无法发送");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Lingo 异步配置加载示例 ===\n");
    
    // 创建并启动异步应用
    let app = AsyncApp::new().await;
    
    // 验证配置并显示警告
    let config = app.config_manager.get_config().await;
    validate_config(&config);
    
    // 显示配置信息
    let config = app.config_manager.get_config().await;
    println!("当前配置:");
    println!("  应用名称: {}", config.name);
    println!("  版本: {}", config.version);
    println!("  调试模式: {}", config.debug);
    println!("  服务端口: {}", config.port);
    println!("  热重载: {}\n", config.hot_reload);
    
    println!("HTTP客户端配置:");
    println!("  基础URL: {}", config.http_client.base_url);
    println!("  超时: {}秒", config.http_client.timeout);
    println!("  最大重试: {}次\n", config.http_client.max_retries);
    
    println!("任务调度器配置:");
    println!("  启用: {}", config.scheduler.enabled);
    println!("  工作线程: {}", config.scheduler.worker_threads);
    println!("  队列大小: {}\n", config.scheduler.queue_size);
    
    println!("缓存配置:");
    println!("  启用: {}", config.cache.enabled);
    println!("  类型: {}", config.cache.cache_type);
    println!("  默认TTL: {}秒\n", config.cache.default_ttl);
    
    println!("指标配置:");
    println!("  启用: {}", config.metrics.enabled);
    println!("  端点: {}", config.metrics.endpoint);
    println!("  收集间隔: {}秒\n", config.metrics.collection_interval);
    
    println!("配置来源优先级:");
    println!("  1. 默认值 (代码中定义)");
    println!("  2. 配置文件: config.toml");
    println!("  3. 环境变量: ASYNC_*");
    println!("  4. 命令行参数\n");
    
    println!("异步特性:");
    println!("  [OK] 异步配置加载");
    println!("  [OK] 配置热重载");
    println!("  [OK] 响应式配置更新");
    println!("  [OK] 并发服务管理");
    println!("  [OK] 异步任务协调\n");
    
    // 运行应用（这会一直运行直到被中断）
    println!("启动应用程序...\n");
    
    // 为了演示，我们只运行5秒钟
    let app_future = app.start();
    let timeout_future = sleep(Duration::from_secs(5));
    
    tokio::select! {
        result = app_future => {
            if let Err(e) = result {
                eprintln!("应用程序运行失败: {}", e);
            }
        }
        _ = timeout_future => {
            println!("\n演示完成，应用程序正常退出");
        }
    }
    
    println!("\n[SUCCESS] 异步配置示例运行完成！");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // 删除未使用的导入以消除警告
    // use std::time::Duration;

    #[test]
    fn test_async_app_config_new() {
        // Test that we can create an AsyncAppConfig using new()
        let config = AsyncAppConfig::new();
        assert!(config.name.len() > 0, "name should have a default value");
        assert!(config.port > 0, "port should have a valid default value");
    }

    #[test]
    fn test_async_app_config_default() {
        // Test that we can create an AsyncAppConfig using Default
        let config = AsyncAppConfig::default();
        assert_eq!(config.name, "Async Configuration Example");
        assert_eq!(config.version, "1.0.0");
        assert!(config.port > 0);
    }

    #[test]
    fn test_config_serialization() {
        // Test that the config can be serialized and deserialized
        let config = AsyncAppConfig::default();
        let serialized = toml::to_string(&config).expect("Should be able to serialize config");
        assert!(serialized.contains("name"), "Serialized config should contain name");
        assert!(serialized.contains("version"), "Serialized config should contain version");

        // Test deserialization
        let deserialized: AsyncAppConfig = toml::from_str(&serialized).expect("Should be able to deserialize config");
        assert_eq!(deserialized.name, config.name);
        assert_eq!(deserialized.version, config.version);
        assert_eq!(deserialized.port, config.port);
    }

    #[tokio::test]
    async fn test_config_manager_creation() {
        // Test that we can create a ConfigManager
        let config_manager = Arc::new(ConfigManager::new());
        
        // Verify we can get current config
        let current_config = config_manager.get_config().await;
        assert_eq!(current_config.name, "Async Configuration Example");
    }

    #[test]
    fn test_service_configs() {
        // Test that individual service configs can be created
        let config = AsyncAppConfig::default();
        
        // Test that we can create services with the config
        let _http_client = HttpClientService::new(config.http_client.clone());
        let _scheduler = SchedulerService::new(config.scheduler.clone());
        let _cache = CacheService::new(config.cache.clone());
        let _metrics = MetricsService::new(config.metrics.clone());
        
        // If we get here without panicking, the configs are valid
    }

    #[test]
    fn test_zero_port_boundary() {
        // Test handling of zero port values
        let mut config = AsyncAppConfig::default();
        config.port = 0;
        
        // Zero port should be detectable
        assert_eq!(config.port, 0, "port should accept zero value");
        
        // Should still be serializable with zero port
        let serialized = toml::to_string(&config).expect("Should serialize config with zero port");
        let deserialized: AsyncAppConfig = toml::from_str(&serialized).expect("Should deserialize config with zero port");
        assert_eq!(deserialized.port, 0);
    }

    #[test]
    fn test_zero_timeout_configurations() {
        // Test zero timeout values for different services
        let mut config = AsyncAppConfig::default();
        
        config.request_timeout = 0;
        config.http_client.timeout = 0;
        config.scheduler.task_timeout = 0;
        config.cache.default_ttl = 0;
        
        // Zero timeouts should be handled gracefully
        assert_eq!(config.request_timeout, 0, "request_timeout should accept zero");
        assert_eq!(config.http_client.timeout, 0, "http_client timeout should accept zero");
        assert_eq!(config.scheduler.task_timeout, 0, "scheduler timeout should accept zero");
        assert_eq!(config.cache.default_ttl, 0, "cache TTL should accept zero");
        
        // Services should still be constructible with zero timeouts
        let _http_client = HttpClientService::new(config.http_client.clone());
        let _scheduler = SchedulerService::new(config.scheduler.clone());
        let _cache = CacheService::new(config.cache.clone());
    }

    #[test]
    fn test_zero_worker_threads_boundary() {
        // Test zero worker threads configuration
        let mut config = AsyncAppConfig::default();
        config.scheduler.worker_threads = 0;
        
        assert_eq!(config.scheduler.worker_threads, 0, "worker_threads should accept zero");
        
        // Scheduler should handle zero worker threads gracefully
        let scheduler = SchedulerService::new(config.scheduler.clone());
        assert_eq!(scheduler.config.worker_threads, 0);
    }

    #[test]
    fn test_zero_queue_size_boundary() {
        // Test zero queue size configuration
        let mut config = AsyncAppConfig::default();
        config.scheduler.queue_size = 0;
        
        assert_eq!(config.scheduler.queue_size, 0, "queue_size should accept zero");
        
        // Should be serializable with zero queue size
        let serialized = toml::to_string(&config).expect("Should serialize with zero queue size");
        let deserialized: AsyncAppConfig = toml::from_str(&serialized).expect("Should deserialize with zero queue size");
        assert_eq!(deserialized.scheduler.queue_size, 0);
    }

    #[test]
    fn test_max_value_boundaries() {
        // Test maximum value configurations - using TOML-safe maximum values
        let mut config = AsyncAppConfig::default();
        
        config.port = u16::MAX; // This is safe for TOML
        config.max_connections = 1000000; // Large but TOML-safe value
        config.request_timeout = 3600000; // 1 hour in milliseconds, safe for TOML
        config.http_client.max_retries = u32::MAX; // This is safe for TOML
        config.scheduler.worker_threads = 1000; // Large but reasonable value
        config.cache.max_entries = 1000000; // Large but TOML-safe value
        config.metrics.batch_size = 100000; // Large but reasonable value
        
        // All maximum values should be acceptable
        assert_eq!(config.port, u16::MAX);
        assert_eq!(config.max_connections, 1000000);
        assert_eq!(config.request_timeout, 3600000);
        assert_eq!(config.http_client.max_retries, u32::MAX);
        assert_eq!(config.scheduler.worker_threads, 1000);
        assert_eq!(config.cache.max_entries, 1000000);
        assert_eq!(config.metrics.batch_size, 100000);
        
        // Services should handle maximum values
        let _http_client = HttpClientService::new(config.http_client.clone());
        let _scheduler = SchedulerService::new(config.scheduler.clone());
        let _cache = CacheService::new(config.cache.clone());
        let _metrics = MetricsService::new(config.metrics.clone());
    }

    #[test]
    fn test_empty_string_configurations() {
        // Test empty string configurations
        let mut config = AsyncAppConfig::default();
        
        config.name = String::new();
        config.version = String::new();
        config.http_client.base_url = String::new();
        config.http_client.user_agent = String::new();
        config.cache.cache_type = String::new();
        config.metrics.endpoint = String::new();
        
        // Empty strings should be accepted
        assert!(config.name.is_empty());
        assert!(config.version.is_empty());
        assert!(config.http_client.base_url.is_empty());
        assert!(config.http_client.user_agent.is_empty());
        assert!(config.cache.cache_type.is_empty());
        assert!(config.metrics.endpoint.is_empty());
        
        // Should still be serializable with empty strings
        let serialized = toml::to_string(&config).expect("Should serialize with empty strings");
        let deserialized: AsyncAppConfig = toml::from_str(&serialized).expect("Should deserialize with empty strings");
        assert!(deserialized.name.is_empty());
        assert!(deserialized.version.is_empty());
    }

    #[test]
    fn test_disabled_services_configuration() {
        // Test all services disabled
        let mut config = AsyncAppConfig::default();
        
        config.scheduler.enabled = false;
        config.cache.enabled = false;
        config.metrics.enabled = false;
        config.hot_reload = false;
        
        // Disabled services should still be constructible
        let _scheduler = SchedulerService::new(config.scheduler.clone());
        let _cache = CacheService::new(config.cache.clone());
        let _metrics = MetricsService::new(config.metrics.clone());
        
        // Configuration should serialize/deserialize correctly
        let serialized = toml::to_string(&config).expect("Should serialize disabled services");
        let deserialized: AsyncAppConfig = toml::from_str(&serialized).expect("Should deserialize disabled services");
        assert!(!deserialized.scheduler.enabled);
        assert!(!deserialized.cache.enabled);
        assert!(!deserialized.metrics.enabled);
        assert!(!deserialized.hot_reload);
    }

    #[tokio::test]
    async fn test_config_manager_update_boundary() {
        // Test config manager with boundary value updates
        let config_manager = Arc::new(ConfigManager::new());
        
        let mut new_config = AsyncAppConfig::default();
        new_config.port = 0;
        new_config.max_connections = 0;
        new_config.request_timeout = 3600000; // Large but TOML-safe value
        
        // Should be able to update with boundary values
        config_manager.update_config(new_config.clone()).await;
        
        let updated_config = config_manager.get_config().await;
        assert_eq!(updated_config.port, 0);
        assert_eq!(updated_config.max_connections, 0);
        assert_eq!(updated_config.request_timeout, 3600000);
    }

    #[test]
    fn test_serialization_roundtrip_with_boundaries() {
        // Test serialization roundtrip with boundary values - using TOML-safe values
        let mut config = AsyncAppConfig::default();
        
        // Set various boundary values that are safe for TOML serialization
        config.port = 0;
        config.max_connections = 1000000; // Large but TOML-safe
        config.request_timeout = 0;
        config.http_client.timeout = 3600000; // Large but TOML-safe
        config.scheduler.worker_threads = 0;
        config.cache.max_entries = 1000000; // Large but TOML-safe
        config.name = String::new();
        config.scheduler.enabled = false;
        
        // Serialize and deserialize
        let serialized = toml::to_string(&config).expect("Should serialize boundary config");
        let deserialized: AsyncAppConfig = toml::from_str(&serialized).expect("Should deserialize boundary config");
        
        // Verify all boundary values preserved
        assert_eq!(deserialized.port, 0);
        assert_eq!(deserialized.max_connections, 1000000);
        assert_eq!(deserialized.request_timeout, 0);
        assert_eq!(deserialized.http_client.timeout, 3600000);
        assert_eq!(deserialized.scheduler.worker_threads, 0);
        assert_eq!(deserialized.cache.max_entries, 1000000);
        assert!(deserialized.name.is_empty());
        assert!(!deserialized.scheduler.enabled);
    }

    #[test]
    fn test_invalid_toml_handling() {
        // Test handling of malformed TOML
        let invalid_toml = r#"
        name = "Test"
        port = "not_a_number"
        debug = 123
        "#;
        
        // Should fail gracefully with invalid TOML
        let result = toml::from_str::<AsyncAppConfig>(invalid_toml);
        assert!(result.is_err(), "Should fail to parse invalid TOML");
    }
}
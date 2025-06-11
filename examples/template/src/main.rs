//! # Lingo Configuration Template Generation Example
//!
//! This example demonstrates how to use Lingo to generate various configuration templates
//! for a typical web application. It showcases:
//!
//! - TOML configuration file generation
//! - Environment variables template generation
//! - Docker Compose configuration generation
//! - Kubernetes deployment configuration generation
//! - Complex nested configuration structures
//! - Production-ready configuration templates

use lingo::Config;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use clap::Parser;
use colored::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// Application name
    pub app_name: String,
    /// Application version
    pub app_version: String,
    /// Environment (development, staging, production)
    pub environment: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum number of connections
    pub max_connections: Option<u32>,
    /// Request timeout in seconds
    pub timeout: Option<u64>,
    /// Enable SSL/TLS
    pub ssl_enabled: Option<bool>,
    /// SSL certificate path
    pub ssl_cert_path: Option<String>,
    /// SSL private key path
    pub ssl_key_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub database: String,
    /// Database username
    pub username: String,
    /// Database password
    pub password: String,
    /// Connection pool size
    pub pool_size: Option<u32>,
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
    /// Enable SSL
    pub ssl_enabled: Option<bool>,
    /// SSL certificate path
    pub ssl_cert_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,
    /// Connection pool size
    pub pool_size: Option<u32>,
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
    /// Enable cluster mode
    pub cluster_enabled: Option<bool>,
    /// Cluster nodes (if cluster mode is enabled)
    pub cluster_nodes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (text, json)
    pub format: String,
    /// Log targets (stdout, file)
    pub targets: Vec<String>,
    /// Log file path (if file target is used)
    pub file_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecurityConfig {
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT expiration time in seconds
    pub jwt_expiration: Option<u64>,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    /// Enable CSRF protection
    pub csrf_enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptionConfig {
    /// Encryption algorithm
    pub algorithm: String,
    /// Encryption key size
    pub key_size: Option<u32>,
    /// Key derivation iterations
    pub iterations: Option<u32>,
    /// Key file path
    pub key_file_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonitoringConfig {
    /// Health check endpoint path
    pub health_check_path: String,
    /// Metrics endpoint path
    pub metrics_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TracingConfig {
    /// Tracing endpoint
    pub endpoint: String,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: Option<f64>,
    /// Service name
    pub service_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlertingConfig {
    /// Enabled alerting channels
    pub channels: Option<Vec<String>>,
    /// Email alerting configuration
    pub email: Option<EmailAlertConfig>,
    /// Slack alerting configuration
    pub slack: Option<SlackAlertConfig>,
    /// Webhook alerting configuration
    pub webhook: Option<WebhookAlertConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailAlertConfig {
    /// SMTP host
    pub smtp_host: String,
    /// SMTP port
    pub smtp_port: Option<u16>,
    /// From email address
    pub from_email: String,
    /// To email addresses
    pub to_emails: Vec<String>,
    /// SMTP username
    pub username: String,
    /// SMTP password
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlackAlertConfig {
    /// Slack webhook URL
    pub webhook_url: String,
    /// Slack channel
    pub channel: String,
    /// Bot username
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookAlertConfig {
    /// Webhook URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// HTTP headers
    pub headers: Option<std::collections::HashMap<String, String>>,
}

#[derive(Config, Serialize, Deserialize, Debug, Clone)]
pub struct TemplateConfig {
    /// Application configuration
    pub app: AppConfig,
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Redis configuration
    pub redis: RedisConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Encryption configuration
    pub encryption: EncryptionConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Tracing configuration
    pub tracing: TracingConfig,
    /// Alerting configuration
    pub alerting: AlertingConfig,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            app: AppConfig {
                app_name: "Template Example App".to_string(),
                app_version: "0.1.0".to_string(),
                environment: "development".to_string(),
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: Some(1000),
                timeout: Some(30),
                ssl_enabled: Some(false),
                ssl_cert_path: None,
                ssl_key_path: None,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "myapp".to_string(),
                username: "postgres".to_string(),
                password: "password".to_string(),
                pool_size: Some(10),
                timeout: Some(30),
                ssl_enabled: Some(false),
                ssl_cert_path: None,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: Some(10),
                timeout: Some(5),
                cluster_enabled: Some(false),
                cluster_nodes: None,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "text".to_string(),
                targets: vec!["stdout".to_string()],
                file_path: None,
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key-here".to_string(),
                jwt_expiration: Some(3600),
                cors_origins: vec!["http://localhost:3000".to_string()],
                csrf_enabled: Some(true),
            },
            encryption: EncryptionConfig {
                algorithm: "AES-256-GCM".to_string(),
                key_size: Some(256),
                iterations: Some(100000),
                key_file_path: None,
            },
            monitoring: MonitoringConfig {
                health_check_path: "/health".to_string(),
                metrics_path: "/metrics".to_string(),
            },
            tracing: TracingConfig {
                endpoint: "http://localhost:14268/api/traces".to_string(),
                sampling_rate: Some(0.1),
                service_name: "template-example".to_string(),
            },
            alerting: AlertingConfig {
                channels: None,
                email: None,
                slack: None,
                webhook: None,
            },
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "template-example")]
#[command(about = "Lingo configuration template generation example")]
struct Args {
    /// Generate configuration file template
    #[arg(long)]
    generate_config: bool,

    /// Generate environment variables template
    #[arg(long)]
    generate_env: bool,

    /// Generate Docker Compose configuration
    #[arg(long)]
    generate_docker: bool,

    /// Generate Kubernetes configuration
    #[arg(long)]
    generate_k8s: bool,

    /// Generate all templates
    #[arg(long)]
    generate_all: bool,

    /// Output directory for generated templates
    #[arg(long, default_value = "./generated")]
    output_dir: String,
}

struct TemplateGenerator {
    config: TemplateConfig,
    output_dir: String,
}

impl TemplateGenerator {
    fn new(config: TemplateConfig, output_dir: String) -> Self {
        Self { config, output_dir }
    }

    fn generate_config_template(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.output_dir)?;
        let config_path = Path::new(&self.output_dir).join("config.toml");
        
        println!("{} Configuration file template generated: {}", "[OK]".green(), config_path.display().to_string().cyan());
        
        let toml_content = self.generate_toml_config();
        fs::write(config_path, toml_content)?;
        
        Ok(())
    }

    fn generate_toml_config(&self) -> String {
        format!(r#"# Template Configuration File
# 
# This is a comprehensive configuration template for a web application.
# Modify the values according to your specific requirements.

[app]
# Application name
app_name = "{}"

# Application version
app_version = "{}"

# Environment (development, staging, production)
environment = "{}"

[server]
# Server host address
host = "{}"

# Server port
port = {}

# Maximum number of connections
max_connections = {}

# Request timeout in seconds
timeout = {}

# Enable SSL/TLS
ssl_enabled = {}

[database]
# Database host
host = "{}"

# Database port
port = {}

# Database name
database = "{}"

# Database username
username = "{}"

# Database password
password = "{}"

# Connection pool size
pool_size = {}

# Connection timeout in seconds
timeout = {}

# Enable SSL
ssl_enabled = {}

[redis]
# Redis connection URL
url = "{}"

# Connection pool size
pool_size = {}

# Connection timeout in seconds
timeout = {}

# Enable cluster mode
cluster_enabled = {}

[logging]
# Log level (trace, debug, info, warn, error)
level = "{}"

# Log format (text, json)
format = "{}"

# Log targets (stdout, file)
targets = ["{}"]

[security]
# JWT secret key (change this in production!)
jwt_secret = "{}"

# JWT expiration time in seconds
jwt_expiration = {}

# CORS allowed origins
cors_origins = ["{}"]

# Enable CSRF protection
csrf_enabled = {}

[encryption]
# Encryption algorithm
algorithm = "{}"

# Encryption key size
key_size = {}

# Key derivation iterations
iterations = {}

[monitoring]
# Health check endpoint path
health_check_path = "{}"

# Metrics endpoint path
metrics_path = "{}"

[tracing]
# Tracing endpoint
endpoint = "{}"

# Sampling rate (0.0 to 1.0)
sampling_rate = {}

# Service name
service_name = "{}"
"#,
            self.config.app.app_name,
            self.config.app.app_version,
            self.config.app.environment,
            self.config.server.host,
            self.config.server.port,
            self.config.server.max_connections.unwrap_or(1000),
            self.config.server.timeout.unwrap_or(30),
            self.config.server.ssl_enabled.unwrap_or(false),
            self.config.database.host,
            self.config.database.port,
            self.config.database.database,
            self.config.database.username,
            self.config.database.password,
            self.config.database.pool_size.unwrap_or(10),
            self.config.database.timeout.unwrap_or(30),
            self.config.database.ssl_enabled.unwrap_or(false),
            self.config.redis.url,
            self.config.redis.pool_size.unwrap_or(10),
            self.config.redis.timeout.unwrap_or(5),
            self.config.redis.cluster_enabled.unwrap_or(false),
            self.config.logging.level,
            self.config.logging.format,
            self.config.logging.targets.join(", "),
            self.config.security.jwt_secret,
            self.config.security.jwt_expiration.unwrap_or(3600),
            self.config.security.cors_origins.join(", "),
            self.config.security.csrf_enabled.unwrap_or(true),
            self.config.encryption.algorithm,
            self.config.encryption.key_size.unwrap_or(256),
            self.config.encryption.iterations.unwrap_or(100000),
            self.config.monitoring.health_check_path,
            self.config.monitoring.metrics_path,
            self.config.tracing.endpoint,
            self.config.tracing.sampling_rate.unwrap_or(0.1),
            self.config.tracing.service_name,
        )
    }

    fn generate_env_template(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.output_dir)?;
        let env_path = Path::new(&self.output_dir).join(".env.example");
        
        println!("{} Environment variables template generated: {}", "[OK]".green(), env_path.display().to_string().cyan());
        
        let env_content = self.generate_env_config();
        fs::write(env_path, env_content)?;
        
        Ok(())
    }

    fn generate_env_config(&self) -> String {
        format!(r#"# Template Environment Variables Configuration
# 
# Copy this file to .env and modify the values according to your environment

# Application Configuration
APP_NAME="{}"
APP_VERSION="{}"
ENVIRONMENT="{}"

# Server Configuration
HOST="{}"
PORT={}
MAX_CONNECTIONS={}
TIMEOUT={}
SSL_ENABLED={}

# Database Configuration
DATABASE_HOST="{}"
DATABASE_PORT={}
DATABASE_DATABASE="{}"
DATABASE_USERNAME="{}"
DATABASE_PASSWORD="{}"
DATABASE_POOL_SIZE={}
DATABASE_TIMEOUT={}
DATABASE_SSL_ENABLED={}

# Redis Configuration
REDIS_URL="{}"
REDIS_POOL_SIZE={}
REDIS_TIMEOUT={}
REDIS_CLUSTER_ENABLED={}

# Logging Configuration
LOGGING_LEVEL="{}"
LOGGING_FORMAT="{}"
LOGGING_TARGETS="{}"

# Security Configuration
SECURITY_JWT_SECRET="{}"
SECURITY_JWT_EXPIRATION={}
SECURITY_CORS_ORIGINS="{}"
SECURITY_CSRF_ENABLED={}

# Encryption Configuration
SECURITY_ENCRYPTION_ALGORITHM="{}"
SECURITY_ENCRYPTION_KEY_SIZE={}
SECURITY_ENCRYPTION_ITERATIONS={}

# Monitoring Configuration
MONITORING_HEALTH_CHECK_PATH="{}"
MONITORING_METRICS_PATH="{}"

# Tracing Configuration
MONITORING_TRACING_ENDPOINT="{}"
MONITORING_TRACING_SAMPLING_RATE={}
MONITORING_TRACING_SERVICE_NAME="{}"
"#,
            self.config.app.app_name,
            self.config.app.app_version,
            self.config.app.environment,
            self.config.server.host,
            self.config.server.port,
            self.config.server.max_connections.unwrap_or(1000),
            self.config.server.timeout.unwrap_or(30),
            self.config.server.ssl_enabled.unwrap_or(false),
            self.config.database.host,
            self.config.database.port,
            self.config.database.database,
            self.config.database.username,
            self.config.database.password,
            self.config.database.pool_size.unwrap_or(10),
            self.config.database.timeout.unwrap_or(30),
            self.config.database.ssl_enabled.unwrap_or(false),
            self.config.redis.url,
            self.config.redis.pool_size.unwrap_or(10),
            self.config.redis.timeout.unwrap_or(5),
            self.config.redis.cluster_enabled.unwrap_or(false),
            self.config.logging.level,
            self.config.logging.format,
            self.config.logging.targets.join(","),
            self.config.security.jwt_secret,
            self.config.security.jwt_expiration.unwrap_or(3600),
            self.config.security.cors_origins.join(","),
            self.config.security.csrf_enabled.unwrap_or(true),
            self.config.encryption.algorithm,
            self.config.encryption.key_size.unwrap_or(256),
            self.config.encryption.iterations.unwrap_or(100000),
            self.config.monitoring.health_check_path,
            self.config.monitoring.metrics_path,
            self.config.tracing.endpoint,
            self.config.tracing.sampling_rate.unwrap_or(0.1),
            self.config.tracing.service_name,
        )
    }

    fn generate_docker_compose(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.output_dir)?;
        let docker_path = Path::new(&self.output_dir).join("docker-compose.yml");
        
        println!("{} Docker Compose configuration generated: {}", "[OK]".green(), docker_path.display().to_string().cyan());
        
        let docker_content = self.generate_docker_config();
        fs::write(docker_path, docker_content)?;
        
        Ok(())
    }

    fn generate_docker_config(&self) -> String {
        format!(r#"# Template Docker Compose Configuration
# This file defines the complete application stack

version: "3.8"

services:
  app:
    build: .
    ports:
      - "{}:{}"
    environment:
      - ENVIRONMENT=production
      - DATABASE_HOST=postgres
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:{}/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  postgres:
    image: postgres:15
    ports:
      - "{}:{}"
    environment:
      - POSTGRES_DB={}
      - POSTGRES_USER={}
      - POSTGRES_PASSWORD={}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U {} -d {}"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:

networks:
  default:
    name: {}
"#,
            self.config.server.port,
            self.config.server.port,
            self.config.server.port,
            self.config.database.port,
            self.config.database.port,
            self.config.database.database,
            self.config.database.username,
            self.config.database.password,
            self.config.database.username,
            self.config.database.database,
            self.config.app.app_name.to_lowercase().replace(" ", "-"),
        )
    }

    fn generate_k8s_deployment(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.output_dir)?;
        let k8s_path = Path::new(&self.output_dir).join("k8s-deployment.yaml");
        
        println!("{} Kubernetes configuration generated: {}", "[OK]".green(), k8s_path.display().to_string().cyan());
        
        let k8s_content = self.generate_k8s_config();
        fs::write(k8s_path, k8s_content)?;
        
        Ok(())
    }

    fn generate_k8s_config(&self) -> String {
        let app_name = self.config.app.app_name.to_lowercase().replace(" ", "-");
        format!(r#"# Template Kubernetes Deployment Configuration
# This file contains all necessary Kubernetes resources

apiVersion: v1
kind: ConfigMap
metadata:
  name: {}-config
  namespace: default
data:
  config.toml: |
    [app]
    app_name = "{}"
    app_version = "{}"
    environment = "production"
    
    [server]
    host = "0.0.0.0"
    port = {}
    
    [database]
    host = "postgres-service"
    port = {}
    database = "{}"
    username = "{}"
    password = "password"
    
    [redis]
    url = "redis://redis-service:6379"
    
    [logging]
    level = "info"
    format = "json"
    targets = ["stdout"]
    
    [monitoring]
    health_check_path = "/health"
    metrics_path = "/metrics"

---
apiVersion: v1
kind: Secret
metadata:
  name: {}-secrets
  namespace: default
type: Opaque
data:
  database-password: cGFzc3dvcmQ=
  jwt-secret: eW91ci1zZWNyZXQta2V5LWhlcmU=

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}-deployment
  namespace: default
  labels:
    app: {}
spec:
  replicas: 3
  selector:
    matchLabels:
      app: {}
  template:
    metadata:
      labels:
        app: {}
    spec:
      containers:
      - name: {}
        image: {}:latest
        ports:
        - containerPort: {}
        env:
        - name: ENVIRONMENT
          value: "production"
        - name: DATABASE_PASSWORD
          valueFrom:
            secretKeyRef:
              name: {}-secrets
              key: database-password
        - name: SECURITY_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: {}-secrets
              key: jwt-secret
        livenessProbe:
          httpGet:
            path: /health
            port: {}
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: {}
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"

---
apiVersion: v1
kind: Service
metadata:
  name: {}-service
  namespace: default
spec:
  selector:
    app: {}
  ports:
  - protocol: TCP
    port: 80
    targetPort: {}
  type: LoadBalancer

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {}-hpa
  namespace: default
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {}-deployment
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
"#,
            app_name,
            self.config.app.app_name,
            self.config.app.app_version,
            self.config.server.port,
            self.config.database.port,
            self.config.database.database,
            self.config.database.username,
            app_name,
            app_name,
            app_name,
            app_name,
            app_name,
            app_name,
            app_name,
            self.config.server.port,
            app_name,
            app_name,
            self.config.server.port,
            self.config.server.port,
            app_name,
            app_name,
            self.config.server.port,
            app_name,
            app_name,
        )
    }

    fn generate_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.generate_config_template()?;
        self.generate_env_template()?;
        self.generate_docker_compose()?;
        self.generate_k8s_deployment()?;
        
        println!("{} All templates generated successfully!", "[SUCCESS]".green());
        println!("Output directory: {}", self.output_dir.cyan());
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("{}", "=== Lingo Configuration Template Generation Example ===".bold().cyan());
    println!();
    
    let config_path = "config.toml";
    let config = if Path::new(config_path).exists() {
        println!("Loading from config file: {}", config_path.cyan());
        TemplateConfig::load_from_file(config_path)?
    } else {
        println!("Using default configuration");
        TemplateConfig::default()
    };
    
    println!();
    println!("Current configuration:");
    println!("  App name: {}", config.app.app_name.green());
    println!("  Version: {}", config.app.app_version.green());
    println!("  Environment: {}", config.app.environment.green());
    println!("  Port: {}", config.server.port.to_string().green());
    println!();
    
    let generator = TemplateGenerator::new(config, args.output_dir);
    
    if args.generate_all {
        generator.generate_all()?;
    } else if args.generate_config {
        generator.generate_config_template()?;
    } else if args.generate_env {
        generator.generate_env_template()?;
    } else if args.generate_docker {
        generator.generate_docker_compose()?;
    } else if args.generate_k8s {
        generator.generate_k8s_deployment()?;
    } else {
        println!("{}", "Please specify the template type to generate, or use --generate-all to generate all templates".yellow());
        println!();
        println!("Available options:");
        println!("  {} Generate configuration file template", "--generate-config".cyan());
        println!("  {} Generate environment variables template", "--generate-env".cyan());
        println!("  {} Generate Docker Compose configuration", "--generate-docker".cyan());
        println!("  {} Generate Kubernetes configuration", "--generate-k8s".cyan());
        println!("  {} Generate all templates", "--generate-all".cyan());
        println!();
        println!("Examples:");
        println!("  {} Generate all templates", "cargo run --example template -- --generate-all".green());
        println!("  {} Generate only config file", "cargo run --example template -- --generate-config".green());
        println!("  {} Specify output directory", "cargo run --example template -- --generate-all --output-dir ./my-templates".green());
        return Ok(());
    }
    
    println!();
    println!("{}", "Configuration template generation features:".bold());
    println!("  {} Automatically generate TOML configuration file templates", "[OK]".green());
    println!("  {} Generate environment variable configuration templates", "[OK]".green());
    println!("  {} Generate Docker Compose deployment configuration", "[OK]".green());
    println!("  {} Generate Kubernetes deployment configuration", "[OK]".green());
    println!("  {} Include detailed configuration descriptions and comments", "[OK]".green());
    println!("  {} Support nested configuration structures", "[OK]".green());
    println!("  {} Production-ready configuration templates", "[OK]".green());
    
    println!();
    println!("{} Template generation completed!", "[SUCCESS]".green());
    
    Ok(())
}
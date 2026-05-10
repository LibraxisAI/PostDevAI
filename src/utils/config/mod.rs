use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub ramlake: RamLakeConfig,
    pub models: ModelsConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamLakeConfig {
    pub path: String,
    pub max_size: u64,
    pub backup_interval: u64,
    pub backup_path: String,
    pub allocation: StoreAllocationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreAllocationConfig {
    pub vector_store: f32,
    pub code_store: f32,
    pub history_store: f32,
    pub metadata_store: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    pub device: String,
    pub memory_limit: f64,
    pub models: Vec<ModelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub path: String,
    pub tokenizer_path: String,
    pub memory_required: f64,
    pub r#type: String,
    pub task: String,
    pub priority: i32,
    pub max_tokens: Option<usize>,
    pub model_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_tls: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub enable_auth: bool,
    pub jwt_secret: Option<String>,
    pub allowed_clients: Option<Vec<String>>,
}

/// Load configuration from TOML file
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    
    // Validate configuration
    validate_config(&config)?;
    
    Ok(config)
}

/// Validate configuration
fn validate_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // Validate server config
    if config.server.port == 0 {
        return Err("Server port cannot be 0".into());
    }
    
    // Validate RAM-Lake config
    if config.ramlake.max_size == 0 {
        return Err("RAM-Lake max size cannot be 0".into());
    }
    
    let allocation_sum = config.ramlake.allocation.vector_store +
                         config.ramlake.allocation.code_store +
                         config.ramlake.allocation.history_store +
                         config.ramlake.allocation.metadata_store;
    
    if (allocation_sum - 1.0).abs() > 0.001 {
        return Err(format!("Store allocation must sum to 1.0, got {}", allocation_sum).into());
    }
    
    // Validate models config
    if config.models.memory_limit <= 0.0 {
        return Err("Models memory limit must be positive".into());
    }
    
    if !["cpu", "gpu"].contains(&config.models.device.as_str()) {
        return Err(format!("Device must be 'cpu' or 'gpu', got {}", config.models.device).into());
    }
    
    // Validate security config
    if config.security.enable_tls {
        if config.security.cert_path.is_none() || config.security.key_path.is_none() {
            return Err("TLS is enabled but cert_path or key_path is missing".into());
        }
    }
    
    if config.security.enable_auth && config.security.jwt_secret.is_none() {
        return Err("Authentication is enabled but jwt_secret is missing".into());
    }
    
    Ok(())
}

/// Create default configuration
pub fn create_default_config() -> Config {
    Config {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 50051,
            workers: 4,
        },
        ramlake: RamLakeConfig {
            path: "/mnt/ramlake".to_string(),
            max_size: 200 * 1024 * 1024 * 1024, // 200GB
            backup_interval: 3600, // 1 hour
            backup_path: "/var/backups/ramlake".to_string(),
            allocation: StoreAllocationConfig {
                vector_store: 0.3,
                code_store: 0.4,
                history_store: 0.2,
                metadata_store: 0.1,
            },
        },
        models: ModelsConfig {
            device: "gpu".to_string(),
            memory_limit: 200.0, // 200GB
            models: vec![
                ModelConfig {
                    name: "embedder".to_string(),
                    path: "/opt/models/nomic-embed-text-v1".to_string(),
                    tokenizer_path: "/opt/models/nomic-embed-text-v1/tokenizer.json".to_string(),
                    memory_required: 1.0, // 1GB
                    r#type: "embedder".to_string(),
                    task: "embedding".to_string(),
                    priority: 10, // Highest priority
                    max_tokens: None,
                    model_type: None,
                },
                ModelConfig {
                    name: "qwen-72b".to_string(),
                    path: "/opt/models/mlx-qwen3-72b".to_string(),
                    tokenizer_path: "/opt/models/mlx-qwen3-72b/tokenizer.json".to_string(),
                    memory_required: 140.0, // 140GB
                    r#type: "llm".to_string(),
                    task: "reasoning".to_string(),
                    priority: 5,
                    max_tokens: Some(32000),
                    model_type: Some("qwen3".to_string()),
                },
                ModelConfig {
                    name: "codellama-34b".to_string(),
                    path: "/opt/models/mlx-codellama-34b".to_string(),
                    tokenizer_path: "/opt/models/mlx-codellama-34b/tokenizer.json".to_string(),
                    memory_required: 65.0, // 65GB
                    r#type: "llm".to_string(),
                    task: "code".to_string(),
                    priority: 6,
                    max_tokens: Some(16000),
                    model_type: Some("llama".to_string()),
                },
                ModelConfig {
                    name: "mistral-7b".to_string(),
                    path: "/opt/models/mlx-mistral-7b-v0.2".to_string(),
                    tokenizer_path: "/opt/models/mlx-mistral-7b-v0.2/tokenizer.json".to_string(),
                    memory_required: 14.0, // 14GB
                    r#type: "llm".to_string(),
                    task: "fast".to_string(),
                    priority: 8,
                    max_tokens: Some(8000),
                    model_type: Some("mistral".to_string()),
                },
            ],
        },
        security: SecurityConfig {
            enable_tls: false,
            cert_path: None,
            key_path: None,
            enable_auth: false,
            jwt_secret: None,
            allowed_clients: None,
        },
    }
}

/// Save configuration to TOML file
pub fn save_config<P: AsRef<Path>>(config: &Config, path: P) -> Result<(), Box<dyn std::error::Error>> {
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}
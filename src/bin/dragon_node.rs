use std::sync::Arc;
use std::path::PathBuf;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::signal;
use tonic::transport::Server;
use parking_lot::RwLock;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt};

use postdevai::core::memory::ramlake::{RamLake, RamLakeConfig, StoreAllocation};
use postdevai::core::network::dragon_node_service::{DragonNodeServiceImpl, DragonNodeServiceServer};
use postdevai::mlx::models::MLXModelManager;
use postdevai::utils::config::{load_config, ModelConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    
    info!("Starting Dragon Node server...");
    
    // Load configuration
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "./config/dragon_node.toml".to_string());
    
    let config = load_config(&config_path)?;
    
    // Setup RAM-Lake
    info!("Initializing RAM-Lake...");
    let ramdisk_path = PathBuf::from(&config.ramlake.path);
    
    let ramlake_config = RamLakeConfig {
        max_size: config.ramlake.max_size,
        backup_interval: config.ramlake.backup_interval,
        backup_path: PathBuf::from(&config.ramlake.backup_path),
        allocation: StoreAllocation {
            vector_store: config.ramlake.allocation.vector_store,
            code_store: config.ramlake.allocation.code_store,
            history_store: config.ramlake.allocation.history_store,
            metadata_store: config.ramlake.allocation.metadata_store,
        },
    };
    
    let ram_lake = RamLake::new(ramdisk_path, ramlake_config)
        .map_err(|e| format!("Failed to initialize RAM-Lake: {}", e))?;
    
    // Start RAM-Lake background tasks
    ram_lake.start()
        .map_err(|e| format!("Failed to start RAM-Lake background tasks: {}", e))?;
    
    let ram_lake = Arc::new(RwLock::new(ram_lake));
    
    // Setup MLX Model Manager
    info!("Initializing MLX Model Manager...");
    
    // Convert ModelConfig to format required by MLXModelManager
    let models_config: std::collections::HashMap<String, ModelConfig> = config.models.models
        .into_iter()
        .map(|m| (m.name.clone(), m))
        .collect();
    
    let model_manager = MLXModelManager::new(
        models_config,
        config.models.memory_limit,
        &config.models.device,
    )?;
    
    let model_manager = Arc::new(RwLock::new(model_manager));
    
    // Create Dragon Node service
    let dragon_service = DragonNodeServiceImpl::new(ram_lake.clone(), model_manager.clone());
    
    // Start gRPC server
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    info!("Starting gRPC server on {}", addr);
    
    Server::builder()
        .add_service(DragonNodeServiceServer::new(dragon_service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    
    info!("Dragon Node server shutting down...");
    
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down...");
        },
        _ = terminate => {
            info!("Received terminate signal, shutting down...");
        },
    }
}
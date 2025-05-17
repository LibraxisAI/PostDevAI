use std::sync::Arc;
use std::path::PathBuf;
use std::error::Error;
use std::time::Duration;
use std::io;

use tokio::runtime::Runtime;
use parking_lot::RwLock;
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt};

use postdevai::core::memory::ramlake::{RamLake, RamLakeConfig, StoreAllocation};
use postdevai::tui::app::{run_app, setup_terminal, restore_terminal, App};
use postdevai::tui::bridge::SystemBridge;
use postdevai::utils::config::{load_config, ModelConfig};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    
    info!("Starting Developer Node...");
    
    // Load configuration
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "./config/developer_node.toml".to_string());
    
    let config = load_config(&config_path)?;
    
    // Set up runtime for async operations
    let rt = Runtime::new()?;
    
    // Set up connection to Dragon Node
    info!("Connecting to Dragon Node at {}:{}", config.dragon_node.host, config.dragon_node.port);
    
    // Initialize local caches and TUI state
    info!("Initializing TUI...");
    
    // Create terminal
    let mut terminal = setup_terminal()?;
    
    // Create app with TUI
    let mut app = App::new(Duration::from_millis(250));
    
    // Run the TUI application
    match run_app(&mut terminal, app) {
        Ok(_) => {
            restore_terminal(&mut terminal)?;
            info!("Shutting down Developer Node...");
            Ok(())
        }
        Err(err) => {
            restore_terminal(&mut terminal)?;
            error!("Error running application: {}", err);
            Err(err.into())
        }
    }
}

// In a full implementation, we'd have actual code to connect to the Dragon Node
// and properly handle loading data from it
fn connect_to_dragon_node(host: &str, port: u16, app: &mut App) -> Result<(), Box<dyn Error>> {
    info!("Connecting to Dragon Node at {}:{}", host, port);
    
    // In a real implementation, this would connect to the Dragon Node
    // and load RAM-Lake and other components
    // Here, we're just setting up dummy data
    
    // Create a dummy RAM-Lake for demonstration
    let ramdisk_path = PathBuf::from("/tmp/ramlake");
    let ramlake_config = RamLakeConfig {
        max_size: 200 * 1024 * 1024 * 1024, // 200 GB
        backup_interval: 3600,               // 1 hour
        backup_path: PathBuf::from("/tmp/ramlake_backup"),
        allocation: StoreAllocation {
            vector_store: 0.4,
            code_store: 0.3,
            history_store: 0.2,
            metadata_store: 0.1,
        },
    };
    
    // We would actually connect to the real RAM-Lake via gRPC
    // For now, create a dummy local instance for the TUI to use
    if let Ok(ram_lake) = RamLake::new(ramdisk_path, ramlake_config) {
        let ram_lake = Arc::new(RwLock::new(ram_lake));
        app.set_ramlake(ram_lake);
    }

    Ok(())
}
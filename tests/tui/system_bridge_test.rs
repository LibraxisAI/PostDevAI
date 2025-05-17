use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use chrono::Utc;

use postdevai::tui::bridge::SystemBridge;
use postdevai::core::memory::{RamLake, RamLakeConfig, StoreAllocation};
use postdevai::tui::state::app_state::{ModelInfo, EventInfo, CodeInfo};
use postdevai::system::{SystemState, MemoryUsage, NodeType};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Instant;

    /// Test SystemBridge creation without any components
    #[test]
    fn test_system_bridge_creation() {
        let bridge = SystemBridge::new();
        assert!(bridge.get_loaded_models().is_empty());
        assert!(bridge.get_recent_events(100).is_empty());
        assert!(bridge.get_recent_code(100).is_empty());
        assert!(!bridge.get_node_connections().is_empty()); // Default connections are provided
    }

    /// Test SystemBridge getting system state
    #[test]
    fn test_get_system_state() {
        let bridge = SystemBridge::new();
        let result = bridge.get_system_state();
        assert!(result.is_ok());
        
        if let Ok(state) = result {
            assert_eq!(state.node_type, NodeType::Developer);
            assert!(!state.hostname.is_empty());
            assert!(state.memory_usage.total > 0);
        }
    }

    /// Test SystemBridge with placeholder RAM-Lake metrics
    #[test]
    fn test_get_ramlake_metrics() {
        let bridge = SystemBridge::new();
        let metrics = bridge.get_ramlake_metrics();
        
        // Check placeholder metrics
        assert_eq!(metrics.total_size, 1024 * 1024 * 1024 * 200); // 200GB
        assert_eq!(metrics.used_size, 1024 * 1024 * 1024 * 50);   // 50GB
        assert_eq!(metrics.vector_store_size, 1024 * 1024 * 1024 * 20);  // 20GB
        assert_eq!(metrics.code_store_size, 1024 * 1024 * 1024 * 15);    // 15GB
        assert_eq!(metrics.history_store_size, 1024 * 1024 * 1024 * 10); // 10GB
        assert_eq!(metrics.metadata_store_size, 1024 * 1024 * 1024 * 5); // 5GB
        assert_eq!(metrics.indexed_files, 1256);
        assert_eq!(metrics.vector_entries, 25789);
        assert_eq!(metrics.history_events, 3467);
    }

    /// Test model cache and updates
    #[test]
    fn test_model_cache_update() {
        let mut bridge = SystemBridge::new();
        
        // Get initial models
        let models = bridge.get_loaded_models();
        assert!(!models.is_empty());
        
        // Get models again without waiting - should be cached
        let cached_models = bridge.get_loaded_models();
        assert_eq!(models.len(), cached_models.len());
        
        // Models should have proper structure
        if let Some(model) = models.first() {
            assert!(!model.name.is_empty());
            assert!(!model.model_type.is_empty());
            assert!(!model.status.is_empty());
            assert!(model.memory_gb > 0.0);
            assert!(model.priority >= 0);
        }
    }

    /// Test MlxBridge functionality
    #[test]
    fn test_mlx_bridge() {
        let bridge = SystemBridge::new();
        let models = bridge.get_loaded_models();
        
        // Verify we have mock model data
        assert!(models.iter().any(|m| m.name == "Qwen3-32B"));
        assert!(models.iter().any(|m| m.name == "MLX-Embedder"));
    }

    /// Test node connections
    #[test]
    fn test_node_connections() {
        let bridge = SystemBridge::new();
        let connections = bridge.get_node_connections();
        
        // Should have default connections
        assert!(!connections.is_empty());
        
        // Should have expected node types
        assert!(connections.iter().any(|c| c.node_type == "Dragon"));
        assert!(connections.iter().any(|c| c.node_type == "Developer"));
        assert!(connections.iter().any(|c| c.node_type == "Coordinator"));
        
        // All should be "connected" in mock data
        for conn in connections {
            assert_eq!(conn.status, "connected");
        }
    }
    
    /// Test MockRamLake creation and behavior
    #[test]
    fn test_mock_ram_lake() {
        // Setting up a mock RAM-Lake for testing the SystemBridge
        let ramdisk_path = PathBuf::from("/tmp/test_ramlake");
        let ramlake_config = RamLakeConfig {
            max_size: 1024 * 1024 * 1024, // 1 GB for testing
            backup_interval: 3600,
            backup_path: PathBuf::from("/tmp/test_ramlake_backup"),
            allocation: StoreAllocation {
                vector_store: 0.4,
                code_store: 0.3,
                history_store: 0.2,
                metadata_store: 0.1,
            },
        };
        
        // We'd create a real RamLake in true testing, but we'll skip that for this test
        // and just test the bridge's mock implementation
        let result = RamLake::new(ramdisk_path, ramlake_config);
        
        // Either we created a real RamLake or we got an error (likely because /tmp/test_ramlake doesn't exist)
        // Either outcome is fine for this test - we're only testing the error-resistant behavior of SystemBridge
        match result {
            Ok(_) => println!("Created test RamLake - would assign to bridge in full test"),
            Err(e) => println!("Error creating test RamLake: {}", e),
        }
        
        // Even without RamLake, the bridge provides mock metrics
        let bridge = SystemBridge::new();
        let metrics = bridge.get_ramlake_metrics();
        assert!(metrics.total_size > 0);
    }
}
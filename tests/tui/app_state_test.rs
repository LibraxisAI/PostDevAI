use std::time::{Duration, Instant};
use chrono::{DateTime, Utc, Local};
use uuid::Uuid;

use postdevai::tui::state::app_state::{AppState, ModelInfo, EventInfo, CodeInfo, NodeConnection};
use postdevai::system::{SystemState, MemoryUsage, NodeType};
use postdevai::core::memory::RamLakeMetrics;

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test that a new AppState is created with default values
    #[test]
    fn test_new_app_state() {
        let state = AppState::new();
        
        // Check default values
        assert_eq!(state.ramlake_metrics.total_size, 0);
        assert_eq!(state.ramlake_metrics.used_size, 0);
        assert!(state.loaded_models.is_empty());
        assert!(state.recent_events.is_empty());
        assert!(state.recent_code.is_empty());
        assert!(state.node_connections.is_empty());
        assert!(state.uptime.as_secs() == 0);
    }
    
    /// Test updating app state from system state
    #[test]
    fn test_update_from_system_state() {
        let mut state = AppState::new();
        
        // Create a test system state
        let system_state = SystemState {
            node_type: NodeType::Developer,
            hostname: "test-host".to_string(),
            uptime: Duration::from_secs(3600), // 1 hour
            memory_usage: MemoryUsage {
                total: 16 * 1024 * 1024 * 1024, // 16 GB
                used: 8 * 1024 * 1024 * 1024,   // 8 GB
                free: 8 * 1024 * 1024 * 1024,   // 8 GB
            },
            cpu_usage: 25.5, // 25.5% CPU usage
        };
        
        // Update state
        state.update(&system_state);
        
        // Check that values were updated
        assert_eq!(state.system_state.hostname, "test-host");
        assert_eq!(state.system_state.uptime.as_secs(), 3600);
        assert_eq!(state.system_state.memory_usage.total, 16 * 1024 * 1024 * 1024);
        assert_eq!(state.system_state.memory_usage.used, 8 * 1024 * 1024 * 1024);
        assert_eq!(state.system_state.cpu_usage, 25.5);
    }
    
    /// Test updating RAM-Lake metrics
    #[test]
    fn test_update_ramlake_metrics() {
        let mut state = AppState::new();
        
        // Create test metrics
        let metrics = RamLakeMetrics {
            total_size: 100 * 1024 * 1024 * 1024, // 100 GB
            used_size: 50 * 1024 * 1024 * 1024,   // 50 GB
            vector_store_size: 20 * 1024 * 1024 * 1024, // 20 GB
            code_store_size: 15 * 1024 * 1024 * 1024,   // 15 GB
            history_store_size: 10 * 1024 * 1024 * 1024, // 10 GB
            metadata_store_size: 5 * 1024 * 1024 * 1024, // 5 GB
            indexed_files: 1000,
            vector_entries: 50000,
            history_events: 5000,
        };
        
        // Update metrics
        state.update_ramlake_metrics(metrics);
        
        // Check metrics were updated
        assert_eq!(state.ramlake_metrics.total_size, 100 * 1024 * 1024 * 1024);
        assert_eq!(state.ramlake_metrics.used_size, 50 * 1024 * 1024 * 1024);
        assert_eq!(state.ramlake_metrics.vector_store_size, 20 * 1024 * 1024 * 1024);
        assert_eq!(state.ramlake_metrics.code_store_size, 15 * 1024 * 1024 * 1024);
        assert_eq!(state.ramlake_metrics.history_store_size, 10 * 1024 * 1024 * 1024);
        assert_eq!(state.ramlake_metrics.metadata_store_size, 5 * 1024 * 1024 * 1024);
        assert_eq!(state.ramlake_metrics.indexed_files, 1000);
        assert_eq!(state.ramlake_metrics.vector_entries, 50000);
        assert_eq!(state.ramlake_metrics.history_events, 5000);
    }
    
    /// Test updating loaded models
    #[test]
    fn test_update_loaded_models() {
        let mut state = AppState::new();
        
        // Create test models
        let models = vec![
            ModelInfo {
                name: "Test Model 1".to_string(),
                model_type: "LLM".to_string(),
                status: "loaded".to_string(),
                memory_gb: 32.0,
                priority: 10,
                last_used: Instant::now(),
            },
            ModelInfo {
                name: "Test Model 2".to_string(),
                model_type: "Embedder".to_string(),
                status: "unloaded".to_string(),
                memory_gb: 2.0,
                priority: 5,
                last_used: Instant::now(),
            },
        ];
        
        // Update models
        state.update_loaded_models(models.clone());
        
        // Check models were updated
        assert_eq!(state.loaded_models.len(), 2);
        assert_eq!(state.loaded_models[0].name, "Test Model 1");
        assert_eq!(state.loaded_models[0].model_type, "LLM");
        assert_eq!(state.loaded_models[0].memory_gb, 32.0);
        assert_eq!(state.loaded_models[1].name, "Test Model 2");
        assert_eq!(state.loaded_models[1].model_type, "Embedder");
        assert_eq!(state.loaded_models[1].memory_gb, 2.0);
    }
    
    /// Test adding events
    #[test]
    fn test_add_event() {
        let mut state = AppState::new();
        
        // Create test event
        let event = EventInfo {
            id: Uuid::new_v4(),
            event_type: "Test".to_string(),
            timestamp: Local::now(),
            source: Some("Test Source".to_string()),
            severity: Some("Info".to_string()),
            summary: "Test Event".to_string(),
        };
        
        // Add event
        state.add_event(event);
        
        // Check event was added
        assert_eq!(state.recent_events.len(), 1);
        assert_eq!(state.recent_events[0].event_type, "Test");
        assert_eq!(state.recent_events[0].source, Some("Test Source".to_string()));
        assert_eq!(state.recent_events[0].severity, Some("Info".to_string()));
        assert_eq!(state.recent_events[0].summary, "Test Event");
    }
    
    /// Test adding events with limit
    #[test]
    fn test_event_limit() {
        let mut state = AppState::new();
        
        // Add 110 events (limit is 100)
        for i in 0..110 {
            let event = EventInfo {
                id: Uuid::new_v4(),
                event_type: "Test".to_string(),
                timestamp: Local::now(),
                source: Some("Test Source".to_string()),
                severity: Some("Info".to_string()),
                summary: format!("Test Event {}", i),
            };
            state.add_event(event);
        }
        
        // Check events were limited to 100
        assert_eq!(state.recent_events.len(), 100);
        
        // Check the first event is event 110
        assert_eq!(state.recent_events[0].summary, "Test Event 109");
    }
    
    /// Test adding code files
    #[test]
    fn test_add_code() {
        let mut state = AppState::new();
        
        // Create test code info
        let code = CodeInfo {
            id: Uuid::new_v4(),
            path: "test/path/file.rs".to_string(),
            language: "Rust".to_string(),
            size: 1024,
            modified_at: Utc::now(),
        };
        
        // Add code
        state.add_code(code);
        
        // Check code was added
        assert_eq!(state.recent_code.len(), 1);
        assert_eq!(state.recent_code[0].path, "test/path/file.rs");
        assert_eq!(state.recent_code[0].language, "Rust");
        assert_eq!(state.recent_code[0].size, 1024);
    }
    
    /// Test clearing events
    #[test]
    fn test_clear_events() {
        let mut state = AppState::new();
        
        // Add some events
        for i in 0..5 {
            let event = EventInfo {
                id: Uuid::new_v4(),
                event_type: "Test".to_string(),
                timestamp: Local::now(),
                source: Some("Test Source".to_string()),
                severity: Some("Info".to_string()),
                summary: format!("Test Event {}", i),
            };
            state.add_event(event);
        }
        
        // Check events were added
        assert_eq!(state.recent_events.len(), 5);
        
        // Clear events
        state.clear_events();
        
        // Check events were cleared
        assert_eq!(state.recent_events.len(), 0);
    }
    
    /// Test updating node connections
    #[test]
    fn test_update_node_connections() {
        let mut state = AppState::new();
        
        // Create test connections
        let connections = vec![
            NodeConnection {
                id: Uuid::new_v4(),
                node_type: "Dragon".to_string(),
                hostname: "dragon-host".to_string(),
                status: "connected".to_string(),
                last_heartbeat: Utc::now(),
            },
            NodeConnection {
                id: Uuid::new_v4(),
                node_type: "Developer".to_string(),
                hostname: "dev-host".to_string(),
                status: "connected".to_string(),
                last_heartbeat: Utc::now(),
            },
        ];
        
        // Update connections
        state.update_node_connections(connections.clone());
        
        // Check connections were updated
        assert_eq!(state.node_connections.len(), 2);
        assert_eq!(state.node_connections[0].node_type, "Dragon");
        assert_eq!(state.node_connections[0].hostname, "dragon-host");
        assert_eq!(state.node_connections[1].node_type, "Developer");
        assert_eq!(state.node_connections[1].hostname, "dev-host");
    }
}
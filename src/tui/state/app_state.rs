use std::collections::VecDeque;
use std::time::{Instant, Duration};
use serde::{Serialize, Deserialize};

use crate::core::memory::ramlake::RamLakeMetrics;
use crate::system::SystemState;

/// Application state for the TUI
pub struct AppState {
    /// Current RAM-Lake metrics
    pub ramlake_metrics: RamLakeMetrics,
    
    /// System state (memory, CPU, etc.)
    pub system_state: SystemState,
    
    /// Loaded models
    pub loaded_models: Vec<ModelInfo>,
    
    /// Recent events
    pub recent_events: VecDeque<EventInfo>,
    
    /// Recent code files
    pub recent_code: VecDeque<CodeInfo>,
    
    /// Application uptime
    pub uptime: Duration,
    
    /// Start time
    pub start_time: Instant,
    
    /// Node connections
    pub node_connections: Vec<NodeConnection>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name
    pub name: String,
    
    /// Model type
    pub model_type: String,
    
    /// Model status
    pub status: String,
    
    /// Memory used in GB
    pub memory_gb: f64,
    
    /// Priority
    pub priority: i32,
    
    /// Last used timestamp as elapsed seconds since UNIX epoch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_secs: Option<u64>,
    
    /// Last used timestamp (not serialized)
    #[serde(skip)]
    pub last_used: Option<Instant>,
}

/// Event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    /// Event ID
    pub id: uuid::Uuid,
    
    /// Event type
    pub event_type: String,
    
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Local>,
    
    /// Event source
    pub source: Option<String>,
    
    /// Event severity
    pub severity: Option<String>,
    
    /// Event summary
    pub summary: String,
}

/// Code file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeInfo {
    /// Code ID
    pub id: uuid::Uuid,
    
    /// File path
    pub path: String,
    
    /// Language
    pub language: String,
    
    /// Size in bytes
    pub size: u64,
    
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

/// Node connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    /// Node ID
    pub id: uuid::Uuid,
    
    /// Node type
    pub node_type: String,
    
    /// Node hostname
    pub hostname: String,
    
    /// Node status
    pub status: String,
    
    /// Last heartbeat timestamp
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        Self {
            ramlake_metrics: RamLakeMetrics {
                total_size: 0,
                used_size: 0,
                vector_store_size: 0,
                code_store_size: 0,
                history_store_size: 0,
                metadata_store_size: 0,
                indexed_files: 0,
                vector_entries: 0,
                history_events: 0,
            },
            system_state: SystemState::default(),
            loaded_models: Vec::new(),
            recent_events: VecDeque::with_capacity(100),
            recent_code: VecDeque::with_capacity(100),
            uptime: Duration::from_secs(0),
            start_time: Instant::now(),
            node_connections: Vec::new(),
        }
    }
    
    /// Update application state from system state
    pub fn update(&mut self, system_state: &SystemState) {
        self.system_state = system_state.clone();
        self.uptime = self.start_time.elapsed();
    }
    
    /// Update RAM-Lake metrics
    pub fn update_ramlake_metrics(&mut self, metrics: RamLakeMetrics) {
        self.ramlake_metrics = metrics;
    }
    
    /// Update loaded models
    pub fn update_loaded_models(&mut self, models: Vec<ModelInfo>) {
        self.loaded_models = models;
    }
    
    /// Add an event
    pub fn add_event(&mut self, event: EventInfo) {
        self.recent_events.push_front(event);
        if self.recent_events.len() > 100 {
            self.recent_events.pop_back();
        }
    }
    
    /// Add a code file
    pub fn add_code(&mut self, code: CodeInfo) {
        self.recent_code.push_front(code);
        if self.recent_code.len() > 100 {
            self.recent_code.pop_back();
        }
    }
    
    /// Clear events
    pub fn clear_events(&mut self) {
        self.recent_events.clear();
    }
    
    /// Update node connections
    pub fn update_node_connections(&mut self, connections: Vec<NodeConnection>) {
        self.node_connections = connections;
    }
}
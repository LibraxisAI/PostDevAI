// PostDevAI - Autonomous RAM-Lake Memory Server for Developer Symbiosis

// Export generated protobuf code
pub mod proto;

// Export core modules
pub mod core {
    pub mod memory {
        pub mod ramlake;
    }
    pub mod indexing;
    pub mod monitoring;
    pub mod network {
        pub mod dragon_node_service;
    }
}

// Export MLX related modules
pub mod mlx {
    pub mod models {
        // Indicates that MLXModelManager is available through FFI as an external type
        extern "Rust" {
            pub type MLXModelManager;
        }
    }
    pub mod embedding;
    pub mod inference;
}

// Export TUI modules
pub mod tui {
    pub mod app;
    pub mod state {
        pub mod app_state;
    }
    pub mod views {
        pub mod dashboard;
        pub mod help;
        pub mod models;
        pub mod ramlake;
        pub mod history;
        pub mod context;
    }
    pub mod bridge {
        pub mod system_bridge;
    }
}

// Export utility modules
pub mod utils {
    pub mod config;
    pub mod filesystem;
    pub mod logging;
}

// Export system types
pub mod system {
    #[derive(Debug, Clone)]
    pub enum NodeType {
        Dragon,
        Developer,
        Coordinator,
    }

    #[derive(Clone)]
    pub struct SystemState {
        pub node_type: NodeType,
        pub hostname: String,
        pub uptime: std::time::Duration,
        pub memory_usage: MemoryUsage,
        pub cpu_usage: f32,
    }

    #[derive(Debug, Clone)]
    pub struct MemoryUsage {
        pub total: u64,
        pub used: u64,
        pub free: u64,
    }

    impl Default for SystemState {
        fn default() -> Self {
            Self {
                node_type: NodeType::Dragon,
                hostname: "unknown".to_string(),
                uptime: std::time::Duration::from_secs(0),
                memory_usage: MemoryUsage {
                    total: 0,
                    used: 0,
                    free: 0,
                },
                cpu_usage: 0.0,
            }
        }
    }
}
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::path::PathBuf;
use parking_lot::RwLock as PLRwLock;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

// Import store implementations from the stores module
mod stores;
use stores::{VectorStore, CodeStore, HistoryStore, MetadataStore, MemoryManager};

/// Main RAM-Lake implementation for PostDevAI
/// Provides high-speed memory storage and indexing
pub struct RamLake {
    /// Base path for the RAM disk mount
    ramdisk_path: PathBuf,
    
    /// Configuration for the RAM-Lake
    config: RamLakeConfig,
    
    /// Memory manager for the RAM-Lake
    memory_manager: Arc<PLRwLock<MemoryManager>>,
    
    /// Vector storage and indices
    vector_store: Arc<PLRwLock<VectorStore>>,
    
    /// Document and code storage
    code_store: Arc<PLRwLock<CodeStore>>,
    
    /// History and event storage
    history_store: Arc<PLRwLock<HistoryStore>>,
    
    /// Metadata and relations storage
    metadata_store: Arc<PLRwLock<MetadataStore>>,
    
    /// Metrics for the RAM-Lake
    metrics: Arc<PLRwLock<RamLakeMetrics>>,
    
    /// Last backup timestamp
    last_backup: Arc<Mutex<Instant>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamLakeConfig {
    /// Maximum size of the RAM-Lake in bytes
    pub max_size: u64,
    
    /// Backup interval in seconds
    pub backup_interval: u64,
    
    /// Path to store backups
    pub backup_path: PathBuf,
    
    /// Percentage allocation for different stores
    pub allocation: StoreAllocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreAllocation {
    /// Percentage for vector store
    pub vector_store: f32,
    
    /// Percentage for code store
    pub code_store: f32,
    
    /// Percentage for history store
    pub history_store: f32,
    
    /// Percentage for metadata store
    pub metadata_store: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamLakeMetrics {
    /// Total RAM-Lake size in bytes
    pub total_size: u64,
    
    /// Used RAM-Lake size in bytes
    pub used_size: u64,
    
    /// Vector store size in bytes
    pub vector_store_size: u64,
    
    /// Code store size in bytes
    pub code_store_size: u64,
    
    /// History store size in bytes
    pub history_store_size: u64,
    
    /// Metadata store size in bytes
    pub metadata_store_size: u64,
    
    /// Number of indexed files
    pub indexed_files: usize,
    
    /// Number of vector entries
    pub vector_entries: usize,
    
    /// Number of history events
    pub history_events: usize,
}

impl RamLake {
    pub fn new(ramdisk_path: PathBuf, config: RamLakeConfig) -> Result<Self, String> {
        // Verify RAM disk exists
        if !ramdisk_path.exists() {
            return Err(format!("RAM disk path does not exist: {:?}", ramdisk_path));
        }
        
        // Create store directories
        let vector_path = ramdisk_path.join("vectors");
        let code_path = ramdisk_path.join("code");
        let history_path = ramdisk_path.join("history");
        let metadata_path = ramdisk_path.join("metadata");
        
        std::fs::create_dir_all(&vector_path)
            .map_err(|e| format!("Failed to create vector directory: {}", e))?;
        std::fs::create_dir_all(&code_path)
            .map_err(|e| format!("Failed to create code directory: {}", e))?;
        std::fs::create_dir_all(&history_path)
            .map_err(|e| format!("Failed to create history directory: {}", e))?;
        std::fs::create_dir_all(&metadata_path)
            .map_err(|e| format!("Failed to create metadata directory: {}", e))?;
        
        // Calculate size allocations
        let total_size = config.max_size;
        let vector_size = (total_size as f64 * config.allocation.vector_store as f64) as u64;
        let code_size = (total_size as f64 * config.allocation.code_store as f64) as u64;
        let history_size = (total_size as f64 * config.allocation.history_store as f64) as u64;
        let metadata_size = (total_size as f64 * config.allocation.metadata_store as f64) as u64;
        
        // Create stores
        let memory_manager = Arc::new(PLRwLock::new(MemoryManager::new(total_size)));
        let vector_store = Arc::new(PLRwLock::new(VectorStore::new(vector_path, vector_size)?));
        let code_store = Arc::new(PLRwLock::new(CodeStore::new(code_path, code_size)?));
        let history_store = Arc::new(PLRwLock::new(HistoryStore::new(history_path, history_size)?));
        let metadata_store = Arc::new(PLRwLock::new(MetadataStore::new(metadata_path, metadata_size)?));
        
        let metrics = Arc::new(PLRwLock::new(RamLakeMetrics {
            total_size,
            used_size: 0,
            vector_store_size: 0,
            code_store_size: 0,
            history_store_size: 0,
            metadata_store_size: 0,
            indexed_files: 0,
            vector_entries: 0,
            history_events: 0,
        }));
        
        Ok(Self {
            ramdisk_path,
            config,
            memory_manager,
            vector_store,
            code_store,
            history_store,
            metadata_store,
            metrics,
            last_backup: Arc::new(Mutex::new(Instant::now())),
        })
    }
    
    /// Start the RAM-Lake background tasks
    pub fn start(&self) -> Result<(), String> {
        // Start backup task
        let backup_interval = Duration::from_secs(self.config.backup_interval);
        let backup_path = self.config.backup_path.clone();
        let last_backup = self.last_backup.clone();
        let ramdisk_path = self.ramdisk_path.clone();
        
        std::thread::spawn(move || {
            loop {
                let now = Instant::now();
                let last = *last_backup.lock().unwrap();
                
                if now.duration_since(last) >= backup_interval {
                    // Perform backup
                    if let Err(e) = Self::backup_ramlake(&ramdisk_path, &backup_path) {
                        eprintln!("Failed to backup RAM-Lake: {}", e);
                    }
                    
                    // Update last backup time
                    *last_backup.lock().unwrap() = Instant::now();
                }
                
                // Sleep for a bit
                std::thread::sleep(Duration::from_secs(1));
            }
        });
        
        // Start metrics collection task
        let metrics = self.metrics.clone();
        let vector_store = self.vector_store.clone();
        let code_store = self.code_store.clone();
        let history_store = self.history_store.clone();
        let metadata_store = self.metadata_store.clone();
        
        std::thread::spawn(move || {
            loop {
                // Update metrics
                let mut m = metrics.write();
                
                m.vector_store_size = vector_store.read().get_size();
                m.code_store_size = code_store.read().get_size();
                m.history_store_size = history_store.read().get_size();
                m.metadata_store_size = metadata_store.read().get_size();
                
                m.used_size = m.vector_store_size + m.code_store_size + m.history_store_size + m.metadata_store_size;
                
                m.indexed_files = code_store.read().get_file_count();
                m.vector_entries = vector_store.read().get_entry_count();
                m.history_events = history_store.read().get_event_count();
                
                // Sleep for a bit
                std::thread::sleep(Duration::from_secs(1));
            }
        });
        
        Ok(())
    }
    
    /// Backup the RAM-Lake to disk
    fn backup_ramlake(ramdisk_path: &PathBuf, backup_path: &PathBuf) -> Result<(), String> {
        // Create backup directory if it doesn't exist
        std::fs::create_dir_all(backup_path)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
        
        // Create a timestamped backup directory
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_dir = backup_path.join(format!("ramlake_backup_{}", timestamp));
        
        std::fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to create timestamped backup directory: {}", e))?;
        
        // Perform rsync-like backup
        let options = fs_extra::dir::CopyOptions::new();
        fs_extra::dir::copy(ramdisk_path, &backup_dir, &options)
            .map_err(|e| format!("Failed to backup RAM-Lake: {}", e))?;
        
        Ok(())
    }
    
    /// Get the current RAM-Lake metrics
    pub fn get_metrics(&self) -> RamLakeMetrics {
        self.metrics.read().clone()
    }
    
    /// Store a code file in the RAM-Lake
    pub fn store_code(&self, path: &str, content: &str, language: &str) -> Result<Uuid, String> {
        // Generate a unique ID for this code
        let id = Uuid::new_v4();
        
        // Store the code
        let mut code_store = self.code_store.write();
        code_store.store_file(id, path, content, language)?;
        
        // Update memory manager
        let mut memory_manager = self.memory_manager.write();
        memory_manager.allocate_with_source(content.len() as u64, &format!("code:{}", path))
            .map_err(|e| format!("Failed to allocate memory: {}", e))?;
        
        Ok(id)
    }
    
    /// Index a code file for vector search
    pub fn index_code(&self, code_id: Uuid, embeddings: Vec<f32>) -> Result<(), String> {
        // Store the embedding
        let mut vector_store = self.vector_store.write();
        vector_store.store_embedding(code_id, embeddings)?;
        
        Ok(())
    }
    
    /// Store a terminal or system event in history
    pub fn store_event(&self, event_type: &str, content: &str) -> Result<Uuid, String> {
        // Generate a unique ID for this event
        let id = Uuid::new_v4();
        
        // Store the event
        let mut history_store = self.history_store.write();
        history_store.store_event(id, event_type, content)?;
        
        // Update memory manager
        let mut memory_manager = self.memory_manager.write();
        memory_manager.allocate_with_source(content.len() as u64, &format!("event:{}", event_type))
            .map_err(|e| format!("Failed to allocate memory: {}", e))?;
        
        Ok(id)
    }
    
    /// Store metadata about relations between entities
    pub fn store_metadata(&self, source_id: Uuid, relation: &str, target_id: Uuid) -> Result<(), String> {
        // Store the metadata
        let mut metadata_store = self.metadata_store.write();
        metadata_store.store_relation(source_id, relation, target_id)?;
        
        Ok(())
    }
    
    /// Search for similar code by vector embedding
    pub fn search_similar(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Uuid, f32)>, String> {
        // Perform vector search
        let vector_store = self.vector_store.read();
        let results = vector_store.search_similar(embedding, limit)?;
        
        Ok(results)
    }
    
    /// Get a code file by ID
    pub fn get_code(&self, id: Uuid) -> Result<(String, String, String), String> {
        // Get the code
        let code_store = self.code_store.read();
        code_store.get_file(id)
    }
    
    /// Get event by ID
    pub fn get_event(&self, id: Uuid) -> Result<(String, String, chrono::DateTime<chrono::Local>), String> {
        // Get the event
        let history_store = self.history_store.read();
        history_store.get_event(id)
    }
    
    /// Get related entities by ID and relation type
    pub fn get_related(&self, id: Uuid, relation: Option<&str>) -> Result<Vec<(Uuid, String, Uuid)>, String> {
        // Get related entities
        let metadata_store = self.metadata_store.read();
        metadata_store.get_relations(id, relation)
    }
}
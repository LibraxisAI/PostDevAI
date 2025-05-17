use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use parking_lot::RwLock;
use chrono::{DateTime, Utc, TimeZone};

/// History Store for RAM-Lake
/// 
/// Stores event history for terminal, logs, errors, etc.
pub struct HistoryStore {
    /// Path to store events
    path: PathBuf,
    
    /// Maximum size of the store in bytes
    max_size: u64,
    
    /// Current size of the store in bytes
    current_size: u64,
    
    /// Index of events
    index: RwLock<EventIndex>,
    
    /// Mapping of UUIDs to event metadata
    metadata: RwLock<HashMap<Uuid, EventMetadata>>,
}

/// Event Index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventIndex {
    /// Number of events
    pub count: usize,
    
    /// Index version
    pub version: u32,
    
    /// UUIDs of events in chronological order
    pub ids: Vec<Uuid>,
    
    /// Type to UUIDs mapping
    pub type_map: HashMap<String, Vec<Uuid>>,
}

/// Event Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// ID of the event
    pub id: Uuid,
    
    /// Type of the event
    pub event_type: String,
    
    /// Size of the event content in bytes
    pub size: u64,
    
    /// Path to the event file in the store
    pub file_path: String,
    
    /// Creation timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Source of the event (e.g., terminal, IDE, etc.)
    pub source: Option<String>,
    
    /// Severity of the event (e.g., info, warning, error)
    pub severity: Option<String>,
}

/// Event with content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Metadata of the event
    pub metadata: EventMetadata,
    
    /// Content of the event
    pub content: String,
}

impl HistoryStore {
    /// Create a new history store
    pub fn new(path: PathBuf, max_size: u64) -> Result<Self, String> {
        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create history store directory: {}", e))?;
        }
        
        // Load or create index
        let index_path = path.join("index.json");
        let index = if index_path.exists() {
            let file = fs::File::open(&index_path)
                .map_err(|e| format!("Failed to open index file: {}", e))?;
            serde_json::from_reader(file)
                .map_err(|e| format!("Failed to parse index file: {}", e))?
        } else {
            EventIndex {
                count: 0,
                version: 1,
                ids: Vec::new(),
                type_map: HashMap::new(),
            }
        };
        
        // Load metadata
        let metadata_path = path.join("metadata.json");
        let metadata = if metadata_path.exists() {
            let file = fs::File::open(&metadata_path)
                .map_err(|e| format!("Failed to open metadata file: {}", e))?;
            serde_json::from_reader(file)
                .map_err(|e| format!("Failed to parse metadata file: {}", e))?
        } else {
            HashMap::new()
        };
        
        // Calculate current size
        let mut current_size = 0;
        for entry in fs::read_dir(&path).map_err(|e| format!("Failed to read history store directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let metadata = entry.metadata().map_err(|e| format!("Failed to read file metadata: {}", e))?;
            current_size += metadata.len();
        }
        
        Ok(Self {
            path,
            max_size,
            current_size,
            index: RwLock::new(index),
            metadata: RwLock::new(metadata),
        })
    }
    
    /// Store an event
    pub fn store_event(&mut self, id: Uuid, event_type: &str, content: &str) -> Result<(), String> {
        // Calculate size
        let content_size = content.len() as u64;
        
        // Check if we have enough space
        if self.current_size + content_size > self.max_size {
            // Try to free up space by removing oldest events
            self.remove_oldest_events(content_size)?;
            
            // Check again
            if self.current_size + content_size > self.max_size {
                return Err("Not enough space in history store".to_string());
            }
        }
        
        // Generate file path
        let file_name = format!("{}.event", id);
        let file_path = self.path.join(&file_name);
        
        // Write content to file
        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("Failed to create event file: {}", e))?;
        
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write event content: {}", e))?;
        
        // Create metadata
        let now = Utc::now();
        let metadata = EventMetadata {
            id,
            event_type: event_type.to_string(),
            size: content_size,
            file_path: file_name,
            timestamp: now,
            source: None,
            severity: None,
        };
        
        // Update index
        {
            let mut index = self.index.write();
            index.ids.push(id);
            
            // Add to type map
            index.type_map.entry(event_type.to_string())
                .or_insert_with(Vec::new)
                .push(id);
            
            index.count += 1;
            index.version += 1;
        }
        
        // Update metadata
        {
            let mut metadata_lock = self.metadata.write();
            metadata_lock.insert(id, metadata);
        }
        
        // Update size
        self.current_size += content_size;
        
        // Persist index and metadata
        self.persist_index()?;
        self.persist_metadata()?;
        
        Ok(())
    }
    
    /// Store an event with additional metadata
    pub fn store_event_with_metadata(
        &mut self,
        id: Uuid,
        event_type: &str,
        content: &str,
        source: Option<&str>,
        severity: Option<&str>,
    ) -> Result<(), String> {
        // Calculate size
        let content_size = content.len() as u64;
        
        // Check if we have enough space
        if self.current_size + content_size > self.max_size {
            // Try to free up space by removing oldest events
            self.remove_oldest_events(content_size)?;
            
            // Check again
            if self.current_size + content_size > self.max_size {
                return Err("Not enough space in history store".to_string());
            }
        }
        
        // Generate file path
        let file_name = format!("{}.event", id);
        let file_path = self.path.join(&file_name);
        
        // Write content to file
        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("Failed to create event file: {}", e))?;
        
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write event content: {}", e))?;
        
        // Create metadata
        let now = Utc::now();
        let metadata = EventMetadata {
            id,
            event_type: event_type.to_string(),
            size: content_size,
            file_path: file_name,
            timestamp: now,
            source: source.map(|s| s.to_string()),
            severity: severity.map(|s| s.to_string()),
        };
        
        // Update index
        {
            let mut index = self.index.write();
            index.ids.push(id);
            
            // Add to type map
            index.type_map.entry(event_type.to_string())
                .or_insert_with(Vec::new)
                .push(id);
            
            index.count += 1;
            index.version += 1;
        }
        
        // Update metadata
        {
            let mut metadata_lock = self.metadata.write();
            metadata_lock.insert(id, metadata);
        }
        
        // Update size
        self.current_size += content_size;
        
        // Persist index and metadata
        self.persist_index()?;
        self.persist_metadata()?;
        
        Ok(())
    }
    
    /// Remove oldest events to free up space
    fn remove_oldest_events(&mut self, required_space: u64) -> Result<(), String> {
        // Calculate how much space to free
        let space_to_free = required_space;
        
        // Get oldest events
        let mut oldest_events = Vec::new();
        {
            let index = self.index.read();
            let metadata = self.metadata.read();
            
            // Get oldest events first
            for &id in &index.ids {
                if let Some(event_metadata) = metadata.get(&id) {
                    oldest_events.push((id, event_metadata.timestamp, event_metadata.size));
                }
            }
        }
        
        // Sort by timestamp (oldest first)
        oldest_events.sort_by(|a, b| a.1.cmp(&b.1));
        
        // Remove events until we have freed enough space
        let mut freed_space = 0;
        let mut removed_ids = Vec::new();
        
        for (id, _, size) in oldest_events {
            if freed_space >= space_to_free {
                break;
            }
            
            // Remove event
            self.delete_event(id)?;
            
            freed_space += size;
            removed_ids.push(id);
        }
        
        Ok(())
    }
    
    /// Persist index to disk
    fn persist_index(&self) -> Result<(), String> {
        let index_path = self.path.join("index.json");
        let index = self.index.read();
        
        let file = fs::File::create(&index_path)
            .map_err(|e| format!("Failed to create index file: {}", e))?;
        
        serde_json::to_writer_pretty(file, &*index)
            .map_err(|e| format!("Failed to write index file: {}", e))?;
        
        Ok(())
    }
    
    /// Persist metadata to disk
    fn persist_metadata(&self) -> Result<(), String> {
        let metadata_path = self.path.join("metadata.json");
        let metadata = self.metadata.read();
        
        let file = fs::File::create(&metadata_path)
            .map_err(|e| format!("Failed to create metadata file: {}", e))?;
        
        serde_json::to_writer_pretty(file, &*metadata)
            .map_err(|e| format!("Failed to write metadata file: {}", e))?;
        
        Ok(())
    }
    
    /// Get an event by UUID
    pub fn get_event(&self, id: Uuid) -> Result<(String, String, DateTime<chrono::Local>), String> {
        // Get metadata
        let metadata_lock = self.metadata.read();
        let metadata = metadata_lock.get(&id)
            .ok_or_else(|| format!("Event with ID {} not found", id))?;
        
        // Open file
        let file_path = self.path.join(&metadata.file_path);
        let mut file = fs::File::open(&file_path)
            .map_err(|e| format!("Failed to open event file: {}", e))?;
        
        // Read content
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read event content: {}", e))?;
        
        // Convert UTC timestamp to local time
        let local_time = chrono::Local.from_utc_datetime(&metadata.timestamp.naive_utc());
        
        Ok((metadata.event_type.clone(), content, local_time))
    }
    
    /// Get event metadata by UUID
    pub fn get_event_metadata(&self, id: Uuid) -> Result<EventMetadata, String> {
        let metadata_lock = self.metadata.read();
        metadata_lock.get(&id)
            .cloned()
            .ok_or_else(|| format!("Event with ID {} not found", id))
    }
    
    /// Delete an event
    pub fn delete_event(&mut self, id: Uuid) -> Result<(), String> {
        // Get metadata
        let mut metadata_lock = self.metadata.write();
        let metadata = metadata_lock.get(&id)
            .ok_or_else(|| format!("Event with ID {} not found", id))?;
        
        // Store event type for index update
        let event_type = metadata.event_type.clone();
        
        // Remove file
        let file_path = self.path.join(&metadata.file_path);
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to remove event file: {}", e))?;
        
        // Update size
        self.current_size -= metadata.size;
        
        // Remove from metadata
        metadata_lock.remove(&id);
        drop(metadata_lock);
        
        // Update index
        {
            let mut index = self.index.write();
            index.ids.retain(|&i| i != id);
            
            // Remove from type map
            if let Some(events) = index.type_map.get_mut(&event_type) {
                events.retain(|&i| i != id);
                
                // Remove empty type entries
                if events.is_empty() {
                    index.type_map.remove(&event_type);
                }
            }
            
            index.count -= 1;
            index.version += 1;
        }
        
        // Persist index and metadata
        self.persist_index()?;
        self.persist_metadata()?;
        
        Ok(())
    }
    
    /// Get the size of the store
    pub fn get_size(&self) -> u64 {
        self.current_size
    }
    
    /// Get the number of events
    pub fn get_event_count(&self) -> usize {
        self.index.read().count
    }
    
    /// Find events by type
    pub fn find_events_by_type(&self, event_type: &str) -> Vec<Uuid> {
        let index = self.index.read();
        index.type_map.get(event_type)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Find events by timestamp range
    pub fn find_events_by_timestamp_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Uuid> {
        let metadata_lock = self.metadata.read();
        metadata_lock.iter()
            .filter(|&(_, metadata)| {
                metadata.timestamp >= start && metadata.timestamp <= end
            })
            .map(|(&id, _)| id)
            .collect()
    }
    
    /// Find events by severity
    pub fn find_events_by_severity(&self, severity: &str) -> Vec<Uuid> {
        let metadata_lock = self.metadata.read();
        metadata_lock.iter()
            .filter(|&(_, metadata)| {
                metadata.severity.as_ref().map_or(false, |s| s == severity)
            })
            .map(|(&id, _)| id)
            .collect()
    }
    
    /// Find events by source
    pub fn find_events_by_source(&self, source: &str) -> Vec<Uuid> {
        let metadata_lock = self.metadata.read();
        metadata_lock.iter()
            .filter(|&(_, metadata)| {
                metadata.source.as_ref().map_or(false, |s| s == source)
            })
            .map(|(&id, _)| id)
            .collect()
    }
    
    /// Get all event metadata
    pub fn get_all_metadata(&self) -> Vec<EventMetadata> {
        let metadata_lock = self.metadata.read();
        metadata_lock.values().cloned().collect()
    }
    
    /// Get recent events
    pub fn get_recent_events(&self, limit: usize) -> Vec<(Uuid, EventMetadata)> {
        let mut events = Vec::new();
        
        let index = self.index.read();
        let metadata = self.metadata.read();
        
        // Get the specified number of most recent events
        for &id in index.ids.iter().rev().take(limit) {
            if let Some(event_metadata) = metadata.get(&id) {
                events.push((id, event_metadata.clone()));
            }
        }
        
        events
    }
    
    /// Export event history to JSON
    pub fn export_to_json(&self, path: &str) -> Result<(), String> {
        // Load all events
        let mut events = Vec::new();
        
        let index = self.index.read();
        let metadata = self.metadata.read();
        
        for &id in &index.ids {
            if let Some(event_metadata) = metadata.get(&id) {
                // Open file
                let file_path = self.path.join(&event_metadata.file_path);
                let mut file = fs::File::open(&file_path)
                    .map_err(|e| format!("Failed to open event file: {}", e))?;
                
                // Read content
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| format!("Failed to read event content: {}", e))?;
                
                // Create event
                let event = Event {
                    metadata: event_metadata.clone(),
                    content,
                };
                
                events.push(event);
            }
        }
        
        // Write to file
        let file = fs::File::create(path)
            .map_err(|e| format!("Failed to create export file: {}", e))?;
        
        serde_json::to_writer_pretty(file, &events)
            .map_err(|e| format!("Failed to write export file: {}", e))?;
        
        Ok(())
    }
}
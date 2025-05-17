use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use parking_lot::RwLock;

/// Code Store for RAM-Lake
/// 
/// Stores code files and their metadata
pub struct CodeStore {
    /// Path to store code files
    path: PathBuf,
    
    /// Maximum size of the store in bytes
    max_size: u64,
    
    /// Current size of the store in bytes
    current_size: u64,
    
    /// Index of code files
    index: RwLock<CodeIndex>,
    
    /// Mapping of UUIDs to code metadata
    metadata: RwLock<HashMap<Uuid, CodeMetadata>>,
}

/// Code Index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIndex {
    /// Number of code files
    pub count: usize,
    
    /// Index version
    pub version: u32,
    
    /// UUIDs of code files
    pub ids: Vec<Uuid>,
    
    /// Path to UUID mapping
    pub path_map: HashMap<String, Uuid>,
}

/// Code Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetadata {
    /// ID of the code file
    pub id: Uuid,
    
    /// Path of the code file
    pub path: String,
    
    /// Programming language
    pub language: String,
    
    /// Size of the file in bytes
    pub size: u64,
    
    /// Path to the code file in the store
    pub file_path: String,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    
    /// SHA-256 hash of the content
    pub hash: String,
}

impl CodeStore {
    /// Create a new code store
    pub fn new(path: PathBuf, max_size: u64) -> Result<Self, String> {
        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create code store directory: {}", e))?;
        }
        
        // Load or create index
        let index_path = path.join("index.json");
        let index = if index_path.exists() {
            let file = fs::File::open(&index_path)
                .map_err(|e| format!("Failed to open index file: {}", e))?;
            serde_json::from_reader(file)
                .map_err(|e| format!("Failed to parse index file: {}", e))?
        } else {
            CodeIndex {
                count: 0,
                version: 1,
                ids: Vec::new(),
                path_map: HashMap::new(),
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
        for entry in fs::read_dir(&path).map_err(|e| format!("Failed to read code store directory: {}", e))? {
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
    
    /// Store a code file
    pub fn store_file(&mut self, id: Uuid, path: &str, content: &str, language: &str) -> Result<(), String> {
        // Calculate size
        let content_size = content.len() as u64;
        
        // Check if we have enough space
        if self.current_size + content_size > self.max_size {
            return Err("Not enough space in code store".to_string());
        }
        
        // Check if path already exists (and get existing ID if it does)
        let existing_id = {
            let index = self.index.read();
            index.path_map.get(path).cloned()
        };
        
        // If path exists, need to delete old file first
        if let Some(existing_id) = existing_id {
            self.delete_file(existing_id)?;
        }
        
        // Generate file path
        let file_name = format!("{}.code", id);
        let file_path = self.path.join(&file_name);
        
        // Write content to file
        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("Failed to create code file: {}", e))?;
        
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write code content: {}", e))?;
        
        // Calculate hash
        let hash = sha256::digest(content);
        
        // Create metadata
        let now = chrono::Utc::now();
        let metadata = CodeMetadata {
            id,
            path: path.to_string(),
            language: language.to_string(),
            size: content_size,
            file_path: file_name,
            created_at: now,
            modified_at: now,
            hash,
        };
        
        // Update index
        {
            let mut index = self.index.write();
            index.ids.push(id);
            index.path_map.insert(path.to_string(), id);
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
    
    /// Get a code file by UUID
    pub fn get_file(&self, id: Uuid) -> Result<(String, String, String), String> {
        // Get metadata
        let metadata_lock = self.metadata.read();
        let metadata = metadata_lock.get(&id)
            .ok_or_else(|| format!("Code file with ID {} not found", id))?;
        
        // Open file
        let file_path = self.path.join(&metadata.file_path);
        let mut file = fs::File::open(&file_path)
            .map_err(|e| format!("Failed to open code file: {}", e))?;
        
        // Read content
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read code content: {}", e))?;
        
        Ok((metadata.path.clone(), content, metadata.language.clone()))
    }
    
    /// Get code file metadata by UUID
    pub fn get_file_metadata(&self, id: Uuid) -> Result<CodeMetadata, String> {
        let metadata_lock = self.metadata.read();
        metadata_lock.get(&id)
            .cloned()
            .ok_or_else(|| format!("Code file with ID {} not found", id))
    }
    
    /// Get code file by path
    pub fn get_file_by_path(&self, path: &str) -> Result<(Uuid, String, String), String> {
        // Get UUID from path
        let id = {
            let index = self.index.read();
            index.path_map.get(path)
                .cloned()
                .ok_or_else(|| format!("Code file with path {} not found", path))?
        };
        
        // Get file
        let (_, content, language) = self.get_file(id)?;
        
        Ok((id, content, language))
    }
    
    /// Delete a code file
    pub fn delete_file(&mut self, id: Uuid) -> Result<(), String> {
        // Get metadata
        let mut metadata_lock = self.metadata.write();
        let metadata = metadata_lock.get(&id)
            .ok_or_else(|| format!("Code file with ID {} not found", id))?;
        
        // Remove file
        let file_path = self.path.join(&metadata.file_path);
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to remove code file: {}", e))?;
        
        // Get path for index update
        let path = metadata.path.clone();
        
        // Update size
        self.current_size -= metadata.size;
        
        // Remove from metadata
        metadata_lock.remove(&id);
        drop(metadata_lock);
        
        // Update index
        {
            let mut index = self.index.write();
            index.ids.retain(|&i| i != id);
            index.path_map.remove(&path);
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
    
    /// Get the number of files
    pub fn get_file_count(&self) -> usize {
        self.index.read().count
    }
    
    /// Find files by language
    pub fn find_files_by_language(&self, language: &str) -> Vec<Uuid> {
        let metadata_lock = self.metadata.read();
        metadata_lock.iter()
            .filter(|&(_, metadata)| metadata.language == language)
            .map(|(&id, _)| id)
            .collect()
    }
    
    /// Find files by path pattern
    pub fn find_files_by_path_pattern(&self, pattern: &str) -> Vec<Uuid> {
        // Simple glob-like pattern matching with * wildcard
        let regex_pattern = pattern.replace("*", ".*");
        let regex = regex::Regex::new(&format!("^{}$", regex_pattern)).unwrap_or_else(|_| {
            // Fallback to exact match if regex is invalid
            regex::Regex::new(&format!("^{}$", regex::escape(pattern))).unwrap()
        });
        
        let index = self.index.read();
        index.path_map.iter()
            .filter(|&(path, _)| regex.is_match(path))
            .map(|(_, &id)| id)
            .collect()
    }
    
    /// Find files modified after a certain time
    pub fn find_files_modified_after(&self, timestamp: chrono::DateTime<chrono::Utc>) -> Vec<Uuid> {
        let metadata_lock = self.metadata.read();
        metadata_lock.iter()
            .filter(|&(_, metadata)| metadata.modified_at > timestamp)
            .map(|(&id, _)| id)
            .collect()
    }
    
    /// Update a code file
    pub fn update_file(&mut self, id: Uuid, content: &str) -> Result<(), String> {
        // Get metadata
        let mut metadata_lock = self.metadata.write();
        let metadata = metadata_lock.get_mut(&id)
            .ok_or_else(|| format!("Code file with ID {} not found", id))?;
        
        // Calculate size difference
        let old_size = metadata.size;
        let new_size = content.len() as u64;
        let size_diff = new_size as i64 - old_size as i64;
        
        // Check if we have enough space for the size increase
        if size_diff > 0 && self.current_size + size_diff as u64 > self.max_size {
            return Err("Not enough space in code store".to_string());
        }
        
        // Open file
        let file_path = self.path.join(&metadata.file_path);
        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("Failed to open code file: {}", e))?;
        
        // Write content
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write code content: {}", e))?;
        
        // Update metadata
        metadata.size = new_size;
        metadata.modified_at = chrono::Utc::now();
        metadata.hash = sha256::digest(content);
        
        // Update size
        if size_diff > 0 {
            self.current_size += size_diff as u64;
        } else {
            self.current_size -= (-size_diff) as u64;
        }
        
        // Persist metadata
        self.persist_metadata()?;
        
        Ok(())
    }
    
    /// Get all file metadata
    pub fn get_all_metadata(&self) -> Vec<CodeMetadata> {
        let metadata_lock = self.metadata.read();
        metadata_lock.values().cloned().collect()
    }
}
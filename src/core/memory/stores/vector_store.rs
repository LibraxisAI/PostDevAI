use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use parking_lot::RwLock;

/// Vector Store for RAM-Lake
/// 
/// Stores and indexes embeddings for vector search
pub struct VectorStore {
    /// Path to store embeddings
    path: PathBuf,
    
    /// Maximum size of the store in bytes
    max_size: u64,
    
    /// Current size of the store in bytes
    current_size: u64,
    
    /// Index of embeddings
    index: RwLock<VectorIndex>,
    
    /// Mapping of UUIDs to embedding metadata
    metadata: RwLock<HashMap<Uuid, EmbeddingMetadata>>,
    
    /// FAISS index
    // Tymczasowo wyłączone z powodu braku feature "static" w faiss
    // #[cfg(feature = "faiss")]
    // faiss_index: RwLock<Option<faiss::Index>>,
}

/// Vector Index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndex {
    /// Dimension of embeddings
    pub dimension: usize,
    
    /// Number of embeddings
    pub count: usize,
    
    /// Index version
    pub version: u32,
    
    /// UUIDs of embeddings in order
    pub ids: Vec<Uuid>,
}

/// Embedding Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    /// ID of the embedding
    pub id: Uuid,
    
    /// Source ID (e.g., code ID, event ID)
    pub source_id: Uuid,
    
    /// Type of embedding (e.g., "code", "text", "event")
    pub embedding_type: String,
    
    /// Dimension of the embedding
    pub dimension: usize,
    
    /// Path to the embedding file
    pub file_path: String,
    
    /// Size of the embedding in bytes
    pub size: u64,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl VectorStore {
    /// Create a new vector store
    pub fn new(path: PathBuf, max_size: u64) -> Result<Self, String> {
        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create vector store directory: {}", e))?;
        }
        
        // Load or create index
        let index_path = path.join("index.json");
        let index = if index_path.exists() {
            let file = fs::File::open(&index_path)
                .map_err(|e| format!("Failed to open index file: {}", e))?;
            serde_json::from_reader(file)
                .map_err(|e| format!("Failed to parse index file: {}", e))?
        } else {
            VectorIndex {
                dimension: 0,
                count: 0,
                version: 1,
                ids: Vec::new(),
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
        for entry in fs::read_dir(&path).map_err(|e| format!("Failed to read vector store directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let metadata = entry.metadata().map_err(|e| format!("Failed to read file metadata: {}", e))?;
            current_size += metadata.len();
        }
        
        // Initialize FAISS index if enabled
        #[cfg(feature = "faiss")]
        let faiss_index = {
            let faiss_path = path.join("faiss.index");
            let index = if faiss_path.exists() && index.dimension > 0 {
                let mut index = faiss::Index::new_with_dimension(index.dimension as i32)
                    .map_err(|e| format!("Failed to create FAISS index: {}", e))?;
                index.read_index(faiss_path.to_str().unwrap())
                    .map_err(|e| format!("Failed to read FAISS index: {}", e))?;
                Some(index)
            } else {
                None
            };
            RwLock::new(index)
        };
        
        Ok(Self {
            path,
            max_size,
            current_size,
            index: RwLock::new(index),
            metadata: RwLock::new(metadata),
            #[cfg(feature = "faiss")]
            faiss_index,
        })
    }
    
    /// Store an embedding
    pub fn store_embedding(&mut self, id: Uuid, embedding: Vec<f32>) -> Result<(), String> {
        // Check if embedding already exists
        let metadata_lock = self.metadata.read();
        if metadata_lock.contains_key(&id) {
            return Err(format!("Embedding with ID {} already exists", id));
        }
        drop(metadata_lock);
        
        // Calculate size
        let embedding_size = (embedding.len() * std::mem::size_of::<f32>()) as u64;
        
        // Check if we have enough space
        if self.current_size + embedding_size > self.max_size {
            return Err("Not enough space in vector store".to_string());
        }
        
        // Generate file path
        let file_name = format!("{}.vec", id);
        let file_path = self.path.join(&file_name);
        
        // Write embedding to file
        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("Failed to create embedding file: {}", e))?;
        
        // Write embedding dimensions as header
        let dimension = embedding.len() as u32;
        file.write_all(&dimension.to_le_bytes())
            .map_err(|e| format!("Failed to write dimension header: {}", e))?;
        
        // Write embedding data
        for &value in &embedding {
            file.write_all(&value.to_le_bytes())
                .map_err(|e| format!("Failed to write embedding data: {}", e))?;
        }
        
        // Create metadata
        let metadata = EmbeddingMetadata {
            id,
            source_id: id, // Default to same ID, can be updated later
            embedding_type: "unknown".to_string(),
            dimension: embedding.len(),
            file_path: file_name,
            size: embedding_size,
            created_at: chrono::Utc::now(),
        };
        
        // Update index
        {
            let mut index = self.index.write();
            
            // Set dimension if this is the first embedding
            if index.count == 0 {
                index.dimension = embedding.len();
            } else if index.dimension != embedding.len() {
                return Err(format!(
                    "Embedding dimension mismatch. Expected {}, got {}",
                    index.dimension, embedding.len()
                ));
            }
            
            index.ids.push(id);
            index.count += 1;
            index.version += 1;
        }
        
        // Add to FAISS index if enabled
        #[cfg(feature = "faiss")]
        {
            let mut faiss_index = self.faiss_index.write();
            
            // Create FAISS index if it doesn't exist
            if faiss_index.is_none() {
                *faiss_index = Some(
                    faiss::Index::new_with_dimension(embedding.len() as i32)
                        .map_err(|e| format!("Failed to create FAISS index: {}", e))?,
                );
            }
            
            // Add embedding to FAISS index
            if let Some(index) = faiss_index.as_mut() {
                index.add_with_ids(
                    &embedding,
                    &[index.ntotal() as i64],
                ).map_err(|e| format!("Failed to add embedding to FAISS index: {}", e))?;
            }
        }
        
        // Update metadata
        {
            let mut metadata_lock = self.metadata.write();
            metadata_lock.insert(id, metadata);
        }
        
        // Update size
        self.current_size += embedding_size;
        
        // Persist index and metadata
        self.persist_index()?;
        self.persist_metadata()?;
        
        #[cfg(feature = "faiss")]
        self.persist_faiss_index()?;
        
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
    
    /// Persist FAISS index to disk
    #[cfg(feature = "faiss")]
    fn persist_faiss_index(&self) -> Result<(), String> {
        let faiss_path = self.path.join("faiss.index");
        let faiss_index = self.faiss_index.read();
        
        if let Some(index) = faiss_index.as_ref() {
            index.write_index(faiss_path.to_str().unwrap())
                .map_err(|e| format!("Failed to write FAISS index: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Load embedding from disk
    pub fn load_embedding(&self, id: Uuid) -> Result<Vec<f32>, String> {
        // Get metadata
        let metadata_lock = self.metadata.read();
        let metadata = metadata_lock.get(&id)
            .ok_or_else(|| format!("Embedding with ID {} not found", id))?;
        
        // Open file
        let file_path = self.path.join(&metadata.file_path);
        let mut file = fs::File::open(&file_path)
            .map_err(|e| format!("Failed to open embedding file: {}", e))?;
        
        // Read dimension header
        let mut dimension_bytes = [0u8; 4];
        file.read_exact(&mut dimension_bytes)
            .map_err(|e| format!("Failed to read dimension header: {}", e))?;
        let dimension = u32::from_le_bytes(dimension_bytes) as usize;
        
        // Verify dimension
        if dimension != metadata.dimension {
            return Err(format!(
                "Embedding dimension mismatch. Expected {}, got {}",
                metadata.dimension, dimension
            ));
        }
        
        // Read embedding data
        let mut embedding = Vec::with_capacity(dimension);
        for _ in 0..dimension {
            let mut value_bytes = [0u8; 4];
            file.read_exact(&mut value_bytes)
                .map_err(|e| format!("Failed to read embedding data: {}", e))?;
            let value = f32::from_le_bytes(value_bytes);
            embedding.push(value);
        }
        
        Ok(embedding)
    }
    
    /// Search for similar embeddings
    pub fn search_similar(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Uuid, f32)>, String> {
        // Check dimension
        let index = self.index.read();
        if index.dimension != embedding.len() {
            return Err(format!(
                "Embedding dimension mismatch. Expected {}, got {}",
                index.dimension, embedding.len()
            ));
        }
        
        // If no embeddings, return empty results
        if index.count == 0 {
            return Ok(Vec::new());
        }
        
        // Use FAISS if enabled
        #[cfg(feature = "faiss")]
        {
            let faiss_index = self.faiss_index.read();
            if let Some(index) = faiss_index.as_ref() {
                let (distances, indices) = index.search(&embedding, limit as i64)
                    .map_err(|e| format!("Failed to search with FAISS: {}", e))?;
                
                // Convert results
                let mut results = Vec::with_capacity(limit);
                for i in 0..indices.len() {
                    let idx = indices[i];
                    if idx >= 0 && idx < index.ids.len() as i64 {
                        let id = index.ids[idx as usize];
                        let distance = distances[i];
                        // Convert distance to similarity score (lower distance = higher similarity)
                        let similarity = 1.0 / (1.0 + distance);
                        results.push((id, similarity));
                    }
                }
                
                return Ok(results);
            }
        }
        
        // Fall back to brute force search
        self.brute_force_search(embedding, limit)
    }
    
    /// Brute force search for similar embeddings
    fn brute_force_search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Uuid, f32)>, String> {
        let index = self.index.read();
        let mut results = Vec::with_capacity(index.count.min(limit));
        
        // Calculate similarity for each embedding
        for &id in &index.ids {
            let stored_embedding = self.load_embedding(id)?;
            let similarity = self.cosine_similarity(&embedding, &stored_embedding);
            results.push((id, similarity));
        }
        
        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit results
        results.truncate(limit);
        
        Ok(results)
    }
    
    /// Calculate cosine similarity
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        
        for i in 0..a.len() {
            dot_product += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        
        let norm_a = norm_a.sqrt();
        let norm_b = norm_b.sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }
    
    /// Get the size of the store
    pub fn get_size(&self) -> u64 {
        self.current_size
    }
    
    /// Get the number of entries
    pub fn get_entry_count(&self) -> usize {
        self.index.read().count
    }
    
    /// Delete an embedding
    pub fn delete_embedding(&mut self, id: Uuid) -> Result<(), String> {
        // Get metadata
        let mut metadata_lock = self.metadata.write();
        let metadata = metadata_lock.get(&id)
            .ok_or_else(|| format!("Embedding with ID {} not found", id))?;
        
        // Calculate size
        let embedding_size = metadata.size;
        
        // Remove file
        let file_path = self.path.join(&metadata.file_path);
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to remove embedding file: {}", e))?;
        
        // Update index
        {
            let mut index = self.index.write();
            index.ids.retain(|&i| i != id);
            index.count -= 1;
            index.version += 1;
        }
        
        // Remove from metadata
        metadata_lock.remove(&id);
        
        // Update size
        self.current_size -= embedding_size;
        
        // Rebuild FAISS index if enabled
        #[cfg(feature = "faiss")]
        {
            // Rebuilding FAISS index means adding all embeddings again
            let mut faiss_index = self.faiss_index.write();
            if faiss_index.is_some() {
                let dimension = self.index.read().dimension;
                *faiss_index = Some(
                    faiss::Index::new_with_dimension(dimension as i32)
                        .map_err(|e| format!("Failed to create FAISS index: {}", e))?,
                );
                
                // Re-add all embeddings
                let index = self.index.read();
                for (i, &id) in index.ids.iter().enumerate() {
                    let embedding = self.load_embedding(id)?;
                    faiss_index.as_mut().unwrap().add_with_ids(
                        &embedding,
                        &[i as i64],
                    ).map_err(|e| format!("Failed to add embedding to FAISS index: {}", e))?;
                }
            }
        }
        
        // Persist index and metadata
        self.persist_index()?;
        self.persist_metadata()?;
        
        #[cfg(feature = "faiss")]
        self.persist_faiss_index()?;
        
        Ok(())
    }
}
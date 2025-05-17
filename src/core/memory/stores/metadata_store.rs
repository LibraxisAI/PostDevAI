use std::path::PathBuf;
use std::fs;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use parking_lot::RwLock;

/// Metadata Store for RAM-Lake
/// 
/// Stores metadata and relations between entities
pub struct MetadataStore {
    /// Path to store metadata
    path: PathBuf,
    
    /// Maximum size of the store in bytes
    max_size: u64,
    
    /// Current size of the store in bytes
    current_size: u64,
    
    /// Relations between entities
    relations: RwLock<RelationGraph>,
}

/// Relation Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationGraph {
    /// Number of relations
    pub count: usize,
    
    /// Version of the graph
    pub version: u32,
    
    /// Forward relations (source -> relation -> targets)
    pub forward: HashMap<Uuid, HashMap<String, HashSet<Uuid>>>,
    
    /// Backward relations (target -> relation -> sources)
    pub backward: HashMap<Uuid, HashMap<String, HashSet<Uuid>>>,
    
    /// All relations (source, relation, target)
    pub all_relations: Vec<(Uuid, String, Uuid)>,
}

impl MetadataStore {
    /// Create a new metadata store
    pub fn new(path: PathBuf, max_size: u64) -> Result<Self, String> {
        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create metadata store directory: {}", e))?;
        }
        
        // Load or create relation graph
        let relations_path = path.join("relations.json");
        let relations = if relations_path.exists() {
            let file = fs::File::open(&relations_path)
                .map_err(|e| format!("Failed to open relations file: {}", e))?;
            serde_json::from_reader(file)
                .map_err(|e| format!("Failed to parse relations file: {}", e))?
        } else {
            RelationGraph {
                count: 0,
                version: 1,
                forward: HashMap::new(),
                backward: HashMap::new(),
                all_relations: Vec::new(),
            }
        };
        
        // Calculate current size
        let current_size = if relations_path.exists() {
            fs::metadata(&relations_path)
                .map_err(|e| format!("Failed to read file metadata: {}", e))?
                .len()
        } else {
            0
        };
        
        Ok(Self {
            path,
            max_size,
            current_size,
            relations: RwLock::new(relations),
        })
    }
    
    /// Store a relation between entities
    pub fn store_relation(&mut self, source_id: Uuid, relation: &str, target_id: Uuid) -> Result<(), String> {
        let mut relations = self.relations.write();
        
        // Check if relation already exists
        let already_exists = relations.forward
            .get(&source_id)
            .and_then(|r| r.get(relation))
            .map(|t| t.contains(&target_id))
            .unwrap_or(false);
        
        if already_exists {
            return Ok(());
        }
        
        // Add to forward relations
        relations.forward
            .entry(source_id)
            .or_insert_with(HashMap::new)
            .entry(relation.to_string())
            .or_insert_with(HashSet::new)
            .insert(target_id);
        
        // Add to backward relations
        relations.backward
            .entry(target_id)
            .or_insert_with(HashMap::new)
            .entry(relation.to_string())
            .or_insert_with(HashSet::new)
            .insert(source_id);
        
        // Add to all relations
        relations.all_relations.push((source_id, relation.to_string(), target_id));
        
        // Update count and version
        relations.count += 1;
        relations.version += 1;
        
        // Persist relations
        drop(relations);
        self.persist_relations()?;
        
        Ok(())
    }
    
    /// Persist relations to disk
    fn persist_relations(&self) -> Result<(), String> {
        let relations_path = self.path.join("relations.json");
        let relations = self.relations.read();
        
        let file = fs::File::create(&relations_path)
            .map_err(|e| format!("Failed to create relations file: {}", e))?;
        
        serde_json::to_writer_pretty(file, &*relations)
            .map_err(|e| format!("Failed to write relations file: {}", e))?;
        
        // Update current size
        let new_size = fs::metadata(&relations_path)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?
            .len();
        
        // This is not atomic but should be fine for this use case
        let mut self_mut = unsafe { &mut *(self as *const Self as *mut Self) };
        self_mut.current_size = new_size;
        
        Ok(())
    }
    
    /// Get relations for an entity
    pub fn get_relations(&self, id: Uuid, relation_type: Option<&str>) -> Result<Vec<(Uuid, String, Uuid)>, String> {
        let relations = self.relations.read();
        
        let mut result = Vec::new();
        
        // Get forward relations
        if let Some(forward) = relations.forward.get(&id) {
            for (relation, targets) in forward {
                // Filter by relation type if specified
                if relation_type.is_none() || relation_type.unwrap() == relation {
                    for &target in targets {
                        result.push((id, relation.clone(), target));
                    }
                }
            }
        }
        
        // Get backward relations
        if let Some(backward) = relations.backward.get(&id) {
            for (relation, sources) in backward {
                // Filter by relation type if specified
                if relation_type.is_none() || relation_type.unwrap() == relation {
                    for &source in sources {
                        result.push((source, relation.clone(), id));
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Get forward relations for an entity
    pub fn get_forward_relations(&self, id: Uuid, relation_type: Option<&str>) -> Result<Vec<(String, Uuid)>, String> {
        let relations = self.relations.read();
        
        let mut result = Vec::new();
        
        // Get forward relations
        if let Some(forward) = relations.forward.get(&id) {
            for (relation, targets) in forward {
                // Filter by relation type if specified
                if relation_type.is_none() || relation_type.unwrap() == relation {
                    for &target in targets {
                        result.push((relation.clone(), target));
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Get backward relations for an entity
    pub fn get_backward_relations(&self, id: Uuid, relation_type: Option<&str>) -> Result<Vec<(Uuid, String)>, String> {
        let relations = self.relations.read();
        
        let mut result = Vec::new();
        
        // Get backward relations
        if let Some(backward) = relations.backward.get(&id) {
            for (relation, sources) in backward {
                // Filter by relation type if specified
                if relation_type.is_none() || relation_type.unwrap() == relation {
                    for &source in sources {
                        result.push((source, relation.clone()));
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Delete a relation between entities
    pub fn delete_relation(&mut self, source_id: Uuid, relation: &str, target_id: Uuid) -> Result<(), String> {
        let mut relations = self.relations.write();
        
        // Check if relation exists
        let exists = relations.forward
            .get(&source_id)
            .and_then(|r| r.get(relation))
            .map(|t| t.contains(&target_id))
            .unwrap_or(false);
        
        if !exists {
            return Ok(());
        }
        
        // Remove from forward relations
        if let Some(forward) = relations.forward.get_mut(&source_id) {
            if let Some(targets) = forward.get_mut(relation) {
                targets.remove(&target_id);
                
                // Remove empty sets
                if targets.is_empty() {
                    forward.remove(relation);
                }
            }
            
            // Remove empty maps
            if forward.is_empty() {
                relations.forward.remove(&source_id);
            }
        }
        
        // Remove from backward relations
        if let Some(backward) = relations.backward.get_mut(&target_id) {
            if let Some(sources) = backward.get_mut(relation) {
                sources.remove(&source_id);
                
                // Remove empty sets
                if sources.is_empty() {
                    backward.remove(relation);
                }
            }
            
            // Remove empty maps
            if backward.is_empty() {
                relations.backward.remove(&target_id);
            }
        }
        
        // Remove from all relations
        relations.all_relations.retain(|&(s, ref r, t)| !(s == source_id && r == relation && t == target_id));
        
        // Update count and version
        relations.count -= 1;
        relations.version += 1;
        
        // Persist relations
        drop(relations);
        self.persist_relations()?;
        
        Ok(())
    }
    
    /// Delete all relations for an entity
    pub fn delete_entity_relations(&mut self, id: Uuid) -> Result<(), String> {
        let mut relations = self.relations.write();
        
        // Get all relations involving this entity
        let mut to_delete = Vec::new();
        
        // Check forward relations
        if let Some(forward) = relations.forward.get(&id) {
            for (relation, targets) in forward {
                for &target in targets {
                    to_delete.push((id, relation.clone(), target));
                }
            }
        }
        
        // Check backward relations
        if let Some(backward) = relations.backward.get(&id) {
            for (relation, sources) in backward {
                for &source in sources {
                    to_delete.push((source, relation.clone(), id));
                }
            }
        }
        
        // Remove all relations involving this entity
        for (source, relation, target) in &to_delete {
            // Remove from forward relations
            if let Some(forward) = relations.forward.get_mut(source) {
                if let Some(targets) = forward.get_mut(relation) {
                    targets.remove(target);
                    
                    // Remove empty sets
                    if targets.is_empty() {
                        forward.remove(relation);
                    }
                }
                
                // Remove empty maps
                if forward.is_empty() {
                    relations.forward.remove(source);
                }
            }
            
            // Remove from backward relations
            if let Some(backward) = relations.backward.get_mut(target) {
                if let Some(sources) = backward.get_mut(relation) {
                    sources.remove(source);
                    
                    // Remove empty sets
                    if sources.is_empty() {
                        backward.remove(relation);
                    }
                }
                
                // Remove empty maps
                if backward.is_empty() {
                    relations.backward.remove(target);
                }
            }
        }
        
        // Remove from all relations
        relations.all_relations.retain(|&(s, _, t)| s != id && t != id);
        
        // Update count and version
        relations.count -= to_delete.len();
        relations.version += 1;
        
        // Persist relations
        drop(relations);
        self.persist_relations()?;
        
        Ok(())
    }
    
    /// Get all relations
    pub fn get_all_relations(&self) -> Vec<(Uuid, String, Uuid)> {
        let relations = self.relations.read();
        relations.all_relations.clone()
    }
    
    /// Get relations by type
    pub fn get_relations_by_type(&self, relation_type: &str) -> Vec<(Uuid, Uuid)> {
        let relations = self.relations.read();
        
        let mut result = Vec::new();
        
        for &(source, ref relation, target) in &relations.all_relations {
            if relation == relation_type {
                result.push((source, target));
            }
        }
        
        result
    }
    
    /// Find entities by relation pattern
    pub fn find_entities_by_relation(&self, relation_pattern: &str) -> Vec<Uuid> {
        let relations = self.relations.read();
        
        let mut result = HashSet::new();
        
        // Simple glob-like pattern matching with * wildcard
        let regex_pattern = relation_pattern.replace("*", ".*");
        let regex = regex::Regex::new(&format!("^{}$", regex_pattern)).unwrap_or_else(|_| {
            // Fallback to exact match if regex is invalid
            regex::Regex::new(&format!("^{}$", regex::escape(relation_pattern))).unwrap()
        });
        
        for (source, relation, target) in &relations.all_relations {
            if regex.is_match(relation) {
                result.insert(*source);
                result.insert(*target);
            }
        }
        
        result.into_iter().collect()
    }
    
    /// Get the size of the store
    pub fn get_size(&self) -> u64 {
        self.current_size
    }
    
    /// Get the number of relations
    pub fn get_relation_count(&self) -> usize {
        self.relations.read().count
    }
    
    /// Check if a relation exists
    pub fn relation_exists(&self, source_id: Uuid, relation: &str, target_id: Uuid) -> bool {
        let relations = self.relations.read();
        
        relations.forward
            .get(&source_id)
            .and_then(|r| r.get(relation))
            .map(|t| t.contains(&target_id))
            .unwrap_or(false)
    }
    
    /// Get entities related to a group
    pub fn get_related_entities(&self, ids: &[Uuid], relation_type: Option<&str>) -> Result<Vec<Uuid>, String> {
        let relations = self.relations.read();
        
        let mut result = HashSet::new();
        
        for &id in ids {
            // Get forward relations
            if let Some(forward) = relations.forward.get(&id) {
                for (relation, targets) in forward {
                    // Filter by relation type if specified
                    if relation_type.is_none() || relation_type.unwrap() == relation {
                        result.extend(targets);
                    }
                }
            }
            
            // Get backward relations
            if let Some(backward) = relations.backward.get(&id) {
                for (relation, sources) in backward {
                    // Filter by relation type if specified
                    if relation_type.is_none() || relation_type.unwrap() == relation {
                        result.extend(sources);
                    }
                }
            }
        }
        
        // Remove original ids from result
        for &id in ids {
            result.remove(&id);
        }
        
        Ok(result.into_iter().collect())
    }
}
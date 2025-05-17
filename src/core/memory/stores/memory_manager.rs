use std::collections::VecDeque;
use chrono::{DateTime, Utc};

/// Memory Manager for RAM-Lake
/// 
/// Tracks and manages memory allocations
pub struct MemoryManager {
    /// Maximum memory size in bytes
    max_size: u64,
    
    /// Current allocated memory in bytes
    current_size: u64,
    
    /// Allocation history
    allocations: VecDeque<MemoryAllocation>,
}

/// Memory Allocation
#[derive(Debug, Clone)]
pub struct MemoryAllocation {
    /// Size of the allocation in bytes
    pub size: u64,
    
    /// Source of the allocation
    pub source: String,
    
    /// Timestamp of the allocation
    pub timestamp: DateTime<Utc>,
}

/// Memory Allocation Error
#[derive(Debug, thiserror::Error)]
pub enum MemoryAllocationError {
    #[error("Not enough memory available")]
    OutOfMemory,
    
    #[error("Invalid allocation size")]
    InvalidSize,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(max_size: u64) -> Self {
        Self {
            max_size,
            current_size: 0,
            allocations: VecDeque::new(),
        }
    }
    
    /// Allocate memory
    pub fn allocate(&mut self, size: u64) -> Result<(), MemoryAllocationError> {
        // Check size
        if size == 0 {
            return Err(MemoryAllocationError::InvalidSize);
        }
        
        // Check if we have enough memory
        if self.current_size + size > self.max_size {
            return Err(MemoryAllocationError::OutOfMemory);
        }
        
        // Record allocation
        let allocation = MemoryAllocation {
            size,
            source: "unknown".to_string(),
            timestamp: Utc::now(),
        };
        
        self.allocations.push_back(allocation);
        
        // Update current size
        self.current_size += size;
        
        // Limit allocation history
        if self.allocations.len() > 1000 {
            self.allocations.pop_front();
        }
        
        Ok(())
    }
    
    /// Allocate memory with source information
    pub fn allocate_with_source(&mut self, size: u64, source: &str) -> Result<(), MemoryAllocationError> {
        // Check size
        if size == 0 {
            return Err(MemoryAllocationError::InvalidSize);
        }
        
        // Check if we have enough memory
        if self.current_size + size > self.max_size {
            return Err(MemoryAllocationError::OutOfMemory);
        }
        
        // Record allocation
        let allocation = MemoryAllocation {
            size,
            source: source.to_string(),
            timestamp: Utc::now(),
        };
        
        self.allocations.push_back(allocation);
        
        // Update current size
        self.current_size += size;
        
        // Limit allocation history
        if self.allocations.len() > 1000 {
            self.allocations.pop_front();
        }
        
        Ok(())
    }
    
    /// Free memory
    pub fn free(&mut self, size: u64) -> Result<(), MemoryAllocationError> {
        // Check size
        if size == 0 {
            return Err(MemoryAllocationError::InvalidSize);
        }
        
        // Check if we have enough allocated memory
        if size > self.current_size {
            return Err(MemoryAllocationError::InvalidSize);
        }
        
        // Update current size
        self.current_size -= size;
        
        // Record free (as negative allocation)
        let allocation = MemoryAllocation {
            size: size,
            source: "free".to_string(),
            timestamp: Utc::now(),
        };
        
        self.allocations.push_back(allocation);
        
        // Limit allocation history
        if self.allocations.len() > 1000 {
            self.allocations.pop_front();
        }
        
        Ok(())
    }
    
    /// Get current memory usage
    pub fn get_current_usage(&self) -> u64 {
        self.current_size
    }
    
    /// Get maximum memory size
    pub fn get_max_size(&self) -> u64 {
        self.max_size
    }
    
    /// Get available memory
    pub fn get_available_memory(&self) -> u64 {
        self.max_size - self.current_size
    }
    
    /// Get memory utilization percentage
    pub fn get_utilization_percentage(&self) -> f64 {
        (self.current_size as f64 / self.max_size as f64) * 100.0
    }
    
    /// Get recent allocations
    pub fn get_recent_allocations(&self, limit: usize) -> Vec<MemoryAllocation> {
        let count = std::cmp::min(limit, self.allocations.len());
        self.allocations.iter().rev().take(count).cloned().collect()
    }
    
    /// Get allocations by source
    pub fn get_allocations_by_source(&self, source: &str) -> Vec<MemoryAllocation> {
        self.allocations.iter()
            .filter(|a| a.source == source)
            .cloned()
            .collect()
    }
    
    /// Reset memory allocations
    pub fn reset(&mut self) {
        self.current_size = 0;
        self.allocations.clear();
    }
    
    /// Increase maximum memory size
    pub fn increase_max_size(&mut self, additional_size: u64) {
        self.max_size += additional_size;
    }
    
    /// Decrease maximum memory size
    pub fn decrease_max_size(&mut self, reduction_size: u64) -> Result<(), MemoryAllocationError> {
        let new_max_size = if reduction_size > self.max_size {
            0
        } else {
            self.max_size - reduction_size
        };
        
        // Check if we have enough free memory
        if self.current_size > new_max_size {
            return Err(MemoryAllocationError::OutOfMemory);
        }
        
        self.max_size = new_max_size;
        
        Ok(())
    }
}
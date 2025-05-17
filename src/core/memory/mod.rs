// Export RAM-Lake implementation
pub mod ramlake;

// Re-export main types
pub use ramlake::{
    RamLake,
    RamLakeConfig,
    StoreAllocation,
    RamLakeMetrics,
};
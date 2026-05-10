// Basic file system operations

use std::fs;
use std::path::{Path, PathBuf};

/// Check if a path exists
pub fn path_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Check if a path is a directory
pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// Check if a path is a file
pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// Create a directory and its parents if they don't exist
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {}", e))
}

/// Read a file as string
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Write string to a file
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
}

/// Get the absolute path from a relative path
pub fn absolute_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, String> {
    std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))
        .map(|mut p| {
            p.push(path);
            p
        })
}

/// Get the canonical (absolute, normalized) path
pub fn canonical_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, String> {
    path.as_ref().canonicalize().map_err(|e| format!("Failed to get canonical path: {}", e))
}

/// Get file size in bytes
pub fn file_size<P: AsRef<Path>>(path: P) -> Result<u64, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("Failed to get file metadata: {}", e))?;
    Ok(metadata.len())
}

/// Get file modification time
pub fn modification_time<P: AsRef<Path>>(path: P) -> Result<std::time::SystemTime, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("Failed to get file metadata: {}", e))?;
    Ok(metadata.modified().map_err(|e| format!("Failed to get modification time: {}", e))?)
}

/// List files in a directory
pub fn list_files<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut files = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
            Err(e) => return Err(format!("Failed to access directory entry: {}", e)),
        }
    }
    
    Ok(files)
}

/// List directories in a directory
pub fn list_dirs<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut dirs = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                }
            }
            Err(e) => return Err(format!("Failed to access directory entry: {}", e)),
        }
    }
    
    Ok(dirs)
}

/// Create a temporary directory
pub fn create_temp_dir(prefix: &str) -> Result<PathBuf, String> {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .map_err(|e| format!("Failed to create temporary directory: {}", e))
        .map(|dir| dir.into_path())
}
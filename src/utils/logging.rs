// Logging utilities for PostDevAI

use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::EnvFilter;

/// Initialize default logging
pub fn init() {
    init_with_filter("info");
}

/// Initialize logging with a filter
pub fn init_with_filter(filter: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter));
    
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stdout.with_max_level(Level::INFO))
        .init();
    
    tracing::debug!("Logging initialized with filter: {}", filter);
}

/// Initialize logging to a file
pub fn init_with_file(file_path: &str, filter: &str) -> Result<(), String> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter));
    
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;
    
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stdout.with_max_level(Level::INFO))
        .with_writer(file.with_max_level(Level::TRACE))
        .init();
    
    tracing::debug!("Logging initialized with filter: {} and file: {}", filter, file_path);
    
    Ok(())
}

/// Create a Logger that supports different level-based targets
pub struct Logger {
    name: String,
}

impl Logger {
    /// Create a new logger with a name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
    
    /// Log a debug message
    pub fn debug(&self, message: &str) {
        tracing::debug!("{}: {}", self.name, message);
    }
    
    /// Log an info message
    pub fn info(&self, message: &str) {
        tracing::info!("{}: {}", self.name, message);
    }
    
    /// Log a warning message
    pub fn warn(&self, message: &str) {
        tracing::warn!("{}: {}", self.name, message);
    }
    
    /// Log an error message
    pub fn error(&self, message: &str) {
        tracing::error!("{}: {}", self.name, message);
    }
    
    /// Log a message with values
    pub fn log(&self, level: Level, message: &str, values: &[(&str, &str)]) {
        let mut msg = String::new();
        msg.push_str(&self.name);
        msg.push_str(": ");
        msg.push_str(message);
        
        if !values.is_empty() {
            msg.push_str(" {");
            for (i, (key, value)) in values.iter().enumerate() {
                if i > 0 {
                    msg.push_str(", ");
                }
                msg.push_str(key);
                msg.push_str("=");
                msg.push_str(value);
            }
            msg.push_str("}");
        }
        
        match level {
            Level::TRACE => tracing::trace!("{}", msg),
            Level::DEBUG => tracing::debug!("{}", msg),
            Level::INFO => tracing::info!("{}", msg),
            Level::WARN => tracing::warn!("{}", msg),
            Level::ERROR => tracing::error!("{}", msg),
        }
    }
}
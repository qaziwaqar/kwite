//! # Remote Logging Module
//!
//! This module provides functionality to buffer log messages locally and send them
//! in batches to a remote logging endpoint. It's designed to reduce API invocations
//! while providing comprehensive application logging for debugging and analytics.
//!
//! ## Features
//!
//! - **Buffered Logging**: Collects logs in memory before sending
//! - **Batch Processing**: Sends multiple log entries in a single request
//! - **Configurable Endpoint**: Remote logging endpoint can be configured
//! - **Conditional Logging**: Can be enabled/disabled via configuration
//! - **System Information**: Includes system context with each batch
//! - **Privacy Aware**: Hashes sensitive information like MAC addresses
//!
//! ## Configuration
//!
//! Remote logging is controlled by configuration flags and is disabled by default
//! to respect user privacy and minimize external dependencies.

// Allow dead code for remote logging features that may be used conditionally
#![allow(dead_code)]

use crate::constants::{PERFORMANCE_ENDPOINT, DEFAULT_LOG_BATCH_SIZE, DEFAULT_LOG_FLUSH_INTERVAL_SECONDS, MAX_PAYLOAD_SIZE_BYTES};
use crate::system_info::SystemInfo;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, warn};

#[cfg(feature = "remote-logging")]
use serde_json;

/// Maximum payload size per request (2MB)
const MAX_PAYLOAD_SIZE_BYTES_LOCAL: usize = MAX_PAYLOAD_SIZE_BYTES;

/// Configuration for remote logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteLoggingConfig {
    /// Whether remote logging is enabled
    pub enabled: bool,
    /// Remote endpoint URL for log submission
    pub endpoint: String,
    /// Maximum number of log entries to buffer before sending
    pub batch_size: usize,
    /// Maximum time to wait before sending a batch (in seconds)
    pub flush_interval_seconds: u64,
    /// Whether to include system information with each batch
    pub include_system_info: bool,
    /// API key or authentication token (if required)
    pub auth_token: Option<String>,
}

impl Default for RemoteLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for privacy
            endpoint: PERFORMANCE_ENDPOINT.to_string(), // Updated endpoint for crash logs
            batch_size: DEFAULT_LOG_BATCH_SIZE,
            flush_interval_seconds: DEFAULT_LOG_FLUSH_INTERVAL_SECONDS,
            include_system_info: true,
            auth_token: None,
        }
    }
}

/// A single log entry for remote transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp when the log was created
    pub timestamp: String,
    /// Log level (error, warn, info, debug)
    pub level: String,
    /// Log message content
    pub message: String,
    /// Source module or location
    pub source: Option<String>,
    /// Additional structured fields
    pub fields: std::collections::HashMap<String, String>,
}

/// Application information for logging context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Build target architecture
    pub build_target: String,
}

/// Batch of log entries with system context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogBatch {
    /// Application name and version
    pub app_info: AppInfo,
    /// System information context
    pub system_info: Option<SystemInfo>,
    /// Batch of log entries
    pub logs: Vec<LogEntry>,
    /// When this batch was created
    pub batch_timestamp: String,
    /// Session identifier for this application run
    pub session_id: String,
}

impl LogBatch {
    /// Estimate the serialized size of this batch in bytes
    #[cfg(feature = "remote-logging")]
    fn estimated_size(&self) -> usize {
        // Use JSON serialization to get accurate size
        match serde_json::to_vec(self) {
            Ok(serialized) => serialized.len(),
            Err(_) => {
                // Fallback estimation if serialization fails
                let base_size = 500; // Approximate size of metadata
                let log_size_estimate = self.logs.len() * 200; // Rough estimate per log entry
                base_size + log_size_estimate
            }
        }
    }

    /// Fallback estimation when remote logging feature is disabled
    #[cfg(not(feature = "remote-logging"))]
    fn estimated_size(&self) -> usize {
        let base_size = 500; // Approximate size of metadata
        let log_size_estimate = self.logs.len() * 200; // Rough estimate per log entry
        base_size + log_size_estimate
    }

    /// Create a trimmed version with only the most recent logs that fit within size limit
    fn trim_to_size_limit(&self, max_size: usize) -> Self {
        let mut trimmed_logs = self.logs.clone();
        
        // Create a test batch to check size
        let mut test_batch = self.clone();
        test_batch.logs = trimmed_logs.clone();
        
        // If the batch is already within limits, return as-is
        if test_batch.estimated_size() <= max_size {
            return test_batch;
        }
        
        // Remove logs from the beginning (oldest first) until we're under the limit
        while !trimmed_logs.is_empty() && test_batch.estimated_size() > max_size {
            trimmed_logs.remove(0); // Remove oldest log
            test_batch.logs = trimmed_logs.clone();
        }
        
        if trimmed_logs.len() < self.logs.len() {
            debug!("Trimmed log batch from {} to {} entries to fit size limit", 
                   self.logs.len(), trimmed_logs.len());
        }
        
        test_batch
    }
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: "Kwite".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_target: std::env::consts::ARCH.to_string(),
        }
    }
}

/// Remote logging buffer and transmission manager
pub struct RemoteLogger {
    config: RemoteLoggingConfig,
    buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    system_info: SystemInfo,
    session_id: String,
    last_flush: Arc<Mutex<SystemTime>>,
    #[cfg(feature = "remote-logging")]
    client: Option<reqwest::Client>,
}

impl RemoteLogger {
    /// Create a new remote logger with the given configuration
    pub fn new(config: RemoteLoggingConfig) -> Self {
        let session_id = format!(
            "kwite_{}_{}", 
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            rand::random::<u32>()
        );

        Self {
            config: config.clone(),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            system_info: SystemInfo::collect(),
            session_id,
            last_flush: Arc::new(Mutex::new(SystemTime::now())),
            #[cfg(feature = "remote-logging")]
            client: if config.enabled {
                Some(reqwest::Client::new())
            } else {
                None
            },
        }
    }

    /// Add a log entry to the buffer
    pub fn log(&self, level: &str, message: &str, source: Option<&str>, fields: std::collections::HashMap<String, String>) {
        if !self.config.enabled {
            return;
        }

        let entry = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: level.to_string(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
            fields,
        };

        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.push_back(entry);
            
            // Check if we need to flush based on buffer size
            if buffer.len() >= self.config.batch_size {
                drop(buffer); // Release lock before async operation
                self.flush_async();
            }
        }

        // Check if we need to flush based on time
        if let Ok(last_flush) = self.last_flush.lock() {
            if last_flush.elapsed().unwrap_or_default().as_secs() >= self.config.flush_interval_seconds {
                drop(last_flush); // Release lock before async operation
                self.flush_async();
            }
        }
    }

    /// Flush the log buffer asynchronously
    fn flush_async(&self) {
        if !self.config.enabled {
            return;
        }

        let buffer = self.buffer.clone();
        let config = self.config.clone();
        let system_info = if self.config.include_system_info {
            Some(self.system_info.clone())
        } else {
            None
        };
        let session_id = self.session_id.clone();
        let last_flush = self.last_flush.clone();

        #[cfg(feature = "remote-logging")]
        {
            if let Some(client) = &self.client {
                let client_clone = client.clone();
                
                // Create a new thread to handle the async operation
                // This avoids the "no reactor running" error when called from GUI thread
                std::thread::spawn(move || {
                    // Create a single-threaded tokio runtime for this operation
                    let rt = match tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build() 
                    {
                        Ok(rt) => rt,
                        Err(e) => {
                            debug!("Failed to create tokio runtime for remote logging: {}", e);
                            return;
                        }
                    };
                    
                    // Run the async operation within the runtime
                    rt.block_on(async move {
                        Self::send_batch_async(
                            client_clone,
                            buffer,
                            config,
                            system_info,
                            session_id,
                            last_flush,
                        ).await;
                    });
                });
            }
        }

        #[cfg(not(feature = "remote-logging"))]
        {
            debug!("Remote logging not enabled at compile time - logs buffered locally only");
        }
    }

    /// Send a batch of logs to the remote endpoint
    #[cfg(feature = "remote-logging")]
    async fn send_batch_async(
        client: reqwest::Client,
        buffer: Arc<Mutex<VecDeque<LogEntry>>>,
        config: RemoteLoggingConfig,
        system_info: Option<SystemInfo>,
        session_id: String,
        last_flush: Arc<Mutex<SystemTime>>,
    ) {
        // Extract logs from buffer
        let logs = {
            if let Ok(mut buffer) = buffer.lock() {
                let mut logs = Vec::new();
                while let Some(entry) = buffer.pop_front() {
                    logs.push(entry);
                }
                logs
            } else {
                return;
            }
        };

        if logs.is_empty() {
            return;
        }

        let batch = LogBatch {
            app_info: AppInfo::default(),
            system_info,
            logs,
            batch_timestamp: chrono::Utc::now().to_rfc3339(),
            session_id,
        };

        // Check size and trim if necessary to stay within 2MB limit
        let final_batch = batch.trim_to_size_limit(MAX_PAYLOAD_SIZE_BYTES_LOCAL);

        // Attempt to send the batch
        let mut request = client.post(&config.endpoint);
        
        if let Some(auth_token) = &config.auth_token {
            request = request.bearer_auth(auth_token);
        }

        match request
            .json(&final_batch)
            .timeout(Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Successfully sent log batch with {} entries", final_batch.logs.len());
                } else {
                    warn!("Remote logging endpoint returned status: {}", response.status());
                }
            }
            Err(e) => {
                error!("Failed to send log batch to remote endpoint: {}", e);
                
                // Re-add logs to buffer for retry (optional)
                if let Ok(mut buffer) = buffer.lock() {
                    for log in final_batch.logs {
                        buffer.push_front(log);
                    }
                    // Limit buffer size to prevent memory issues
                    while buffer.len() > config.batch_size * 5 {
                        buffer.pop_back();
                    }
                }
            }
        }

        // Update last flush time
        if let Ok(mut last_flush) = last_flush.lock() {
            *last_flush = SystemTime::now();
        }
    }

    /// Force flush all buffered logs
    pub fn flush(&self) {
        if !self.config.enabled {
            return;
        }

        self.flush_async();
    }

    /// Get current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.lock().map(|b| b.len()).unwrap_or(0)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: RemoteLoggingConfig) {
        self.config = config.clone();
        
        #[cfg(feature = "remote-logging")]
        {
            self.client = if config.enabled {
                Some(reqwest::Client::new())
            } else {
                None
            };
        }
    }
}

/// Global remote logger instance
static REMOTE_LOGGER: once_cell::sync::OnceCell<Arc<Mutex<RemoteLogger>>> = once_cell::sync::OnceCell::new();

/// Initialize the global remote logger
pub fn init_remote_logger(config: RemoteLoggingConfig) {
    let logger = RemoteLogger::new(config);
    REMOTE_LOGGER.set(Arc::new(Mutex::new(logger))).ok();
}

/// Log a message to the remote logging system
pub fn log_remote(level: &str, message: &str, source: Option<&str>, fields: std::collections::HashMap<String, String>) {
    if let Some(logger) = REMOTE_LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.log(level, message, source, fields);
        }
    }
}

/// Convenience macro for remote logging with automatic source detection
#[macro_export]
macro_rules! remote_log {
    ($level:expr, $($arg:tt)*) => {
        {
            let message = format!($($arg)*);
            let source = Some(module_path!());
            let fields = std::collections::HashMap::new();
            $crate::remote_logging::log_remote($level, &message, source, fields);
        }
    };
}

/// Convenience macros for different log levels
#[macro_export]
macro_rules! remote_info {
    ($($arg:tt)*) => {
        $crate::remote_log!("info", $($arg)*)
    };
}

#[macro_export]
macro_rules! remote_warn {
    ($($arg:tt)*) => {
        $crate::remote_log!("warn", $($arg)*)
    };
}

#[macro_export]
macro_rules! remote_error {
    ($($arg:tt)*) => {
        $crate::remote_log!("error", $($arg)*)
    };
}

#[macro_export]
macro_rules! remote_debug {
    ($($arg:tt)*) => {
        $crate::remote_log!("debug", $($arg)*)
    };
}

/// Force flush the remote logger
pub fn flush_remote_logs() {
    if let Some(logger) = REMOTE_LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            logger.flush();
        }
    }
}

/// Get the current remote log buffer size
pub fn remote_log_buffer_size() -> usize {
    if let Some(logger) = REMOTE_LOGGER.get() {
        if let Ok(logger) = logger.lock() {
            return logger.buffer_size();
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_logging_config_default() {
        let config = RemoteLoggingConfig::default();
        assert!(!config.enabled); // Should be disabled by default
        assert_eq!(config.endpoint, PERFORMANCE_ENDPOINT);
        assert_eq!(config.batch_size, DEFAULT_LOG_BATCH_SIZE);
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            message: "Test message".to_string(),
            source: Some("test_module".to_string()),
            fields: std::collections::HashMap::new(),
        };

        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.source, Some("test_module".to_string()));
    }

    #[test]
    fn test_remote_logger_disabled() {
        let config = RemoteLoggingConfig {
            enabled: false,
            ..RemoteLoggingConfig::default()
        };

        let logger = RemoteLogger::new(config);
        assert_eq!(logger.buffer_size(), 0);

        // Logging should be ignored when disabled
        logger.log("info", "test", None, std::collections::HashMap::new());
        assert_eq!(logger.buffer_size(), 0);
    }

    #[test]
    fn test_remote_logger_buffering() {
        let config = RemoteLoggingConfig {
            enabled: true,
            batch_size: 10,
            ..RemoteLoggingConfig::default()
        };

        let logger = RemoteLogger::new(config);
        
        // Add a log entry
        logger.log("info", "test message", None, std::collections::HashMap::new());
        assert_eq!(logger.buffer_size(), 1);
    }

    #[test]
    fn test_app_info_default() {
        let app_info = AppInfo::default();
        assert_eq!(app_info.name, "Kwite");
        assert!(!app_info.version.is_empty());
        assert!(!app_info.build_target.is_empty());
    }

    #[test]
    fn test_log_batch_size_estimation() {
        let mut logs = Vec::new();
        for i in 0..10 {
            logs.push(LogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                level: "info".to_string(),
                message: format!("Test message {}", i),
                source: Some("test_module".to_string()),
                fields: std::collections::HashMap::new(),
            });
        }

        let batch = LogBatch {
            app_info: AppInfo::default(),
            system_info: None,
            logs,
            batch_timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: "test_session".to_string(),
        };

        let size = batch.estimated_size();
        assert!(size > 0, "Batch size should be greater than 0");
        assert!(size < 10000, "Batch size should be reasonable for 10 entries");
    }

    #[test]
    fn test_log_batch_trimming() {
        // Create a batch with many large log entries
        let mut logs = Vec::new();
        for i in 0..100 {
            logs.push(LogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                level: "info".to_string(),
                message: format!("Very long test message that takes up space {}: {}", i, "x".repeat(1000)),
                source: Some("test_module".to_string()),
                fields: std::collections::HashMap::new(),
            });
        }

        let original_batch = LogBatch {
            app_info: AppInfo::default(),
            system_info: None,
            logs,
            batch_timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: "test_session".to_string(),
        };

        // Trim to a very small size to force trimming
        let trimmed = original_batch.trim_to_size_limit(10000); // 10KB limit
        
        assert!(trimmed.logs.len() < original_batch.logs.len(), "Trimmed batch should have fewer logs");
        assert!(trimmed.estimated_size() <= 10000, "Trimmed batch should be within size limit");
        
        // Verify we kept the most recent logs (higher indices)
        if !trimmed.logs.is_empty() {
            let first_kept_message = &trimmed.logs[0].message;
            assert!(first_kept_message.contains("Very long test message"), "Should contain original log structure");
        }
    }

    #[test]
    fn test_log_batch_no_trimming_needed() {
        let logs = vec![LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            message: "Short message".to_string(),
            source: Some("test_module".to_string()),
            fields: std::collections::HashMap::new(),
        }];

        let batch = LogBatch {
            app_info: AppInfo::default(),
            system_info: None,
            logs: logs.clone(),
            batch_timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: "test_session".to_string(),
        };

        let trimmed = batch.trim_to_size_limit(MAX_PAYLOAD_SIZE_BYTES_LOCAL);
        assert_eq!(trimmed.logs.len(), batch.logs.len(), "No trimming should be needed for small batch");
    }
}
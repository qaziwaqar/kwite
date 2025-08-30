//! # Logging Infrastructure Module
//! 
//! This module provides centralized logging capabilities for the Kwite application.
//! It uses the `tracing` ecosystem for structured, high-performance logging with
//! configurable output levels and formatting.
//! 
//! ## Design Goals
//! 
//! - **Performance**: Zero-cost when disabled, minimal overhead when enabled
//! - **Structured Logging**: Support for key-value pairs and contextual information
//! - **Configurable**: Environment-based log level control for debugging
//! - **Thread Safety**: Safe logging from multiple audio processing threads
//! - **Development Friendly**: Include source locations and thread IDs for debugging
//! 
//! ## Log Levels
//! 
//! - **ERROR**: System failures, audio device errors, critical issues
//! - **WARN**: Recoverable issues, device disconnections, fallback actions
//! - **INFO**: Normal operation events, device selection, configuration changes
//! - **DEBUG**: Detailed execution flow, parameter changes, performance metrics
//! 
//! ## Environment Configuration
//! 
//! Set the `RUST_LOG` environment variable to control log output:
//! - `RUST_LOG=kwite=debug` - Show all logs from this application
//! - `RUST_LOG=warn` - Show only warnings and errors globally
//! - `RUST_LOG=kwite::audio=debug,warn` - Debug audio module, warn others

use tracing_subscriber::{fmt, EnvFilter, prelude::*};
use once_cell::sync::Lazy;

/// Initialize the global logger.
/// 
/// This function sets up the tracing infrastructure with appropriate formatters
/// and filters. It's designed to be called once from `main.rs` and will safely
/// ignore subsequent calls.
/// 
/// ## Configuration
/// 
/// The logger is configured with:
/// - **Environment-based filtering**: Respects `RUST_LOG` environment variable
/// - **Default level**: `debug` for this application, `warn` for dependencies
/// - **Thread information**: Includes thread IDs for multi-threaded debugging
/// - **Source locations**: Shows file and line numbers for development builds
/// - **Human-readable format**: Clean output suitable for console viewing
/// 
/// ## Why Lazy Initialization?
/// 
/// Using `Lazy<()>` ensures the logger is initialized exactly once, even if
/// this function is called multiple times. This prevents duplicate subscriber
/// registration which would cause runtime panics.
/// 
/// Should be called once from `main.rs`.
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    // Use Lazy to ensure initialization happens exactly once
    // Multiple calls to this function are safe and will be ignored
    static INIT: Lazy<()> = Lazy::new(|| {
        // Create environment filter with sensible defaults
        // Falls back to "kwite=debug,warn" if RUST_LOG is not set
        // This provides detailed logging for our code while limiting noise from dependencies
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("kwite=debug,warn"));

        // Configure the tracing subscriber with multiple layers
        tracing_subscriber::registry()
            .with(env_filter) // Apply the environment-based filtering
            .with(fmt::layer()
                .with_target(false)     // Don't show module paths (cleaner output)
                .with_thread_ids(true)  // Include thread IDs for debugging multi-threaded code
                .with_level(true)       // Show log levels (ERROR, WARN, INFO, DEBUG)
                .with_line_number(true) // Include source line numbers for development
            )
            .init(); // Install as the global subscriber
    });
    
    // Force initialization of the lazy static
    Lazy::force(&INIT);
    Ok(())
}

/// Convenience re-export of log macros
/// 
/// This module provides a clean interface for logging throughout the application.
/// All code can import `crate::logger::log` and use the familiar log macros.
/// 
/// ## Usage Examples
/// 
/// ```rust
/// use kwite::logger::log;
/// 
/// log::info!("Starting audio processing with device: {}", "test_device");
/// log::warn!("Device {} not found, falling back to default", "test_id");
/// log::error!("Failed to initialize audio stream: {}", "test error");
/// log::debug!("Processing {} samples with sensitivity {}", 1024, 0.5);
/// ```
/// 
/// ## Performance Notes
/// 
/// The tracing macros are zero-cost when the log level is disabled. This means
/// you can include detailed debug logging without impacting release performance.
pub mod log {
    pub use tracing::{debug, error, info, warn};
}
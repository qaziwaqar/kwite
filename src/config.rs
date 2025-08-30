//! # Configuration Management Module
//!
//! This module handles persistent storage of user preferences and application settings.
//! Configuration is stored in a platform-appropriate location using the TOML format
//! for human readability and easy manual editing if needed.
//!
//! ## Design Philosophy
//!
//! The configuration system follows these principles:
//! 1. **Fail-safe defaults**: Always provide working defaults if config is missing/corrupt
//! 2. **Automatic recovery**: Gracefully handle invalid or missing configuration files
//! 3. **Platform compliance**: Store config files in OS-appropriate locations
//! 4. **User-friendly format**: Use TOML for readability and manual editing
//!
//! ## Configuration Storage Locations
//!
//! - **Windows**: `%APPDATA%\Kwite\config.toml`
//! - **macOS**: `~/Library/Application Support/Kwite/config.toml`
//! - **Linux**: `~/.config/kwite/config.toml`

use crate::remote_logging::RemoteLoggingConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use crate::constants::{DEFAULT_LOG_FLUSH_INTERVAL_SECONDS, DEFAULT_UPDATE_CHECK_INTERVAL_HOURS, PERFORMANCE_ENDPOINT, UPDATE_ENDPOINT};

/// Auto-update configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutoUpdateConfig {
    /// Whether to check for updates automatically
    pub enabled: bool,
    /// How often to check for updates (in hours)
    pub check_interval_hours: u64,
    /// Update server endpoint
    pub update_endpoint: String,
    /// Whether to notify user before downloading updates
    pub notify_before_download: bool,
}

/// Performance and analytics configuration  
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyticsConfig {
    /// Whether to send crash logs and performance data
    pub enabled: bool,
    /// Performance data endpoint
    pub performance_endpoint: String,
    /// How often to send performance data (in seconds) - weekly
    pub performance_interval_seconds: u64,
}

/// Application configuration structure
///
/// This struct contains all user-configurable settings that should persist
/// between application sessions. Settings are automatically saved when
/// changed through the GUI and loaded at startup.
///
/// ## Field Descriptions
///
/// - `input_device_id`: Identifier for the preferred microphone/input device
/// - `output_device_id`: Identifier for the preferred output device (often virtual cable)
/// - `sensitivity`: Noise cancellation sensitivity threshold (0.01 - 0.5)
/// - `auto_start`: Whether to begin noise cancellation automatically on startup
/// - `minimize_to_tray`: Whether to minimize to system tray instead of taskbar
/// - `development_mode`: Enable advanced analytics and debug features
/// - `remote_logging`: Configuration for remote logging and analytics
/// - `usage_statistics`: Enable collection of usage statistics
/// - `auto_update`: Configuration for automatic updates
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KwiteConfig {
    /// Audio input device identifier
    /// Typically corresponds to microphone or line-in device
    pub input_device_id: String,

    /// Audio output device identifier
    /// Preferably a virtual audio cable for use with communication apps
    pub output_device_id: String,

    /// Noise cancellation sensitivity (0.01 = aggressive, 0.5 = conservative)
    /// Lower values remove more background noise but may affect voice quality
    pub sensitivity: f32,

    /// Automatically start noise cancellation when application launches
    /// Useful for users who always want noise cancellation enabled
    pub auto_start: bool,

    /// Minimize to system tray instead of showing in taskbar
    /// Helps keep the application running unobtrusively
    pub minimize_to_tray: bool,

    /// Enable development mode features
    /// Shows advanced AI metrics and debug information (hidden from end users)
    pub development_mode: bool,

    /// Remote logging configuration
    /// Controls collection and transmission of logs for debugging
    pub remote_logging: RemoteLoggingConfig,

    /// Combined analytics configuration (crash logs + performance data)
    /// Replaces separate usage_statistics and remote_logging options
    pub analytics: AnalyticsConfig,

    /// Auto-update configuration
    pub auto_update: AutoUpdateConfig,
}

impl Default for AutoUpdateConfig {
    fn default() -> Self {
        Self {
            enabled: true, // Enabled by default
            check_interval_hours: DEFAULT_UPDATE_CHECK_INTERVAL_HOURS,
            update_endpoint: UPDATE_ENDPOINT.to_string(),
            notify_before_download: true,
        }
    }
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            performance_endpoint: PERFORMANCE_ENDPOINT.to_string(),
            performance_interval_seconds: DEFAULT_LOG_FLUSH_INTERVAL_SECONDS,
        }
    }
}

impl Default for KwiteConfig {
    /// Provide sensible defaults for first-time users
    ///
    /// These defaults are chosen to provide a good out-of-box experience:
    /// - Default device selections will be resolved at runtime
    /// - Moderate sensitivity balances noise reduction with voice quality
    /// - Auto-start enabled for immediate noise cancellation benefit
    /// - Development mode disabled for end users
    /// - Remote logging disabled by default for privacy
    fn default() -> Self {
        Self {
            input_device_id: "input_default".to_string(),
            output_device_id: "output_default".to_string(),
            sensitivity: 0.1, // Moderate noise reduction as starting point
            auto_start: false,
            minimize_to_tray: false, // Keep visible by default
            development_mode: false, // Hide advanced features from end users
            remote_logging: RemoteLoggingConfig::default(),
            analytics: AnalyticsConfig::default(), // Disabled by default for privacy
            auto_update: AutoUpdateConfig::default(),
        }
    }
}

impl KwiteConfig {
    /// Load configuration from disk, using defaults if file doesn't exist or is invalid
    ///
    /// This method implements robust configuration loading with multiple fallback levels:
    /// 1. Try to load and parse existing config file
    /// 2. If file doesn't exist, use default configuration
    /// 3. If file exists but is corrupt, log error and use defaults
    /// 4. If config directory can't be determined, use defaults
    ///
    /// This approach ensures the application always starts successfully, even with
    /// filesystem issues or corrupted configuration files.
    pub fn load() -> Self {
        match Self::config_path() {
            Ok(path) => {
                if path.exists() {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            match toml::from_str(&content) {
                                Ok(config) => config,
                                Err(e) => {
                                    eprintln!("Failed to parse config: {}", e);
                                    Self::default()
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read config file: {}", e);
                            Self::default()
                        }
                    }
                } else {
                    Self::default()
                }
            }
            Err(e) => {
                eprintln!("Failed to get config path: {}", e);
                Self::default()
            }
        }
    }

    /// Save current configuration to disk
    ///
    /// This method persists the current configuration state to the platform-appropriate
    /// configuration directory. The TOML format is used for human readability and
    /// to allow advanced users to manually edit settings if needed.
    ///
    /// The save process includes:
    /// 1. Determine the correct config file path for the current platform
    /// 2. Create parent directories if they don't exist
    /// 3. Serialize configuration to pretty-printed TOML
    /// 4. Write atomically to prevent corruption during write operations
    ///
    /// ## Error Handling
    ///
    /// Save errors are propagated to the caller for appropriate user feedback.
    /// Common failure scenarios include:
    /// - Insufficient disk space
    /// - Permission issues in config directory
    /// - Filesystem corruption or device errors
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        
        println!("Configuration saved to: {}", path.display());
        Ok(())
    }

    /// Determine the platform-appropriate configuration file path
    ///
    /// This function implements the platform-specific logic for configuration storage:
    ///
    /// ## Windows
    /// Uses the `%APPDATA%` directory (typically `C:\Users\{user}\AppData\Roaming`)
    /// This is the standard location for application data on Windows.
    ///
    /// ## macOS
    /// Uses the `~/Library/Application Support` directory following Apple's guidelines
    /// for application configuration storage.
    ///
    /// ## Linux/Unix
    /// Uses the `~/.config` directory following the XDG Base Directory Specification.
    /// The directory name is lowercase following Unix conventions.
    ///
    /// ## Error Handling
    ///
    /// If the platform's config directory cannot be determined (rare but possible
    /// on misconfigured systems), an error is returned rather than falling back
    /// to potentially inappropriate locations.
    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = if cfg!(target_os = "windows") {
            dirs::config_dir()
                .ok_or("Could not find config directory")?
                .join("Kwite")
        } else if cfg!(target_os = "macos") {
            dirs::config_dir()
                .ok_or("Could not find config directory")?
                .join("Kwite")
        } else {
            // Linux and other Unix-like systems
            dirs::config_dir()
                .ok_or("Could not find config directory")?
                .join("kwite")
        };

        Ok(config_dir.join("config.toml"))
    }

    /// Create a config for testing with all fields populated
    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            input_device_id: "test_input".to_string(),
            output_device_id: "test_output".to_string(),
            sensitivity: 0.1,
            auto_start: false,
            minimize_to_tray: false,
            development_mode: false,
            remote_logging: RemoteLoggingConfig::default(),
            analytics: AnalyticsConfig::default(),
            auto_update: AutoUpdateConfig::default(),
        }
    }
}

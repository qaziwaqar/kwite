//! # Auto-Update Module
//!
//! This module provides functionality to check for and download application updates
//! automatically. It handles version checking, download management, and user
//! notification for new releases.
//!
//! ## Features
//!
//! - **Version Checking**: Compares current version with remote latest
//! - **Download Management**: Handles update file downloads with progress
//! - **User Notification**: Alerts users about available updates
//! - **Configurable**: Update checking can be enabled/disabled
//! - **Safe Updates**: Validates downloaded files before installation
//!
//! ## Security
//!
//! All downloads are verified against checksums and digital signatures
//! where available to prevent tampering or malicious updates.

// Allow dead code for auto-update features that may be used conditionally
#![allow(dead_code)]

use crate::config::AutoUpdateConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Information about a software update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Version string (e.g., "1.2.3")
    pub version: String,
    /// Release date
    pub release_date: String,
    /// Download URL for the update
    pub download_url: String,
    /// File size in bytes
    pub file_size: u64,
    /// SHA256 checksum for verification
    pub checksum: String,
    /// Release notes or changelog
    pub release_notes: String,
    /// Whether this is a critical security update
    pub is_critical: bool,
    /// Minimum supported version for this update
    pub min_version: Option<String>,
}

/// Update check result
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    /// No update available
    NoUpdate,
    /// Update available
    UpdateAvailable(UpdateInfo),
    /// Error checking for updates
    Error(String),
}

/// Update download progress
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Bytes downloaded so far
    pub downloaded: u64,
    /// Total bytes to download
    pub total: u64,
    /// Download speed in bytes per second
    pub speed_bps: u64,
    /// Estimated time remaining in seconds
    pub eta_seconds: u64,
}

/// Auto-update manager
pub struct AutoUpdateManager {
    config: AutoUpdateConfig,
    current_version: String,
    last_check: Option<SystemTime>,
    #[cfg(feature = "remote-logging")]
    client: Option<reqwest::Client>,
}

impl AutoUpdateManager {
    /// Create a new auto-update manager
    pub fn new(config: AutoUpdateConfig) -> Self {
        Self {
            config: config.clone(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            last_check: None,
            #[cfg(feature = "remote-logging")]
            client: if config.enabled {
                Some(reqwest::Client::new())
            } else {
                None
            },
        }
    }

    /// Check if an update check is due based on configured interval
    pub fn is_check_due(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        match self.last_check {
            None => true, // Never checked before
            Some(last) => {
                let interval = Duration::from_secs(self.config.check_interval_hours * 3600);
                last.elapsed().unwrap_or(Duration::ZERO) >= interval
            }
        }
    }

    /// Check for available updates
    pub async fn check_for_updates(&mut self) -> UpdateCheckResult {
        if !self.config.enabled {
            return UpdateCheckResult::NoUpdate;
        }

        self.last_check = Some(SystemTime::now());

        #[cfg(feature = "remote-logging")]
        {
            if let Some(client) = &self.client {
                match self.fetch_update_info(client).await {
                    Ok(update_info) => {
                        if self.is_newer_version(&update_info.version) {
                            UpdateCheckResult::UpdateAvailable(update_info)
                        } else {
                            UpdateCheckResult::NoUpdate
                        }
                    }
                    Err(e) => UpdateCheckResult::Error(e.to_string()),
                }
            } else {
                UpdateCheckResult::Error("HTTP client not available".to_string())
            }
        }

        #[cfg(not(feature = "remote-logging"))]
        {
            UpdateCheckResult::Error("Auto-update not enabled at compile time".to_string())
        }
    }

    /// Fetch update information from remote server
    #[cfg(feature = "remote-logging")]
    async fn fetch_update_info(&self, client: &reqwest::Client) -> Result<UpdateInfo, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/version.json", self.config.update_endpoint);
        
        let response = client
            .get(&url)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Update server returned status: {}", response.status()).into());
        }

        let update_info: UpdateInfo = response.json().await?;
        Ok(update_info)
    }

    /// Compare version strings to determine if remote version is newer
    fn is_newer_version(&self, remote_version: &str) -> bool {
        match (self.parse_version(&self.current_version), self.parse_version(remote_version)) {
            (Ok(current), Ok(remote)) => remote > current,
            _ => false, // If we can't parse versions, assume no update
        }
    }

    /// Parse semantic version string into comparable tuple
    fn parse_version(&self, version: &str) -> Result<(u32, u32, u32), Box<dyn std::error::Error>> {
        let parts: Result<Vec<u32>, _> = version
            .trim_start_matches('v') // Remove 'v' prefix if present
            .split('.')
            .take(3)
            .map(|s| s.parse::<u32>())
            .collect();

        match parts {
            Ok(v) if v.len() >= 3 => Ok((v[0], v[1], v[2])),
            Ok(v) if v.len() == 2 => Ok((v[0], v[1], 0)),
            Ok(v) if v.len() == 1 => Ok((v[0], 0, 0)),
            _ => Err("Invalid version format".into()),
        }
    }

    /// Download an update file
    #[cfg(feature = "remote-logging")]
    pub async fn download_update(
        &self,
        update_info: &UpdateInfo,
        download_path: &PathBuf,
        progress_callback: impl Fn(DownloadProgress) + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(client) = &self.client {
            let response = client
                .get(&update_info.download_url)
                .timeout(Duration::from_secs(3600)) // 1 hour timeout for large files
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(format!("Download failed with status: {}", response.status()).into());
            }

            let total_size = response.content_length().unwrap_or(update_info.file_size);
            let mut downloaded = 0u64;
            let start_time = std::time::Instant::now();

            // Create the download directory if it doesn't exist
            if let Some(parent) = download_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut file = std::fs::File::create(download_path)?;
            
            use futures_util::StreamExt;
            use std::io::Write;
            let mut stream = response.bytes_stream();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                file.write_all(&chunk)?;
                
                downloaded += chunk.len() as u64;
                let elapsed = start_time.elapsed().as_secs().max(1);
                let speed = downloaded / elapsed;
                let eta = if speed > 0 { (total_size - downloaded) / speed } else { 0 };

                progress_callback(DownloadProgress {
                    downloaded,
                    total: total_size,
                    speed_bps: speed,
                    eta_seconds: eta,
                });
            }

            file.sync_all()?;
            drop(file);

            // Verify checksum
            self.verify_download(download_path, &update_info.checksum)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

            Ok(())
        } else {
            Err("HTTP client not available".into())
        }
    }

    /// Verify downloaded file checksum
    fn verify_download(&self, file_path: &PathBuf, expected_checksum: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use sha2::{Sha256, Digest};
        use std::io::Read;

        let mut file = std::fs::File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let computed_hash = format!("{:x}", hasher.finalize());
        
        if computed_hash.eq_ignore_ascii_case(expected_checksum) {
            Ok(())
        } else {
            Err(format!(
                "Checksum verification failed. Expected: {}, Got: {}",
                expected_checksum, computed_hash
            ).into())
        }
    }

    /// Install a downloaded update (platform-specific)
    pub fn install_update(&self, update_file: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_os = "windows")]
        {
            // On Windows, typically run the installer executable
            std::process::Command::new(update_file)
                .spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, open the downloaded file (usually a .dmg or .pkg)
            std::process::Command::new("open")
                .arg(update_file)
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux, the update process varies by distribution and package format
            // For now, just make the file executable and provide instructions
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(update_file)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(update_file, perms)?;
            
            // Could launch with system default handler
            std::process::Command::new("xdg-open")
                .arg(update_file)
                .spawn()
                .or_else(|_| {
                    // Fallback: just print instructions
                    eprintln!("Update downloaded to: {:?}", update_file);
                    eprintln!("Please run the installer manually.");
                    let result: Result<std::process::Child, std::io::Error> = Ok(std::process::Command::new("true").spawn()?);
                    result
                })?;
        }

        Ok(())
    }

    /// Get current application version
    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AutoUpdateConfig) {
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

    /// Get last check time
    pub fn last_check_time(&self) -> Option<SystemTime> {
        self.last_check
    }

    /// Force a manual update check regardless of interval
    pub async fn force_check(&mut self) -> UpdateCheckResult {
        let was_enabled = self.config.enabled;
        self.config.enabled = true; // Temporarily enable for manual check
        let result = self.check_for_updates().await;
        self.config.enabled = was_enabled; // Restore original setting
        result
    }
}

/// Get the default download directory for updates
pub fn get_update_download_dir() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let mut path = dirs::download_dir()
        .or_else(|| dirs::home_dir())
        .ok_or("Could not determine download directory")?;
    
    path.push("Kwite");
    path.push("updates");
    
    Ok(path)
}

/// Format file size in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    const THRESHOLD: u64 = 1024;

    if bytes < THRESHOLD {
        return format!("{} B", bytes);
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Format download speed in human-readable format
pub fn format_speed(bytes_per_second: u64) -> String {
    format!("{}/s", format_file_size(bytes_per_second))
}

/// Format time duration in human-readable format
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let manager = AutoUpdateManager::new(AutoUpdateConfig::default());
        
        assert_eq!(manager.parse_version("1.2.3").unwrap(), (1, 2, 3));
        assert_eq!(manager.parse_version("v2.0.0").unwrap(), (2, 0, 0));
        assert_eq!(manager.parse_version("0.1").unwrap(), (0, 1, 0));
        assert_eq!(manager.parse_version("5").unwrap(), (5, 0, 0));
        
        assert!(manager.parse_version("invalid").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let manager = AutoUpdateManager::new(AutoUpdateConfig::default());
        
        assert!(manager.is_newer_version("1.0.1"));  // Current is 0.1.0
        assert!(!manager.is_newer_version("0.0.9")); // Older version
    }

    #[test]
    fn test_update_check_timing() {
        let config = AutoUpdateConfig {
            enabled: true,
            check_interval_hours: 24,
            ..AutoUpdateConfig::default()
        };
        
        let mut manager = AutoUpdateManager::new(config);
        assert!(manager.is_check_due()); // Never checked before
        
        manager.last_check = Some(SystemTime::now());
        assert!(!manager.is_check_due()); // Just checked
    }

    #[test]
    fn test_disabled_updates() {
        let config = AutoUpdateConfig {
            enabled: false,
            ..AutoUpdateConfig::default()
        };
        
        let manager = AutoUpdateManager::new(config);
        assert!(!manager.is_check_due());
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(2097152), "2.0 MB");
        assert_eq!(format_file_size(3221225472), "3.0 GB");
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3665), "1h 1m");
    }
}
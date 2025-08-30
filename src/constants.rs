//! # Application Constants
//!
//! This module contains all application-wide constants including API endpoints,
//! configuration defaults, and other shared values that need to be consistent
//! across the application.
//!
//! ## Endpoint Constants
//!
//! All external service endpoints are defined here to ensure consistency
//! and make it easy to update them in a single location.

/// Performance and usage analytics endpoint
/// Used for sending crash logs, performance metrics, and usage statistics
// TODO: Update with actual endpoint when available
pub const PERFORMANCE_ENDPOINT: &str = "";

/// Auto-update version checking endpoint  
/// Used for checking the latest application version and update information
// TODO: Update with actual endpoint when available
pub const UPDATE_ENDPOINT: &str = "";

/// Default batch size for remote logging
pub const DEFAULT_LOG_BATCH_SIZE: usize = 50;

/// Default flush interval for remote logging (7 days in seconds)
pub const DEFAULT_LOG_FLUSH_INTERVAL_SECONDS: u64 = 604800;

/// Default auto-update check interval (24 hours)
pub const DEFAULT_UPDATE_CHECK_INTERVAL_HOURS: u64 = 24;

/// Maximum payload size per logging request (2MB)
pub const MAX_PAYLOAD_SIZE_BYTES: usize = 2 * 1024 * 1024;
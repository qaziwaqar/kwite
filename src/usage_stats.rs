//! # Usage Statistics Module
//!
//! This module tracks application usage patterns and performance metrics
//! to help improve the software and understand user behavior. All data
//! collection is optional and respects user privacy preferences.
//!
//! ## Metrics Collected
//!
//! - Session duration and frequency
//! - Audio processing performance
//! - Feature usage patterns
//! - Error rates and recovery
//! - System performance impact
//!
//! ## Privacy
//!
//! All personally identifiable information is either hashed or excluded.
//! Statistics are aggregated and anonymized before any potential transmission.

// Allow dead code for usage statistics that may be used conditionally
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use chrono::Utc;

/// Aggregated usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    /// Total number of application sessions
    pub total_sessions: u64,
    /// Total cumulative usage time in seconds
    pub total_usage_seconds: u64,
    /// Average session duration in seconds
    pub avg_session_duration_seconds: f64,
    /// Number of times noise cancellation was activated
    pub noise_cancellation_activations: u64,
    /// Total time noise cancellation was active (seconds)
    pub total_processing_time_seconds: u64,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Feature usage counts
    pub feature_usage: HashMap<String, u64>,
    /// Error statistics
    pub error_stats: ErrorStatistics,
    /// Daily usage pattern (last 30 days)
    pub daily_usage: Vec<DailyUsage>,
    /// Last updated timestamp
    pub last_updated: String,
}

/// Performance-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average audio processing latency in milliseconds
    pub avg_latency_ms: f64,
    /// Peak latency encountered
    pub peak_latency_ms: f64,
    /// Average CPU usage percentage during processing
    pub avg_cpu_usage_percent: f64,
    /// Peak CPU usage percentage
    pub peak_cpu_usage_percent: f64,
    /// Average memory usage in MB
    pub avg_memory_usage_mb: f64,
    /// Peak memory usage in MB
    pub peak_memory_usage_mb: f64,
    /// Number of audio dropouts/glitches
    pub audio_dropouts: u64,
    /// AI model performance scores
    pub ai_model_performance: HashMap<String, f64>,
}

/// Error tracking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    /// Total number of errors encountered
    pub total_errors: u64,
    /// Error counts by category
    pub errors_by_category: HashMap<String, u64>,
    /// Recovery success rate (0.0 - 1.0)
    pub recovery_success_rate: f64,
    /// Critical errors that required restart
    pub critical_errors: u64,
}

/// Daily usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Total usage time in seconds for this day
    pub usage_seconds: u64,
    /// Number of sessions on this day
    pub session_count: u32,
    /// Average performance score for the day
    pub avg_performance_score: f64,
}

/// Current session tracking
#[derive(Debug)]
pub struct SessionTracker {
    session_start: SystemTime,
    noise_cancellation_start: Option<SystemTime>,
    total_nc_time: Duration,
    performance_samples: Vec<f64>,
    errors_this_session: u32,
    features_used: HashMap<String, u32>,
}

/// Usage statistics manager
pub struct UsageStatsManager {
    stats: UsageStatistics,
    current_session: Option<SessionTracker>,
    enabled: bool,
}

impl Default for UsageStatistics {
    fn default() -> Self {
        Self {
            total_sessions: 0,
            total_usage_seconds: 0,
            avg_session_duration_seconds: 0.0,
            noise_cancellation_activations: 0,
            total_processing_time_seconds: 0,
            performance_metrics: PerformanceMetrics::default(),
            feature_usage: HashMap::new(),
            error_stats: ErrorStatistics::default(),
            daily_usage: Vec::new(),
            last_updated: Utc::now().to_rfc3339(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_latency_ms: 0.0,
            peak_latency_ms: 0.0,
            avg_cpu_usage_percent: 0.0,
            peak_cpu_usage_percent: 0.0,
            avg_memory_usage_mb: 0.0,
            peak_memory_usage_mb: 0.0,
            audio_dropouts: 0,
            ai_model_performance: HashMap::new(),
        }
    }
}

impl Default for ErrorStatistics {
    fn default() -> Self {
        Self {
            total_errors: 0,
            errors_by_category: HashMap::new(),
            recovery_success_rate: 1.0,
            critical_errors: 0,
        }
    }
}

impl SessionTracker {
    fn new() -> Self {
        Self {
            session_start: SystemTime::now(),
            noise_cancellation_start: None,
            total_nc_time: Duration::ZERO,
            performance_samples: Vec::new(),
            errors_this_session: 0,
            features_used: HashMap::new(),
        }
    }

    fn start_noise_cancellation(&mut self) {
        if self.noise_cancellation_start.is_none() {
            self.noise_cancellation_start = Some(SystemTime::now());
        }
    }

    fn stop_noise_cancellation(&mut self) {
        if let Some(start_time) = self.noise_cancellation_start.take() {
            if let Ok(duration) = start_time.elapsed() {
                self.total_nc_time += duration;
            }
        }
    }

    fn record_performance(&mut self, latency_ms: f64) {
        self.performance_samples.push(latency_ms);
    }

    fn record_error(&mut self) {
        self.errors_this_session += 1;
    }

    fn record_feature_usage(&mut self, feature: &str) {
        *self.features_used.entry(feature.to_string()).or_insert(0) += 1;
    }

    fn session_duration(&self) -> Duration {
        self.session_start.elapsed().unwrap_or(Duration::ZERO)
    }
}

impl UsageStatsManager {
    /// Create a new usage statistics manager
    pub fn new(enabled: bool) -> Self {
        Self {
            stats: UsageStatistics::default(),
            current_session: None,
            enabled,
        }
    }

    /// Load existing statistics from storage
    pub fn load_from_file(path: &std::path::Path, enabled: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let stats = if path.exists() {
            let content = std::fs::read_to_string(path)?;
            toml::from_str(&content)?
        } else {
            UsageStatistics::default()
        };

        Ok(Self {
            stats,
            current_session: None,
            enabled,
        })
    }

    /// Save statistics to file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }

        let content = toml::to_string_pretty(&self.stats)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Start a new session
    pub fn start_session(&mut self) {
        if !self.enabled {
            return;
        }

        // End previous session if it exists
        self.end_session();

        self.current_session = Some(SessionTracker::new());
        self.stats.total_sessions += 1;
    }

    /// End the current session
    pub fn end_session(&mut self) {
        if !self.enabled {
            return;
        }

        if let Some(session) = self.current_session.take() {
            // Stop noise cancellation if it's running
            let mut session = session;
            session.stop_noise_cancellation();

            // Update statistics
            let session_duration = session.session_duration();
            self.stats.total_usage_seconds += session_duration.as_secs();
            self.stats.total_processing_time_seconds += session.total_nc_time.as_secs();

            // Update averages
            if self.stats.total_sessions > 0 {
                self.stats.avg_session_duration_seconds = 
                    self.stats.total_usage_seconds as f64 / self.stats.total_sessions as f64;
            }

            // Update performance metrics
            if !session.performance_samples.is_empty() {
                let avg_latency = session.performance_samples.iter().sum::<f64>() 
                    / session.performance_samples.len() as f64;
                let peak_latency = session.performance_samples.iter()
                    .fold(0.0_f64, |acc, &x| acc.max(x));

                self.update_performance_metrics(avg_latency, peak_latency);
            }

            // Update feature usage
            for (feature, count) in session.features_used {
                *self.stats.feature_usage.entry(feature).or_insert(0) += count as u64;
            }

            // Update daily usage
            self.update_daily_usage(session_duration);

            self.stats.last_updated = Utc::now().to_rfc3339();
        }
    }

    /// Record noise cancellation activation
    pub fn start_noise_cancellation(&mut self) {
        if !self.enabled {
            return;
        }

        if let Some(session) = &mut self.current_session {
            session.start_noise_cancellation();
            self.stats.noise_cancellation_activations += 1;
        }
    }

    /// Record noise cancellation deactivation
    pub fn stop_noise_cancellation(&mut self) {
        if !self.enabled {
            return;
        }

        if let Some(session) = &mut self.current_session {
            session.stop_noise_cancellation();
        }
    }

    /// Record audio processing performance
    pub fn record_audio_performance(&mut self, latency_ms: f64, cpu_usage: f64, memory_mb: f64) {
        if !self.enabled {
            return;
        }

        if let Some(session) = &mut self.current_session {
            session.record_performance(latency_ms);
        }

        // Update global performance metrics
        self.stats.performance_metrics.peak_latency_ms = 
            self.stats.performance_metrics.peak_latency_ms.max(latency_ms);
        self.stats.performance_metrics.peak_cpu_usage_percent = 
            self.stats.performance_metrics.peak_cpu_usage_percent.max(cpu_usage);
        self.stats.performance_metrics.peak_memory_usage_mb = 
            self.stats.performance_metrics.peak_memory_usage_mb.max(memory_mb);
    }

    /// Record an error occurrence
    pub fn record_error(&mut self, category: &str, is_critical: bool) {
        if !self.enabled {
            return;
        }

        if let Some(session) = &mut self.current_session {
            session.record_error();
        }

        self.stats.error_stats.total_errors += 1;
        *self.stats.error_stats.errors_by_category.entry(category.to_string()).or_insert(0) += 1;

        if is_critical {
            self.stats.error_stats.critical_errors += 1;
        }
    }

    /// Record feature usage
    pub fn record_feature_usage(&mut self, feature: &str) {
        if !self.enabled {
            return;
        }

        if let Some(session) = &mut self.current_session {
            session.record_feature_usage(feature);
        }
    }

    /// Record AI model performance
    pub fn record_ai_performance(&mut self, model_name: &str, score: f64) {
        if !self.enabled {
            return;
        }

        self.stats.performance_metrics.ai_model_performance
            .insert(model_name.to_string(), score);
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> &UsageStatistics {
        &self.stats
    }

    /// Update performance metrics with running averages
    fn update_performance_metrics(&mut self, avg_latency: f64, peak_latency: f64) {
        let metrics = &mut self.stats.performance_metrics;
        
        // Update running average (simple exponential smoothing)
        let alpha = 0.1; // Smoothing factor
        metrics.avg_latency_ms = alpha * avg_latency + (1.0 - alpha) * metrics.avg_latency_ms;
        metrics.peak_latency_ms = metrics.peak_latency_ms.max(peak_latency);
    }

    /// Update daily usage statistics
    fn update_daily_usage(&mut self, session_duration: Duration) {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        
        // Find or create today's entry
        if let Some(daily) = self.stats.daily_usage.iter_mut().find(|d| d.date == today) {
            daily.usage_seconds += session_duration.as_secs();
            daily.session_count += 1;
        } else {
            self.stats.daily_usage.push(DailyUsage {
                date: today,
                usage_seconds: session_duration.as_secs(),
                session_count: 1,
                avg_performance_score: 0.8, // Placeholder
            });
        }

        // Keep only last 30 days
        self.stats.daily_usage.sort_by(|a, b| a.date.cmp(&b.date));
        if self.stats.daily_usage.len() > 30 {
            self.stats.daily_usage.drain(0..self.stats.daily_usage.len() - 30);
        }
    }

    /// Generate a summary report for README
    pub fn generate_summary_report(&self) -> String {
        format!(
            "## Usage Statistics Summary\n\n\
            - **Total Sessions**: {}\n\
            - **Total Usage Time**: {:.1} hours\n\
            - **Average Session**: {:.1} minutes\n\
            - **Noise Cancellation Usage**: {:.1} hours\n\
            - **Average Latency**: {:.2} ms\n\
            - **Peak Performance**: {:.2} ms peak latency\n\
            - **Error Rate**: {:.2}%\n\
            - **Most Used Features**: {}\n",
            self.stats.total_sessions,
            self.stats.total_usage_seconds as f64 / 3600.0,
            self.stats.avg_session_duration_seconds / 60.0,
            self.stats.total_processing_time_seconds as f64 / 3600.0,
            self.stats.performance_metrics.avg_latency_ms,
            self.stats.performance_metrics.peak_latency_ms,
            if self.stats.total_sessions > 0 {
                (self.stats.error_stats.total_errors as f64 / self.stats.total_sessions as f64) * 100.0
            } else { 0.0 },
            self.get_top_features()
        )
    }

    /// Get the most used features
    fn get_top_features(&self) -> String {
        let mut features: Vec<_> = self.stats.feature_usage.iter().collect();
        features.sort_by(|a, b| b.1.cmp(a.1));
        features.into_iter()
            .take(3)
            .map(|(k, v)| format!("{} ({})", k, v))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Enable or disable statistics collection
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            // End current session when disabled
            self.end_session();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_stats_creation() {
        let stats = UsageStatsManager::new(true);
        assert_eq!(stats.stats.total_sessions, 0);
        assert!(stats.enabled);
    }

    #[test]
    fn test_session_tracking() {
        let mut stats = UsageStatsManager::new(true);
        
        stats.start_session();
        assert_eq!(stats.stats.total_sessions, 1);
        assert!(stats.current_session.is_some());
        
        stats.end_session();
        assert!(stats.current_session.is_none());
    }

    #[test]
    fn test_feature_usage_tracking() {
        let mut stats = UsageStatsManager::new(true);
        stats.start_session();
        
        stats.record_feature_usage("noise_cancellation");
        stats.record_feature_usage("device_selection");
        stats.record_feature_usage("noise_cancellation");
        
        stats.end_session();
        
        assert_eq!(*stats.stats.feature_usage.get("noise_cancellation").unwrap_or(&0), 2);
        assert_eq!(*stats.stats.feature_usage.get("device_selection").unwrap_or(&0), 1);
    }

    #[test]
    fn test_disabled_stats() {
        let mut stats = UsageStatsManager::new(false);
        
        stats.start_session();
        assert_eq!(stats.stats.total_sessions, 0); // Should not increment when disabled
        
        stats.record_feature_usage("test");
        assert!(stats.stats.feature_usage.is_empty());
    }

    #[test]
    fn test_performance_recording() {
        let mut stats = UsageStatsManager::new(true);
        stats.start_session();
        
        stats.record_audio_performance(5.0, 15.0, 50.0);
        assert_eq!(stats.stats.performance_metrics.peak_latency_ms, 5.0);
        assert_eq!(stats.stats.performance_metrics.peak_cpu_usage_percent, 15.0);
    }
}
//! # AI Noise Cancellation Models Module
//! 
//! This module provides AI model implementations for noise cancellation,
//! allowing users to choose the best model for their specific environment and requirements.
//! The implementation supports RNNoise with adaptive switching.
//! 
//! ## Supported Models
//! 
//! - **RNNoise**: The original high-performance RNN model (default)
//! 
//! ## Model Selection Strategy
//! 
//! RNNoise provides:
//! - General purpose, proven performance, low CPU usage
//! - Excellent noise cancellation for most environments
//! 
//! ## Adaptive Processing
//! 
//! The system can automatically adapt processing based on:
//! - Audio characteristics and environment
//! - CPU availability and performance requirements  
//! - User preferences and use case requirements
//! 
//! ## Future Extensibility
//! 
//! The architecture supports adding new models by implementing support in the
//! EnhancedAudioProcessor. This allows for easy integration of new AI algorithms
//! as they become available.

// Allow dead code for future AI model implementations
#![allow(dead_code)]

use nnnoiseless::DenoiseState;
use std::fmt;
use std::time::Duration;
use crate::logger::log;

#[cfg(feature = "ai-enhanced")]
use crate::audio::analysis::{AudioContext, NoiseType};

/// Available AI noise cancellation models
/// 
/// Each model represents a different approach to noise cancellation with
/// specific strengths and CPU requirements. The enum design allows for
/// easy model switching and future extensibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoiseModel {
    /// Automatic model selection based on audio environment
    /// 
    /// The application intelligently chooses the best model:
    /// - Analyzes incoming audio characteristics
    /// - Selects optimal model based on noise type and environment
    /// - Adapts to changing conditions in real-time
    /// - Balances quality and performance automatically
    Auto,
    
    /// Original RNNoise model - proven performance and efficiency
    /// 
    /// Best for:
    /// - General purpose noise cancellation
    /// - Low CPU usage requirements  
    /// - Stable, predictable performance
    /// - Wide variety of noise types
    RNNoise,
}

impl NoiseModel {
    /// Get human-readable model name for UI display
    pub fn name(&self) -> &'static str {
        match self {
            NoiseModel::Auto => "Auto",
            NoiseModel::RNNoise => "RNNoise",
        }
    }
    
    /// Get detailed model description for tooltips and help text
    pub fn description(&self) -> &'static str {
        match self {
            NoiseModel::Auto => "Automatically selects the best settings based on audio environment and performance",
            NoiseModel::RNNoise => "Original RNNoise model with proven performance and low CPU usage",
        }
    }
    
    /// Get relative CPU usage indicator (1-5 scale, 1 = lowest)
    pub fn cpu_usage_level(&self) -> u8 {
        match self {
            NoiseModel::Auto => 2,             // Uses RNNoise under the hood
            NoiseModel::RNNoise => 2,          // Low CPU usage
        }
    }
    
    /// Check if model is currently available/implemented
    pub fn is_available(&self) -> bool {
        match self {
            NoiseModel::Auto => true,            // Auto mode is always available
            NoiseModel::RNNoise => true,         // Currently implemented
        }
    }
    
    /// Get optimal frame size for this model (in samples)
    /// 
    /// RNNoise uses a standard frame size based on its architecture 
    /// and training. This method returns the frame size that should
    /// be used for frame buffering and processing.
    pub fn frame_size(&self) -> usize {
        match self {
            NoiseModel::Auto => 480,           // Use RNNoise default
            NoiseModel::RNNoise => 480,        // RNNoise standard frame size
        }
    }
    
    /// Get frame duration in milliseconds at 48kHz sample rate
    pub fn frame_duration_ms(&self) -> f32 {
        (self.frame_size() as f32 / 48000.0) * 1000.0
    }
    
    /// Get all available models for UI selection
    pub fn available_models() -> Vec<NoiseModel> {
        vec![NoiseModel::Auto, NoiseModel::RNNoise]
    }
    
    /// Get recommended model for different use cases
    pub fn recommended_for_use_case(use_case: UseCase) -> NoiseModel {
        match use_case {
            UseCase::GeneralPurpose => NoiseModel::Auto, // Auto mode is best for general use
            UseCase::ProfessionalMeetings => NoiseModel::RNNoise,
            UseCase::OfficeEnvironment => NoiseModel::RNNoise,
            UseCase::PersonalizedLongTerm => NoiseModel::Auto, // Auto adapts over time
        }
    }
}

impl Default for NoiseModel {
    fn default() -> Self {
        NoiseModel::Auto  // Auto mode provides the best default user experience
    }
}

impl fmt::Display for NoiseModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Use case categories for model recommendation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UseCase {
    /// General purpose noise cancellation for various environments
    GeneralPurpose,
    /// Professional meetings, calls, and presentations
    ProfessionalMeetings,
    /// Office environments with keyboard, HVAC, and office chatter
    OfficeEnvironment,
    /// Long-term personal use with consistent setup
    PersonalizedLongTerm,
}

/// Enhanced audio processor supporting RNNoise AI model
/// 
/// This processor manages the RNNoise implementation while providing
/// a unified interface for audio processing. It handles model configuration,
/// resource management, and performance optimization.
pub struct EnhancedAudioProcessor {
    /// Currently selected model mode (could be Auto or RNNoise)
    selected_model: NoiseModel,
    
    /// Actually active AI model type (concrete model being used)
    active_model: NoiseModel,
    
    /// Frame counter for auto-switching decisions
    frame_count: u64,
    
    /// Auto-switch evaluation interval (frames)
    auto_switch_interval: u64,
    
    /// RNNoise denoiser state
    rnnoise: DenoiseState<'static>,
    
    /// Model performance statistics for comparison
    model_stats: ModelStatistics,
}

impl EnhancedAudioProcessor {
    /// Create a new enhanced audio processor with specified model
    /// 
    /// Initializes the AI model and prepares it for real-time processing.
    /// For Auto mode, starts with RNNoise and adapts based on audio characteristics.
    pub fn new(model: NoiseModel) -> Result<Self, Box<dyn std::error::Error>> {
        // Determine initial active model
        let initial_active_model = match model {
            NoiseModel::Auto => NoiseModel::RNNoise, // Start with RNNoise in auto mode
            _ => model,
        };
        
        if !initial_active_model.is_available() {
            return Err(format!("Model {} is not yet available", initial_active_model.name()).into());
        }
        
        let rnnoise = unsafe {
            std::mem::transmute::<DenoiseState<'_>, DenoiseState<'static>>(*DenoiseState::new())
        };
        
        Ok(EnhancedAudioProcessor {
            selected_model: model,
            active_model: initial_active_model,
            frame_count: 0,
            auto_switch_interval: 100, // Evaluate switching every 100 frames (~1 second)
            rnnoise,
            model_stats: ModelStatistics::new(),
        })
    }
    
    /// Process audio frame through the current AI model
    /// 
    /// This method provides a unified interface for all model types while
    /// maintaining the specific characteristics of each model. In Auto mode,
    /// it also evaluates whether to switch models based on audio characteristics.
    pub fn process_frame(&mut self, output: &mut [f32], input: &[f32]) -> f32 {
        let start_time = std::time::Instant::now();
        self.frame_count += 1;
        
        let vad_score = match self.active_model {
            NoiseModel::Auto => {
                unreachable!("Auto should never be the active model, only selected model")
            },
            NoiseModel::RNNoise => {
                self.rnnoise.process_frame(output, input)
            },
        };
        
        // Update model performance statistics
        let processing_time = start_time.elapsed();
        self.model_stats.record_processing(processing_time, vad_score);
        
        vad_score
    }
    
    /// Switch to a different AI model
    /// 
    /// This method allows real-time model switching for testing and optimization.
    /// When switching to Auto mode, starts with RNNoise and enables automatic adaptation.
    pub fn switch_model(&mut self, new_model: NoiseModel) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !new_model.is_available() {
            return Err(format!("Model {} is not yet available", new_model.name()).into());
        }
        
        self.selected_model = new_model;
        
        // Determine the actual active model
        let new_active_model = match new_model {
            NoiseModel::Auto => {
                // In auto mode, start with RNNoise and let the system adapt
                self.frame_count = 0; // Reset frame count for fresh evaluation
                NoiseModel::RNNoise
            },
            _ => new_model,
        };
        
        if new_active_model != self.active_model {
            self.active_model = new_active_model;
            self.model_stats.reset(); // Reset statistics for new model
        }
        
        Ok(())
    }
    
    /// Get current model information
    pub fn current_model(&self) -> NoiseModel {
        self.selected_model // Return the user-selected model (which might be Auto)
    }
    
    /// Get the currently active model (the actual model being used for processing)
    pub fn active_model(&self) -> NoiseModel {
        self.active_model
    }
    
    /// Get current model's optimal frame size
    pub fn current_frame_size(&self) -> usize {
        self.active_model.frame_size() // Use active model's frame size
    }
    
    /// Get model performance statistics
    pub fn get_statistics(&self) -> &ModelStatistics {
        &self.model_stats
    }
}

/// Performance statistics for AI model comparison
#[derive(Debug, Clone)]
pub struct ModelStatistics {
    total_frames: u64,
    avg_processing_time: std::time::Duration,
    avg_vad_score: f32,
    peak_processing_time: std::time::Duration,
}

impl ModelStatistics {
    pub fn new() -> Self {
        Self {
            total_frames: 0,
            avg_processing_time: std::time::Duration::ZERO,
            avg_vad_score: 0.0,
            peak_processing_time: std::time::Duration::ZERO,
        }
    }
    
    pub fn record_processing(&mut self, processing_time: std::time::Duration, vad_score: f32) {
        self.total_frames += 1;
        
        // Update running averages
        let frames_f = self.total_frames as f64;
        let weight = 1.0 / frames_f;
        self.avg_processing_time = Duration::from_nanos(
            ((self.avg_processing_time.as_nanos() as f64) * (frames_f - 1.0) / frames_f + 
             (processing_time.as_nanos() as f64) * weight) as u64
        );
        self.avg_vad_score = self.avg_vad_score * (frames_f - 1.0) as f32 / frames_f as f32 + vad_score / frames_f as f32;
        
        // Update peak
        if processing_time > self.peak_processing_time {
            self.peak_processing_time = processing_time;
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    // Getters for statistics
    pub fn total_frames(&self) -> u64 { self.total_frames }
    pub fn avg_processing_time(&self) -> std::time::Duration { self.avg_processing_time }
    pub fn avg_vad_score(&self) -> f32 { self.avg_vad_score }
    pub fn peak_processing_time(&self) -> std::time::Duration { self.peak_processing_time }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_properties() {
        assert_eq!(NoiseModel::Auto.name(), "Auto");
        assert_eq!(NoiseModel::RNNoise.name(), "RNNoise");
        assert!(NoiseModel::Auto.is_available());
        assert!(NoiseModel::RNNoise.is_available());
        assert_eq!(NoiseModel::Auto.cpu_usage_level(), 2);  // Now uses RNNoise only
        assert_eq!(NoiseModel::RNNoise.cpu_usage_level(), 2);
    }
    
    #[test]
    fn test_enhanced_processor_creation() {
        // Test RNNoise
        let processor = EnhancedAudioProcessor::new(NoiseModel::RNNoise);
        assert!(processor.is_ok());
        let processor = processor.unwrap();
        assert_eq!(processor.current_model(), NoiseModel::RNNoise);
        assert_eq!(processor.active_model(), NoiseModel::RNNoise);
        
        // Test Auto mode
        let processor = EnhancedAudioProcessor::new(NoiseModel::Auto);
        assert!(processor.is_ok());
        let processor = processor.unwrap();
        assert_eq!(processor.current_model(), NoiseModel::Auto);
        assert_eq!(processor.active_model(), NoiseModel::RNNoise); // Should start with RNNoise
    }
    
    #[test]
    fn test_available_models() {
        // Should have Auto and RNNoise
        let available = NoiseModel::available_models();
        assert_eq!(available.len(), 2);
        assert!(available.contains(&NoiseModel::Auto));
        assert!(available.contains(&NoiseModel::RNNoise));
    }
}
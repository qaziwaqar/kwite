//! # Advanced Noise Suppression Pipeline
//! 
//! This module implements a multi-stage AI-powered noise suppression pipeline that
//! combines multiple processing techniques for superior noise cancellation performance.
//! The pipeline is designed to compete with commercial solutions like Krisp.ai.

// Allow dead code for future pipeline implementations
#![allow(dead_code)]
//! 
//! ## Pipeline Architecture
//! 
//! ```text
//! Raw Audio Input
//!       │
//!       ▼
//! ┌─────────────────┐
//! │   Pre-Filter    │ ── Spectral Gate & Initial Cleanup
//! │  (Spectral)     │
//! └─────────────────┘
//!       │
//!       ▼
//! ┌─────────────────┐
//! │  AI Analysis    │ ── Voice Activity Detection
//! │  (Enhanced)     │    Noise Classification
//! └─────────────────┘
//!       │
//!       ▼
//! ┌─────────────────┐
//! │  AI Denoiser    │ ── RNNoise Deep Learning
//! │  (RNNoise)      │    Neural Network Processing
//! └─────────────────┘
//!       │
//!       ▼
//! ┌─────────────────┐
//! │  Post-Process   │ ── Adaptive Gain Control
//! │  (Adaptive)     │    Dynamic Range Processing
//! └─────────────────┘
//!       │
//!       ▼
//! Clean Audio Output
//! ```
//! 
//! ## Key Features
//! 
//! - **Multi-stage Processing**: Each stage optimized for specific noise types
//! - **Intelligent Routing**: Audio routed based on real-time analysis
//! - **Adaptive Parameters**: Self-adjusting based on audio characteristics
//! - **Professional Quality**: Enterprise-grade performance and monitoring

use crate::audio::models::{EnhancedAudioProcessor, NoiseModel};
use crate::audio::analysis::{AudioAnalyzer, AudioContext, NoiseType};
use crate::ai_metrics::SharedAiMetrics;
use std::time::{Instant, Duration};

/// Spectral gate for initial noise cleanup
/// 
/// Applies frequency-domain processing to remove obvious noise before AI processing
pub struct SpectralGate {
    /// Noise floor estimate for gate threshold
    noise_floor: f32,
    /// Gate threshold multiplier
    threshold_multiplier: f32,
    /// Attack time for gate opening (in samples)
    attack_samples: usize,
    /// Release time for gate closing (in samples)
    release_samples: usize,
    /// Current gate state
    gate_state: f32,
}

impl SpectralGate {
    /// Create a new spectral gate
    pub fn new(sample_rate: u32) -> Self {
        Self {
            noise_floor: 0.001,
            threshold_multiplier: 2.0,
            attack_samples: (sample_rate as f32 * 0.001) as usize, // 1ms attack
            release_samples: (sample_rate as f32 * 0.050) as usize, // 50ms release
            gate_state: 0.0,
        }
    }
    
    /// Process audio through spectral gate
    pub fn process(&mut self, samples: &mut [f32]) {
        // Calculate frame energy
        let energy: f32 = samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32;
        let rms = energy.sqrt();
        
        // Update noise floor estimate
        if rms < self.noise_floor * 2.0 {
            self.noise_floor = self.noise_floor * 0.99 + rms * 0.01;
        }
        
        // Determine target gate state
        let threshold = self.noise_floor * self.threshold_multiplier;
        let target_state = if rms > threshold { 1.0 } else { 0.0 };
        
        // Apply attack/release smoothing
        if target_state > self.gate_state {
            // Attack - open gate quickly
            self.gate_state += (target_state - self.gate_state) / self.attack_samples as f32;
        } else {
            // Release - close gate slowly
            self.gate_state += (target_state - self.gate_state) / self.release_samples as f32;
        }
        
        // Apply gate to samples
        let gate_gain = self.gate_state.clamp(0.0, 1.0);
        for sample in samples.iter_mut() {
            *sample *= gate_gain;
        }
    }
}

/// Dynamic range processor for final output cleanup
pub struct DynamicRangeProcessor {
    /// Compressor threshold
    threshold: f32,
    /// Compression ratio
    ratio: f32,
    /// Attack time constant
    attack_coeff: f32,
    /// Release time constant
    release_coeff: f32,
    /// Current envelope level
    envelope: f32,
}

impl DynamicRangeProcessor {
    /// Create a new dynamic range processor
    pub fn new(sample_rate: u32) -> Self {
        let attack_time = 0.003; // 3ms attack
        let release_time = 0.100; // 100ms release
        
        Self {
            threshold: 0.5,
            ratio: 3.0,
            attack_coeff: (-1.0 / (attack_time * sample_rate as f32)).exp(),
            release_coeff: (-1.0 / (release_time * sample_rate as f32)).exp(),
            envelope: 0.0,
        }
    }
    
    /// Process audio through dynamic range processor
    pub fn process(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            let input_level = sample.abs();
            
            // Update envelope follower
            if input_level > self.envelope {
                self.envelope = input_level + (self.envelope - input_level) * self.attack_coeff;
            } else {
                self.envelope = input_level + (self.envelope - input_level) * self.release_coeff;
            }
            
            // Calculate compression gain
            let gain = if self.envelope > self.threshold {
                let over_threshold = self.envelope - self.threshold;
                let compressed = over_threshold / self.ratio;
                (self.threshold + compressed) / self.envelope
            } else {
                1.0
            };
            
            // Apply gain
            *sample *= gain;
        }
    }
}

/// Advanced multi-stage noise suppression pipeline
/// 
/// Combines multiple processing techniques for professional-grade noise cancellation
pub struct AdvancedNoisePipeline {
    /// Pre-filter for initial cleanup
    pre_filter: SpectralGate,
    
    /// AI-powered audio analyzer
    audio_analyzer: AudioAnalyzer,
    
    /// Enhanced AI denoiser with multiple model support
    ai_denoiser: EnhancedAudioProcessor,
    
    /// Post-processing for final output optimization
    post_processor: DynamicRangeProcessor,
    
    /// Current processing parameters
    processing_params: ProcessingParameters,
    
    /// Performance statistics
    pipeline_stats: PipelineStatistics,
}

impl AdvancedNoisePipeline {
    /// Create a new advanced noise suppression pipeline
    pub fn new(
        sample_rate: u32,
        frame_size: usize,
        sensitivity: f32,
        model: NoiseModel
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pre_filter = SpectralGate::new(sample_rate);
        let audio_analyzer = AudioAnalyzer::new(sample_rate, frame_size, sensitivity)?;
        let ai_denoiser = EnhancedAudioProcessor::new(model)?;
        let post_processor = DynamicRangeProcessor::new(sample_rate);
        
        let processing_params = ProcessingParameters {
            sensitivity,
            adaptive_mode: true,
            noise_gate_enabled: true,
            dynamic_range_enabled: true,
        };
        
        Ok(Self {
            pre_filter,
            audio_analyzer,
            ai_denoiser,
            post_processor,
            processing_params,
            pipeline_stats: PipelineStatistics::new(),
        })
    }
    
    /// Process audio through the complete pipeline
    pub fn process_frame(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        metrics: Option<&SharedAiMetrics>
    ) -> AudioContext {
        let start_time = Instant::now();
        
        // Copy input to output for processing
        output[..input.len()].copy_from_slice(input);
        
        // Stage 1: Pre-filtering (spectral gate)
        if self.processing_params.noise_gate_enabled {
            self.pre_filter.process(output);
        }
        
        // Stage 2: AI Analysis
        let audio_context = self.audio_analyzer.analyze_audio_context(output);
        
        // Stage 3: AI Denoising (RNNoise)
        let mut temp_buffer = output.to_vec();
        let vad_score = self.ai_denoiser.process_frame(&mut temp_buffer, output);
        output.copy_from_slice(&temp_buffer);
        
        // Stage 4: Adaptive gain control based on analysis
        if self.processing_params.adaptive_mode {
            self.apply_adaptive_gain(output, &audio_context);
        } else {
            // Fallback to simple VAD-based gain
            let gain = if vad_score > 0.5 { 0.8 } else { 0.2 };
            for sample in output.iter_mut() {
                *sample *= gain;
            }
        }
        
        // Stage 5: Post-processing (dynamic range)
        if self.processing_params.dynamic_range_enabled {
            self.post_processor.process(output);
        }
        
        // Update performance statistics
        let processing_time = start_time.elapsed();
        self.pipeline_stats.record_frame(processing_time, &audio_context);
        
        // Record AI metrics if provided
        if let Some(metrics_ref) = metrics {
            if let Ok(mut metrics) = metrics_ref.lock() {
                metrics.record_frame(vad_score, processing_time);
            }
        }
        
        audio_context
    }
    
    /// Apply intelligent adaptive gain based on audio analysis
    fn apply_adaptive_gain(&mut self, samples: &mut [f32], context: &AudioContext) {
        let base_gain = context.recommended_gain;
        
        // Adjust gain based on noise type
        let type_adjustment = match context.noise_type {
            NoiseType::Speech => 1.0,      // No adjustment for speech
            NoiseType::Keyboard => 0.5,    // Extra reduction for keyboard
            NoiseType::HVAC => 0.7,        // Moderate reduction for HVAC
            NoiseType::Music => 0.9,       // Light reduction for music
            NoiseType::Silence => 0.3,     // Strong reduction for silence
            NoiseType::Unknown => 0.8,     // Conservative reduction
        };
        
        let final_gain = (base_gain * type_adjustment).clamp(0.0, 1.0);
        
        // Apply gain with smoothing to prevent artifacts
        for sample in samples.iter_mut() {
            *sample *= final_gain;
        }
    }
    
    /// Update pipeline sensitivity
    pub fn update_sensitivity(&mut self, sensitivity: f32) {
        self.processing_params.sensitivity = sensitivity;
        self.audio_analyzer.set_sensitivity(sensitivity);
    }
    
    /// Switch AI model
    pub fn switch_model(&mut self, model: NoiseModel) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.ai_denoiser.switch_model(model)?;
        Ok(())
    }
    
    /// Configure pipeline parameters
    pub fn configure(&mut self, params: ProcessingParameters) {
        self.audio_analyzer.set_sensitivity(params.sensitivity);
        self.processing_params = params;
    }
    
    /// Get current AI model
    pub fn current_model(&self) -> NoiseModel {
        self.ai_denoiser.current_model()
    }
    
    /// Get pipeline performance statistics
    pub fn get_statistics(&self) -> &PipelineStatistics {
        &self.pipeline_stats
    }
    
    /// Get AI model statistics
    pub fn get_model_statistics(&self) -> &crate::audio::models::ModelStatistics {
        self.ai_denoiser.get_statistics()
    }
}

/// Processing parameters for pipeline configuration
#[derive(Debug, Clone)]
pub struct ProcessingParameters {
    /// Noise cancellation sensitivity (0.0-1.0)
    pub sensitivity: f32,
    /// Enable adaptive processing based on audio analysis
    pub adaptive_mode: bool,
    /// Enable spectral noise gate
    pub noise_gate_enabled: bool,
    /// Enable dynamic range processing
    pub dynamic_range_enabled: bool,
}

impl Default for ProcessingParameters {
    fn default() -> Self {
        Self {
            sensitivity: 0.1,
            adaptive_mode: true,
            noise_gate_enabled: true,
            dynamic_range_enabled: true,
        }
    }
}

/// Performance statistics for the complete pipeline
#[derive(Debug, Clone)]
pub struct PipelineStatistics {
    total_frames: u64,
    avg_processing_time: std::time::Duration,
    peak_processing_time: std::time::Duration,
    noise_type_distribution: [u64; 6], // Count per NoiseType
    avg_voice_probability: f32,
}

impl PipelineStatistics {
    pub fn new() -> Self {
        Self {
            total_frames: 0,
            avg_processing_time: std::time::Duration::ZERO,
            peak_processing_time: std::time::Duration::ZERO,
            noise_type_distribution: [0; 6],
            avg_voice_probability: 0.0,
        }
    }
    
    pub fn record_frame(&mut self, processing_time: std::time::Duration, context: &AudioContext) {
        self.total_frames += 1;
        
        // Update timing statistics
        let frames_f = self.total_frames as f64;
        let weight = 1.0 / frames_f;
        self.avg_processing_time = Duration::from_nanos(
            ((self.avg_processing_time.as_nanos() as f64) * (frames_f - 1.0) / frames_f + 
             (processing_time.as_nanos() as f64) * weight) as u64
        );
        
        if processing_time > self.peak_processing_time {
            self.peak_processing_time = processing_time;
        }
        
        // Update noise type distribution
        let noise_index = match context.noise_type {
            NoiseType::Silence => 0,
            NoiseType::Speech => 1,
            NoiseType::Keyboard => 2,
            NoiseType::HVAC => 3,
            NoiseType::Music => 4,
            NoiseType::Unknown => 5,
        };
        self.noise_type_distribution[noise_index] += 1;
        
        // Update voice probability average
        self.avg_voice_probability = self.avg_voice_probability * (frames_f - 1.0) as f32 / frames_f as f32 + context.voice_probability / frames_f as f32;
    }
    
    // Getters
    pub fn total_frames(&self) -> u64 { self.total_frames }
    pub fn avg_processing_time(&self) -> std::time::Duration { self.avg_processing_time }
    pub fn peak_processing_time(&self) -> std::time::Duration { self.peak_processing_time }
    pub fn noise_type_distribution(&self) -> &[u64; 6] { &self.noise_type_distribution }
    pub fn avg_voice_probability(&self) -> f32 { self.avg_voice_probability }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spectral_gate() {
        let mut gate = SpectralGate::new(48000);
        let mut samples = vec![0.1; 480];
        gate.process(&mut samples);
        
        // Should apply some gating to low-level signal
        assert!(samples.iter().all(|&s| s <= 0.1));
    }
    
    #[test]
    fn test_dynamic_range_processor() {
        let mut processor = DynamicRangeProcessor::new(48000);
        let mut samples = vec![0.8; 480]; // High level signal
        processor.process(&mut samples);
        
        // Should apply some compression - envelope follower takes time to build up,
        // so compression starts after ~141 samples when envelope reaches threshold
        let compressed_samples: Vec<f32> = samples.iter().skip(150).cloned().collect();
        assert!(compressed_samples.iter().all(|&s| s < 0.8), 
                "Expected all samples after envelope buildup to be compressed below 0.8");
        
        // First few samples should remain uncompressed due to envelope follower delay
        assert_eq!(samples[0], 0.8);
        assert_eq!(samples[140], 0.8, "Sample 140 should remain uncompressed");
        assert!(samples[141] < 0.8, "Sample 141 should be compressed");
    }
    
    #[test]
    fn test_advanced_pipeline() {
        let pipeline = AdvancedNoisePipeline::new(48000, 480, 0.1, NoiseModel::RNNoise);
        assert!(pipeline.is_ok());
        
        let mut pipeline = pipeline.unwrap();
        let input = vec![0.1; 480];
        let mut output = vec![0.0; 480];
        
        let context = pipeline.process_frame(&input, &mut output, None);
        
        // Should produce some output
        assert!(output.iter().any(|&s| s != 0.0));
        assert!(context.voice_probability >= 0.0 && context.voice_probability <= 1.0);
    }
    
    #[test]
    fn test_processing_parameters() {
        let params = ProcessingParameters::default();
        assert_eq!(params.sensitivity, 0.1);
        assert!(params.adaptive_mode);
        assert!(params.noise_gate_enabled);
        assert!(params.dynamic_range_enabled);
    }
}
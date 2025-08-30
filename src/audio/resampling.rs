//! # Audio Resampling and Frame Adaptation Module
//!
//! This module provides utilities for handling different sample rates and adapting
//! audio frames for optimal AI processing. It's particularly important for virtual
//! audio device configurations that might use 44.1kHz instead of the optimal 48kHz.
//!
//! ## Key Features
//!
//! - **Sample Rate Detection**: Identify and adapt to different sample rates
//! - **Frame Size Calculation**: Calculate optimal frame sizes for different sample rates
//! - **Simple Resampling**: Basic resampling for small sample rate differences
//! - **Quality Preservation**: Maintain audio quality during adaptation

use std::collections::VecDeque;

/// Audio resampler for handling sample rate differences
pub struct SimpleResampler {
    /// Input sample rate
    input_rate: u32,
    /// Output sample rate 
    output_rate: u32,
    /// Internal buffer for resampling
    buffer: VecDeque<f32>,
    /// Interpolation ratio
    ratio: f64,
    /// Current fractional position
    position: f64,
}

impl SimpleResampler {
    /// Create a new resampler for the given sample rates
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        Self {
            input_rate,
            output_rate,
            buffer: VecDeque::new(),
            ratio: input_rate as f64 / output_rate as f64,
            position: 0.0,
        }
    }
    
    /// Check if resampling is needed
    pub fn needs_resampling(&self) -> bool {
        self.input_rate != self.output_rate
    }
    
    /// Process audio samples through the resampler
    /// 
    /// Uses linear interpolation for basic resampling. For production use with
    /// significant sample rate differences, consider using a proper resampling library.
    pub fn process(&mut self, input: &[f32], output: &mut Vec<f32>) {
        if !self.needs_resampling() {
            // No resampling needed, direct copy
            output.clear();
            output.extend_from_slice(input);
            return;
        }
        
        // Add input samples to buffer
        self.buffer.extend(input.iter());
        
        output.clear();
        
        // Generate output samples using linear interpolation
        while self.position < self.buffer.len() as f64 - 1.0 {
            let index = self.position as usize;
            let fraction = self.position - index as f64;
            
            if index + 1 < self.buffer.len() {
                // Linear interpolation between two samples
                let sample1 = self.buffer[index];
                let sample2 = self.buffer[index + 1];
                let interpolated = sample1 + fraction as f32 * (sample2 - sample1);
                output.push(interpolated);
            }
            
            // Advance position by the resampling ratio
            self.position += self.ratio;
        }
        
        // Remove consumed samples from buffer, keeping some for next iteration
        let consumed = self.position as usize;
        if consumed > 0 {
            for _ in 0..consumed.min(self.buffer.len()) {
                self.buffer.pop_front();
            }
            self.position -= consumed as f64;
        }
    }
}

/// Calculate optimal frame size for RNNoise based on sample rate
/// 
/// RNNoise expects 10ms frames, so this calculates the number of samples
/// needed for 10ms at the given sample rate.
pub fn calculate_frame_size_for_sample_rate(sample_rate: u32) -> usize {
    // 10ms worth of samples at the given sample rate
    (sample_rate as f64 * 0.01) as usize
}

/// Calculate the actual frame duration for a given frame size and sample rate
pub fn calculate_frame_duration_ms(frame_size: usize, sample_rate: u32) -> f32 {
    (frame_size as f32 / sample_rate as f32) * 1000.0
}

/// Check if the current configuration is optimal for noise cancellation
pub fn is_optimal_configuration(sample_rate: u32, frame_size: usize) -> bool {
    // Optimal configuration is 48kHz with 480-sample frames (10ms)
    sample_rate == 48000 && frame_size == 480
}

/// Get recommended configuration message for different sample rates
pub fn get_configuration_advice(sample_rate: u32) -> String {
    match sample_rate {
        48000 => "✅ Optimal configuration (48kHz) for AI noise cancellation".to_string(),
        44100 => "⚠️  44.1kHz detected - consider setting to 48kHz for optimal AI performance".to_string(),
        _ => format!("⚠️  {}Hz detected - 48kHz recommended for best noise cancellation", sample_rate),
    }
}

/// Adapt frame size to work with RNNoise's requirements
/// 
/// RNNoise expects exactly 480 samples per frame. This function either:
/// 1. Passes through frames that are already 480 samples
/// 2. Resamples frames to 480 samples for different sample rates
/// 3. Provides warnings for suboptimal configurations
pub fn adapt_frame_for_rnnoise(
    input: &[f32], 
    sample_rate: u32,
    output: &mut Vec<f32>
) -> Result<(), String> {
    const RNNOISE_FRAME_SIZE: usize = 480;
    const OPTIMAL_SAMPLE_RATE: u32 = 48000;
    
    if sample_rate == OPTIMAL_SAMPLE_RATE && input.len() == RNNOISE_FRAME_SIZE {
        // Optimal case - direct copy
        output.clear();
        output.extend_from_slice(input);
        return Ok(());
    }
    
    if sample_rate == OPTIMAL_SAMPLE_RATE {
        // Correct sample rate but wrong frame size
        if input.len() < RNNOISE_FRAME_SIZE {
            // Pad with zeros
            output.clear();
            output.extend_from_slice(input);
            output.resize(RNNOISE_FRAME_SIZE, 0.0);
        } else {
            // Truncate to correct size
            output.clear();
            output.extend_from_slice(&input[..RNNOISE_FRAME_SIZE]);
        }
        return Ok(());
    }
    
    // Different sample rate - need to resample
    if sample_rate == 44100 {
        // Common case: 44.1kHz to 48kHz
        // 10ms at 44.1kHz = 441 samples
        // We need to resample 441 samples to 480 samples
        if input.len() != 441 {
            return Err(format!("Expected 441 samples for 44.1kHz (10ms), got {}", input.len()));
        }
        
        output.clear();
        output.resize(RNNOISE_FRAME_SIZE, 0.0);
        
        // Simple linear interpolation resampling
        let ratio = input.len() as f64 / RNNOISE_FRAME_SIZE as f64;
        for i in 0..RNNOISE_FRAME_SIZE {
            let src_pos = i as f64 * ratio;
            let src_index = src_pos as usize;
            let fraction = src_pos - src_index as f64;
            
            if src_index + 1 < input.len() {
                let sample1 = input[src_index];
                let sample2 = input[src_index + 1];
                output[i] = sample1 + fraction as f32 * (sample2 - sample1);
            } else if src_index < input.len() {
                output[i] = input[src_index];
            }
        }
        return Ok(());
    }
    
    Err(format!("Unsupported sample rate: {}Hz. Supported rates: 48000Hz (optimal), 44100Hz", sample_rate))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_size_calculation() {
        assert_eq!(calculate_frame_size_for_sample_rate(48000), 480);
        assert_eq!(calculate_frame_size_for_sample_rate(44100), 441);
        assert_eq!(calculate_frame_size_for_sample_rate(16000), 160);
    }
    
    #[test]
    fn test_frame_duration_calculation() {
        assert!((calculate_frame_duration_ms(480, 48000) - 10.0).abs() < 0.01);
        assert!((calculate_frame_duration_ms(441, 44100) - 10.0).abs() < 0.01);
    }
    
    #[test]
    fn test_optimal_configuration() {
        assert!(is_optimal_configuration(48000, 480));
        assert!(!is_optimal_configuration(44100, 441));
        assert!(!is_optimal_configuration(48000, 441));
    }
    
    #[test]
    fn test_configuration_advice() {
        let advice_48k = get_configuration_advice(48000);
        assert!(advice_48k.contains("Optimal"));
        
        let advice_44k = get_configuration_advice(44100);
        assert!(advice_44k.contains("44.1kHz"));
        assert!(advice_44k.contains("48kHz"));
    }
    
    #[test]
    fn test_frame_adaptation_optimal() {
        let input = vec![0.1; 480];
        let mut output = Vec::new();
        
        let result = adapt_frame_for_rnnoise(&input, 48000, &mut output);
        assert!(result.is_ok());
        assert_eq!(output.len(), 480);
        assert_eq!(output, input);
    }
    
    #[test]
    fn test_frame_adaptation_44khz() {
        let input = vec![0.1; 441]; // 10ms at 44.1kHz
        let mut output = Vec::new();
        
        let result = adapt_frame_for_rnnoise(&input, 44100, &mut output);
        assert!(result.is_ok());
        assert_eq!(output.len(), 480);
        // Output should be close to input value due to interpolation
        assert!((output[0] - 0.1).abs() < 0.01);
    }
    
    #[test]
    fn test_simple_resampler() {
        let mut resampler = SimpleResampler::new(44100, 48000);
        assert!(resampler.needs_resampling());
        
        let input = vec![0.1; 441];
        let mut output = Vec::new();
        resampler.process(&input, &mut output);
        
        // Should produce approximately 480 samples
        assert!(output.len() > 470 && output.len() < 490);
    }
}
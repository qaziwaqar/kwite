//! # Audio Processing Module
//! 
//! This module implements the core AI-powered noise cancellation processing using the
//! RNNoise deep learning model. The processing is designed for real-time operation
//! with minimal latency while providing effective background noise removal.
//! 
//! ## AI Model Overview
//! 
//! The noise cancellation uses RNNoise, a recurrent neural network specifically trained
//! for real-time noise suppression. Key characteristics:
//! 
//! - **Model Type**: Gated Recurrent Unit (GRU) based RNN
//! - **Training Data**: Diverse noise environments and speech patterns  
//! - **Frame Size**: 480 samples (10ms at 48kHz)
//! - **Latency**: Sub-frame processing for minimal delay
//! - **CPU Usage**: Optimized for real-time operation
//! 
//! ## Processing Strategy
//! 
//! The algorithm combines AI noise detection with traditional audio processing:
//! 
//! 1. **Voice Activity Detection (VAD)**: AI determines speech probability
//! 2. **Adaptive Gain Control**: Different processing for speech vs. noise
//! 3. **Frame-based Processing**: Handles audio in optimal chunks
//! 4. **Graceful Fade Handling**: Smooth transitions for partial frames
//! 
//! ## Why This Approach?
//! 
//! - **RNNoise Effectiveness**: Proven performance in real-world scenarios
//! - **Real-time Capable**: Designed specifically for low-latency applications  

// Allow dead code for future AI processing implementations
#![allow(dead_code)]
//! - **Balanced Processing**: Preserves speech quality while removing noise
//! - **Adaptive Behavior**: Different strategies for speech vs. background noise
//! 
//! ## Limitations
//! 
//! - **Not Suitable for Music**: Designed for human speech, not music signals
//! - **Mono Audio Only**: Expects single-channel (mono) audio input
//! - **Fixed Frame Size**: Input must be a multiple of 480 samples
//! 
//! ## Future Improvements
//! 
//! - **Dynamic Frame Sizing**: Adapt frame size to input signal characteristics
//! - **Multi-channel Support**: Process stereo or multi-channel audio
//! - **Enhanced VAD**: Improve voice activity detection accuracy
//! - **Music Mode**: Special processing mode for music signals

use std::time::Instant;
use crate::ai_metrics::SharedAiMetrics;
use crate::audio::models::EnhancedAudioProcessor;
use crate::audio::analysis::AudioContext;
use nnnoiseless::DenoiseState;

/// Process audio through AI noise cancellation
/// 
/// This function applies sophisticated noise cancellation to incoming audio using
/// a combination of AI voice activity detection and adaptive gain control. The
/// processing is optimized for real-time operation while maintaining audio quality.
/// 
/// ## Parameters
/// 
/// - `input`: Raw audio samples from microphone (mono, f32)
/// - `output`: Buffer for processed audio samples  
/// - `denoiser`: AI model state (maintains context between calls)
/// - `metrics`: Optional AI performance metrics collector for monitoring
/// 
/// ## Processing Algorithm
/// 
/// The algorithm operates on fixed-size frames (480 samples) for optimal AI performance:
/// 
/// 1. **Frame Extraction**: Split input into processing frames
/// 2. **AI Analysis**: RNNoise provides voice activity detection (VAD) score
/// 3. **Adaptive Gain**: Apply different gain based on speech probability
/// 4. **Output Assembly**: Combine processed frames into output buffer
/// 5. **Remainder Handling**: Process incomplete frames with fade-out
/// 6. **Performance Tracking**: Record AI metrics for monitoring (if provided)
/// 
/// ## Voice Activity Detection (VAD)
/// 
/// The AI model returns a VAD score (0.0 to 1.0) indicating speech probability:
/// - **0.0**: Likely background noise (silence, fan noise, keyboard clicks)
/// - **0.5**: Uncertain (mixed speech and noise)  
/// - **1.0**: Likely human speech (voice, singing, speaking)
/// 
/// ## Adaptive Gain Strategy
/// 
/// Different gain levels are applied based on VAD score:
/// - **Speech (VAD > 0.5)**: High gain (0.8) to preserve voice clarity
/// - **Noise (VAD ≤ 0.5)**: Low gain (0.1) to suppress background sounds
/// 
/// This approach provides more natural-sounding results than binary on/off switching.
/// 
/// ## Frame Size Rationale
/// 
/// The 480-sample frame size (10ms at 48kHz) is chosen because:
/// - **AI Optimization**: RNNoise is trained and optimized for this frame size
/// - **Latency**: Small enough for real-time processing (sub-20ms total latency)
/// - **Quality**: Large enough for effective frequency analysis
/// - **Efficiency**: Optimal balance between CPU usage and processing quality
pub fn process_audio(
    input: &[f32], 
    output: &mut [f32], 
    denoiser: &mut DenoiseState<'static>,
    metrics: Option<&SharedAiMetrics>
) {
    // Use the AI model's optimal frame size for processing
    // This constant is defined by the nnnoiseless library based on RNNoise requirements
    const FRAME_SIZE: usize = nnnoiseless::FRAME_SIZE;

    // Initialize output buffer to silence
    // This ensures clean output even if processing fails partway through
    output.fill(0.0);

    // Process complete frames using the AI model
    // Each frame is processed independently, allowing for frame-level parallelization
    for (i, chunk) in input.chunks_exact(FRAME_SIZE).enumerate() {
        let start_time = Instant::now();
        
        // Create temporary buffer for AI processing
        // The AI model modifies this buffer in-place during processing
        let mut frame = vec![0.0; FRAME_SIZE];

        // Apply AI noise cancellation to the frame
        // The model returns a Voice Activity Detection (VAD) score
        // VAD ranges from 0.0 (noise) to 1.0 (speech)
        let vad = denoiser.process_frame(&mut frame, chunk);

        // Record AI performance metrics if provided
        if let Some(metrics_ref) = metrics {
            let processing_time = start_time.elapsed();
            if let Ok(mut metrics) = metrics_ref.lock() {
                metrics.record_frame(vad, processing_time);
            }
        }

        // Apply adaptive gain based on voice activity detection
        // This creates more natural-sounding noise suppression than binary switching
        let gain = if vad < 0.5 { 
            0.1  // Low gain for background noise (aggressive suppression)
        } else { 
            0.8  // High gain for detected speech (preserve voice quality)
        };

        // Copy processed frame to output buffer with applied gain
        // The gain adjustment provides final volume control after AI processing
        let start = i * FRAME_SIZE;
        for (out, processed) in output[start..start + FRAME_SIZE].iter_mut()
            .zip(frame.iter()) {
            *out = processed * gain;
        }
    }

    // Handle remaining samples that don't fill a complete frame
    // This occurs when input length is not a multiple of FRAME_SIZE
    let processed_samples = (input.len() / FRAME_SIZE) * FRAME_SIZE;
    if processed_samples < input.len() {
        let remain = input.len() - processed_samples;
        
        // Apply fade-out to remaining samples to prevent audio artifacts
        // This creates a smooth transition instead of an abrupt cutoff
        for i in 0..remain {
            // Calculate fade factor: 1.0 at start, 0.0 at end
            let fade = 1.0 - (i as f32 / remain as f32);
            
            // Apply fade-out with reduced gain for safety
            // 0.5 gain prevents potential volume spikes from unprocessed audio
            output[processed_samples + i] = input[processed_samples + i] * fade * 0.5;
        }
    }
}

/// Enhanced AI processing with intelligent environmental adaptation
/// 
/// This function provides enterprise-grade AI processing that automatically adapts
/// to the user's environment and audio context. Unlike the basic processing function,
/// this system analyzes incoming audio and intelligently adjusts processing parameters.
/// 
/// ## Enhanced Features
/// 
/// - **Automatic Model Selection**: Chooses optimal AI model based on audio analysis
/// - **Environmental Adaptation**: Adapts to background noise characteristics
/// - **Context-Aware Processing**: Different strategies for speech vs. noise vs. music
/// - **Performance Optimization**: Intelligent parameter tuning for efficiency
/// - **Professional Metrics**: Comprehensive AI performance tracking
/// 
/// ## Parameters
/// 
/// - `input`: Raw audio samples from microphone (mono, f32)
/// - `output`: Buffer for processed audio samples  
/// - `processor`: Enhanced AI processor with multi-model support
/// - `context`: Audio analysis context with environmental information
/// - `metrics`: Optional AI performance metrics collector for monitoring
/// 
/// ## Processing Intelligence
/// 
/// The enhanced system makes intelligent decisions based on audio context:
/// 
/// - **Speech Detection**: Uses advanced VAD with confidence scoring
/// - **Noise Classification**: Identifies specific noise types (keyboard, HVAC, music)
/// - **Adaptive Gain**: Adjusts processing strength based on noise characteristics
/// - **Model Optimization**: Selects best AI model for current environment
/// - **Quality Preservation**: Maintains voice quality while maximizing noise reduction
pub fn process_audio_enhanced(
    input: &[f32], 
    output: &mut [f32], 
    processor: &mut EnhancedAudioProcessor,
    context: &AudioContext,
    metrics: Option<&SharedAiMetrics>
) {
    // Use the AI model's optimal frame size for processing
    const FRAME_SIZE: usize = 480; // RNNoise optimal frame size
    
    // Initialize output buffer to silence
    output.fill(0.0);
    
    // Get intelligent processing parameters based on audio context
    let processing_params = determine_processing_parameters(context);
    
    // Process complete frames using the enhanced AI system
    for (i, chunk) in input.chunks_exact(FRAME_SIZE).enumerate() {
        let start_time = Instant::now();
        
        // Create temporary buffer for AI processing
        let mut frame = vec![0.0; FRAME_SIZE];
        
        // Apply enhanced AI processing with environmental context
        let vad_score = processor.process_frame(&mut frame, chunk);
        
        // Record comprehensive AI performance metrics
        if let Some(metrics_ref) = metrics {
            let processing_time = start_time.elapsed();
            if let Ok(mut metrics) = metrics_ref.lock() {
                metrics.record_frame(vad_score, processing_time);
                metrics.update_noise_type(context.noise_type.clone());
                metrics.update_confidence(context.voice_probability);
            }
        }
        
        // Apply intelligent adaptive gain based on context and VAD
        let gain = calculate_intelligent_gain(vad_score, context, &processing_params);
        
        // Copy processed frame to output buffer with intelligent gain
        let start = i * FRAME_SIZE;
        for (out, processed) in output[start..start + FRAME_SIZE].iter_mut()
            .zip(frame.iter()) {
            *out = processed * gain;
        }
    }
    
    // Handle remaining samples with intelligent fade-out
    let processed_samples = (input.len() / FRAME_SIZE) * FRAME_SIZE;
    if processed_samples < input.len() {
        let remain = input.len() - processed_samples;
        
        // Apply intelligent fade-out based on audio context
        let fade_gain = processing_params.fade_gain;
        for i in 0..remain {
            let fade = 1.0 - (i as f32 / remain as f32);
            output[processed_samples + i] = input[processed_samples + i] * fade * fade_gain;
        }
    }
}

/// Processing parameters determined by intelligent audio analysis
#[derive(Debug, Clone)]
struct ProcessingParameters {
    /// Base gain multiplier for speech
    speech_gain: f32,
    /// Base gain multiplier for noise
    noise_gain: f32,
    /// Gain for fade-out of remaining samples
    fade_gain: f32,
    /// Confidence threshold for speech detection
    speech_threshold: f32,
}

/// Determine intelligent processing parameters based on audio context
/// 
/// This function analyzes the current audio environment and selects optimal
/// processing parameters for maximum effectiveness while preserving audio quality.
fn determine_processing_parameters(context: &AudioContext) -> ProcessingParameters {
    use crate::audio::analysis::NoiseType;
    
    // Base parameters optimized for general use
    let mut params = ProcessingParameters {
        speech_gain: 0.85,
        noise_gain: 0.15,
        fade_gain: 0.6,
        speech_threshold: 0.5,
    };
    
    // Adjust parameters based on detected noise type
    match context.noise_type {
        NoiseType::Speech => {
            // Preserve speech quality - reduce noise suppression strength
            params.speech_gain = 0.9;
            params.noise_gain = 0.3;
            params.speech_threshold = 0.4; // More sensitive to speech
        },
        NoiseType::Keyboard => {
            // Aggressive suppression for keyboard noise - it's very distinct
            params.speech_gain = 0.8;
            params.noise_gain = 0.05;
            params.speech_threshold = 0.6; // Less sensitive to avoid key clicks
        },
        NoiseType::HVAC => {
            // Moderate suppression for continuous background noise
            params.speech_gain = 0.85;
            params.noise_gain = 0.1;
            params.speech_threshold = 0.5;
        },
        NoiseType::Music => {
            // Conservative processing - music has complex harmonics
            params.speech_gain = 0.9;
            params.noise_gain = 0.4;
            params.speech_threshold = 0.3; // More sensitive to preserve musical elements
        },
        NoiseType::Silence => {
            // Very light processing for silence - preserve any subtle sounds
            params.speech_gain = 0.95;
            params.noise_gain = 0.05;
            params.speech_threshold = 0.3; // Sensitive to detect any activity
        },
        NoiseType::Unknown => {
            // Use conservative defaults for unknown noise types
            params.speech_gain = 0.8;
            params.noise_gain = 0.2;
            params.speech_threshold = 0.5;
        },
    }
    
    // Adjust based on voice probability confidence
    if context.voice_probability > 0.8 {
        // High confidence speech - preserve more audio
        params.speech_gain = (params.speech_gain + 0.1).min(1.0);
    } else if context.voice_probability < 0.2 {
        // High confidence noise - suppress more aggressively
        params.noise_gain = (params.noise_gain * 0.5).max(0.02);
    }
    
    params
}

/// Calculate intelligent gain based on VAD score, context, and processing parameters
/// 
/// This function provides more sophisticated gain control than simple threshold-based
/// switching, taking into account the audio environment and confidence levels.
fn calculate_intelligent_gain(
    vad_score: f32, 
    context: &AudioContext, 
    params: &ProcessingParameters
) -> f32 {
    // Use smooth interpolation instead of hard threshold
    let speech_confidence = if vad_score > params.speech_threshold {
        // Scale speech confidence from threshold to 1.0
        (vad_score - params.speech_threshold) / (1.0 - params.speech_threshold)
    } else {
        0.0
    };
    
    // Interpolate between noise and speech gains based on confidence
    let base_gain = params.noise_gain + speech_confidence * (params.speech_gain - params.noise_gain);
    
    // Apply additional adjustments based on environmental context
    let environmental_adjustment = match context.noise_type {
        crate::audio::analysis::NoiseType::Keyboard => {
            // Extra suppression during detected keyboard activity
            if vad_score < 0.3 { 0.7 } else { 1.0 }
        },
        crate::audio::analysis::NoiseType::HVAC => {
            // Consistent suppression for continuous background noise
            0.9
        },
        _ => 1.0, // No adjustment for other noise types
    };
    
    // Ensure final gain is within reasonable bounds
    (base_gain * environmental_adjustment).clamp(0.02, 1.0)
}
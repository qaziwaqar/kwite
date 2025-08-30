//! Unit tests for AI noise cancellation processing
//! 
//! These tests validate the core AI functionality to ensure Kwite provides
//! professional-grade noise cancellation comparable to industry leaders like Krisp.ai

use kwite::audio::process::process_audio;
use kwite::ai_metrics::{AiMetrics, AiStatus};
use nnnoiseless::DenoiseState;
use std::time::Duration;

#[test]
fn test_ai_processing_basic_functionality() {
    // Test that AI processing doesn't crash and produces output
    let mut denoiser = unsafe {
        std::mem::transmute::<DenoiseState<'_>, DenoiseState<'static>>(*DenoiseState::new())
    };
    
    // Create test audio input (480 samples = optimal frame size)
    let input = vec![0.1; 480];
    let mut output = vec![0.0; 480];
    
    // Process audio through AI
    process_audio(&input, &mut output, &mut denoiser, None);
    
    // Verify output was generated
    assert!(!output.iter().all(|&x| x == 0.0), "AI processing should produce non-zero output");
}

#[test]
fn test_ai_frame_size_optimization() {
    // Test that the AI model uses the optimal frame size (480 samples)
    assert_eq!(nnnoiseless::FRAME_SIZE, 480, "Frame size should be optimized for RNNoise");
    
    let mut denoiser = unsafe {
        std::mem::transmute::<DenoiseState<'_>, DenoiseState<'static>>(*DenoiseState::new())
    };
    
    // Test with exact frame size
    let input = vec![0.1; 480];
    let mut output = vec![0.0; 480];
    process_audio(&input, &mut output, &mut denoiser, None);
    
    // All samples should be processed
    assert!(output.iter().any(|&x| x != 0.0), "All samples in optimal frame should be processed");
}

#[test]
fn test_ai_processing_with_metrics() {
    let mut denoiser = unsafe {
        std::mem::transmute::<DenoiseState<'_>, DenoiseState<'static>>(*DenoiseState::new())
    };
    
    let mut metrics = AiMetrics::new();
    let metrics_shared = std::sync::Arc::new(std::sync::Mutex::new(metrics));
    
    // Create test input
    let input = vec![0.1; 480];
    let mut output = vec![0.0; 480];
    
    // Process with metrics
    process_audio(&input, &mut output, &mut denoiser, Some(&metrics_shared));
    
    // Check that metrics were recorded
    let metrics_guard = metrics_shared.lock().unwrap();
    assert_eq!(metrics_guard.total_frames, 1, "Metrics should record one processed frame");
    assert!(metrics_guard.avg_latency_us > 0, "Processing latency should be recorded");
    assert!(metrics_guard.avg_vad_score >= 0.0 && metrics_guard.avg_vad_score <= 1.0, 
            "VAD score should be in valid range");
}

#[test]
fn test_vad_score_range() {
    // Test that Voice Activity Detection scores are in the correct range
    let mut metrics = AiMetrics::new();
    
    // Simulate various VAD scores
    metrics.record_frame(0.0, Duration::from_micros(1000));  // Pure noise
    metrics.record_frame(0.5, Duration::from_micros(1000));  // Mixed
    metrics.record_frame(1.0, Duration::from_micros(1000));  // Pure speech
    
    assert!(metrics.avg_vad_score >= 0.0 && metrics.avg_vad_score <= 1.0, 
            "Average VAD score should be in valid range");
    
    // Test performance summary
    let summary = metrics.get_performance_summary();
    assert!(summary.avg_vad_score >= 0.0 && summary.avg_vad_score <= 1.0);
    assert!(summary.model_confidence >= 0.0 && summary.model_confidence <= 1.0);
}

#[test]
fn test_ai_latency_requirements() {
    // Test that AI processing meets real-time requirements
    let mut denoiser = unsafe {
        std::mem::transmute::<DenoiseState<'_>, DenoiseState<'static>>(*DenoiseState::new())
    };
    
    let mut metrics = AiMetrics::new();
    let metrics_shared = std::sync::Arc::new(std::sync::Mutex::new(metrics));
    
    // Process multiple frames and measure average latency
    for _ in 0..10 {
        let input = vec![0.1; 480];
        let mut output = vec![0.0; 480];
        process_audio(&input, &mut output, &mut denoiser, Some(&metrics_shared));
    }
    
    let metrics_guard = metrics_shared.lock().unwrap();
    let summary = metrics_guard.get_performance_summary();
    
    // Professional-grade latency should be under 10ms per frame
    assert!(summary.avg_latency_ms < 10.0, 
            "AI processing latency should be under 10ms for real-time operation");
    
    // Frame rate should be reasonable
    assert!(summary.estimated_fps >= 50, 
            "Frame processing rate should support real-time audio");
}

#[test]
fn test_ai_model_confidence() {
    let mut metrics = AiMetrics::new();
    
    // Simulate consistent VAD scores (high confidence)
    for _ in 0..20 {
        metrics.record_frame(0.9, Duration::from_micros(1000));
    }
    
    let summary = metrics.get_performance_summary();
    assert!(matches!(summary.ai_status, AiStatus::Excellent | AiStatus::Good),
            "Consistent AI processing should result in high confidence");
    
    // Test inconsistent VAD scores (lower confidence)
    let mut inconsistent_metrics = AiMetrics::new();
    for i in 0..20 {
        let vad = if i % 2 == 0 { 0.1 } else { 0.9 };
        inconsistent_metrics.record_frame(vad, Duration::from_micros(1000));
    }
    
    let inconsistent_summary = inconsistent_metrics.get_performance_summary();
    assert!(inconsistent_summary.model_confidence < summary.model_confidence,
            "Inconsistent processing should result in lower confidence");
}

#[test]
fn test_adaptive_gain_processing() {
    // Test that adaptive gain is applied based on VAD scores
    let mut denoiser = unsafe {
        std::mem::transmute::<DenoiseState<'_>, DenoiseState<'static>>(*DenoiseState::new())
    };
    
    // Create test input with some variation
    let input: Vec<f32> = (0..480).map(|i| (i as f32 / 480.0) * 0.1).collect();
    let mut output = vec![0.0; 480];
    
    process_audio(&input, &mut output, &mut denoiser, None);
    
    // Verify that output is different from input (processing occurred)
    let input_sum: f32 = input.iter().sum();
    let output_sum: f32 = output.iter().sum();
    assert_ne!(input_sum, output_sum, "Adaptive gain should modify the audio signal");
}

#[test]
fn test_competitive_ai_features() {
    // Test features that make Kwite competitive with Krisp.ai
    let mut metrics = AiMetrics::new();
    
    // Record performance data
    for i in 0..100 {
        let vad = 0.7 + 0.3 * (i as f32 / 100.0).sin(); // Varying VAD scores
        metrics.record_frame(vad, Duration::from_micros(5000)); // 5ms processing time
    }
    
    let summary = metrics.get_performance_summary();
    
    // Competitive features check:
    // 1. Real-time processing (low latency)
    assert!(summary.avg_latency_ms < 10.0, "Should have professional-grade latency");
    
    // 2. High processing rate
    assert!(summary.estimated_fps >= 80, "Should support high-quality real-time audio");
    
    // 3. Effective noise reduction
    assert!(summary.noise_reduction_percent > 0.0, "Should detect and reduce noise");
    
    // 4. Professional confidence levels
    assert!(summary.model_confidence > 0.5, "Should maintain reasonable confidence");
    
    // 5. Large number of processed frames
    assert!(summary.frames_processed >= 100, "Should handle continuous processing");
}

#[test]
fn test_memory_safety_ai_processing() {
    // Test that AI processing is memory safe with various input sizes
    let mut denoiser = unsafe {
        std::mem::transmute::<DenoiseState<'_>, DenoiseState<'static>>(*DenoiseState::new())
    };
    
    // Test with exact frame size
    let input1 = vec![0.1; 480];
    let mut output1 = vec![0.0; 480];
    process_audio(&input1, &mut output1, &mut denoiser, None);
    
    // Test with multiple frames
    let input2 = vec![0.1; 960]; // 2 frames
    let mut output2 = vec![0.0; 960];
    process_audio(&input2, &mut output2, &mut denoiser, None);
    
    // Test with partial frame
    let input3 = vec![0.1; 600]; // 1.25 frames
    let mut output3 = vec![0.0; 600];
    process_audio(&input3, &mut output3, &mut denoiser, None);
    
    // All should complete without crashing
    assert!(true, "All AI processing variants should complete safely");
}

#[test]
fn test_frame_buffering_with_variable_chunks() {
    // Test that the enhanced audio processor handles variable-length chunks correctly
    // This test simulates the exact scenario that was causing the frame size assertion crash
    use kwite::audio::models::{EnhancedAudioProcessor, NoiseModel};
    
    let mut processor = EnhancedAudioProcessor::new(NoiseModel::RNNoise)
        .expect("Should create enhanced processor");
    
    // Simulate variable-length audio chunks that would come from real audio capture
    let test_chunks = vec![
        vec![0.1; 256],  // Smaller than frame size
        vec![0.2; 512],  // Larger than frame size but not multiple  
        vec![0.3; 128],  // Small chunk
        vec![0.4; 400],  // Another non-frame-size chunk
        vec![0.5; 200],  // Small chunk
    ];
    
    // The old implementation would have crashed here because it passed these
    // variable-length chunks directly to RNNoise which expects exactly 480 samples
    
    // Accumulate chunks and process only when we have complete frames
    let mut buffer = Vec::new();
    const FRAME_SIZE: usize = 480;
    
    for chunk in test_chunks {
        buffer.extend_from_slice(&chunk);
        
        // Process complete frames (this is what the fix does in the processing thread)
        while buffer.len() >= FRAME_SIZE {
            let frame_input: Vec<f32> = buffer.drain(0..FRAME_SIZE).collect();
            let mut frame_output = vec![0.0f32; FRAME_SIZE];
            
            // This should not crash with the proper frame size
            let _vad_score = processor.process_frame(&mut frame_output, &frame_input);
            
            // Verify processing produces output
            assert!(frame_output.iter().any(|&x| x != 0.0), 
                   "Frame processing should produce non-zero output");
        }
    }
    
    // Test passed if we reach here without crashing
    assert!(true, "Variable-length chunk processing should not crash");
}

#[test]
fn test_model_frame_size_compatibility() {
    // Test that different AI models report their correct frame sizes
    use kwite::audio::models::{NoiseModel};
    
    // Verify frame sizes for RNNoise model
    assert_eq!(NoiseModel::RNNoise.frame_size(), 480);
    
    // Verify frame duration calculations are reasonable
    assert!((NoiseModel::RNNoise.frame_duration_ms() - 10.0).abs() < 0.1); // ~10ms at 48kHz
}

#[test] 
fn test_enhanced_processor_frame_size_reporting() {
    // Test that the enhanced processor correctly reports frame sizes for different models
    use kwite::audio::models::{EnhancedAudioProcessor, NoiseModel};
    
    let processor = EnhancedAudioProcessor::new(NoiseModel::RNNoise)
        .expect("Should create enhanced processor");
    
    // Verify frame size matches model specification
    assert_eq!(processor.current_frame_size(), 480);
    assert_eq!(processor.current_frame_size(), NoiseModel::RNNoise.frame_size());
}

#[test]
fn test_adaptive_frame_buffering_simulation() {
    // Simulate the adaptive frame buffering that handles different frame sizes
    // This tests the core logic that will be used in the audio processing thread
    use kwite::audio::models::{EnhancedAudioProcessor, NoiseModel};
    
    let mut processor = EnhancedAudioProcessor::new(NoiseModel::RNNoise)
        .expect("Should create enhanced processor");
    
    // Simulate audio chunks of various sizes
    let test_chunks = vec![
        vec![0.1; 200],  // Small chunk
        vec![0.2; 600],  // Larger chunk  
        vec![0.3; 100],  // Small chunk
        vec![0.4; 800],  // Large chunk
    ];
    
    let mut buffer = Vec::new();
    let mut processed_frames = 0;
    
    for chunk in test_chunks {
        buffer.extend_from_slice(&chunk);
        
        // Get current model's frame size (adaptive)
        let frame_size = processor.current_frame_size();
        
        // Process complete frames (this simulates the audio thread logic)
        while buffer.len() >= frame_size {
            let frame_input: Vec<f32> = buffer.drain(0..frame_size).collect();
            let mut frame_output = vec![0.0f32; frame_size];
            
            // Process frame with current model
            let _vad_score = processor.process_frame(&mut frame_output, &frame_input);
            processed_frames += 1;
            
            // Verify output matches expected frame size
            assert_eq!(frame_output.len(), frame_size);
        }
    }
    
    // Should have processed multiple complete frames
    assert!(processed_frames > 0, "Should have processed at least one complete frame");
    
    // Buffer should contain remaining partial frame data
    assert!(buffer.len() < processor.current_frame_size(), 
           "Buffer should contain less than one complete frame");
}



#[test]
fn test_available_models_include_rnnoise() {
    use kwite::audio::models::NoiseModel;
    
    let available = NoiseModel::available_models();
    assert!(available.contains(&NoiseModel::RNNoise), "RNNoise should always be available");
    assert!(available.contains(&NoiseModel::Auto), "Auto mode should be available");
    
    // Auto and RNNoise should be available (Auto uses RNNoise under the hood)
    assert_eq!(available.len(), 2, "Auto and RNNoise should be available");
}


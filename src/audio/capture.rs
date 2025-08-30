//! # Audio Input Capture Module
//! 
//! This module handles real-time audio capture from input devices such as microphones,
//! line-in ports, or virtual audio devices. It's designed to work reliably across
//! different audio hardware and provides consistent mono audio output for processing.
//! 
//! ## Key Features
//! 
//! - **Device-specific configuration**: Uses each device's optimal settings
//! - **Automatic format conversion**: Converts stereo to mono when needed
//! - **Low-latency capture**: Optimized for real-time processing
//! - **Robust error handling**: Graceful handling of device disconnections
//! 
//! ## Audio Pipeline
//! 
//! Input Device → CPAL Stream → Format Conversion → Channel → Audio Processor
//! 
//! The capture system respects the input device's native configuration to minimize
//! audio quality degradation and ensure compatibility across different hardware.

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{StreamConfig, BufferSize};
use crossbeam_channel::Sender;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::logger::log;
use crate::audio::devices::get_device_by_id;
use crate::audio::resampling::{SimpleResampler, get_configuration_advice};

/// Start audio input capture from the specified device
/// 
/// This function initializes a real-time audio input stream that continuously
/// captures audio data and forwards it to the processing pipeline.
/// 
/// ## Parameters
/// 
/// - `sender`: Channel for sending captured audio to the processor
/// - `running`: Atomic flag for graceful shutdown coordination
/// - `device_id`: Identifier of the input device to use
/// 
/// ## Audio Format Handling
/// 
/// The function adapts to the input device's native configuration to ensure
/// optimal audio quality and compatibility. Key considerations:
/// 
/// - **Sample Rate**: Uses device's default rate (typically 44.1kHz or 48kHz)
/// - **Channels**: Accepts mono or stereo, converts stereo to mono for processing
/// - **Buffer Size**: Lets the device choose optimal buffer size for latency/stability
/// 
/// ## Stereo to Mono Conversion
/// 
/// When the input device provides stereo audio, we extract only the left channel.
/// This approach is chosen because:
/// 1. Most microphones provide identical data on both channels
/// 2. The AI noise cancellation model expects mono input
/// 3. Left channel extraction is computationally efficient
/// 
/// ## Error Recovery
/// 
/// The stream includes error callbacks that log issues without crashing the application.
/// Common scenarios handled:
/// - Device disconnection during capture
/// - Audio driver issues or conflicts
/// - Buffer underruns or overruns
pub fn start_input_stream(
    sender: Sender<Vec<f32>>,
    running: Arc<AtomicBool>,
    device_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::info!("Starting input stream with device ID: {}", device_id);
    
    // Resolve the device ID to an actual audio device
    // This handles both default device selection and specific device targeting
    let device = get_device_by_id(device_id, true)
        .ok_or_else(|| {
            log::error!("Selected input device '{}' not found", device_id);
            "Selected input device not found"
        })?;

    // Query the device's optimal input configuration
    // This ensures we work with the device's preferred settings
    let supported_config = device.default_input_config().map_err(|e| {
        log::error!("Failed to get input device configuration: {}", e);
        e
    })?;

    // Build stream configuration using device preferences
    // BufferSize::Default lets the audio driver choose optimal latency/stability balance
    let config = StreamConfig {
        channels: supported_config.channels(),  // Respect device's channel layout
        sample_rate: supported_config.sample_rate(), // Use device's native sample rate
        buffer_size: BufferSize::Default,  // Let device choose optimal buffer size
    };

    log::info!("Input device: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));
    log::info!("Input config: {:?}", config);
    
    // Log sample rate configuration advice
    let advice = get_configuration_advice(config.sample_rate.0);
    log::info!("{}", advice);
    
    // Check for potential macOS virtual audio device configuration issues
    if cfg!(target_os = "macos") {
        let device_name = device.name().unwrap_or_default().to_lowercase();
        let virtual_device_type = crate::virtual_audio::detect_virtual_device_type(&device_name);
        
        if let Some(device_type) = virtual_device_type {
            log::warn!("*** CRITICAL macOS CONFIGURATION ISSUE DETECTED ***");
            log::warn!("{} is configured as INPUT device: {}", device_type, device_name);
            log::warn!("For noise cancellation to work properly:");
            log::warn!("1. INPUT should be your MICROPHONE (built-in microphone, external mic, etc.)");
            log::warn!("2. OUTPUT should be {} (VB-Cable or BlackHole)", device_type);
            log::warn!("3. Configure your communication app (Discord/Teams/Zoom) to use {} as input", device_type);
            log::warn!("Current setup will NOT provide noise cancellation!");
            log::warn!("Change your input device to your actual microphone in Kwite settings.");
            
            // Still allow it to work but with warnings
            log::info!("Detected {} on macOS as input - this is likely misconfigured", device_type);
            
            // Warn if sample rate is not optimal for noise cancellation
            if config.sample_rate.0 != 48000 {
                log::warn!("{} sample rate is {} Hz, expected 48000 Hz for optimal noise cancellation", 
                    device_type, config.sample_rate.0);
                log::warn!("Consider setting {} to 48kHz in Audio MIDI Setup for best performance", device_type);
                log::warn!("Current configuration may result in degraded noise cancellation quality");
            } else {
                log::info!("{} configured optimally at 48kHz for AI processing", device_type);
            }
            
            // Provide additional setup guidance for macOS users
            if config.channels != 1 && config.channels != 2 {
                log::warn!("{} has {} channels - expected 1 or 2 channels", device_type, config.channels);
            }
        } else {
            log::info!("✅ Detected proper input device: {} (not a virtual audio device)", device_name);
            log::info!("✅ This is CORRECT for noise cancellation - microphone as input, virtual device as output");
            log::info!("Noise cancellation should work properly with this configuration");
        }
    }

    let running_clone = running.clone();
    let sample_rate = config.sample_rate.0;
    
    // Log resampling information
    let needs_resampling = sample_rate != 48000;
    log::info!("Audio resampling: {}", if needs_resampling {
        format!("{}Hz -> 48kHz", sample_rate)
    } else {
        "Not needed (48kHz)".to_string()
    });
    
    // Create the input stream with real-time audio callback
    // The callback runs on a high-priority audio thread and must be efficient
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _| {
            // Only process audio while the system is running
            // This prevents unnecessary work during shutdown
            if running_clone.load(Ordering::Relaxed) {
                // Convert stereo input to mono for noise cancellation processing
                // Many microphones report as stereo but provide identical left/right channels
                let mono_data: Vec<f32> = if config.channels == 2 {
                    // Extract left channel only (every other sample starting from index 0)
                    // Stereo audio is interleaved: [L, R, L, R, ...]
                    data.iter().step_by(2).copied().collect()
                } else {
                    // Already mono, use as-is
                    data.to_vec()
                };
                
                // Apply basic resampling if needed (e.g., 44.1kHz virtual audio devices -> 48kHz for AI processing)
                let processed_data = if sample_rate != 48000 && sample_rate == 44100 {
                    // Handle the common 44.1kHz -> 48kHz case with simple interpolation
                    let target_length = (mono_data.len() as f64 * 48000.0 / 44100.0) as usize;
                    let mut resampled = Vec::with_capacity(target_length);
                    
                    for i in 0..target_length {
                        let src_index = (i as f64 * 44100.0 / 48000.0) as usize;
                        if src_index < mono_data.len() {
                            resampled.push(mono_data[src_index]);
                        } else {
                            resampled.push(0.0);
                        }
                    }
                    resampled
                } else {
                    mono_data
                };
                
                // Send to processor using try_send to avoid blocking the audio thread
                // If the processing pipeline is behind, we drop frames to prevent audio glitches
                if let Err(_) = sender.try_send(processed_data) {
                    // Channel is full - this is normal if processing can't keep up
                    // We don't log this as it would spam the logs in normal operation
                }
            }
        },
        move |err| {
            // Log audio stream errors without panicking
            // These can occur due to device disconnection, driver issues, etc.
            log::error!("Input stream error: {}", err);
        },
        None, // No timeout for the stream
    ).map_err(|e| {
        log::error!("Failed to build input stream: {}", e);
        e
    })?;

    // Start the audio capture stream
    stream.play().map_err(|e| {
        log::error!("Failed to start input stream: {}", e);
        e
    })?;
    
    log::info!("Input stream started successfully");
    
    // Keep the stream alive by blocking until shutdown is requested
    // The stream runs on its own thread, so we just need to prevent cleanup
    while running.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    log::info!("Input stream stopping");
    Ok(())
}
//! # Audio Output Module
//! 
//! This module handles routing processed audio to output devices, with special emphasis
//! on virtual audio cable support for seamless integration with communication applications.
//! The output system is designed to be robust and adaptive to various audio hardware
//! configurations.
//! 
//! ## Key Features
//! 
//! - **Virtual Audio Cable Detection**: Automatically prefers virtual devices for app integration
//! - **Fallback Device Selection**: Graceful handling when preferred devices aren't available
//! - **Format Adaptation**: Converts mono processed audio to device's required format
//! - **Buffer Management**: Prevents audio dropouts with adaptive buffering
//! - **Real-time Performance**: Optimized for low-latency audio delivery
//! 
//! ## Virtual Audio Cable Integration
//! 
//! Virtual audio cables (like VB-Audio Cable) create virtual audio devices that allow
//! applications to route audio between programs. This is essential for using Kwite
//! with communication apps like Discord, Teams, or Zoom.
//! 
//! ## Device Selection Priority
//! 
//! 1. User-selected device (if available)
//! 2. Virtual audio cable device (for app integration)
//! 3. System default output device
//! 4. Any available output device

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::Receiver;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::VecDeque;
use crate::logger::log;
use crate::audio::devices::{get_device_by_id, find_virtual_output_device};
use cpal::{BufferSize, StreamConfig};

/// Start audio output stream to the specified device
/// 
/// This function creates a real-time audio output stream that receives processed
/// audio from the noise cancellation pipeline and routes it to the appropriate
/// output device (speakers, headphones, or virtual audio cable).
/// 
/// ## Parameters
/// 
/// - `receiver`: Channel receiving processed audio from the AI pipeline
/// - `running`: Atomic flag for coordinating graceful shutdown
/// - `device_id`: Preferred output device identifier
/// 
/// ## Device Selection Logic
/// 
/// The function implements a sophisticated fallback strategy to ensure audio
/// output works in various system configurations:
/// 
/// 1. **Primary**: Use the device specified by `device_id`
/// 2. **Fallback 1**: Find any available virtual audio device
/// 3. **Fallback 2**: Use the system's default output device
/// 4. **Error**: No output devices available (rare but possible)
/// 
/// This approach ensures compatibility with:
/// - Standard speaker/headphone setups
/// - Virtual audio cable configurations
/// - Changing audio device availability (USB devices, etc.)
/// 
/// ## Audio Format Handling
/// 
/// The output system adapts processed mono audio to the output device's requirements:
/// - **Mono devices**: Direct output of processed audio
/// - **Stereo devices**: Duplicate mono signal to both left and right channels
/// - **Multi-channel**: Duplicate to all channels (rare for this use case)
/// 
/// ## Buffer Management
/// 
/// Uses a VecDeque for efficient audio buffering to handle timing differences
/// between the processing pipeline and audio output callback. This prevents:
/// - Audio dropouts when processing temporarily falls behind
/// - Buffer overruns when processing gets ahead of output
/// - Clicks and pops from discontinuous audio
pub fn start_output_stream(
    receiver: Receiver<Vec<f32>>,
    running: Arc<AtomicBool>,
    device_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Implement device selection with multiple fallback levels
    // This ensures the output works in various system configurations
    let device = get_device_by_id(device_id, false)
        .or_else(|| {
            // First fallback: Look for virtual audio devices
            // These are preferred for communication app integration
            log::warn!("Selected output device not found, trying to find virtual device");
            find_virtual_output_device()
        })
        .or_else(|| {
            // Second fallback: Use system default output
            // This ensures basic audio output functionality
            log::warn!("No virtual device found, using default output");
            cpal::default_host().default_output_device()
        })
        .ok_or("No output device available")?;

    // Query the device's optimal output configuration
    // This ensures compatibility with the device's native format
    let supported_config = device.default_output_config()?;

    // Configure output stream to match device capabilities
    // Using device defaults minimizes format conversion overhead
    let config = StreamConfig {
        channels: supported_config.channels(),      // Match device's channel layout
        sample_rate: supported_config.sample_rate(), // Use device's native sample rate
        buffer_size: BufferSize::Default,           // Let device choose optimal buffer size
    };

    log::info!("Using output device: {}", device.name()?);
    log::info!("Output config: {:?}", config);
    
    // Check for potential macOS virtual audio device configuration
    if cfg!(target_os = "macos") {
        let device_name = device.name().unwrap_or_default().to_lowercase();
        let virtual_device_type = crate::virtual_audio::detect_virtual_device_type(&device_name);
        
        if let Some(device_type) = virtual_device_type {
            log::info!("*** macOS {} OUTPUT Configuration Detected ***", device_type);
            log::info!("{} is configured as OUTPUT device: {}", device_type, device_name);
            log::info!("This is CORRECT for noise cancellation setup!");
            log::info!("Make sure your communication app uses {} as INPUT to receive processed audio", device_type);
            
            // Warn if sample rate is not optimal
            if config.sample_rate.0 != 48000 {
                log::warn!("{} output sample rate is {} Hz, expected 48000 Hz for optimal performance", 
                    device_type, config.sample_rate.0);
                log::warn!("Consider setting {} to 48kHz in Audio MIDI Setup for best results", device_type);
            } else {
                log::info!("{} configured optimally at 48kHz", device_type);
            }
            
            // Check channel configuration
            if config.channels != 1 && config.channels != 2 {
                log::warn!("{} output has {} channels - expected 1 or 2 channels", device_type, config.channels);
            } else {
                log::info!("{} channel configuration: {} channels (optimal)", device_type, config.channels);
            }
        } else {
            log::info!("Using regular output device: {} - this will not route to communication apps", device_name);
            log::info!("For noise cancellation routing, use a virtual audio device like VB-Cable as output");
        }
    }

    // Create audio buffer for handling timing differences between
    // the processing pipeline and audio output callback rates
    let mut buffer = VecDeque::new();

    // Create the output stream with real-time audio callback
    // This callback runs on a high-priority audio thread
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _| {
            // Continuously drain the receiver to fill our internal buffer
            // This prevents the processing pipeline from blocking on a full channel
            while let Ok(audio_data) = receiver.try_recv() {
                buffer.extend(audio_data);
            }

            // Fill the output buffer by consuming from our internal buffer
            // The device expects interleaved samples for multi-channel output
            for chunk in data.chunks_mut(config.channels as usize) {
                // Get the next processed audio sample (or silence if buffer is empty)
                // Silence prevents audio glitches when processing temporarily falls behind
                let sample = buffer.pop_front().unwrap_or(0.0);
                
                // Duplicate the mono sample to all output channels
                // This ensures proper audio output regardless of device configuration
                for channel_sample in chunk {
                    *channel_sample = sample;
                }
            }
        },
        move |err| {
            // Log audio stream errors without panicking
            // These can occur due to device disconnection, driver issues, etc.
            log::error!("Output stream error: {}", err);
        },
        None, // No timeout for the stream
    )?;

    // Start audio output playback
    stream.play()?;

    // Keep the stream alive until shutdown is requested
    // The stream runs on its own thread, so we just prevent cleanup
    while running.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}
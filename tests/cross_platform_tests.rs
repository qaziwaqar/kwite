use kwite::config::KwiteConfig;
use serial_test::serial;
use std::path::PathBuf;

#[test]
#[serial]
fn test_platform_specific_config_paths() {
    // Test that config path logic works for each platform
    // Since we can't easily test the private config_path function,
    // we test the expected behavior through serialization
    
    let config = KwiteConfig::default();
    
    // Test that serialization works on current platform
    let toml_content = toml::to_string_pretty(&config)
        .expect("Config serialization should work on all platforms");
    
    // Test that deserialization works
    let _loaded_config: KwiteConfig = toml::from_str(&toml_content)
        .expect("Config deserialization should work on all platforms");
    
    // Test platform-specific path expectations
    if cfg!(target_os = "windows") {
        // On Windows, config should go to %APPDATA%\Kwite\config.toml
        println!("Testing Windows config path expectations");
        // We can't directly test the path without accessing private functions
        // but we can verify the logic doesn't panic
    } else if cfg!(target_os = "macos") {
        // On macOS, config should go to ~/Library/Application Support/Kwite/config.toml
        println!("Testing macOS config path expectations");
    } else {
        // On Linux/Unix, config should go to ~/.config/kwite/config.toml
        println!("Testing Linux/Unix config path expectations");
    }
}

#[test]
#[serial]
fn test_cross_platform_device_naming() {
    use kwite::audio::devices::{list_input_devices, list_output_devices};
    
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    
    // Test that device names are valid UTF-8 on all platforms
    for device in &input_devices {
        assert!(!device.name.is_empty(), "Device name should not be empty");
        assert!(!device.id.is_empty(), "Device ID should not be empty");
        
        // Test that device names don't contain null bytes (common cross-platform issue)
        assert!(!device.name.contains('\0'), "Device name should not contain null bytes");
        assert!(!device.id.contains('\0'), "Device ID should not contain null bytes");
    }
    
    for device in &output_devices {
        assert!(!device.name.is_empty(), "Device name should not be empty");
        assert!(!device.id.is_empty(), "Device ID should not be empty");
        
        assert!(!device.name.contains('\0'), "Device name should not contain null bytes");
        assert!(!device.id.contains('\0'), "Device ID should not contain null bytes");
    }
}

#[test]
#[serial]
fn test_platform_specific_virtual_device_detection() {
    use kwite::audio::devices::list_output_devices;
    
    let output_devices = list_output_devices();
    
    // Test virtual device detection patterns across platforms
    for device in &output_devices {
        let name_lower = device.name.to_lowercase();
        
        // Windows-specific virtual devices
        if cfg!(target_os = "windows") {
            if name_lower.contains("vb-audio") || 
               name_lower.contains("voicemeeter") ||
               name_lower.contains("virtual audio cable") {
                println!("Found Windows virtual device: {}", device.name);
            }
        }
        
        // macOS-specific virtual devices  
        if cfg!(target_os = "macos") {
            if name_lower.contains("blackhole") ||
               name_lower.contains("soundflower") ||
               name_lower.contains("loopback") {
                println!("Found macOS virtual device: {}", device.name);
            }
        }
        
        // Linux-specific virtual devices
        if cfg!(target_os = "linux") {
            if name_lower.contains("pulse") ||
               name_lower.contains("virtual") ||
               name_lower.contains("null") {
                println!("Found Linux virtual device: {}", device.name);
            }
        }
    }
}

#[test]
#[serial]
fn test_unicode_device_names_cross_platform() {
    use kwite::audio::devices::{list_input_devices, list_output_devices};
    use kwite::config::KwiteConfig;
    
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    
    // Create configs with potential unicode device names
    let test_unicode_names = vec![
        "Микрофон".to_string(),           // Russian
        "マイクロフォン".to_string(),         // Japanese
        "麦克风".to_string(),               // Chinese
        "Mikrofón".to_string(),           // Slovak/Czech
        "Micrófono".to_string(),          // Spanish
        "Mikrofon🎤".to_string(),         // With emoji
    ];
    
    for unicode_name in test_unicode_names {
        let config = KwiteConfig {
            input_device_id: unicode_name.clone(),
            output_device_id: format!("output_{}", unicode_name),
            sensitivity: 0.2,
            auto_start: false,
            minimize_to_tray: false,
            development_mode: false,
            remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
            analytics: kwite::config::AnalyticsConfig::default(),
            auto_update: kwite::config::AutoUpdateConfig::default(),
        };
        
        // Test that unicode survives serialization/deserialization
        let toml_content = toml::to_string_pretty(&config)
            .expect("Should serialize unicode device names");
        
        let loaded_config: KwiteConfig = toml::from_str(&toml_content)
            .expect("Should deserialize unicode device names");
        
        assert_eq!(config.input_device_id, loaded_config.input_device_id);
        assert_eq!(config.output_device_id, loaded_config.output_device_id);
    }
}

#[test]
#[serial]
fn test_platform_audio_backend_compatibility() {
    use kwite::audio::devices::{list_input_devices, list_output_devices};
    
    // Test that audio enumeration works regardless of platform backend
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    
    // Should always have at least fallback devices
    assert!(!input_devices.is_empty(), "Should have input devices on all platforms");
    assert!(!output_devices.is_empty(), "Should have output devices on all platforms");
    
    // Test platform-specific behaviors
    if cfg!(target_os = "windows") {
        println!("Testing Windows WASAPI backend compatibility");
        // Windows should typically have multiple devices
    } else if cfg!(target_os = "macos") {
        println!("Testing macOS Core Audio backend compatibility");
        // macOS should have at least built-in devices
    } else if cfg!(target_os = "linux") {
        println!("Testing Linux ALSA/PulseAudio backend compatibility");
        // Linux might have virtual ALSA devices even without hardware
    }
    
    // Verify device enumeration is stable
    let input_devices_2 = list_input_devices();
    let output_devices_2 = list_output_devices();
    
    assert_eq!(input_devices.len(), input_devices_2.len(), 
               "Device enumeration should be stable");
    assert_eq!(output_devices.len(), output_devices_2.len(), 
               "Device enumeration should be stable");
}

#[test]
#[serial]
fn test_cross_platform_path_handling() {
    use std::path::Path;
    
    // Test that config handles different path separators
    let config = KwiteConfig {
        input_device_id: "test/input".to_string(),
        output_device_id: "test\\output".to_string(),  // Mixed separators
        sensitivity: 0.3,
        auto_start: false,
        minimize_to_tray: false,
        development_mode: false,
        remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
        analytics: kwite::config::AnalyticsConfig::default(),
        auto_update: kwite::config::AutoUpdateConfig::default(),
    };
    
    // Serialization should preserve the strings as-is
    let toml_content = toml::to_string_pretty(&config)
        .expect("Should handle path-like strings in device IDs");
    
    let loaded_config: KwiteConfig = toml::from_str(&toml_content)
        .expect("Should deserialize path-like device IDs");
    
    assert_eq!(config.input_device_id, loaded_config.input_device_id);
    assert_eq!(config.output_device_id, loaded_config.output_device_id);
}

#[test]
#[serial]
fn test_platform_specific_line_endings() {
    // Test that config handles different line endings properly
    let config = KwiteConfig::default();
    
    let toml_content = toml::to_string_pretty(&config)
        .expect("Should serialize config");
    
    // Test with different line endings
    let unix_content = toml_content.replace("\r\n", "\n");
    let windows_content = toml_content.replace("\n", "\r\n");
    
    // All should deserialize correctly
    let _unix_config: KwiteConfig = toml::from_str(&unix_content)
        .expect("Should handle Unix line endings");
    
    let _windows_config: KwiteConfig = toml::from_str(&windows_content)
        .expect("Should handle Windows line endings");
        
    // Note: Classic Mac line endings (\r only) are not supported by TOML spec
    // This is expected behavior as TOML requires either \n or \r\n
    println!("Line ending tests completed. Classic Mac \\r-only not supported by TOML spec.");
}

#[test]
#[serial]
fn test_platform_floating_point_precision() {
    // Test that sensitivity values maintain precision across platforms
    let test_values = [
        0.1, 0.01, 0.001, 0.33333333, 0.666666666, 0.123456789
    ];
    
    for &value in &test_values {
        let config = KwiteConfig {
            input_device_id: "test".to_string(),
            output_device_id: "test".to_string(),
            sensitivity: value,
            auto_start: false,
            minimize_to_tray: false,
            development_mode: false,
            remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
            analytics: kwite::config::AnalyticsConfig::default(),
            auto_update: kwite::config::AutoUpdateConfig::default(),
        };
        
        let toml_content = toml::to_string_pretty(&config)
            .expect("Should serialize floating point values");
        
        let loaded_config: KwiteConfig = toml::from_str(&toml_content)
            .expect("Should deserialize floating point values");
        
        // Allow for small floating point precision differences
        let diff = (config.sensitivity - loaded_config.sensitivity).abs();
        assert!(diff < 1e-6, "Floating point precision should be maintained across platforms: {} vs {}", 
                config.sensitivity, loaded_config.sensitivity);
    }
}

#[cfg(target_family = "unix")]
#[test]
#[serial]
fn test_unix_specific_features() {
    // Test Unix-specific functionality
    println!("Testing Unix-specific audio features");
    
    use kwite::audio::devices::list_output_devices;
    let devices = list_output_devices();
    
    // Unix systems might have ALSA, PulseAudio, or JACK devices
    for device in &devices {
        let name_lower = device.name.to_lowercase();
        if name_lower.contains("alsa") || 
           name_lower.contains("pulse") || 
           name_lower.contains("jack") {
            println!("Found Unix audio device: {}", device.name);
        }
    }
}

#[cfg(target_os = "windows")]
#[test]
#[serial]
fn test_windows_specific_features() {
    // Test Windows-specific functionality
    println!("Testing Windows-specific audio features");
    
    use kwite::audio::devices::list_output_devices;
    let devices = list_output_devices();
    
    // Windows should have WASAPI devices
    for device in &devices {
        let name_lower = device.name.to_lowercase();
        if name_lower.contains("speakers") || 
           name_lower.contains("headphones") ||
           name_lower.contains("realtek") ||
           name_lower.contains("microsoft") {
            println!("Found Windows audio device: {}", device.name);
        }
    }
}

#[cfg(target_os = "macos")]
#[test]
#[serial] 
fn test_macos_specific_features() {
    // Test macOS-specific functionality
    println!("Testing macOS-specific audio features");
    
    use kwite::audio::devices::list_output_devices;
    let devices = list_output_devices();
    
    // macOS should have Core Audio devices
    for device in &devices {
        let name_lower = device.name.to_lowercase();
        if name_lower.contains("built-in") ||
           name_lower.contains("airpods") ||
           name_lower.contains("usb") {
            println!("Found macOS audio device: {}", device.name);
        }
    }
}
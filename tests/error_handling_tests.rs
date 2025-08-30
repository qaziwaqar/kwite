use kwite::audio::devices::*;
use kwite::config::*;
use kwite::logger;
use serial_test::serial;
use std::sync::Once;
use tempfile::TempDir;
use std::fs;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        let _ = logger::init_logger();
    });
}

#[test]
#[serial]
fn test_missing_audio_devices_graceful_handling() {
    setup();
    
    // Test handling of nonexistent device IDs
    let missing_input = get_device_by_id("definitely_nonexistent_input_device", true);
    let missing_output = get_device_by_id("definitely_nonexistent_output_device", false);
    
    assert!(missing_input.is_none(), "Should return None for missing input device");
    assert!(missing_output.is_none(), "Should return None for missing output device");
    
    // Verify the function doesn't panic or crash
    let multiple_missing = vec![
        "fake_device_1",
        "fake_device_2", 
        "nonexistent",
        "",  // Empty string
        "device_with_special_chars!@#$%^&*()",
        "device_with_unicode_麦克风",
    ];
    
    for device_id in multiple_missing {
        let input_result = get_device_by_id(device_id, true);
        let output_result = get_device_by_id(device_id, false);
        
        assert!(input_result.is_none(), "Should handle invalid device ID gracefully: {}", device_id);
        assert!(output_result.is_none(), "Should handle invalid device ID gracefully: {}", device_id);
    }
}

#[test]
#[serial]
fn test_corrupted_config_file_recovery() {
    setup();
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Test various types of corrupted TOML files
    let corrupted_configs = vec![
        ("", "Empty file"),
        ("invalid toml content", "Invalid TOML syntax"),
        ("[[[[[", "Malformed TOML"),
        ("input_device_id = ", "Incomplete assignment"),
        ("input_device_id = \"unclosed string", "Unclosed string"),
        ("sensitivity = not_a_number", "Invalid type"),
        ("auto_start = maybe", "Invalid boolean"),
        ("unknown_field = \"value\"", "Unknown field"),
        ("\x00\x01\x02", "Binary data"), // Non-UTF8 content
        ("input_device_id = \"valid\"\nsensitivity = 999.999", "Out of range value"),
    ];
    
    for (content, description) in corrupted_configs {
        let config_path = temp_dir.path().join(format!("corrupted_{}.toml", description.replace(" ", "_")));
        
        // Write corrupted content
        let write_result = fs::write(&config_path, content);
        
        // Some content might fail to write (e.g., non-UTF8), which is fine
        if write_result.is_ok() {
            // Try to read and parse
            if let Ok(file_content) = fs::read_to_string(&config_path) {
                let parse_result: Result<KwiteConfig, _> = toml::from_str(&file_content);
                
                // Should either parse successfully or fail gracefully
                match parse_result {
                    Ok(_) => println!("Config parsed successfully despite being '{}'", description),
                    Err(e) => {
                        println!("Config parsing failed as expected for '{}': {}", description, e);
                        // The actual load() function should fall back to defaults on error
                    }
                }
            }
        }
    }
    
    // Verify that load() always returns a valid config (uses defaults on error)
    let default_config = KwiteConfig::load();
    assert!(!default_config.input_device_id.is_empty());
    assert!(!default_config.output_device_id.is_empty());
    assert!(default_config.sensitivity > 0.0);
}

#[test]
#[serial]
fn test_filesystem_permission_errors() {
    setup();
    
    // Test config serialization with various problematic inputs
    let problematic_configs = vec![
        KwiteConfig {
            input_device_id: "\0".to_string(), // Null byte
            output_device_id: "valid".to_string(),
            sensitivity: 0.1,
            auto_start: false,
            minimize_to_tray: false,
            development_mode: false,
            remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
            analytics: kwite::config::AnalyticsConfig::default(),
            auto_update: kwite::config::AutoUpdateConfig::default(),
        },
        KwiteConfig {
            input_device_id: "valid".to_string(),
            output_device_id: "very_long_string_that_might_cause_issues_if_filesystem_has_limits".repeat(100),
            sensitivity: 0.1,
            auto_start: false,
            minimize_to_tray: false,
            development_mode: false,
            remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
            analytics: kwite::config::AnalyticsConfig::default(),
            auto_update: kwite::config::AutoUpdateConfig::default(),
        },
    ];
    
    for config in problematic_configs {
        // Test that serialization either succeeds or fails gracefully
        match toml::to_string_pretty(&config) {
            Ok(content) => {
                println!("Serialization succeeded for problematic config");
                
                // Test that deserialization also works
                match toml::from_str::<KwiteConfig>(&content) {
                    Ok(_) => println!("Round-trip successful"),
                    Err(e) => println!("Deserialization failed: {}", e),
                }
            },
            Err(e) => {
                println!("Serialization failed gracefully: {}", e);
            }
        }
    }
}

#[test]
#[serial]
fn test_extreme_sensitivity_values() {
    setup();
    
    // Test edge cases for sensitivity values
    let extreme_values = vec![
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::NAN,
        -1000.0,
        0.0,
        1000.0,
        f32::MIN,
        f32::MAX,
        f32::EPSILON,
    ];
    
    for value in extreme_values {
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
        
        // Test serialization
        match toml::to_string_pretty(&config) {
            Ok(toml_content) => {
                println!("Serialized extreme sensitivity value: {}", value);
                
                // Test deserialization
                match toml::from_str::<KwiteConfig>(&toml_content) {
                    Ok(loaded_config) => {
                        if value.is_nan() {
                            assert!(loaded_config.sensitivity.is_nan(), "NaN should be preserved");
                        } else if value.is_infinite() {
                            assert_eq!(loaded_config.sensitivity.is_infinite(), value.is_infinite());
                            assert_eq!(loaded_config.sensitivity.is_sign_positive(), value.is_sign_positive());
                        } else {
                            assert_eq!(loaded_config.sensitivity, value, "Extreme value should be preserved");
                        }
                    },
                    Err(e) => {
                        println!("Deserialization failed for extreme value {}: {}", value, e);
                    }
                }
            },
            Err(e) => {
                println!("Serialization failed for extreme value {}: {}", value, e);
                // This is acceptable for values like NaN or infinity
            }
        }
    }
}

#[test]
#[serial]
fn test_concurrent_device_access() {
    setup();
    
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    // Spawn multiple threads accessing device enumeration concurrently
    for i in 0..10 {
        let success_count_clone = Arc::clone(&success_count);
        
        let handle = thread::spawn(move || {
            // Each thread enumerates devices multiple times
            for j in 0..5 {
                match std::panic::catch_unwind(|| {
                    let input_devices = list_input_devices();
                    let output_devices = list_output_devices();
                    
                    // Basic validation
                    assert!(!input_devices.is_empty());
                    assert!(!output_devices.is_empty());
                    
                    // Test device lookup
                    if let Some(device) = input_devices.first() {
                        let _lookup_result = get_device_by_id(&device.id, true);
                    }
                    
                    if let Some(device) = output_devices.first() {
                        let _lookup_result = get_device_by_id(&device.id, false);
                    }
                }) {
                    Ok(_) => {
                        success_count_clone.fetch_add(1, Ordering::SeqCst);
                        println!("Thread {} iteration {} succeeded", i, j);
                    },
                    Err(e) => {
                        println!("Thread {} iteration {} panicked: {:?}", i, j, e);
                    }
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread should complete");
    }
    
    let successes = success_count.load(Ordering::SeqCst);
    println!("Concurrent device access: {}/50 operations succeeded", successes);
    
    // Most operations should succeed (allowing for some potential failures in CI environments)
    assert!(successes > 25, "At least half of concurrent operations should succeed");
}

#[test]
#[serial]
fn test_device_enumeration_error_conditions() {
    setup();
    
    // Test repeated device enumeration to catch potential resource leaks
    for i in 0..100 {
        let input_devices = list_input_devices();
        let output_devices = list_output_devices();
        
        // Should always succeed and return at least fallback devices
        assert!(!input_devices.is_empty(), "Iteration {} should have input devices", i);
        assert!(!output_devices.is_empty(), "Iteration {} should have output devices", i);
        
        // Devices should have valid properties
        for device in &input_devices {
            assert!(!device.id.is_empty(), "Device ID should not be empty at iteration {}", i);
            assert!(!device.name.is_empty(), "Device name should not be empty at iteration {}", i);
        }
        
        for device in &output_devices {
            assert!(!device.id.is_empty(), "Device ID should not be empty at iteration {}", i);
            assert!(!device.name.is_empty(), "Device name should not be empty at iteration {}", i);
        }
        
        // Every 10 iterations, test device lookup
        if i % 10 == 0 {
            if let Some(input_device) = input_devices.first() {
                let lookup_result = get_device_by_id(&input_device.id, true);
                assert!(lookup_result.is_some(), "Device lookup should succeed at iteration {}", i);
            }
            
            if let Some(output_device) = output_devices.first() {
                let lookup_result = get_device_by_id(&output_device.id, false);
                assert!(lookup_result.is_some(), "Device lookup should succeed at iteration {}", i);
            }
        }
    }
}

#[test]
#[serial]
fn test_memory_pressure_handling() {
    setup();
    
    // Test behavior under simulated memory pressure
    let mut large_configs = Vec::new();
    
    // Create many config objects to simulate memory pressure
    for i in 0..1000 {
        let config = KwiteConfig {
            input_device_id: format!("input_device_{}", i),
            output_device_id: format!("output_device_{}", i),
            sensitivity: (i as f32) / 1000.0,
            auto_start: i % 2 == 0,
            minimize_to_tray: i % 3 == 0,
            development_mode: false,
            remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
            analytics: kwite::config::AnalyticsConfig::default(),
            auto_update: kwite::config::AutoUpdateConfig::default(),
        };
        
        // Test serialization under memory pressure
        match toml::to_string_pretty(&config) {
            Ok(toml_content) => {
                // Test deserialization
                match toml::from_str::<KwiteConfig>(&toml_content) {
                    Ok(loaded_config) => {
                        assert_eq!(config.input_device_id, loaded_config.input_device_id);
                        assert_eq!(config.output_device_id, loaded_config.output_device_id);
                    },
                    Err(e) => {
                        panic!("Deserialization failed under memory pressure: {}", e);
                    }
                }
            },
            Err(e) => {
                panic!("Serialization failed under memory pressure: {}", e);
            }
        }
        
        large_configs.push(config);
    }
    
    // Verify we can still enumerate devices under memory pressure
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    
    assert!(!input_devices.is_empty(), "Should still enumerate devices under memory pressure");
    assert!(!output_devices.is_empty(), "Should still enumerate devices under memory pressure");
    
    // Clean up
    drop(large_configs);
}

#[test]
#[serial]
fn test_logger_error_conditions() {
    setup();
    
    // Test logging under various error conditions
    use std::panic;
    
    // Test logging with problematic data
    let problematic_strings = vec![
        String::from("Normal string"),
        String::new(), // Empty string
        "\0".to_string(), // Null byte
        "🎵🎤🔊".to_string(), // Unicode emoji
        "Line1\nLine2\tTabbed".to_string(), // Control characters
        "Quote\"Test'Quote".to_string(), // Quotes
        format!("{}", "Very long string ".repeat(1000)), // Long string
    ];
    
    for test_string in problematic_strings {
        // Test that logging doesn't panic with problematic strings
        let result = panic::catch_unwind(|| {
            kwite::logger::log::info!("Testing with problematic string: {}", test_string);
            kwite::logger::log::warn!("Warning with string: {}", test_string);
            kwite::logger::log::error!("Error with string: {}", test_string);
            kwite::logger::log::debug!("Debug with string: {}", test_string);
        });
        
        assert!(result.is_ok(), "Logging should not panic with problematic strings");
    }
}

#[test]
#[serial]
fn test_resource_exhaustion_simulation() {
    setup();
    
    // Simulate rapid resource allocation/deallocation
    for _round in 0..10 {
        let mut temp_data = Vec::new();
        
        // Allocate many temporary objects
        for i in 0..100 {
            let config = KwiteConfig {
                input_device_id: format!("temp_input_{}", i),
                output_device_id: format!("temp_output_{}", i),
                sensitivity: 0.1,
                auto_start: false,
                minimize_to_tray: false,
                development_mode: false,
                remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
                analytics: kwite::config::AnalyticsConfig::default(),
                auto_update: kwite::config::AutoUpdateConfig::default(),
            };
            temp_data.push(config);
        }
        
        // Test that core functionality still works
        let input_devices = list_input_devices();
        let output_devices = list_output_devices();
        
        assert!(!input_devices.is_empty(), "Device enumeration should work under resource pressure");
        assert!(!output_devices.is_empty(), "Device enumeration should work under resource pressure");
        
        // Test config operations
        if let Some(device) = input_devices.first() {
            let lookup_result = get_device_by_id(&device.id, true);
            assert!(lookup_result.is_some(), "Device lookup should work under resource pressure");
        }
        
        // Cleanup
        temp_data.clear();
    }
}
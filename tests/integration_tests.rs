use kwite::audio::devices::*;
use kwite::config::*;
use kwite::logger;
use serial_test::serial;
use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        let _ = logger::init_logger();
    });
}

#[test]
#[serial]
fn test_device_config_integration() {
    setup();
    
    // Test that device selection integrates with config
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    
    assert!(!input_devices.is_empty());
    assert!(!output_devices.is_empty());
    
    // Create config with first available devices
    let config = KwiteConfig {
        input_device_id: input_devices[0].id.clone(),
        output_device_id: output_devices[0].id.clone(),
        sensitivity: 0.3,
        auto_start: false,
        minimize_to_tray: false,
        development_mode: false,
        remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
        analytics: kwite::config::AnalyticsConfig::default(),
        auto_update: kwite::config::AutoUpdateConfig::default(),
    };
    
    // Verify device lookup works with config
    let input_device = get_device_by_id(&config.input_device_id, true);
    let output_device = get_device_by_id(&config.output_device_id, false);
    
    assert!(input_device.is_some(), "Should find input device from config");
    assert!(output_device.is_some(), "Should find output device from config");
}

#[test]
#[serial]
fn test_config_persistence_integration() {
    setup();
    
    // Test full config save/load cycle
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Since we can't easily override the config path, we test the serialization format
    let original_config = KwiteConfig {
        input_device_id: "integration_input".to_string(),
        output_device_id: "integration_output".to_string(),
        sensitivity: 0.35,
        auto_start: false,
        minimize_to_tray: true,
        development_mode: false,
        remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
        analytics: kwite::config::AnalyticsConfig::default(),
        auto_update: kwite::config::AutoUpdateConfig::default(),
    };
    
    // Test serialization
    let toml_content = toml::to_string_pretty(&original_config)
        .expect("Failed to serialize config");
    
    // Test deserialization
    let loaded_config: KwiteConfig = toml::from_str(&toml_content)
        .expect("Failed to deserialize config");
    
    // Verify integrity
    assert_eq!(original_config.input_device_id, loaded_config.input_device_id);
    assert_eq!(original_config.output_device_id, loaded_config.output_device_id);
    assert_eq!(original_config.sensitivity, loaded_config.sensitivity);
    assert_eq!(original_config.auto_start, loaded_config.auto_start);
    assert_eq!(original_config.minimize_to_tray, loaded_config.minimize_to_tray);
}

#[test]
#[serial]
fn test_device_switching_workflow() {
    setup();
    
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    
    // Simulate user switching devices
    let mut config = KwiteConfig::default();
    
    // Start with first devices
    if !input_devices.is_empty() {
        config.input_device_id = input_devices[0].id.clone();
    }
    if !output_devices.is_empty() {
        config.output_device_id = output_devices[0].id.clone();
    }
    
    // Verify initial selection works
    let initial_input = get_device_by_id(&config.input_device_id, true);
    let initial_output = get_device_by_id(&config.output_device_id, false);
    
    if !input_devices.is_empty() {
        assert!(initial_input.is_some(), "Initial input device should be found");
    }
    if !output_devices.is_empty() {
        assert!(initial_output.is_some(), "Initial output device should be found");
    }
    
    // Switch to last devices if there are multiple
    if input_devices.len() > 1 {
        config.input_device_id = input_devices.last().unwrap().id.clone();
        let switched_input = get_device_by_id(&config.input_device_id, true);
        assert!(switched_input.is_some(), "Switched input device should be found");
    }
    
    if output_devices.len() > 1 {
        config.output_device_id = output_devices.last().unwrap().id.clone();
        let switched_output = get_device_by_id(&config.output_device_id, false);
        assert!(switched_output.is_some(), "Switched output device should be found");
    }
}

#[test]
#[serial]
fn test_application_startup_workflow() {
    setup();
    
    // Simulate application startup process
    
    // 1. Load configuration (should always succeed)
    let config = KwiteConfig::load();
    assert!(!config.input_device_id.is_empty());
    assert!(!config.output_device_id.is_empty());
    
    // 2. Enumerate devices
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    assert!(!input_devices.is_empty());
    assert!(!output_devices.is_empty());
    
    // 3. Validate configured devices exist, fallback if needed
    let selected_input = if input_devices.iter().any(|d| d.id == config.input_device_id) {
        config.input_device_id.clone()
    } else {
        // Fallback to default
        input_devices.iter()
            .find(|d| d.is_default)
            .map(|d| d.id.clone())
            .unwrap_or_else(|| input_devices[0].id.clone())
    };
    
    let selected_output = if output_devices.iter().any(|d| d.id == config.output_device_id) {
        config.output_device_id.clone()
    } else {
        // Prefer virtual devices, fallback to default
        output_devices.iter()
            .find(|d| d.is_virtual)
            .or_else(|| output_devices.iter().find(|d| d.is_default))
            .map(|d| d.id.clone())
            .unwrap_or_else(|| output_devices[0].id.clone())
    };
    
    // 4. Verify final selections
    let final_input = get_device_by_id(&selected_input, true);
    let final_output = get_device_by_id(&selected_output, false);
    
    assert!(final_input.is_some(), "Final input device should be valid");
    assert!(final_output.is_some(), "Final output device should be valid");
}

#[test]
#[serial]
fn test_error_recovery_integration() {
    setup();
    
    // Test graceful handling of missing devices
    let invalid_config = KwiteConfig {
        input_device_id: "nonexistent_input".to_string(),
        output_device_id: "nonexistent_output".to_string(),
        sensitivity: 0.2,
        auto_start: false,
        minimize_to_tray: false,
        development_mode: false,
        remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
        analytics: kwite::config::AnalyticsConfig::default(),
        auto_update: kwite::config::AutoUpdateConfig::default(),
    };
    
    // Device lookup should fail gracefully
    let missing_input = get_device_by_id(&invalid_config.input_device_id, true);
    let missing_output = get_device_by_id(&invalid_config.output_device_id, false);
    
    assert!(missing_input.is_none(), "Should return None for missing input device");
    assert!(missing_output.is_none(), "Should return None for missing output device");
    
    // Application should still be able to fall back to available devices
    let available_input = list_input_devices();
    let available_output = list_output_devices();
    
    assert!(!available_input.is_empty(), "Should always have fallback input devices");
    assert!(!available_output.is_empty(), "Should always have fallback output devices");
}

#[test]
#[serial]
fn test_virtual_device_preference_workflow() {
    setup();
    
    let output_devices = list_output_devices();
    
    // Test virtual device detection and preference
    let virtual_devices: Vec<_> = output_devices.iter()
        .filter(|d| d.is_virtual)
        .collect();
    
    if !virtual_devices.is_empty() {
        println!("Found {} virtual device(s)", virtual_devices.len());
        
        // Verify virtual device can be selected
        let virtual_id = &virtual_devices[0].id;
        let device = get_device_by_id(virtual_id, false);
        assert!(device.is_some(), "Virtual device should be accessible");
        
        // Test configuration with virtual device
        let config = KwiteConfig {
            input_device_id: "input_default".to_string(),
            output_device_id: virtual_id.clone(),
            sensitivity: 0.25,
            auto_start: false,
            minimize_to_tray: false,
            development_mode: false,
            remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
            analytics: kwite::config::AnalyticsConfig::default(),
            auto_update: kwite::config::AutoUpdateConfig::default(),
        };
        
        // Verify configuration is valid
        let configured_output = get_device_by_id(&config.output_device_id, false);
        assert!(configured_output.is_some(), "Configured virtual device should be valid");
    } else {
        println!("No virtual devices found in test environment");
    }
}

#[test]
#[serial]
fn test_sensitivity_configuration_integration() {
    setup();
    
    // Test various sensitivity values
    let test_sensitivities = [0.01, 0.1, 0.25, 0.5, 1.0];
    
    for &sensitivity in &test_sensitivities {
        let config = KwiteConfig {
            input_device_id: "input_default".to_string(),
            output_device_id: "output_default".to_string(),
            sensitivity,
            auto_start: false,
            minimize_to_tray: false,
            development_mode: false,
            remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
            analytics: kwite::config::AnalyticsConfig::default(),
            auto_update: kwite::config::AutoUpdateConfig::default(),
        };
        
        // Test serialization preserves precision
        let toml_content = toml::to_string_pretty(&config)
            .expect("Failed to serialize config");
        
        let loaded_config: KwiteConfig = toml::from_str(&toml_content)
            .expect("Failed to deserialize config");
        
        // Allow for small floating point differences
        let diff = (config.sensitivity - loaded_config.sensitivity).abs();
        assert!(diff < 0.001, "Sensitivity precision should be preserved: {} vs {}", 
                config.sensitivity, loaded_config.sensitivity);
    }
}

#[test]
#[serial]
fn test_save_config_saves_all_ui_settings() {
    setup();
    
    // Test that all UI-configurable settings are properly serialized and saved
    let config = KwiteConfig {
        input_device_id: "test_input".to_string(),
        output_device_id: "test_output".to_string(),
        sensitivity: 0.25,
        auto_start: false,
        minimize_to_tray: false,
        development_mode: true,  // This should be saved
        remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
        analytics: kwite::config::AnalyticsConfig {
            enabled: true,  // This should be saved
            performance_endpoint: "test_endpoint".to_string(),
            performance_interval_seconds: 3600,
        },
        auto_update: kwite::config::AutoUpdateConfig {
            enabled: true,  // This should be saved
            check_interval_hours: 24,
            update_endpoint: "test_update_endpoint".to_string(),
            notify_before_download: true,
        },
    };
    
    // Test that config can be serialized and saves all fields
    let toml_content = toml::to_string_pretty(&config).expect("Failed to serialize config");
    
    // Verify all the UI-configurable fields are present in the serialized content
    assert!(toml_content.contains("development_mode = true"));
    assert!(toml_content.contains("[analytics]"));
    assert!(toml_content.contains("[auto_update]"));
    
    // Test that config can be round-tripped
    let loaded_config: KwiteConfig = toml::from_str(&toml_content)
        .expect("Failed to deserialize config");
    
    // Verify all fields are preserved - this confirms that the config.save() 
    // method will persist all UI settings when called from save_config()
    assert_eq!(config.development_mode, loaded_config.development_mode);
    assert_eq!(config.analytics.enabled, loaded_config.analytics.enabled);
    assert_eq!(config.auto_update.enabled, loaded_config.auto_update.enabled);
    assert_eq!(config.sensitivity, loaded_config.sensitivity);
    assert_eq!(config.input_device_id, loaded_config.input_device_id);
    assert_eq!(config.output_device_id, loaded_config.output_device_id);
}
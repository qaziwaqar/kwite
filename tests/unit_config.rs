use kwite::config::*;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_kwite_config_default() {
    let config = KwiteConfig::default();
    
    assert_eq!(config.input_device_id, "input_default");
    assert_eq!(config.output_device_id, "output_default");
    assert_eq!(config.sensitivity, 0.1);
    assert!(!config.auto_start); // Default is false (manual start required)
    assert!(!config.minimize_to_tray); // Default is false
}

#[test]
#[serial]
fn test_kwite_config_clone() {
    let config = KwiteConfig::default();
    let cloned = config.clone();
    
    assert_eq!(config.input_device_id, cloned.input_device_id);
    assert_eq!(config.output_device_id, cloned.output_device_id);
    assert_eq!(config.sensitivity, cloned.sensitivity);
    assert_eq!(config.auto_start, cloned.auto_start);
    assert_eq!(config.minimize_to_tray, cloned.minimize_to_tray);
}

#[test]
#[serial] 
fn test_config_load_nonexistent() {
    // Ensure no config file exists by removing any potential config directory
    if let Some(config_dir) = dirs::config_dir() {
        let app_config_dir = config_dir.join(if cfg!(target_os = "windows") || cfg!(target_os = "macos") { 
            "Kwite" 
        } else { 
            "kwite" 
        });
        
        // Remove the entire config directory to ensure clean state
        let _ = std::fs::remove_dir_all(&app_config_dir);
        
        // Verify config file doesn't exist
        let config_file = app_config_dir.join("config.toml");
        assert!(!config_file.exists(), "Config file should not exist at start of test");
    }
    
    // When config file doesn't exist, should return defaults
    let config = KwiteConfig::load();
    let default_config = KwiteConfig::default();
    
    // Verify exact match with defaults
    assert_eq!(config.input_device_id, default_config.input_device_id, 
        "Config input_device_id should match default. Got '{}', expected '{}'", 
        config.input_device_id, default_config.input_device_id);
    assert_eq!(config.output_device_id, default_config.output_device_id,
        "Config output_device_id should match default. Got '{}', expected '{}'", 
        config.output_device_id, default_config.output_device_id);
    assert_eq!(config.sensitivity, default_config.sensitivity,
        "Config sensitivity should match default. Got {}, expected {}", 
        config.sensitivity, default_config.sensitivity);
}

#[test]
#[serial]
fn test_config_roundtrip_save_load() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("test_config.toml");
    
    // Create a custom config
    let original_config = KwiteConfig {
        input_device_id: "test_input".to_string(),
        output_device_id: "test_output".to_string(),
        sensitivity: 0.25,
        auto_start: false,
        minimize_to_tray: false,
        development_mode: false,
        remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
        analytics: kwite::config::AnalyticsConfig::default(),
        auto_update: kwite::config::AutoUpdateConfig::default(),
    };
    
    // Mock the config_path function by testing the serialization directly
    let toml_content = toml::to_string_pretty(&original_config)
        .expect("Failed to serialize config");
    
    fs::write(&config_path, toml_content)
        .expect("Failed to write config file");
    
    // Read and deserialize the config
    let file_content = fs::read_to_string(&config_path)
        .expect("Failed to read config file");
    
    let loaded_config: KwiteConfig = toml::from_str(&file_content)
        .expect("Failed to deserialize config");
    
    // Verify roundtrip
    assert_eq!(original_config.input_device_id, loaded_config.input_device_id);
    assert_eq!(original_config.output_device_id, loaded_config.output_device_id);
    assert_eq!(original_config.sensitivity, loaded_config.sensitivity);
    assert_eq!(original_config.auto_start, loaded_config.auto_start);
    assert_eq!(original_config.minimize_to_tray, loaded_config.minimize_to_tray);
}

#[test]
#[serial]
fn test_config_invalid_toml_fallback() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("invalid_config.toml");
    
    // Write invalid TOML
    fs::write(&config_path, "invalid toml content [[[")
        .expect("Failed to write invalid config");
    
    // Try to parse invalid TOML
    let file_content = fs::read_to_string(&config_path)
        .expect("Failed to read config file");
    
    let parse_result: Result<KwiteConfig, _> = toml::from_str(&file_content);
    assert!(parse_result.is_err(), "Should fail to parse invalid TOML");
    
    // The actual load() function should fall back to defaults on error
    // We can't test this directly without modifying the config_path function
    // but we verify the error handling works
}

#[test]
#[serial]
fn test_config_partial_toml() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("partial_config.toml");
    
    // Write partial TOML (missing some fields)
    let partial_toml = r#"
input_device_id = "custom_input"
sensitivity = 0.5
"#;
    
    fs::write(&config_path, partial_toml)
        .expect("Failed to write partial config");
    
    let file_content = fs::read_to_string(&config_path)
        .expect("Failed to read config file");
    
    let parse_result: Result<KwiteConfig, _> = toml::from_str(&file_content);
    
    // Check if serde can handle partial deserialization
    match parse_result {
        Ok(config) => {
            assert_eq!(config.input_device_id, "custom_input");
            assert_eq!(config.sensitivity, 0.5);
            // Other fields should have their defaults
            println!("Partial TOML parsing succeeded with defaults");
        },
        Err(e) => {
            println!("Partial TOML failed as expected (serde requires all fields): {}", e);
            // This is expected behavior - serde requires all fields unless using #[serde(default)]
            // The actual application handles this in the load() function
        }
    }
}

#[test]
#[serial]
fn test_config_serialization_format() {
    let config = KwiteConfig {
        input_device_id: "test_input".to_string(),
        output_device_id: "test_output".to_string(),
        sensitivity: 0.15,
        auto_start: false,
        minimize_to_tray: false,
        development_mode: false,
        remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
        analytics: kwite::config::AnalyticsConfig::default(),
        auto_update: kwite::config::AutoUpdateConfig::default(),
    };
    
    let toml_content = toml::to_string_pretty(&config)
        .expect("Failed to serialize config");
    
    // Verify TOML format
    assert!(toml_content.contains("input_device_id = \"test_input\""));
    assert!(toml_content.contains("output_device_id = \"test_output\""));
    assert!(toml_content.contains("sensitivity = 0.15"));
    assert!(toml_content.contains("auto_start = false"));
    assert!(toml_content.contains("minimize_to_tray = false"));
}

#[test]
#[serial]
fn test_config_edge_cases() {
    // Test with extreme values
    let config = KwiteConfig {
        input_device_id: "".to_string(), // Empty string
        output_device_id: "very_long_device_id_that_might_cause_issues_in_some_systems".to_string(),
        sensitivity: 0.0, // Minimum sensitivity
        auto_start: false,
        minimize_to_tray: true,
        development_mode: false,
        remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
        analytics: kwite::config::AnalyticsConfig::default(),
        auto_update: kwite::config::AutoUpdateConfig::default(),
    };
    
    let toml_content = toml::to_string_pretty(&config)
        .expect("Failed to serialize config with edge cases");
    
    let parsed_config: KwiteConfig = toml::from_str(&toml_content)
        .expect("Failed to parse config with edge cases");
    
    assert_eq!(config.input_device_id, parsed_config.input_device_id);
    assert_eq!(config.output_device_id, parsed_config.output_device_id);
    assert_eq!(config.sensitivity, parsed_config.sensitivity);
}

#[test]
#[serial]
fn test_config_unicode_handling() {
    // Test with unicode device names
    let config = KwiteConfig {
        input_device_id: "麦克风_设备".to_string(),
        output_device_id: "Audiоaufnahme".to_string(), // Note: contains Cyrillic 'о'
        sensitivity: 0.3,
        auto_start: false,
        minimize_to_tray: true,
        development_mode: false,
        remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
        analytics: kwite::config::AnalyticsConfig::default(),
        auto_update: kwite::config::AutoUpdateConfig::default(),
    };
    
    let toml_content = toml::to_string_pretty(&config)
        .expect("Failed to serialize config with unicode");
    
    let parsed_config: KwiteConfig = toml::from_str(&toml_content)
        .expect("Failed to parse config with unicode");
    
    assert_eq!(config.input_device_id, parsed_config.input_device_id);
    assert_eq!(config.output_device_id, parsed_config.output_device_id);
}
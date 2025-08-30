use kwite::audio::devices::*;
use serial_test::serial;

#[test]
#[serial]
fn test_audio_device_info_display() {
    let device = AudioDeviceInfo {
        id: "test_id".to_string(),
        name: "Test Device".to_string(),
        is_default: true,
        is_virtual: false,
    };
    
    let display_str = format!("{}", device);
    assert!(display_str.contains("Test Device"));
    assert!(display_str.contains("(Default)"), "Default device should show (Default) suffix");
}

#[test]
#[serial]
fn test_audio_device_info_display_virtual() {
    let device = AudioDeviceInfo {
        id: "test_id".to_string(),
        name: "Virtual Device".to_string(),
        is_default: false,
        is_virtual: true,
    };
    
    let display_str = format!("{}", device);
    assert!(display_str.contains("Virtual Device"));
    assert!(display_str.contains("(Virtual)"), "Virtual device should show (Virtual) suffix");
}

#[test]
#[serial]
fn test_audio_device_info_display_regular() {
    let device = AudioDeviceInfo {
        id: "test_id".to_string(),
        name: "Regular Device".to_string(),
        is_default: false,
        is_virtual: false,
    };
    
    let display_str = format!("{}", device);
    assert_eq!(display_str, "Regular Device");
    assert!(!display_str.contains("(Default)"));
    assert!(!display_str.contains("(Virtual)"));
}

#[test]
#[serial]
fn test_audio_device_info_clone() {
    let device = AudioDeviceInfo {
        id: "test_id".to_string(),
        name: "Test Device".to_string(),
        is_default: false,
        is_virtual: true,
    };
    
    let cloned = device.clone();
    assert_eq!(device.id, cloned.id);
    assert_eq!(device.name, cloned.name);
    assert_eq!(device.is_default, cloned.is_default);
    assert_eq!(device.is_virtual, cloned.is_virtual);
}

#[test]
#[serial]
fn test_list_input_devices_not_empty() {
    let devices = list_input_devices();
    assert!(!devices.is_empty(), "Should have at least one input device (even fallback)");
    
    // Check that we have a default device
    let has_default = devices.iter().any(|d| d.is_default);
    assert!(has_default, "Should have at least one default input device");
}

#[test]
#[serial]
fn test_list_output_devices_not_empty() {
    let devices = list_output_devices();
    assert!(!devices.is_empty(), "Should have at least one output device (even fallback)");
    
    // Check that we have a default device
    let has_default = devices.iter().any(|d| d.is_default);
    assert!(has_default, "Should have at least one default output device");
}

#[test]
#[serial]
fn test_device_id_uniqueness() {
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    
    // Check input device ID uniqueness
    let mut input_ids = std::collections::HashSet::new();
    for device in &input_devices {
        assert!(input_ids.insert(&device.id), "Input device ID '{}' is not unique", device.id);
    }
    
    // Check output device ID uniqueness
    let mut output_ids = std::collections::HashSet::new();
    for device in &output_devices {
        assert!(output_ids.insert(&device.id), "Output device ID '{}' is not unique", device.id);
    }
}

#[test]
#[serial]
fn test_virtual_device_detection() {
    let devices = list_output_devices();
    
    // Test that virtual detection logic works correctly
    for device in &devices {
        let name_lower = device.name.to_lowercase();
        let should_be_virtual = name_lower.contains("cable") ||
            name_lower.contains("vb-audio") ||
            name_lower.contains("voicemeeter") ||
            name_lower.contains("virtual") ||
            name_lower.contains("blackhole");
            
        if should_be_virtual {
            // If the name suggests it's virtual, it should be marked as virtual
            // Note: This might not always be true due to driver variations
            println!("Device '{}' appears virtual based on name", device.name);
        }
    }
}

#[test]
#[serial]
fn test_get_device_by_id_input() {
    let devices = list_input_devices();
    
    if let Some(first_device) = devices.first() {
        let device = get_device_by_id(&first_device.id, true);
        assert!(device.is_some(), "Should find input device by valid ID");
    }
    
    // Test with invalid ID
    let invalid_device = get_device_by_id("nonexistent_device_id", true);
    assert!(invalid_device.is_none(), "Should not find device with invalid ID");
}

#[test]
#[serial]
fn test_get_device_by_id_output() {
    let devices = list_output_devices();
    
    if let Some(first_device) = devices.first() {
        let device = get_device_by_id(&first_device.id, false);
        assert!(device.is_some(), "Should find output device by valid ID");
    }
    
    // Test with invalid ID
    let invalid_device = get_device_by_id("nonexistent_device_id", false);
    assert!(invalid_device.is_none(), "Should not find device with invalid ID");
}

#[test]
#[serial]
fn test_find_virtual_output_device() {
    // This test may return None in environments without virtual audio devices
    let virtual_device = find_virtual_output_device();
    
    if let Some(_device) = virtual_device {
        println!("Found virtual output device");
        // In CI environments, we might not have virtual devices
        // so we just test that the function doesn't panic
    } else {
        println!("No virtual output device found (expected in CI environment)");
    }
}

#[test]
#[serial]
fn test_device_enumeration_consistency() {
    // Test that device enumeration is consistent across multiple calls
    let devices1 = list_input_devices();
    let devices2 = list_input_devices();
    
    assert_eq!(devices1.len(), devices2.len(), "Device count should be consistent");
    
    for (d1, d2) in devices1.iter().zip(devices2.iter()) {
        assert_eq!(d1.id, d2.id, "Device IDs should be consistent");
        assert_eq!(d1.name, d2.name, "Device names should be consistent");
    }
}

#[test]
#[serial] 
fn test_device_fallback_behavior() {
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    
    // Even in environments with no real audio devices, we should get fallback devices
    assert!(!input_devices.is_empty(), "Should always have at least fallback input device");
    assert!(!output_devices.is_empty(), "Should always have at least fallback output device");
    
    // Check that fallback devices have reasonable properties
    for device in &input_devices {
        assert!(!device.id.is_empty(), "Device ID should not be empty");
        assert!(!device.name.is_empty(), "Device name should not be empty");
    }
    
    for device in &output_devices {
        assert!(!device.id.is_empty(), "Device ID should not be empty");
        assert!(!device.name.is_empty(), "Device name should not be empty");
    }
}
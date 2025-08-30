// Test to verify device detection works
use kwite::audio::devices::{list_input_devices, list_output_devices};

#[test]
fn test_device_detection() {
    println!("Testing device detection...\n");
    
    println!("Available Input Devices:");
    let input_devices = list_input_devices();
    assert!(!input_devices.is_empty(), "Should have at least one input device");
    for device in &input_devices {
        println!("  - {} (ID: {})", device, device.id);
    }
    
    println!("\nAvailable Output Devices:");
    let output_devices = list_output_devices();
    assert!(!output_devices.is_empty(), "Should have at least one output device");
    for device in &output_devices {
        println!("  - {} (ID: {})", device, device.id);
        if device.is_virtual {
            println!("    ✓ Virtual audio device detected!");
        }
    }
    
    println!("\nDevice detection test completed successfully!");
}
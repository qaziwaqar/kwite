/// Virtual Audio Device Management
/// 
/// This module provides OS-specific guidance for installing and configuring
/// virtual audio devices, making the setup process painless for users.

use std::fmt;

#[derive(Debug, Clone)]
pub struct VirtualAudioInfo {
    pub name: &'static str,
    pub download_url: &'static str,
    pub description: &'static str,
    pub setup_instructions: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

impl fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperatingSystem::Windows => write!(f, "Windows"),
            OperatingSystem::MacOS => write!(f, "macOS"),
            OperatingSystem::Linux => write!(f, "Linux"),
            OperatingSystem::Unknown => write!(f, "Unknown OS"),
        }
    }
}

/// Detect the current operating system
pub fn detect_os() -> OperatingSystem {
    if cfg!(target_os = "windows") {
        OperatingSystem::Windows
    } else if cfg!(target_os = "macos") {
        OperatingSystem::MacOS
    } else if cfg!(target_os = "linux") {
        OperatingSystem::Linux
    } else {
        OperatingSystem::Unknown
    }
}

/// Get virtual audio device information for the current OS
pub fn get_virtual_audio_info() -> VirtualAudioInfo {
    match detect_os() {
        OperatingSystem::Windows => VirtualAudioInfo {
            name: "VB-Audio Cable",
            download_url: "https://www.vb-audio.com/Cable/",
            description: "Virtual Audio Cable for Windows - Creates virtual audio devices for routing audio between applications",
            setup_instructions: vec![
                "1. Download VB-Audio Cable from the official website",
                "2. Run the installer as Administrator",
                "3. Restart your computer after installation",
                "4. In Windows Sound settings, set 'CABLE Input' as your default playbook device in Discord/Teams",
                "5. Restart Kwite to detect the new virtual device",
            ],
        },
        OperatingSystem::MacOS => VirtualAudioInfo {
            name: "VB-Audio Cable",
            download_url: "https://www.vb-audio.com/Cable/",
            description: "VB-Audio Cable for macOS - Virtual Audio Cable for creating virtual audio devices",
            setup_instructions: vec![
                "1. Download VB-Audio Cable from the official website",
                "2. Install the .pkg file (you may need to allow installation in Security preferences)",
                "3. Open Audio MIDI Setup (found in /Applications/Utilities/)",
                "4. Select VB-Cable in the device list",
                "5. IMPORTANT: Set Format to 48000.0 Hz, 32-bit Float for optimal noise cancellation",
                "6. Create a Multi-Output Device including VB-Cable + your speakers",
                "7. Set VB-Cable as input device in your communication apps",
                "8. Set the Multi-Output Device as your system output (for monitoring)",
                "9. In Kwite, select VB-Cable as both input and output device",
                "10. Restart Kwite to detect the properly configured virtual device",
            ],
        },
        OperatingSystem::Linux => VirtualAudioInfo {
            name: "PulseAudio Virtual Sink",
            download_url: "",
            description: "PulseAudio virtual sink - Use built-in PulseAudio features to create virtual audio devices",
            setup_instructions: vec![
                "1. Open a terminal",
                "2. Create a null sink: pactl load-module module-null-sink sink_name=kwite_output",
                "3. Create a loopback from your mic: pactl load-module module-loopback source=<your_mic> sink=kwite_output",
                "4. Find your mic name with: pactl list sources short",
                "5. Use pavucontrol to manage virtual devices graphically",
                "6. Restart Kwite to detect the new virtual device",
            ],
        },
        OperatingSystem::Unknown => VirtualAudioInfo {
            name: "Virtual Audio Device",
            download_url: "",
            description: "Virtual audio setup varies by operating system",
            setup_instructions: vec![
                "1. Virtual audio setup depends on your operating system",
                "2. Windows: Install VB-Audio Cable",
                "3. macOS: Install VB-Audio Cable",
                "4. Linux: Use PulseAudio virtual sinks",
                "5. Consult the documentation for your OS",
            ],
        },
    }
}

/// Check if virtual audio devices are available on the current system
pub fn has_virtual_devices(output_devices: &[crate::audio::devices::AudioDeviceInfo]) -> bool {
    output_devices.iter().any(|d| d.is_virtual)
}

/// Enhanced virtual device detection with OS-specific patterns
pub fn detect_virtual_device_type(device_name: &str) -> Option<&'static str> {
    let name_lower = device_name.to_lowercase();
    
    // Cross-platform VB-Audio Cable detection
    if name_lower.contains("vb-audio") || name_lower.contains("vb-cable") || 
       (name_lower.contains("cable") && (name_lower.contains("input") || name_lower.contains("output"))) {
        return Some("VB-Audio Cable");
    }
    
    // Windows virtual devices
    if name_lower.contains("voicemeeter") {
        return Some("Voicemeeter");
    }
    
    // macOS virtual devices
    if name_lower.contains("blackhole") {
        return Some("BlackHole");
    }
    if name_lower.contains("soundflower") {
        return Some("Soundflower");
    }
    if name_lower.contains("loopback") {
        return Some("Loopback");
    }
    
    // Linux virtual devices
    if name_lower.contains("null") {
        return Some("PulseAudio Null Sink");
    }
    if name_lower.contains("monitor") {
        return Some("PulseAudio Monitor");
    }
    
    // Generic virtual device indicators
    if name_lower.contains("virtual") {
        return Some("Virtual Audio Device");
    }
    
    None
}

/// Get user-friendly setup status message
pub fn get_setup_status_message(has_virtual_devices: bool) -> (String, egui::Color32) {
    if has_virtual_devices {
        ("✅ Virtual audio device detected and ready!".to_string(), egui::Color32::GREEN)
    } else {
        let os = detect_os();
        let info = get_virtual_audio_info();
        let message = match os {
            OperatingSystem::Windows => format!("⚠ Install {} for best Discord/Teams compatibility", info.name),
            OperatingSystem::MacOS => format!("⚠ Install {} for best audio app compatibility", info.name),
            OperatingSystem::Linux => "⚠ Set up PulseAudio virtual sink for best compatibility".to_string(),
            OperatingSystem::Unknown => "⚠ Install virtual audio device for best compatibility".to_string(),
        };
        (message, egui::Color32::GRAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_detection() {
        let os = detect_os();
        assert!(matches!(os, OperatingSystem::Windows | OperatingSystem::MacOS | OperatingSystem::Linux | OperatingSystem::Unknown));
    }

    #[test]
    fn test_virtual_device_detection() {
        assert_eq!(detect_virtual_device_type("VB-Audio Cable"), Some("VB-Audio Cable"));
        assert_eq!(detect_virtual_device_type("VB-Cable Input"), Some("VB-Audio Cable"));
        assert_eq!(detect_virtual_device_type("VB-Cable Output"), Some("VB-Audio Cable"));
        assert_eq!(detect_virtual_device_type("BlackHole 2ch"), Some("BlackHole"));
        assert_eq!(detect_virtual_device_type("Null Output"), Some("PulseAudio Null Sink"));
        assert_eq!(detect_virtual_device_type("Regular Speakers"), None);
    }

    #[test]
    fn test_setup_status_message() {
        let (message, color) = get_setup_status_message(true);
        assert!(message.contains("ready"));
        assert_eq!(color, egui::Color32::GREEN);

        let (message, color) = get_setup_status_message(false);
        assert!(message.contains("⚠"));
        assert_eq!(color, egui::Color32::GRAY);
    }
}
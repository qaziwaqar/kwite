use cpal::traits::{DeviceTrait, HostTrait};
use std::fmt;

#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_virtual: bool,
}

impl fmt::Display for AudioDeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_default {
            write!(f, "{} (Default)", self.name)
        } else if self.is_virtual {
            write!(f, "{} (Virtual)", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

pub fn list_input_devices() -> Vec<AudioDeviceInfo> {
    let mut devices = Vec::new();
    let host = cpal::default_host();
    
    // Get default device
    let default_device = host.default_input_device();
    let _default_name = default_device.as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_else(|| "Unknown".to_string());

    // Enumerate all input devices
    if let Ok(device_iter) = host.input_devices() {
        for (index, device) in device_iter.enumerate() {
            if let Ok(name) = device.name() {
                let is_default = default_device.as_ref()
                    .map(|d| d.name().ok() == Some(name.clone()))
                    .unwrap_or(false);

                devices.push(AudioDeviceInfo {
                    id: format!("input_{}", index),
                    name: name.clone(),
                    is_default,
                    is_virtual: false,
                });
            }
        }
    }

    // If no devices found, add a fallback
    if devices.is_empty() {
        devices.push(AudioDeviceInfo {
            id: "input_default".to_string(),
            name: "Default Microphone".to_string(),
            is_default: true,
            is_virtual: false,
        });
    }

    devices
}

pub fn list_output_devices() -> Vec<AudioDeviceInfo> {
    let mut devices = Vec::new();
    let host = cpal::default_host();
    
    // Get default device
    let default_device = host.default_output_device();
    let _default_name = default_device.as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_else(|| "Unknown".to_string());

    // Enumerate all output devices
    if let Ok(device_iter) = host.output_devices() {
        for (index, device) in device_iter.enumerate() {
            if let Ok(name) = device.name() {
                let is_default = default_device.as_ref()
                    .map(|d| d.name().ok() == Some(name.clone()))
                    .unwrap_or(false);

                let is_virtual = crate::virtual_audio::detect_virtual_device_type(&name).is_some();

                devices.push(AudioDeviceInfo {
                    id: format!("output_{}", index),
                    name: name.clone(),
                    is_default,
                    is_virtual,
                });
            }
        }
    }

    // If no devices found, add fallback
    if devices.is_empty() {
        devices.push(AudioDeviceInfo {
            id: "output_default".to_string(),
            name: "Default Speakers".to_string(),
            is_default: true,
            is_virtual: false,
        });
    }

    devices
}

pub fn get_device_by_id(device_id: &str, is_input: bool) -> Option<cpal::Device> {
    let host = cpal::default_host();
    
    if is_input {
        if device_id == "input_default" {
            return host.default_input_device();
        }
        
        if let Ok(device_iter) = host.input_devices() {
            for (index, device) in device_iter.enumerate() {
                if format!("input_{}", index) == device_id {
                    return Some(device);
                }
            }
        }
    } else {
        if device_id == "output_default" {
            return host.default_output_device();
        }
        
        if let Ok(device_iter) = host.output_devices() {
            for (index, device) in device_iter.enumerate() {
                if format!("output_{}", index) == device_id {
                    return Some(device);
                }
            }
        }
    }
    
    None
}

pub fn find_virtual_output_device() -> Option<cpal::Device> {
    let host = cpal::default_host();
    
    if let Ok(device_iter) = host.output_devices() {
        for device in device_iter {
            if let Ok(name) = device.name() {
                if crate::virtual_audio::detect_virtual_device_type(&name).is_some() {
                    return Some(device);
                }
            }
        }
    }
    
    None
}
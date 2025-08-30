//! # Kwite GUI Application Module
//! 
//! This module contains the main GUI application logic for Kwite, a real-time AI noise cancellation tool.
//! The application provides a user-friendly interface for configuring audio devices, adjusting sensitivity,
//! and controlling the noise cancellation process.
//! 
//! ## Architecture Overview
//! 
//! The GUI follows the immediate mode paradigm using the `egui` framework, which means the entire UI
//! is rebuilt every frame based on the current application state. This approach simplifies state
//! management and ensures the UI always reflects the current state accurately.
//! 
//! ## Key Components
//! 
//! - **Device Management**: Handles audio device enumeration and selection
//! - **Configuration Management**: Persistent storage of user preferences
//! - **Audio Processing Control**: Start/stop noise cancellation with real-time parameter updates
//! - **Real-time Feedback**: Visual indicators for system status and configuration changes

use eframe::egui;
use egui::{CentralPanel, TopBottomPanel, Button, Slider, ComboBox, Color32, RichText};
use crate::logger::log;
use crate::audio::{AudioManager, devices::{AudioDeviceInfo, list_input_devices, list_output_devices}};
use crate::config::KwiteConfig;
use crate::ai_metrics::{SharedAiMetrics, PerformanceSummary};
use crate::virtual_audio::{get_virtual_audio_info, has_virtual_devices, get_setup_status_message, detect_os};
use crate::remote_logging::{init_remote_logger, log_remote};
use crate::usage_stats::UsageStatsManager;
use crate::auto_update::AutoUpdateManager;
use crate::system_info::SystemInfo;
use std::sync::{Arc, Mutex};

/// Main Kwite App state
/// 
/// This struct maintains all the application state including:
/// - Audio device configurations and selections
/// - Real-time processing parameters
/// - UI state and configuration persistence
/// 
/// The state is designed to be reactive - any changes to critical parameters
/// like device selection or sensitivity immediately trigger updates to the
/// underlying audio processing system.
pub struct KwiteApp {
    /// Whether noise cancellation is currently active
    /// This directly controls the AudioManager lifecycle
    enabled: bool,
    
    /// List of available input devices (microphones, line-in, etc.)
    /// Refreshed periodically to handle device hotplug events
    input_devices: Vec<AudioDeviceInfo>,
    
    /// List of available output devices (speakers, virtual cables, etc.)
    /// Virtual audio devices are preferred for applications like Discord/Teams
    output_devices: Vec<AudioDeviceInfo>,
    
    /// Currently selected input device ID
    /// Persisted in configuration for session continuity
    selected_input_device: String,
    
    /// Currently selected output device ID
    /// Automatically prefers virtual audio devices when available
    selected_output_device: String,
    
    /// Noise cancellation sensitivity threshold (0.01 - 0.5)
    /// Lower values = more aggressive noise removal
    /// Higher values = preserve more original audio
    sensitivity: f32,
    
    /// Thread-safe reference to the audio processing manager
    /// Wrapped in Arc<Mutex<>> for safe sharing between GUI and audio threads
    audio_manager: Arc<Mutex<Option<AudioManager>>>,
    
    /// Timestamp of last device enumeration
    /// Used to implement automatic device refresh every 5 seconds
    last_device_refresh: std::time::Instant,
    
    /// Persistent configuration storage
    /// Automatically saved when critical settings change
    config: KwiteConfig,
    
    /// Flag indicating unsaved configuration changes
    /// Triggers visual indicator and save button in UI
    config_changed: bool,
    
    /// AI performance metrics for real-time display
    /// Shows VAD scores, processing latency, and model confidence
    ai_metrics: Option<SharedAiMetrics>,
    
    /// Cached AI performance summary for display
    /// Updated periodically to avoid excessive mutex locking
    ai_performance: Option<PerformanceSummary>,
    
    /// Last time AI metrics were updated
    last_ai_update: std::time::Instant,
    
    /// Track if sensitivity slider is being dragged (for update-on-release behavior)
    sensitivity_dragging: bool,
    sensitivity_pending_update: Option<f32>,
    

    
    /// Flag to show virtual audio device setup dialog
    show_virtual_setup_dialog: bool,
    
    /// Flag to show macOS audio configuration dialog
    show_macos_audio_dialog: bool,

    /// Flag to show configuration dialog
    show_config_dialog: bool,
    
    /// Show advanced AI controls
    show_advanced_controls: bool,

    /// Maximum test mode for extreme noise cancellation debugging
    /// When enabled, uses extremely aggressive settings to test if noise cancellation is working at all
    max_test_mode: bool,
    
    /// Pipeline verification mode for testing audio routing
    /// When enabled, adds a test tone to verify audio is flowing through the processing pipeline
    pipeline_verification_mode: bool,

    /// Usage statistics manager for tracking application metrics
    usage_stats: Option<UsageStatsManager>,

    /// Auto-update manager for checking and downloading updates
    auto_update_manager: Option<AutoUpdateManager>,

    /// System information collected at startup
    system_info: SystemInfo,
}

impl KwiteApp {
    /// Initialize the application with default or saved configuration
    /// 
    /// This constructor performs several important initialization tasks:
    /// 1. Load persistent configuration from disk
    /// 2. Enumerate available audio devices
    /// 3. Validate and select appropriate default devices
    /// 4. Set up the application state for immediate use
    /// 
    /// Device selection priority:
    /// - Input: Use saved device if available, otherwise use system default
    /// - Output: Prefer virtual audio devices, fallback to saved/default
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = KwiteConfig::load();
        let input_devices = list_input_devices();
        let output_devices = list_output_devices();
        
        // Use config devices if they exist, otherwise select defaults
        let selected_input = if input_devices.iter().any(|d| d.id == config.input_device_id) {
            config.input_device_id.clone()
        } else {
            input_devices.iter()
                .find(|d| d.is_default)
                .map(|d| d.id.clone())
                .unwrap_or_else(|| input_devices.first().map(|d| d.id.clone()).unwrap_or_default())
        };
            
        let selected_output = if output_devices.iter().any(|d| d.id == config.output_device_id) {
            config.output_device_id.clone()
        } else {
            output_devices.iter()
                .find(|d| d.is_virtual)
                .or_else(|| output_devices.iter().find(|d| d.is_default))
                .map(|d| d.id.clone())
                .unwrap_or_else(|| output_devices.first().map(|d| d.id.clone()).unwrap_or_default())
        };

        // Initialize remote logging if enabled
        if config.remote_logging.enabled {
            init_remote_logger(config.remote_logging.clone());
            log_remote("info", "Kwite application started", Some("gui::app"), std::collections::HashMap::new());
        }

        // Initialize usage statistics if analytics enabled
        let usage_stats = if config.analytics.enabled {
            let mut stats = UsageStatsManager::new(true);
            stats.start_session();
            Some(stats)
        } else {
            None
        };

        // Initialize auto-update manager if enabled
        let auto_update_manager = if config.auto_update.enabled {
            Some(AutoUpdateManager::new(config.auto_update.clone()))
        } else {
            None
        };

        // Collect system information
        let system_info = SystemInfo::collect();

        // Log system information for analytics (if remote logging is enabled)
        if config.remote_logging.enabled {
            let mut fields = system_info.to_fields();
            fields.insert("startup".to_string(), "true".to_string());
            log_remote("info", &system_info.to_log_string(), Some("system_info"), fields);
        }

        let mut app = KwiteApp {
            enabled: false, // Will be set based on auto_start config below
            input_devices,
            output_devices,
            selected_input_device: selected_input,
            selected_output_device: selected_output,
            sensitivity: config.sensitivity,
            audio_manager: Arc::new(Mutex::new(None)),
            last_device_refresh: std::time::Instant::now(),
            config,
            config_changed: false,
            ai_metrics: None,
            ai_performance: None,
            last_ai_update: std::time::Instant::now(),
            sensitivity_dragging: false,
            sensitivity_pending_update: None,
            show_advanced_controls: false,
            max_test_mode: std::env::var("KWITE_MAX_TEST").is_ok(), // Initialize from environment variable
            pipeline_verification_mode: false, // Disabled by default
            show_virtual_setup_dialog: false,
            show_macos_audio_dialog: false,
            show_config_dialog: false,
            usage_stats,
            auto_update_manager,
            system_info,
        };

        // Auto-start noise cancellation if configured
        if app.config.auto_start {
            log::info!("Auto-starting noise cancellation as configured");
            log::info!("Input device: {} | Output device: {}", 
                      &app.selected_input_device, &app.selected_output_device);
            app.toggle_audio_processing();
            
            // Wait a moment and verify the processing started
            std::thread::sleep(std::time::Duration::from_millis(100));
            if app.enabled {
                log::info!("✅ Auto-start successful - noise cancellation is ACTIVE");
            } else {
                log::warn!("❌ Auto-start failed - noise cancellation is NOT active");
            }
        } else {
            log::info!("Auto-start disabled in configuration - noise cancellation will be started manually");
        }

        app
    }

    /// Persist current configuration to disk
    /// 
    /// This method ensures user preferences survive application restarts.
    /// Configuration includes device selections, sensitivity settings, and all
    /// other settings that can be modified through the UI settings dialog.
    /// Called automatically when users modify settings or manually via save button.
    fn save_config(&mut self) {
        // Update audio-related settings
        self.config.input_device_id = self.selected_input_device.clone();
        self.config.output_device_id = self.selected_output_device.clone();
        self.config.sensitivity = self.sensitivity;
        
        // Note: Other settings like development_mode, analytics, auto_update, and 
        // remote_logging are already updated directly in the UI handlers when
        // checkboxes are modified, so they don't need to be updated here.
        // This ensures all configuration changes made through the UI are persisted.
        
        if let Err(e) = self.config.save() {
            log::error!("Failed to save configuration: {}", e);
        } else {
            self.config_changed = false;
            log::info!("Configuration saved successfully");
        }
    }

    /// Refresh the list of available audio devices
    /// 
    /// CRITICAL SAFETY: This method should NEVER be called during active audio processing
    /// Device enumeration can cause audio driver conflicts and thread panics.
    /// All calling code must verify audio processing is completely stopped.
    /// 
    /// Modern audio systems support device hotplug (USB headsets, virtual cables, etc.)
    /// This method re-enumerates devices and validates current selections.
    /// 
    /// Automatic fallback logic:
    /// - If selected device disappears, choose a reasonable default
    /// - For outputs, prefer virtual audio devices for application compatibility
    /// - Mark configuration as changed if selections are updated
    fn refresh_devices(&mut self) {
        // Don't refresh devices while audio processing is active
        if self.enabled {
            return;
        }
        
        self.input_devices = list_input_devices();
        self.output_devices = list_output_devices();
        self.last_device_refresh = std::time::Instant::now();
        log::info!("Refreshed audio devices - Input: {}, Output: {}", 
                  self.input_devices.len(), self.output_devices.len());
        
        // Validate current selections
        if !self.input_devices.iter().any(|d| d.id == self.selected_input_device) {
            self.selected_input_device = self.input_devices.first()
                .map(|d| d.id.clone())
                .unwrap_or_default();
            self.config_changed = true;
        }
        
        if !self.output_devices.iter().any(|d| d.id == self.selected_output_device) {
            self.selected_output_device = self.output_devices.iter()
                .find(|d| d.is_virtual)
                .or_else(|| self.output_devices.first())
                .map(|d| d.id.clone())
                .unwrap_or_default();
            self.config_changed = true;
        }
    }

    /// Toggle the noise cancellation processing on/off
    /// 
    /// This is the core functionality that starts/stops the audio processing pipeline.
    /// When enabled:
    /// 1. Creates new AudioManager with current device selections
    /// 2. Starts input capture, processing, and output threads
    /// 3. Begins real-time noise cancellation
    /// 
    /// When disabled:
    /// 1. Stops all audio processing threads gracefully
    /// 2. Releases audio device handles
    /// 3. Returns system to normal audio routing
    fn toggle_audio_processing(&mut self) {
        self.enabled = !self.enabled;
        log::info!("Noise cancellation toggled: {}", self.enabled);

        // Record feature usage in statistics
        if let Some(ref mut stats) = self.usage_stats {
            if self.enabled {
                stats.start_noise_cancellation();
                stats.record_feature_usage("noise_cancellation_start");
            } else {
                stats.stop_noise_cancellation();
                stats.record_feature_usage("noise_cancellation_stop");
            }
        }

        // Log remote event if enabled
        if self.config.remote_logging.enabled {
            let mut fields = std::collections::HashMap::new();
            fields.insert("action".to_string(), if self.enabled { "start" } else { "stop" }.to_string());
            fields.insert("device_input".to_string(), self.selected_input_device.clone());
            fields.insert("device_output".to_string(), self.selected_output_device.clone());
            fields.insert("sensitivity".to_string(), self.sensitivity.to_string());
            log_remote("info", &format!("Noise cancellation {}", if self.enabled { "started" } else { "stopped" }), Some("audio_processing"), fields);
        }

        let mut manager = self.audio_manager.lock().unwrap();

        if self.enabled {
            // Start audio processing
            match AudioManager::new(self.sensitivity, &self.selected_input_device, &self.selected_output_device) {
                Ok(audio_mgr) => {
                    // Capture AI metrics reference for monitoring
                    self.ai_metrics = Some(audio_mgr.get_ai_metrics());
                    *manager = Some(audio_mgr);
                    log::info!("Audio processing started successfully with AI metrics monitoring");
                }
                Err(e) => {
                    log::error!("Failed to start audio processing: {}", e);
                    self.enabled = false;
                    self.ai_metrics = None;
                    
                    // Record error in statistics
                    if let Some(ref mut stats) = self.usage_stats {
                        stats.record_error("audio_start_failed", false);
                    }
                    
                    // Log error remotely if enabled
                    if self.config.remote_logging.enabled {
                        let mut fields = std::collections::HashMap::new();
                        fields.insert("error".to_string(), e.to_string());
                        log_remote("error", "Failed to start audio processing", Some("audio_processing"), fields);
                    }
                }
            }
        } else {
            // Stop audio processing
            *manager = None;
            self.ai_metrics = None;
            self.ai_performance = None;
            log::info!("Audio processing stopped");
        }
    }

    /// Update noise cancellation sensitivity in real-time with rate limiting
    /// 
    /// The sensitivity parameter controls how aggressively the AI model
    /// removes background noise. This can be adjusted while processing
    /// is active for immediate feedback.
    /// 
    /// Rate limiting prevents overwhelming the audio processing thread during
    /// rapid slider movements, batching updates to occur at most every 50ms.
    /// 
    /// Range: 0.01 (very aggressive) to 0.5 (preserve more audio)
    /// Logarithmic scale provides better control granularity
    /// Update noise cancellation sensitivity 
    /// Only called when the slider is released to avoid overwhelming the audio thread
    fn update_sensitivity(&mut self, new_sensitivity: f32) {
        self.sensitivity = new_sensitivity.clamp(0.01, 0.5);
        
        // Update the audio manager with new sensitivity
        if let Ok(mut manager) = self.audio_manager.lock() {
            if let Some(audio_mgr) = manager.as_mut() {
                audio_mgr.update_sensitivity(self.sensitivity);
                log::debug!("Updated sensitivity to: {}", self.sensitivity);
            }
        }
        
        self.config_changed = true;
    }
    
    /// Update AI performance metrics display
    /// 
    /// Called periodically to refresh the AI metrics display without
    /// excessive mutex locking that could impact audio performance
    fn update_ai_metrics(&mut self) {
        if self.last_ai_update.elapsed().as_millis() > 100 {  // Update every 100ms
            if let Some(ref metrics) = self.ai_metrics {
                if let Ok(metrics_guard) = metrics.lock() {
                    self.ai_performance = Some(metrics_guard.get_performance_summary());
                }
            }
            self.last_ai_update = std::time::Instant::now();
        }
    }
}

impl eframe::App for KwiteApp {
    /// Main UI update loop - called every frame
    /// 
    /// This method implements the complete user interface using immediate mode GUI.
    /// The UI is structured into logical sections:
    /// 
    /// 1. **Top Panel**: Application title and configuration status
    /// 2. **Central Panel**: Main controls organized in groups
    ///    - Device Selection: Input/output device choosers with refresh
    ///    - Sensitivity Control: Real-time threshold adjustment
    ///    - Processing Control: Main enable/disable button with status
    /// 
    /// The UI provides immediate feedback for all user actions and clearly
    /// indicates system status through colors and icons.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Automatic device refresh every 5 seconds to handle hotplug events
        // This ensures the device list stays current without manual intervention
        // CRITICAL SAFETY: Skip device refresh if audio processing is active OR
        // if there's been any recent sensitivity changes to prevent interference
        // with active audio streams during rapid parameter adjustments
        let _audio_manager_active = {
            if let Ok(manager) = self.audio_manager.try_lock() {
                manager.is_some()
            } else {
                true // If we can't check, assume it's active for safety
            }
        };
        
        // Auto-refresh devices every 5 seconds when not processing audio
        let should_refresh = self.last_device_refresh.elapsed().as_secs() > 5 && !self.enabled;
            
        if should_refresh {
            self.refresh_devices();
        }

        // Top panel shows application branding and configuration status
        // The configuration indicator helps users understand when settings need saving
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Kwite — Intelligent AI Noise Cancellation");
                
                // Show current noise cancellation status
                if self.enabled {
                    ui.separator();
                    ui.label(RichText::new("RNNoise Active").small().italics());
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.config_changed {
                        if ui.button("💾 Save Config").on_hover_text("Save current settings").clicked() {
                            self.save_config();
                        }
                        ui.colored_label(egui::Color32::GRAY, "●");
                    } else {
                        ui.colored_label(egui::Color32::GREEN, "●");
                    }
                    ui.small("Config:");
                    
                    ui.separator();
                    
                    if ui.small_button("⚙ Settings").on_hover_text("Open application settings").clicked() {
                        self.show_config_dialog = true;
                    }
                });
            });
        });

        // Central panel contains all main application controls
        // Organized vertically with consistent spacing and grouping
        CentralPanel::default().show(ctx, |ui| {
            // Update AI metrics periodically for display
            self.update_ai_metrics();
            
            ui.vertical_centered_justified(|ui| {
                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("🎙 Input Device:");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Disable device refresh during active audio processing to prevent interference
                                let refresh_enabled = !self.enabled;
                                let refresh_tooltip = if refresh_enabled {
                                    "Refresh devices"
                                } else {
                                    "Device refresh disabled during active audio processing"
                                };
                                
                                if ui.add_enabled(refresh_enabled, egui::Button::new("🔄").small())
                                    .on_hover_text(refresh_tooltip)
                                    .clicked() 
                                {
                                    self.refresh_devices();
                                }
                            });
                        });
                        
                        let selected_input_name = self.input_devices.iter()
                            .find(|d| d.id == self.selected_input_device)
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "No device selected".to_string());
                            
                        ComboBox::from_id_salt("input_device")
                            .selected_text(selected_input_name)
                            .show_ui(ui, |ui| {
                                for device in &self.input_devices {
                                    if ui.selectable_value(&mut self.selected_input_device, device.id.clone(), device.to_string()).clicked() {
                                        self.config_changed = true;
                                    }
                                }
                            });

                        ui.add_space(10.0);

                        ui.label("🔊 Output Device:");
                        let selected_output_name = self.output_devices.iter()
                            .find(|d| d.id == self.selected_output_device)
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "No device selected".to_string());
                            
                        ComboBox::from_id_salt("output_device")
                            .selected_text(selected_output_name)
                            .show_ui(ui, |ui| {
                                for device in &self.output_devices {
                                    if ui.selectable_value(&mut self.selected_output_device, device.id.clone(), device.to_string()).clicked() {
                                        self.config_changed = true;
                                    }
                                }
                            });
                            
                        // Enhanced virtual device setup guidance
                        ui.add_space(5.0);
                        let has_virtual = has_virtual_devices(&self.output_devices);
                        let (status_message, status_color) = get_setup_status_message(has_virtual);
                        
                        ui.horizontal(|ui| {
                            ui.colored_label(status_color, &status_message);
                            
                            if !has_virtual {
                                if ui.small_button("📋 Setup Guide").on_hover_text("Show detailed setup instructions").clicked() {
                                    self.show_virtual_setup_dialog = true;
                                }
                            }
                        });
                        
                        // macOS Virtual Audio Device Configuration Warning
                        if cfg!(target_os = "macos") {
                            ui.add_space(5.0);
                            
                            // Check for configuration issues
                            let input_device_name = self.input_devices.iter()
                                .find(|d| d.id == self.selected_input_device)
                                .map(|d| d.name.clone())
                                .unwrap_or_else(|| "Unknown".to_string());
                            let output_device_name = self.output_devices.iter()
                                .find(|d| d.id == self.selected_output_device)
                                .map(|d| d.name.clone())
                                .unwrap_or_else(|| "Unknown".to_string());
                            
                            let input_virtual_type = crate::virtual_audio::detect_virtual_device_type(&input_device_name);
                            let output_virtual_type = crate::virtual_audio::detect_virtual_device_type(&output_device_name);
                            
                            if input_virtual_type.is_some() || output_virtual_type.is_some() {
                                ui.horizontal(|ui| {
                                    if input_virtual_type.is_some() {
                                        ui.colored_label(Color32::from_rgb(255, 100, 100), "🚨 Virtual audio device configuration issue detected!");
                                    } else {
                                        ui.colored_label(Color32::from_rgb(255, 165, 0), "🍎 Virtual audio device detected - verify setup");
                                    }
                                    
                                    if ui.small_button("⚙️ macOS Audio Setup").on_hover_text("Show virtual audio device configuration guide").clicked() {
                                        self.show_macos_audio_dialog = true;
                                    }
                                });
                                
                                if let Some(device_type) = input_virtual_type {
                                    ui.colored_label(Color32::from_rgb(255, 100, 100), format!("Input should be your microphone, not {}!", device_type));
                                }
                            }
                        }
                    });
                });

                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Sensitivity Threshold:");
                        
                        let slider_response = ui.add(Slider::new(&mut self.sensitivity, 0.01..=0.5)
                            .text("Sensitivity")
                            .logarithmic(true));

                        // Track if user is dragging the slider
                        if slider_response.is_pointer_button_down_on() {
                            self.sensitivity_dragging = true;
                            self.sensitivity_pending_update = Some(self.sensitivity);
                        } else if self.sensitivity_dragging && !slider_response.is_pointer_button_down_on() {
                            // User just released the slider - apply the update
                            self.sensitivity_dragging = false;
                            if let Some(pending_value) = self.sensitivity_pending_update.take() {
                                self.update_sensitivity(pending_value);
                            }
                        }

                        ui.small(format!("Current: {:.3}", self.sensitivity));
                    });
                });

                ui.add_space(20.0);

                let button_text = if self.enabled { "🛑 Disable" } else { "▶ Enable" };
                let button_color = if self.enabled {
                    egui::Color32::from_rgb(220, 53, 69)
                } else {
                    egui::Color32::from_rgb(40, 167, 69)
                };

                ui.scope(|ui| {
                    ui.style_mut().visuals.widgets.inactive.bg_fill = button_color;
                    ui.style_mut().visuals.widgets.hovered.bg_fill = button_color;
                    ui.style_mut().visuals.widgets.active.bg_fill = button_color;

                    if ui.add_sized([200.0, 40.0], Button::new(button_text)).clicked() {
                        self.toggle_audio_processing();
                    }
                });

                ui.add_space(20.0);

                // AI Performance Metrics Display (when active and in development mode)
                if self.enabled && self.ai_performance.is_some() && self.config.development_mode {
                    if let Some(ref perf) = self.ai_performance {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("🧠 AI Performance (Dev Mode)").strong());
                                    
                                    // AI Status indicator with color
                                    let (r, g, b) = perf.ai_status.color();
                                    let status_color = Color32::from_rgb(r, g, b);
                                    ui.colored_label(status_color, format!("● {}", perf.ai_status.as_str()));
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("⚙").on_hover_text("Advanced AI Controls").clicked() {
                                            self.show_advanced_controls = !self.show_advanced_controls;
                                        }
                                    });
                                });
                                
                                ui.add_space(5.0);
                                
                                // Real-time AI metrics in columns
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.small("Voice Activity:");
                                        ui.label(format!("{:.1}%", perf.avg_vad_score * 100.0));
                                        
                                        ui.small("Model Confidence:");
                                        ui.label(format!("{:.1}%", perf.model_confidence * 100.0));
                                    });
                                    
                                    ui.separator();
                                    
                                    ui.vertical(|ui| {
                                        ui.small("Processing Latency:");
                                        ui.label(format!("{:.1}ms", perf.avg_latency_ms));
                                        
                                        ui.small("Noise Reduction:");
                                        ui.label(format!("{:.1}%", perf.noise_reduction_percent));
                                    });
                                    
                                    ui.separator();
                                    
                                    ui.vertical(|ui| {
                                        ui.small("Frames Processed:");
                                        ui.label(format!("{}", perf.frames_processed));
                                        
                                        ui.small("Est. Frame Rate:");
                                        ui.label(format!("{} fps", perf.estimated_fps));
                                    });
                                });
                                
                                // Show simplified controls for advanced users
                                if self.show_advanced_controls {
                                    ui.add_space(10.0);
                                    ui.separator();
                                    
                                    // Show AI performance metrics if available
                                    if let Some(ref ai_performance) = self.ai_performance {
                                        ui.horizontal(|ui| {
                                            ui.label("⚡ Performance:");
                                            ui.colored_label(Color32::BLUE, format!("Latency: {:.1}ms", ai_performance.avg_latency_ms));
                                            ui.colored_label(Color32::BLUE, format!("VAD: {:.1}%", ai_performance.avg_vad_score * 100.0));
                                        });
                                    }
                                } else {
                                    // Show simple status for basic users
                                    ui.horizontal(|ui| {
                                        ui.small("Noise Cancellation: ");
                                        if self.enabled {
                                            ui.colored_label(Color32::GREEN, "RNNoise ✓");
                                        } else {
                                            ui.colored_label(Color32::GRAY, "Inactive");
                                        }
                                    });
                                }
                                
                                // Professional comparison note
                                ui.add_space(5.0);
                                ui.small(RichText::new("Professional AI noise cancellation powered by RNNoise").italics().color(Color32::GRAY));
                            });
                        });
                        
                        ui.add_space(10.0);
                    }
                } else if self.enabled && !self.config.development_mode {
                    // Show simple status for end users
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🧠 Noise Cancellation").strong());
                            ui.colored_label(Color32::GREEN, "● Active");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.small("RNNoise AI processing active");
                            });
                        });
                    });
                    ui.add_space(10.0);
                }

                let status_text = if self.enabled { "✅ Noise Cancellation Active" } else { "⌛ Inactive" };
                let status_color = if self.enabled {
                    egui::Color32::from_rgb(40, 167, 69)
                } else {
                    egui::Color32::GRAY
                };

                ui.colored_label(status_color, status_text);
            });
        });

        // Virtual Audio Device Setup Dialog
        if self.show_virtual_setup_dialog {
            self.show_virtual_setup_window(ctx);
        }
        
        // macOS Audio Configuration Dialog
        if self.show_macos_audio_dialog {
            self.show_macos_audio_window(ctx);
        }

        // Configuration Settings Dialog
        if self.show_config_dialog {
            self.show_config_window(ctx);
        }
    }
}

impl KwiteApp {
    /// Show virtual audio device setup dialog with OS-specific instructions
    fn show_virtual_setup_window(&mut self, ctx: &egui::Context) {
        let mut close_dialog = false;
        let mut open = true;
        
        egui::Window::new("Virtual Audio Device Setup")
            .open(&mut open)
            .default_width(600.0)
            .default_height(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                let os = detect_os();
                let info = get_virtual_audio_info();
                
                ui.heading(format!("Setup for {}", os));
                ui.add_space(10.0);
                
                ui.label(egui::RichText::new("Why do I need a virtual audio device?").strong());
                ui.label("Virtual audio devices allow Kwite to route processed audio to applications like Discord, Teams, Zoom, etc. This creates a seamless noise cancellation experience.");
                ui.add_space(10.0);
                
                ui.label(egui::RichText::new(info.name).heading());
                ui.label(info.description);
                ui.add_space(10.0);
                
                if !info.download_url.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Download:");
                        if ui.link(info.download_url).clicked() {
                            if let Err(e) = webbrowser::open(info.download_url) {
                                log::error!("Failed to open browser: {}", e);
                            }
                        }
                    });
                    ui.add_space(10.0);
                }
                
                ui.label(egui::RichText::new("Setup Instructions:").strong());
                
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (i, instruction) in info.setup_instructions.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}.", i + 1));
                                ui.label(*instruction);
                            });
                            ui.add_space(5.0);
                        }
                    });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("🔄 Refresh Devices").on_hover_text("Check for newly installed devices").clicked() {
                        close_dialog = true; // Mark for refresh and close
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            close_dialog = true;
                        }
                    });
                });
            });
        
        // Handle dialog state changes
        if close_dialog || !open {
            self.show_virtual_setup_dialog = false;
            if close_dialog {
                self.refresh_devices();
            }
        }
    }
    
    /// Show macOS audio configuration dialog
    fn show_macos_audio_window(&mut self, ctx: &egui::Context) {
        let mut open = true;
        
        egui::Window::new("🍎 macOS Audio Configuration")
            .open(&mut open)
            .default_width(650.0)
            .default_height(550.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Virtual Audio Device Configuration on macOS");
                ui.add_space(10.0);
                
                // Current Configuration Status
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("📊 Current Configuration Status").heading());
                        ui.add_space(5.0);
                        
                        // Find current input device info
                        let input_device_name = self.input_devices.iter()
                            .find(|d| d.id == self.selected_input_device)
                            .map(|d| d.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());
                        
                        // Find current output device info
                        let output_device_name = self.output_devices.iter()
                            .find(|d| d.id == self.selected_output_device)
                            .map(|d| d.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());
                        
                        // Check for configuration issues using generic virtual device detection
                        let input_virtual_type = crate::virtual_audio::detect_virtual_device_type(&input_device_name);
                        let output_virtual_type = crate::virtual_audio::detect_virtual_device_type(&output_device_name);
                        
                        ui.horizontal(|ui| {
                            ui.label("Input Device:");
                            if input_virtual_type.is_some() {
                                ui.colored_label(Color32::from_rgb(255, 100, 100), format!("❌ {} (INCORRECT)", input_device_name));
                            } else {
                                ui.colored_label(Color32::GREEN, format!("✅ {}", input_device_name));
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Output Device:");
                            if output_virtual_type.is_some() {
                                ui.colored_label(Color32::GREEN, format!("✅ {}", output_device_name));
                            } else {
                                ui.colored_label(Color32::from_rgb(255, 165, 0), format!("⚠️ {} (may not route to apps)", output_device_name));
                            }
                        });
                        
                        if let Some(device_type) = input_virtual_type {
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.colored_label(Color32::from_rgb(255, 100, 100), "🚨 CRITICAL ISSUE:");
                                ui.label(format!("{} as input will NOT provide noise cancellation!", device_type));
                            });
                            ui.label("Change input to your microphone for proper noise cancellation.");
                        }
                        
                        if output_virtual_type.is_none() {
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.colored_label(Color32::from_rgb(255, 165, 0), "ℹ️ NOTE:");
                                ui.label("Output device should be a virtual audio device (VB-Cable/BlackHole) to route to communication apps.");
                            });
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                ui.label(egui::RichText::new("For optimal noise cancellation performance on macOS:").strong());
                ui.add_space(5.0);
                
                // Sample Rate Configuration
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("1. Set Virtual Audio Device to 48kHz Sample Rate").heading());
                        ui.add_space(5.0);
                        ui.label("• Open Audio MIDI Setup (/Applications/Utilities/)");
                        ui.label("• Select your virtual audio device (VB-Cable/BlackHole) in the device list");
                        ui.label("• Set Format to: 48000.0 Hz, 32-bit Float");
                        ui.label("• This ensures optimal AI processing frame alignment");
                        
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.colored_label(Color32::from_rgb(255, 165, 0), "⚠️ Important:");
                            ui.label("Using 44.1kHz can cause audio quality issues with AI noise cancellation");
                        });
                    });
                });
                
                ui.add_space(10.0);
                
                // Device Configuration - Correct Setup
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("2. Correct Device Configuration").heading());
                        ui.add_space(5.0);
                        ui.label("🎤 In Kwite (this app):");
                        ui.label("  • Input Device: Your microphone (Built-in, USB, etc.)");
                        ui.label("  • Output Device: Virtual audio device (VB-Cable/BlackHole)");
                        ui.add_space(5.0);
                        ui.label("💬 In communication apps (Discord, Teams, Zoom):");
                        ui.label("  • Input Device: Virtual audio device (VB-Cable/BlackHole)");
                        ui.label("  • Output Device: Your speakers/headphones");
                    });
                });
                
                ui.add_space(10.0);
                
                // Multi-Output Device Setup
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("3. Create Multi-Output Device (Optional)").heading());
                        ui.add_space(5.0);
                        ui.label("• In Audio MIDI Setup, click '+' and select 'Create Multi-Output Device'");
                        ui.label("• Check both your virtual audio device and your speakers/headphones");
                        ui.label("• Set this Multi-Output Device as your system output");
                        ui.label("• This allows you to hear the processed audio locally");
                    });
                });
                
                ui.add_space(10.0);
                
                // Troubleshooting
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("🔧 Troubleshooting").heading());
                        ui.add_space(5.0);
                        ui.label("If you still hear background noise:");
                        ui.label("• Verify input device is your MICROPHONE, not virtual audio device");
                        ui.label("• Check that virtual audio device is set to 48kHz (not 44.1kHz)");
                        ui.label("• Verify your microphone input levels aren't too high");
                        ui.label("• Try adjusting Kwite's sensitivity slider");
                        ui.label("• Restart applications after changing audio settings");
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.colored_label(Color32::GREEN, "✅ Tip:");
                            ui.label("Check Kwite's log messages for configuration warnings");
                        });
                    });
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("🔄 Refresh Devices").on_hover_text("Check audio device configurations").clicked() {
                        self.refresh_devices();
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            self.show_macos_audio_dialog = false;
                        }
                    });
                });
            });
        
        if !open {
            self.show_macos_audio_dialog = false;
        }
    }

    /// Show configuration settings dialog
    fn show_config_window(&mut self, ctx: &egui::Context) {
        let mut open = true;
        
        egui::Window::new("⚙ Application Settings")
            .open(&mut open)
            .default_width(500.0)
            .default_height(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("General Settings");
                    ui.add_space(10.0);
                    
                    // Development Mode Toggle (only in debug builds)
                    #[cfg(debug_assertions)]
                    {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("🔧 Geek Mode:");
                                    if ui.checkbox(&mut self.config.development_mode, "Enable advanced analytics")
                                        .on_hover_text("Shows detailed AI metrics, performance data")
                                        .changed() {
                                        self.config_changed = true;
                                    }
                                });
                                
                                if self.config.development_mode {
                                    ui.small(RichText::new("⚠ Geek mode shows advanced technical information").color(Color32::GRAY));
                                    
                                    ui.add_space(5.0);
                                    ui.horizontal(|ui| {
                                        ui.label("🚨 Debug Testing:");
                                        if ui.checkbox(&mut self.max_test_mode, "Maximum Test Mode")
                                            .on_hover_text("EXTREME noise cancellation settings for debugging. Reduces background noise to 1% volume - should be VERY noticeable if noise cancellation is working at all.")
                                            .changed() {
                                            // Update the global flag so audio processing thread sees the change
                                            crate::audio::set_max_test_mode(self.max_test_mode);
                                        }
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("🔧 Audio Routing:");
                                        if ui.checkbox(&mut self.pipeline_verification_mode, "Pipeline Verification Mode")
                                            .on_hover_text("Adds a subtle 440Hz test tone to verify audio is flowing through the noise cancellation pipeline. If you can't hear the tone, audio routing is incorrect.")
                                            .changed() {
                                            // Update the global flag so audio processing thread sees the change
                                            crate::audio::set_pipeline_verification_mode(self.pipeline_verification_mode);
                                        }
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("🔍 Diagnostics:");
                                        if ui.button("Run Comprehensive Diagnostics")
                                            .on_hover_text("Logs detailed diagnostic information to help troubleshoot noise cancellation issues. Check the logs for detailed analysis.")
                                            .clicked() {
                                            crate::audio::log_comprehensive_diagnostics();
                                            log::warn!("📋 Comprehensive diagnostics logged - check the console/logs for detailed analysis");
                                        }
                                    });
                                    
                                    if self.max_test_mode {
                                        ui.small(RichText::new("🔥 EXTREME settings active: 1% background noise volume").color(Color32::RED));
                                    }
                                    
                                    if self.pipeline_verification_mode {
                                        ui.small(RichText::new("🎵 Test tone active: 440Hz tone should be audible").color(Color32::GRAY));
                                    }
                                    
                                    // Additional diagnostic hints based on current state
                                    if self.max_test_mode && self.pipeline_verification_mode {
                                        ui.small(RichText::new("🔧 FULL DIAGNOSTIC MODE: Both extreme noise reduction and test tone active").color(Color32::LIGHT_BLUE));
                                        ui.small(RichText::new("   If you hear neither effect, there's a fundamental setup issue").color(Color32::LIGHT_BLUE));
                                    }
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                    }
                    
                    // Privacy & Analytics Settings
                    ui.heading("Privacy & Analytics");
                    ui.add_space(5.0);
                    
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            // Combined Analytics Option
                            ui.horizontal(|ui| {
                                ui.label("📊 Help us making it better:");
                                if ui.checkbox(&mut self.config.analytics.enabled, "Send anonymous crash/performance logs")
                                    .on_hover_text("Sends performance data weekly and crash logs to help improve the application. Includes IP address for analytics.")
                                    .changed() {
                                    self.config_changed = true;
                                    
                                    // Update usage stats manager and remote logging
                                    if self.config.analytics.enabled {
                                        if self.usage_stats.is_none() {
                                            let mut stats = UsageStatsManager::new(true);
                                            stats.start_session();
                                            self.usage_stats = Some(stats);
                                        }
                                        // Also enable remote logging for crash logs
                                        self.config.remote_logging.enabled = true;
                                    } else {
                                        if let Some(ref mut stats) = self.usage_stats {
                                            stats.set_enabled(false);
                                        }
                                        self.config.remote_logging.enabled = false;
                                    }
                                }
                            });
                            
                            // if self.config.analytics.enabled {
                            //     ui.small(RichText::new("ℹ Performance data sent weekly to www.amazon.com/joker").color(Color32::GRAY));
                            //     ui.small(RichText::new("ℹ Crash logs sent to www.amazon.com/joker").color(Color32::GRAY));
                            // }
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Auto-Update Settings
                    ui.heading("Updates");
                    ui.add_space(5.0);
                    
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("🔄 Auto-Updates:");
                                if ui.checkbox(&mut self.config.auto_update.enabled, "Check for updates automatically")
                                    .on_hover_text("Automatically checks for and notifies about new versions")
                                    .changed() {
                                    self.config_changed = true;
                                    
                                    // Update auto-update manager
                                    if self.config.auto_update.enabled {
                                        if self.auto_update_manager.is_none() {
                                            self.auto_update_manager = Some(AutoUpdateManager::new(self.config.auto_update.clone()));
                                        }
                                    } else {
                                        self.auto_update_manager = None;
                                    }
                                }
                            });
                            
                            // Manual check for updates button
                            ui.horizontal(|ui| {
                                if ui.button("🔍 Check for Updates").on_hover_text("Manually check for available updates").clicked() {
                                    log::info!("Manual update check triggered");
                                    // Note: In a real implementation, this would trigger an async update check
                                    // For now, we just log that the check was requested
                                }
                            });
                            
                            if self.config.auto_update.enabled {
/*                                ui.horizontal(|ui| {
                                    ui.label("Check interval:");
                                    ui.add(Slider::new(&mut self.config.auto_update.check_interval_hours, 1..=168)
                                        .suffix(" hours")
                                        .text("Frequency"));
                                });*/
                            }
                        });
                    });
                    
                    ui.add_space(15.0);
                    
                    // System Information Display (if development mode and debug build)
                    #[cfg(debug_assertions)]
                    if self.config.development_mode {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.heading("🖥 System Information");
                                ui.add_space(5.0);
                                
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.small("Operating System:");
                                        ui.label(format!("{} {}", self.system_info.os_name, self.system_info.os_version));
                                        
                                        ui.small("Architecture:");
                                        ui.label(&self.system_info.architecture);
                                        
                                        ui.small("IP Address:");
                                        ui.label(&self.system_info.ip_address);
                                    });
                                    
                                    ui.separator();
                                    
                                    ui.vertical(|ui| {
                                        ui.small("Memory:");
                                        ui.label(format!("{} MB total", self.system_info.total_memory_mb));
                                        
                                        ui.small("CPU:");
                                        ui.label(format!("{} cores", self.system_info.cpu_cores));
                                    });
                                });
                                
                                ui.add_space(5.0);
                                ui.small(RichText::new("MAC Address is hashed for privacy").italics().color(Color32::GRAY));
                            });
                        });
                        
                        ui.add_space(15.0);
                    }
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save Settings").clicked() {
                            self.save_config();
                            self.show_config_dialog = false;
                        }
                        
                        if ui.button("❌ Cancel").clicked() {
                            // Reload config to undo changes
                            self.config = KwiteConfig::load();
                            self.config_changed = false;
                            self.show_config_dialog = false;
                        }
                        

                    });
                });
            });
        
        if !open {
            self.show_config_dialog = false;
        }
    }
}
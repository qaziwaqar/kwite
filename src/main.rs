//! # Kwite - AI-Powered Real-Time Noise Cancellation
//! 
//! Kwite is a real-time noise cancellation application that uses AI models to remove
//! background noise from audio input. It's designed for use with communication applications
//! like Discord, Teams, or Zoom to provide crystal-clear voice transmission.
//! 
//! ## Application Architecture
//! 
//! The application follows a modular architecture with clear separation of concerns:
//! 
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   GUI Module    │    │  Audio Module   │    │ Config Module   │
//! │                 │    │                 │    │                 │
//! │ • User Interface│◄──►│ • Input Capture │    │ • Settings Save │
//! │ • Device Config │    │ • AI Processing │◄──►│ • Persistence   │
//! │ • Real-time UI  │    │ • Output Stream │    │ • Defaults      │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!           │                       │                       │
//!           └───────────────────────┼───────────────────────┘
//!                                   │
//!                          ┌─────────────────┐
//!                          │ Logger Module   │
//!                          │                 │
//!                          │ • Tracing       │
//!                          │ • Debug Support │
//!                          │ • Performance   │
//!                          └─────────────────┘
//! ```
//! 
//! ## Key Features
//! 
//! - **Real-time Processing**: Sub-20ms latency for natural conversation flow
//! - **AI Noise Cancellation**: Uses RNNoise-based deep learning model
//! - **Device Auto-detection**: Automatically finds and configures audio devices
//! - **Virtual Audio Cable Support**: Seamless integration with communication apps
//! - **Persistent Configuration**: Remembers user preferences between sessions
//! - **Cross-platform**: Supports Windows, macOS, and Linux
//! 
//! ## Technical Stack
//! 
//! - **GUI Framework**: egui (immediate mode GUI)
//! - **Audio Processing**: CPAL (Cross-platform Audio Library)
//! - **AI Model**: nnnoiseless (RNNoise implementation)
//! - **Configuration**: TOML with serde serialization
//! - **Logging**: tracing ecosystem for structured logging

// Module declarations - organize code into logical components
mod logger;     // Centralized logging infrastructure
mod gui;        // User interface and interaction handling  
mod audio;      // Audio capture, processing, and output
mod config;     // Configuration persistence and management
mod ai_metrics; // AI performance metrics and monitoring
mod virtual_audio; // Virtual audio device management and guidance
mod system_info; // System information collection for analytics
mod remote_logging; // Remote logging and analytics
mod usage_stats; // Usage statistics and performance tracking
mod auto_update; // Automatic software updates

mod constants; // Application-wide constants and configuration values

use gui::app::KwiteApp;
use eframe::egui::ViewportBuilder;

/// Application entry point
/// 
/// This function performs the essential startup sequence:
/// 1. Initialize the logging system for debugging and monitoring
/// 2. Configure the native GUI framework with appropriate window settings
/// 3. Launch the main application event loop
/// 
/// ## Window Configuration
/// 
/// The application window is configured with:
/// - **Resizable window**: 480x400 pixels default with 400x350 minimum to ensure all controls are visible
/// - **Descriptive title**: Clearly identifies the application purpose
/// - **Native styling**: Uses OS-appropriate window decorations and behavior
/// 
/// ## Error Handling
/// 
/// Critical startup failures (like logging initialization) will cause the application
/// to exit with an appropriate error message. GUI framework errors are handled by
/// eframe and will display user-friendly error dialogs.
fn main() -> eframe::Result<()> {
    // Initialize the logging system first, before any other operations
    // This ensures we can capture and debug any startup issues
    logger::init_logger().expect("Failed to initialize logger");

    // Configure the native window and application options
    // These settings provide an optimal user experience for the control interface
    let options = eframe::NativeOptions {
        // Configure the application window viewport
        viewport: ViewportBuilder::default()
            .with_inner_size((480.0, 400.0))    // Increased size to prevent UI elements from being hidden
            .with_title("Kwite — AI Noise Cancellation") // Clear, descriptive title
            .with_min_inner_size((400.0, 350.0)),    // Minimum size to ensure all controls are visible
        
        // Use default values for all other native options
        // This includes vsync, multisampling, and platform-specific settings
        ..Default::default()
    };

    // Launch the main application event loop
    // eframe handles the platform-specific window creation and event processing
    // The closure creates our main application instance when the GUI is ready
    eframe::run_native(
        "Kwite — AI Noise Cancellation", // Application identifier for the OS
        options,                         // Window and rendering configuration  
        Box::new(|cc| Ok(Box::new(KwiteApp::new(cc)))), // Application factory function
    )
}
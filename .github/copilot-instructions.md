# Project Overview
This is kwite, a professional AI-powered real-time noise cancellation software built in Rust. The application provides advanced audio processing capabilities with both real-time noise reduction and optional AI-enhanced features for professional audio environments.
## Folder Structure
kwite/
├── src/                    # Main source code
│   ├── audio/             # Audio processing modules
│   │   ├── capture.rs     # Audio input handling
│   │   ├── output.rs      # Audio output handling
│   │   ├── pipeline.rs    # Audio processing pipeline
│   │   ├── process.rs     # Core audio processing logic
│   │   ├── devices.rs     # Audio device management
│   │   ├── analysis.rs    # Audio analysis utilities
│   │   └── models.rs      # Audio processing models
│   ├── gui/               # User interface components
│   │   ├── app.rs         # Main application GUI
│   │   └── mod.rs         # GUI module exports
│   ├── ai_metrics.rs      # AI performance metrics
│   ├── auto_update.rs     # Automatic update functionality
│   ├── config.rs          # Configuration management
│   ├── logger.rs          # Logging system
│   ├── remote_logging.rs  # Remote telemetry logging
│   ├── system_info.rs     # System information gathering
│   ├── usage_stats.rs     # Usage analytics
│   └── virtual_audio.rs   # Virtual audio device handling
├── tests/                 # Test suite
├── benches/              # Performance benchmarks
├── assets/               # Application assets
├── scripts/              # Build and deployment scripts
└── installer/            # Installation packages
## Libraries and Frameworks
### Core Audio Processing
    cpal: Cross-platform audio I/O for real-time audio capture and playback
    nnnoiseless: RNNoise implementation for core noise cancellation
    rustfft: Fast Fourier Transform for frequency domain analysis
    webrtc-vad: Voice Activity Detection for enhanced processing
### GUI Framework
    egui: Immediate mode GUI framework
    eframe: Application framework for egui applications
### System Integration
    windows: Windows API bindings for Media and Audio APIs
    crossbeam-channel: Thread-safe communication channels
### Networking & Telemetry
    reqwest: HTTP client for remote logging (optional)
    tokio: Async runtime for network operations (optional)
### Development & Testing
    tracing: Structured logging with environment filtering
    criterion: Performance benchmarking framework
    mockall: Mock object generation for testing

## Coding Standards
### Rust-Specific Standards
    Use Rust 2021 edition features and idioms
    Follow standard Rust naming conventions (snake_case for functions/variables, PascalCase for types)
    Implement comprehensive error handling using Result<T, E> types
    Use #[cfg(feature = "...")] attributes for conditional compilation
    Prefer Vec<T> and slice operations for audio buffer management

### Code Organization
    Separate modules by functionality (audio/, gui/, etc.)
    Use mod.rs files for module organization and exports
    Keep audio processing logic separate from GUI components
    Implement both library (lib.rs) and binary (main.rs) targets
### Performance Standards
    Minimize allocations in real-time audio processing paths
    Use appropriate buffer sizes for low-latency audio processing
    Implement benchmarks for performance-critical audio algorithms
    Profile memory usage and CPU utilization regularly4
## UI guidelines
The user interface prioritizes professional audio software aesthetics with a clean, functional design that emphasizes real-time feedback through audio level meters, device status indicators, and noise reduction controls that update continuously without interfering with audio processing performance. The interface includes device selection controls, visual feedback for noise reduction strength, conditional AI enhancement displays when features are enabled, and maintains accessibility through clear visual indicators, keyboard shortcuts for critical functions, responsive layouts, and actionable error messaging that helps users resolve audio device issues quickly during live sessions.
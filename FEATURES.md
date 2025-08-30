# Kwite Feature Configuration Guide

This document explains how to enable and configure different features in Kwite, including advanced audio processing capabilities.

## Feature Overview

Kwite uses Cargo features to provide flexible builds that can be customized for different use cases and system requirements.

### Available Features

| Feature | Description | Dependencies | Impact |
|---------|-------------|--------------|--------|
| `ai-enhanced` | Enables advanced audio analysis and processing features | `webrtc-vad`, `rustfft`, `spectrum-analyzer`, `apodize` | +~10MB binary size, +15% CPU usage |
| `remote-logging` | Enables optional telemetry and remote diagnostics | `reqwest`, `tokio`, `futures-util` | Network capabilities, privacy controls |

### Default Configuration

By default, Kwite enables both `ai-enhanced` and `remote-logging` features:

```toml
[features]
default = ["ai-enhanced", "remote-logging"]
```

## Enabling AI-Enhanced Features

The `ai-enhanced` feature provides advanced audio processing capabilities beyond the core RNNoise model:

### Option 1: Default Build (Recommended)

```bash
# Clone and build with all features enabled
git clone https://github.com/qaziwaqar/kwite.git
cd kwite
cargo build --release
```

**Result**: AI-enhanced features are available alongside RNNoise

### Option 2: Explicit Feature Selection

```bash
# Build with only AI-enhanced features (no remote logging)
cargo build --release --no-default-features --features ai-enhanced

# Build with specific features
cargo build --release --features "ai-enhanced,remote-logging"
```

### Option 3: Development Build

```bash
# Development build with AI-enhanced features for testing
cargo build --features ai-enhanced
cargo run --features ai-enhanced
```

## Build Configurations

### Full Build (Default)
```bash
cargo build --release
```
- **Includes**: RNNoise, AI-Enhanced Processing, Voice Activity Detection, Remote Logging
- **Binary Size**: ~20MB
- **Memory Usage**: ~60MB
- **Best For**: Full-featured installation

### AI-Enhanced Only
```bash
cargo build --release --no-default-features --features ai-enhanced
```
- **Includes**: RNNoise, Advanced Audio Analysis, Voice Activity Detection
- **Excludes**: Remote logging and telemetry
- **Binary Size**: ~15MB
- **Memory Usage**: ~50MB
- **Best For**: Privacy-focused users who want advanced audio processing

### Minimal Build
### Minimal Build
```bash
cargo build --release --no-default-features
```
- **Includes**: RNNoise only
- **Binary Size**: ~8MB
- **Memory Usage**: ~30MB
- **Best For**: Embedded systems, minimal installations

### Remote Logging Only
```bash
cargo build --release --no-default-features --features remote-logging
```
- **Includes**: RNNoise, Remote diagnostics
- **Binary Size**: ~12MB
- **Best For**: Standard installation with diagnostics support

## Runtime Feature Detection

Your application can detect which features are available at runtime:

```rust
use kwite::audio::models::{NoiseModel, UseCase};

fn main() {
    // Check available models
    let models = NoiseModel::available_models();
    println!("Available models: {:?}", models);
    
    // Check if AI-enhanced features are available
    let has_ai_enhanced = models.len() > 1; // More than just RNNoise
    
    if has_ai_enhanced {
        println!("✅ AI-enhanced features are available!");
        
        // Get recommended model for professional meetings
        let recommended = NoiseModel::recommended_for_use_case(UseCase::ProfessionalMeetings);
        println!("Recommended for meetings: {}", recommended);
    } else {
        println!("ℹ️  Only RNNoise is available. Build with --features ai-enhanced to enable advanced processing.");
    }
}
```

## Feature Compilation

### Conditional Compilation

The codebase uses conditional compilation to include features only when enabled:

```rust
// AI-enhanced features are only compiled with ai-enhanced feature
#[cfg(feature = "ai-enhanced")]
use webrtc_vad::VadOptions;

#[cfg(feature = "ai-enhanced")]
use rustfft::FftPlanner;

// Model enum includes all available processing modes
#[derive(Debug, Clone, Copy)]
pub enum NoiseModel {
    RNNoise,
    Auto, // Available when ai-enhanced is enabled
}
```

### Feature Gates in Tests

Tests also respect feature gates:

```bash
# Run all tests with AI-enhanced features
cargo test --features ai-enhanced

# Run tests for specific module
cargo test --features ai-enhanced audio::models

# Run only basic tests (no AI-enhanced features)
cargo test --no-default-features
```

## Performance Impact

### CPU Usage by Configuration

| Configuration | Idle CPU | Active Processing | Memory |
|---------------|----------|-------------------|---------|
| Minimal (RNNoise only) | 1-2% | 8-12% | 30MB |
| AI-Enhanced (RNNoise + Advanced Processing) | 2-3% | 12-18% | 50MB |
| Full (All features) | 2-4% | 12-18% | 60MB |

### Model Performance Comparison

| Model | CPU Usage | Quality | Latency | Best For |
|-------|-----------|---------|---------|----------|
| RNNoise | ⭐⭐ Low | ⭐⭐⭐⭐ High | <5ms | General use, battery devices |
| Auto | ⭐⭐⭐ Medium | ⭐⭐⭐⭐⭐ Very High | <8ms | Professional, adaptive processing |

## Deployment Recommendations

### Desktop Applications
```bash
# Full build recommended for desktop users
cargo build --release
```

### Server/Headless Deployment  
```bash
# AI-enhanced without remote logging for privacy
cargo build --release --no-default-features --features ai-enhanced
```

### Embedded/IoT Devices
```bash
# Minimal build for resource-constrained environments
cargo build --release --no-default-features
```

### Development/Testing
```bash
# Development with all features for testing
cargo build --features "ai-enhanced,remote-logging"
```

## Troubleshooting Feature Issues

### Feature Not Working

1. **Verify feature was enabled during build**:
   ```bash
   cargo metadata --format-version 1 | grep -A 10 '"name":"kwite"'
   ```

2. **Check compilation logs**:
   ```bash
   cargo build --features ai-enhanced -v 2>&1 | grep -i "deep_filter\|ai-enhanced"
   ```

3. **Test feature availability**:
   ```bash
   cargo test test_model_availability --features ai-enhanced
   ```

### Build Failures

1. **Missing system dependencies** (Linux):
   ```bash
   # Install ALSA development headers
   sudo apt-get install libasound2-dev  # Ubuntu/Debian
   sudo dnf install alsa-lib-devel      # Fedora
   ```

2. **Feature dependency conflicts**:
   ```bash
   # Clean build directory
   cargo clean
   cargo build --release --features ai-enhanced
   ```

### Runtime Issues

1. **AI-enhanced features not appearing in UI**:
   - Verify build included `ai-enhanced` feature
   - Check application logs: `RUST_LOG=kwite::audio::models=debug cargo run`

2. **High memory usage**:
   - AI-enhanced features use more memory for advanced processing
   - Consider building without `ai-enhanced` for memory-constrained systems

3. **Audio processing issues**:
   - Ensure your system supports the required sample rates
   - Check that audio drivers are up to date
   - Monitor CPU usage during processing

## Future Features

The feature system is designed to be extensible. Planned features include:

- `gpu-acceleration`: CUDA/OpenCL support for AI models
- `custom-models`: Support for user-provided AI models  
- `advanced-ui`: Additional GUI features and visualizations
- `cloud-sync`: Configuration synchronization across devices

To stay updated on new features, watch the repository or check the roadmap in README.md.
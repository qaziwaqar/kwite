# Building Kwite Installers

This guide explains how to build platform-specific installers for Kwite.

## Quick Start

### All Platforms (using build script)
```bash
# Make the build script executable
chmod +x installer/build-installers.sh

# Run the automated build script
./installer/build-installers.sh
```

### Manual Building

#### Linux (DEB Package)
```bash
# Install cargo-deb
cargo install cargo-deb

# Build the application
cargo build --release

# Create DEB package
cargo deb
```

The DEB package will be created in `target/debian/kwite_0.1.0-1_amd64.deb`

#### Linux (AppImage)
```bash
# Install cargo-bundle and appimagetool
cargo install cargo-bundle
# Download appimagetool from https://appimage.github.io/

# Build and create AppImage
./installer/build-installers.sh
```

#### macOS (DMG)
```bash
# Install cargo-bundle (on macOS)
cargo install cargo-bundle

# Build app bundle
cargo bundle --release

# Create DMG (requires macOS)
./installer/build-installers.sh
```

#### Windows (MSI)
```bash
# Install cargo-wix and WiX Toolset
cargo install cargo-wix

# Build MSI installer
cargo wix --package kwite
```

## Package Contents

Each installer includes:
- Kwite binary with AI-enhanced noise cancellation (RNNoise + advanced processing)
- Desktop integration files
- Application icon
- System requirements and dependencies

## Build Features

### Default Build (Recommended)
```bash
cargo build --release
# Includes: RNNoise + AI-Enhanced Processing + Remote Logging
```

### AI-Enhanced Only (Privacy-focused)
```bash
cargo build --release --no-default-features --features ai-enhanced
# Includes: RNNoise + Advanced Audio Analysis (no telemetry)
```

### Minimal Build (Resource-constrained)
```bash
cargo build --release --no-default-features
# Includes: RNNoise only
```

## System Requirements

### Linux
- Ubuntu 18.04+ / Debian 10+ / Fedora 30+
- ALSA audio system
- libasound2-dev (for development builds)

### macOS
- macOS 10.14 (Mojave) or later
- CoreAudio framework

### Windows
- Windows 10 version 1903 or later
- WASAPI audio support

## Troubleshooting

### Linux: Missing ALSA
```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev pkg-config

# Fedora/CentOS
sudo dnf install alsa-lib-devel pkgconfig
```

### macOS: Code Signing
For distribution outside the App Store, you'll need to sign the app:
```bash
codesign --deep --force --verify --verbose --sign "Developer ID Application: Your Name" target/release/bundle/osx/Kwite.app
```

### Windows: MSI Building
Requires WiX Toolset v3.11+:
1. Download from https://wixtoolset.org/
2. Install Visual Studio Build Tools
3. Add WiX to PATH
# Kwite Installer Scripts

This directory contains platform-specific installer configuration and build scripts.

## Directory Structure

- `windows/` - Windows MSI installer configuration using WiX
- `macos/` - macOS app bundle and DMG creation scripts  
- `linux/` - Linux package configuration (DEB/RPM)

## Build Instructions

### Windows MSI Installer
```bash
# Install cargo-wix
cargo install cargo-wix

# Build the installer
cargo wix --package kwite
```

### macOS DMG
```bash
# Install cargo-bundle
cargo install cargo-bundle

# Build app bundle and DMG
cargo bundle --release --target x86_64-apple-darwin
```

### Linux Packages
```bash
# Install cargo-deb for Debian packages
cargo install cargo-deb

# Build DEB package
cargo deb

# Install cargo-rpm for Red Hat packages  
cargo install cargo-rpm

# Build RPM package
cargo rpm build
```

## Prerequisites

### Windows
- WiX Toolset v3.11 or later
- Visual Studio Build Tools

### macOS  
- Xcode Command Line Tools
- macOS 10.14+ for target compatibility

### Linux
- Standard development tools (gcc, make)
- Platform-specific package dependencies
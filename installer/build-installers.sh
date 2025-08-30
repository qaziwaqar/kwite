#!/bin/bash

# Kwite Cross-Platform Installer Build Script
# This script creates platform-specific installers for Kwite

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "kwite") | .version')

echo "Building Kwite v$VERSION installers..."

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install packaging tools if not present
install_packaging_tools() {
    echo "Installing packaging tools..."
    
    # Install cargo-bundle for macOS DMG creation
    if ! command_exists cargo-bundle; then
        echo "Installing cargo-bundle..."
        cargo install cargo-bundle
    fi
    
    # Install cargo-deb for Debian packages
    if ! command_exists cargo-deb; then
        echo "Installing cargo-deb..."
        cargo install cargo-deb
    fi
    
    # Install cargo-wix for Windows MSI (on Windows systems)
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        if ! command_exists cargo-wix; then
            echo "Installing cargo-wix..."
            cargo install cargo-wix
        fi
    fi
}

# Build the application
build_application() {
    echo "Building Kwite application..."
    cd "$PROJECT_ROOT"
    
    # Build with all features for full functionality
    cargo build --release --features "ai-enhanced,remote-logging"
    
    echo "Build completed successfully."
}

# Create Linux packages
build_linux_packages() {
    echo "Creating Linux packages..."
    cd "$PROJECT_ROOT"
    
    # Create DEB package
    if command_exists cargo-deb; then
        echo "Creating .deb package..."
        cargo deb --variant kwite
        echo "DEB package created in target/debian/"
    else
        echo "cargo-deb not available, skipping DEB package creation"
    fi
    
    # Create AppImage (if appimagetool is available)
    if command_exists appimagetool; then
        echo "Creating AppImage..."
        mkdir -p target/appimage/kwite.AppDir/usr/bin
        mkdir -p target/appimage/kwite.AppDir/usr/share/applications  
        mkdir -p target/appimage/kwite.AppDir/usr/share/pixmaps
        
        # Copy binary
        cp target/release/kwite target/appimage/kwite.AppDir/usr/bin/
        
        # Copy desktop file
        cp assets/kwite.desktop target/appimage/kwite.AppDir/
        cp assets/kwite.desktop target/appimage/kwite.AppDir/usr/share/applications/
        
        # Copy icon (create a simple placeholder if none exists)
        if [ ! -f assets/icon.png ]; then
            # Create a simple 64x64 PNG icon placeholder
            echo "Creating placeholder icon..."
            convert -size 64x64 xc:blue -pointsize 12 -fill white -gravity center -annotate 0 "KWITE" assets/icon.png 2>/dev/null || {
                echo "ImageMagick not available, copying desktop file as icon placeholder"
                touch assets/icon.png
            }
        fi
        cp assets/icon.png target/appimage/kwite.AppDir/
        cp assets/icon.png target/appimage/kwite.AppDir/usr/share/pixmaps/kwite.png
        
        # Create AppRun
        cat > target/appimage/kwite.AppDir/AppRun << 'EOF'
#!/bin/bash
APPDIR="$(dirname "$(readlink -f "$0")")"
exec "$APPDIR/usr/bin/kwite" "$@"
EOF
        chmod +x target/appimage/kwite.AppDir/AppRun
        
        # Create AppImage
        cd target/appimage
        appimagetool kwite.AppDir kwite-$VERSION-x86_64.AppImage
        cd "$PROJECT_ROOT"
        
        echo "AppImage created: target/appimage/kwite-$VERSION-x86_64.AppImage"
    else
        echo "appimagetool not available, skipping AppImage creation"
    fi
}

# Create macOS packages
build_macos_packages() {
    echo "Creating macOS packages..."
    cd "$PROJECT_ROOT"
    
    if command_exists cargo-bundle && [[ "$OSTYPE" == "darwin"* ]]; then
        echo "Creating .app bundle..."
        cargo bundle --release
        
        echo "Creating DMG..."
        # Create DMG from app bundle
        APP_NAME="target/release/bundle/osx/Kwite.app"
        DMG_NAME="target/release/Kwite-$VERSION.dmg"
        
        if [ -d "$APP_NAME" ]; then
            # Create temporary directory for DMG contents
            mkdir -p target/dmg
            cp -R "$APP_NAME" target/dmg/
            ln -sf /Applications target/dmg/Applications
            
            # Create DMG
            hdiutil create -volname "Kwite $VERSION" -srcfolder target/dmg -ov -format UDZO "$DMG_NAME"
            
            echo "DMG created: $DMG_NAME"
            
            # Cleanup
            rm -rf target/dmg
        else
            echo "App bundle not found, cannot create DMG"
        fi
    else
        echo "cargo-bundle not available or not on macOS, skipping macOS package creation"
    fi
}

# Create Windows packages  
build_windows_packages() {
    echo "Creating Windows packages..."
    cd "$PROJECT_ROOT"
    
    if command_exists cargo-wix && ([[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]); then
        echo "Creating MSI installer..."
        cargo wix --package kwite
        echo "MSI installer created in target/wix/"
    else
        echo "cargo-wix not available or not on Windows, skipping MSI creation"
    fi
}

# Main build process
main() {
    echo "Starting Kwite installer build process..."
    
    # Install required tools
    install_packaging_tools
    
    # Build the application
    build_application
    
    # Detect platform and build appropriate packages
    case "$OSTYPE" in
        linux-gnu*)
            build_linux_packages
            ;;
        darwin*)
            build_macos_packages
            ;;
        msys*|win32*)
            build_windows_packages
            ;;
        *)
            echo "Unknown platform: $OSTYPE"
            echo "Building for Linux by default..."
            build_linux_packages
            ;;
    esac
    
    echo ""
    echo "Installer build completed!"
    echo "Check the following directories for packages:"
    echo "  - target/debian/ (Linux DEB packages)"
    echo "  - target/appimage/ (Linux AppImage)"  
    echo "  - target/release/bundle/osx/ (macOS app bundle)"
    echo "  - target/release/ (macOS DMG)"
    echo "  - target/wix/ (Windows MSI)"
    echo ""
    echo "Built packages:"
    find target -name "*.deb" -o -name "*.AppImage" -o -name "*.dmg" -o -name "*.msi" 2>/dev/null | sed 's/^/  /'
}

# Run main function
main "$@"
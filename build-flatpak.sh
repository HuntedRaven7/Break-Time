#!/bin/bash

# Exit on error
set -e

APP_ID="io.github.HuntedRaven7.BreakTime"
MANIFEST="${APP_ID}.yml"
BUILD_DIR="build-dir"
REPO_DIR="repo"

echo "🚀 Starting Flatpak build process for ${APP_ID}..."

# 1. Check for required tools
if ! command -v flatpak &> /dev/null; then
    echo "❌ Error: flatpak is not installed."
    exit 1
fi

if ! command -v flatpak-builder &> /dev/null; then
    echo "❌ Error: flatpak-builder is not installed."
    exit 1
fi

if ! command -v appstreamcli &> /dev/null; then
    echo "⚠️ Warning: appstreamcli is not installed. Flatpak validation might fail."
    echo "💡 Please install it using your package manager:"
    echo "   - Ubuntu/Debian: sudo apt install appstream librsvg2-bin libgdk-pixbuf2.0-bin"
    echo "   - Fedora: sudo dnf install appstream librsvg2 libgdk-pixbuf2"
    echo "   - Arch Linux: sudo pacman -S appstream librsvg gdk-pixbuf2"
    echo ""
fi

# 2. Set up Flathub remote if missing
echo "📦 Checking for Flathub remote..."
flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# 3. Install/Update SDKs and Platform
echo "📥 Installing/Updating required SDKs and Platform (this may take a while)..."
flatpak install --user -y flathub org.gnome.Sdk//50 org.gnome.Platform//50
flatpak install --user -y flathub org.freedesktop.Sdk.Extension.rust-stable//25.08

# 4. Build the Flatpak
echo "🛠 Building the Flatpak (using PNG icon for host compatibility)..."
flatpak-builder --user --install --force-clean --ccache \
    --install-deps-from=flathub \
    ${BUILD_DIR} ${MANIFEST}

echo "✅ Build complete!"
echo "🎉 You can now run the app with: flatpak run ${APP_ID}"

#!/bin/bash

# Exit on error
set -e

APP_ID="io.github.HuntedRaven7.BreakTime"
MANIFEST="${APP_ID}.yml"
BUILD_DIR="build-dir"
REPO_DIR="repo"
BUNDLE_FILE="${APP_ID}.flatpak"

echo "🚀 Starting Flatpak bundle creation for ${APP_ID}..."

# 1. Check for required tools
if ! command -v flatpak &> /dev/null; then
    echo "❌ Error: flatpak is not installed."
    exit 1
fi

if ! command -v flatpak-builder &> /dev/null; then
    echo "❌ Error: flatpak-builder is not installed."
    exit 1
fi

# 2. Set up Flathub remote if missing
echo "📦 Checking for Flathub remote..."
flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# 3. Build the Flatpak into a local repository
echo "🛠 Building the Flatpak into local repo '${REPO_DIR}'..."
flatpak-builder --user --force-clean --ccache --repo=${REPO_DIR} \
    --install-deps-from=flathub \
    ${BUILD_DIR} ${MANIFEST}

# 4. Create the bundle file
echo "📦 Exporting bundle to ${BUNDLE_FILE}..."
flatpak build-bundle ${REPO_DIR} ${BUNDLE_FILE} ${APP_ID}

echo "✅ Success! Bundle created: ${BUNDLE_FILE}"
echo ""
echo "🎉 You can distribute this file. To install it, users can run:"
echo "   flatpak install ./${BUNDLE_FILE}"
echo ""
echo "💡 Note: You can also run it without installing for testing:"
echo "   flatpak run ${APP_ID}"

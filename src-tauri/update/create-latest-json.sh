#!/bin/bash

set -e

# =====================================
# Config
# =====================================

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# src-tauri/target/release/bundle/macos
BUNDLE="$SCRIPT_DIR/../target/release/bundle/macos"

TAURI_CONFIG="$SCRIPT_DIR/../tauri.conf.json"

GITHUB_USER="minhdc-react-native"
REPO="vaOne-update"


echo "Scanning macOS updater artifact..."


# =====================================
# Find artifact
# =====================================

PACKAGE=$(find "$BUNDLE" -name "*.app.tar.gz" | head -n 1)

if [ -z "$PACKAGE" ]; then
    echo "ERROR: app.tar.gz not found"
    exit 1
fi


SIG_FILE="${PACKAGE}.sig"

if [ ! -f "$SIG_FILE" ]; then
    echo "ERROR: signature file not found:"
    echo "$SIG_FILE"
    exit 1
fi


FILE_NAME=$(basename "$PACKAGE")


# =====================================
# Read version from tauri.conf.json
# =====================================

VERSION=$(grep '"version"' "$TAURI_CONFIG" \
    | head -1 \
    | sed -E 's/.*"version": "([^"]+)".*/\1/')


if [ -z "$VERSION" ]; then
    echo "ERROR: Cannot read version"
    exit 1
fi


echo "Version : $VERSION"
echo "Package : $FILE_NAME"


# =====================================
# Read signature
# =====================================

SIGNATURE=$(cat "$SIG_FILE" | tr -d '\n')


# =====================================
# Detect architecture
# =====================================

ARCH="darwin-aarch64"

if [[ "$FILE_NAME" == *"x86_64"* ]]; then
    ARCH="darwin-x86_64"
fi


URL="https://github.com/${GITHUB_USER}/${REPO}/releases/download/v${VERSION}/${FILE_NAME}"


# =====================================
# Create latest-mac.json
# =====================================

cat > "$BUNDLE/latest-mac.json" <<EOF
{
    "version": "$VERSION",
    "notes": "Update vaOne plugin",
    "pub_date": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "platforms": {
        "$ARCH": {
            "signature": "$SIGNATURE",
            "url": "$URL"
        }
    }
}
EOF


echo ""
echo "================================"
echo "latest-mac.json created"
echo "Version : $VERSION"
echo "Platform: $ARCH"
echo "Output  : $BUNDLE/latest-mac.json"
echo "================================"
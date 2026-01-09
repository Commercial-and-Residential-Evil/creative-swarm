#!/bin/bash
# Sign APK and AAB for Google Play Store release
# Requires: Android SDK build-tools (zipalign, apksigner), JDK (jarsigner)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEY_PROPERTIES="$SCRIPT_DIR/android/key.properties"

# APK paths
UNSIGNED_APK="$SCRIPT_DIR/android/app/build/outputs/apk/release/app-release-unsigned.apk"
ALIGNED_APK="$SCRIPT_DIR/android/app/build/outputs/apk/release/app-release-aligned.apk"
SIGNED_APK="$SCRIPT_DIR/android/app/build/outputs/apk/release/app-release-signed.apk"

# AAB paths
UNSIGNED_AAB="$SCRIPT_DIR/android/app/build/outputs/bundle/release/app-release.aab"
SIGNED_AAB="$SCRIPT_DIR/android/app/build/outputs/bundle/release/app-release-signed.aab"

echo -e "${GREEN}=== Signing for Google Play Store ===${NC}"

# Check if key.properties exists
if [ ! -f "$KEY_PROPERTIES" ]; then
    echo -e "${RED}Error: key.properties not found at $KEY_PROPERTIES${NC}"
    echo "Create android/key.properties with your signing credentials."
    exit 1
fi

# Read key.properties
echo -e "${YELLOW}Reading signing configuration...${NC}"
STORE_PASSWORD=$(grep "storePassword" "$KEY_PROPERTIES" | cut -d'=' -f2)
KEY_PASSWORD=$(grep "keyPassword" "$KEY_PROPERTIES" | cut -d'=' -f2)
KEY_ALIAS=$(grep "keyAlias" "$KEY_PROPERTIES" | cut -d'=' -f2)
STORE_FILE=$(grep "storeFile" "$KEY_PROPERTIES" | cut -d'=' -f2)

# Expand ~ in path if present
STORE_FILE="${STORE_FILE/#\~/$HOME}"

# Validate keystore exists
if [ ! -f "$STORE_FILE" ]; then
    echo -e "${RED}Error: Keystore not found at $STORE_FILE${NC}"
    exit 1
fi

echo "  Keystore: $STORE_FILE"
echo "  Key alias: $KEY_ALIAS"

# Find Android build tools
if [ -z "$ANDROID_HOME" ]; then
    ANDROID_HOME="$HOME/Android/Sdk"
fi

BUILD_TOOLS_DIR=$(ls -d "$ANDROID_HOME/build-tools/"* 2>/dev/null | sort -V | tail -1)
if [ -z "$BUILD_TOOLS_DIR" ]; then
    echo -e "${RED}Error: Android build-tools not found in $ANDROID_HOME/build-tools/${NC}"
    exit 1
fi

ZIPALIGN="$BUILD_TOOLS_DIR/zipalign"
APKSIGNER="$BUILD_TOOLS_DIR/apksigner"

echo "  Build tools: $BUILD_TOOLS_DIR"

# ============================================================================
# Sign AAB (for Google Play Store)
# ============================================================================
if [ -f "$UNSIGNED_AAB" ]; then
    echo ""
    echo -e "${YELLOW}Signing AAB for Google Play Store...${NC}"

    # Copy to signed location first
    cp "$UNSIGNED_AAB" "$SIGNED_AAB"

    # Sign with jarsigner (AAB uses JAR signing, not apksigner)
    jarsigner -verbose \
        -sigalg SHA256withRSA \
        -digestalg SHA-256 \
        -keystore "$STORE_FILE" \
        -storepass "$STORE_PASSWORD" \
        -keypass "$KEY_PASSWORD" \
        -storetype PKCS12 \
        "$SIGNED_AAB" \
        "$KEY_ALIAS"

    # Verify signature
    echo -e "${YELLOW}Verifying AAB signature...${NC}"
    jarsigner -verify -verbose -certs "$SIGNED_AAB" | head -5

    echo -e "${GREEN}AAB signed successfully!${NC}"
    echo -e "  Output: ${YELLOW}$SIGNED_AAB${NC}"
else
    echo -e "${YELLOW}No AAB found at $UNSIGNED_AAB (skipping)${NC}"
fi

# ============================================================================
# Sign APK (for direct installation/testing)
# ============================================================================
if [ -f "$UNSIGNED_APK" ]; then
    echo ""
    echo -e "${YELLOW}Signing APK for direct installation...${NC}"

    # Zipalign
    rm -f "$ALIGNED_APK"
    "$ZIPALIGN" -v -p 4 "$UNSIGNED_APK" "$ALIGNED_APK"

    # Sign with apksigner
    rm -f "$SIGNED_APK"
    "$APKSIGNER" sign \
        --ks "$STORE_FILE" \
        --ks-key-alias "$KEY_ALIAS" \
        --ks-pass "pass:$STORE_PASSWORD" \
        --key-pass "pass:$KEY_PASSWORD" \
        --out "$SIGNED_APK" \
        "$ALIGNED_APK"

    # Verify
    echo -e "${YELLOW}Verifying APK signature...${NC}"
    "$APKSIGNER" verify --verbose "$SIGNED_APK" | head -5

    # Cleanup
    rm -f "$ALIGNED_APK"

    echo -e "${GREEN}APK signed successfully!${NC}"
    echo -e "  Output: ${YELLOW}$SIGNED_APK${NC}"
else
    echo -e "${YELLOW}No APK found at $UNSIGNED_APK (skipping)${NC}"
fi

# ============================================================================
# Summary
# ============================================================================
echo ""
echo -e "${GREEN}=== Signing Complete! ===${NC}"
echo ""

if [ -f "$SIGNED_AAB" ]; then
    echo -e "For Google Play Store upload:"
    echo -e "  ${YELLOW}$SIGNED_AAB${NC}"
    echo ""
fi

if [ -f "$SIGNED_APK" ]; then
    echo -e "For direct installation (testing):"
    echo -e "  ${YELLOW}$SIGNED_APK${NC}"
    echo -e "  Install: adb install $SIGNED_APK"
fi

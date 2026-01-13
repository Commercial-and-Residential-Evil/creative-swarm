#!/bin/bash
# Run Whirled Peas on a connected Android device and stream app logs
#
# Usage:
#   ./runlocaldevice.sh          # Install and run, show logs
#   ./runlocaldevice.sh --build  # Build first, then install and run
#   ./runlocaldevice.sh --logs   # Just show logs (app already running)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

PACKAGE="commercial_and_residential_evil.whirled_peas"
ACTIVITY="commercial_and_residential_evil.whirled_peas.MainActivity"
DEBUG_KEYSTORE="$HOME/.android/debug.keystore"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Parse arguments
BUILD=false
LOGS_ONLY=false
DEV_MODE=false
for arg in "$@"; do
    case $arg in
        --build|-b)
            BUILD=true
            ;;
        --logs|-l)
            LOGS_ONLY=true
            ;;
        --dev|-d)
            DEV_MODE=true
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --build, -b    Build the APK before installing"
            echo "  --dev, -d      Use debug APK (faster builds, ARM64 only)"
            echo "  --logs, -l     Just show logs (don't install/launch)"
            echo "  --help, -h     Show this help"
            exit 0
            ;;
    esac
done

# Set APK path based on mode
if [ "$DEV_MODE" = true ]; then
    APK_PATH="android/app/build/outputs/apk/debug/app-debug.apk"
else
    APK_PATH="android/app/build/outputs/apk/release/app-release-unsigned.apk"
fi

# Check for adb
if ! command -v adb &> /dev/null; then
    echo -e "${RED}Error: adb not found. Please install Android SDK platform-tools.${NC}"
    exit 1
fi

# Check device connection
if ! adb get-state &> /dev/null; then
    echo -e "${RED}Error: No Android device connected.${NC}"
    echo "Connect a device via USB and enable USB debugging."
    exit 1
fi

DEVICE=$(adb devices | grep -v "List" | grep "device$" | head -1 | cut -f1)
echo -e "${GREEN}Device connected:${NC} $DEVICE"

# Logs only mode
if [ "$LOGS_ONLY" = true ]; then
    echo -e "${CYAN}Showing logs for $PACKAGE...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""

    PID=$(adb shell pidof "$PACKAGE" 2>/dev/null || echo "")
    if [ -n "$PID" ]; then
        adb logcat --pid="$PID" -v color 2>/dev/null || adb logcat --pid="$PID"
    else
        echo -e "${YELLOW}App not running. Showing filtered logs...${NC}"
        adb logcat -v color "*:S" "RustStdoutStderr:V" "bevy:V" "wgpu:V" "whirled_peas:V" 2>/dev/null || \
        adb logcat "*:S" "RustStdoutStderr:V" "bevy:V" "wgpu:V" "whirled_peas:V"
    fi
    exit 0
fi

# Build if requested
if [ "$BUILD" = true ]; then
    echo -e "${YELLOW}Building APK...${NC}"
    if [ "$DEV_MODE" = true ]; then
        ./build-android.sh --dev
    else
        ./build-android.sh
    fi
fi

# Check APK exists
if [ ! -f "$APK_PATH" ]; then
    echo -e "${RED}Error: APK not found at $APK_PATH${NC}"
    echo "Run with --build flag or run ./build-android.sh first"
    exit 1
fi

# Debug APKs from Gradle are already signed, release APKs need signing
if [ "$DEV_MODE" = true ]; then
    # Debug APK is already signed by Gradle
    SIGNED_APK="$APK_PATH"
else
    # Sign release APK with debug key
    SIGNED_APK="${APK_PATH%.apk}-debug-signed.apk"
    if [ ! -f "$DEBUG_KEYSTORE" ]; then
        echo -e "${YELLOW}Creating debug keystore...${NC}"
        mkdir -p "$(dirname "$DEBUG_KEYSTORE")"
        keytool -genkey -v -keystore "$DEBUG_KEYSTORE" \
            -storepass android -alias androiddebugkey -keypass android \
            -keyalg RSA -keysize 2048 -validity 10000 \
            -dname "CN=Android Debug,O=Android,C=US"
    fi

    # Copy and sign
    echo -e "${YELLOW}Signing APK with debug key...${NC}"
    cp "$APK_PATH" "$SIGNED_APK"

    # Use apksigner if available, otherwise jarsigner
    if command -v apksigner &> /dev/null; then
        apksigner sign --ks "$DEBUG_KEYSTORE" --ks-pass pass:android "$SIGNED_APK"
    elif command -v jarsigner &> /dev/null; then
        jarsigner -keystore "$DEBUG_KEYSTORE" -storepass android "$SIGNED_APK" androiddebugkey
        # Zipalign if available
        if command -v zipalign &> /dev/null; then
            zipalign -f 4 "$SIGNED_APK" "${SIGNED_APK%.apk}-aligned.apk"
            mv "${SIGNED_APK%.apk}-aligned.apk" "$SIGNED_APK"
        fi
    else
        echo -e "${RED}Error: Neither apksigner nor jarsigner found.${NC}"
        exit 1
    fi
fi

# Uninstall old version (ignore errors)
echo -e "${YELLOW}Uninstalling previous version...${NC}"
adb uninstall "$PACKAGE" 2>/dev/null || true

# Install
echo -e "${YELLOW}Installing APK...${NC}"
adb install "$SIGNED_APK"

# Clear old logs
adb logcat -c

# Launch app
echo -e "${GREEN}Launching $PACKAGE...${NC}"
adb shell am start -n "$PACKAGE/$ACTIVITY"

# Wait for app to start
sleep 1

# Get PID
PID=""
for i in {1..10}; do
    PID=$(adb shell pidof "$PACKAGE" 2>/dev/null || echo "")
    if [ -n "$PID" ]; then
        break
    fi
    sleep 0.5
done

if [ -z "$PID" ]; then
    echo -e "${RED}Warning: Could not get app PID. Showing filtered logs...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""
    adb logcat -v color "*:S" "RustStdoutStderr:V" "bevy:V" "wgpu:V" "whirled_peas:V" 2>/dev/null || \
    adb logcat "*:S" "RustStdoutStderr:V" "bevy:V" "wgpu:V" "whirled_peas:V"
else
    echo -e "${GREEN}App running with PID: $PID${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""
    # Stream logs filtered to our PID
    adb logcat --pid="$PID" -v color 2>/dev/null || adb logcat --pid="$PID"
fi

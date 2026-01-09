#!/bin/bash
# Build script for Whirled Peas Visualiser Android APK
#
# Prerequisites:
# 1. Android SDK installed (via Android Studio or command line tools)
# 2. Android NDK installed (r25+ recommended)
# 3. Environment variables set:
#    - ANDROID_SDK_ROOT or ANDROID_HOME
#    - ANDROID_NDK_HOME
# 4. Rust Android targets installed:
#    rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
# 5. NDK toolchain in PATH:
#    export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Whirled Peas Visualiser Android Build ===${NC}"

# Check for required environment
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo -e "${RED}Error: ANDROID_NDK_HOME not set${NC}"
    echo "Please set ANDROID_NDK_HOME to your NDK installation path"
    echo "Example: export ANDROID_NDK_HOME=~/Android/Sdk/ndk/25.2.9519653"
    exit 1
fi

# Add NDK toolchain to PATH
export PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"

# Check for required targets
echo -e "${YELLOW}Checking Rust targets...${NC}"
TARGETS="aarch64-linux-android armv7-linux-androideabi x86_64-linux-android"
for target in $TARGETS; do
    if ! rustup target list --installed | grep -q "$target"; then
        echo -e "${YELLOW}Installing target: $target${NC}"
        rustup target add "$target"
    fi
done

# Detect minimum API level (default to 21 for wide compatibility)
API_LEVEL=${ANDROID_API_LEVEL:-21}
TOOLCHAIN_DIR="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"

# Build for each Android architecture
echo -e "${YELLOW}Building native libraries (API level $API_LEVEL)...${NC}"

echo "Building for ARM64 (aarch64)..."
export CC_aarch64_linux_android="$TOOLCHAIN_DIR/aarch64-linux-android${API_LEVEL}-clang"
export CXX_aarch64_linux_android="$TOOLCHAIN_DIR/aarch64-linux-android${API_LEVEL}-clang++"
export AR_aarch64_linux_android="$TOOLCHAIN_DIR/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN_DIR/aarch64-linux-android${API_LEVEL}-clang"
cargo build --target aarch64-linux-android --release

echo "Building for ARM32 (armv7)..."
export CC_armv7_linux_androideabi="$TOOLCHAIN_DIR/armv7a-linux-androideabi${API_LEVEL}-clang"
export CXX_armv7_linux_androideabi="$TOOLCHAIN_DIR/armv7a-linux-androideabi${API_LEVEL}-clang++"
export AR_armv7_linux_androideabi="$TOOLCHAIN_DIR/llvm-ar"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$TOOLCHAIN_DIR/armv7a-linux-androideabi${API_LEVEL}-clang"
cargo build --target armv7-linux-androideabi --release

echo "Building for x86_64..."
export CC_x86_64_linux_android="$TOOLCHAIN_DIR/x86_64-linux-android${API_LEVEL}-clang"
export CXX_x86_64_linux_android="$TOOLCHAIN_DIR/x86_64-linux-android${API_LEVEL}-clang++"
export AR_x86_64_linux_android="$TOOLCHAIN_DIR/llvm-ar"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$TOOLCHAIN_DIR/x86_64-linux-android${API_LEVEL}-clang"
cargo build --target x86_64-linux-android --release

# Create jniLibs directory structure
echo -e "${YELLOW}Copying native libraries...${NC}"
mkdir -p android/app/src/main/jniLibs/{arm64-v8a,armeabi-v7a,x86_64}

# Copy our Rust libraries
cp target/aarch64-linux-android/release/libwhirled_peas.so \
   android/app/src/main/jniLibs/arm64-v8a/

cp target/armv7-linux-androideabi/release/libwhirled_peas.so \
   android/app/src/main/jniLibs/armeabi-v7a/

cp target/x86_64-linux-android/release/libwhirled_peas.so \
   android/app/src/main/jniLibs/x86_64/

# Copy libc++_shared.so from NDK (required by oboe/C++ dependencies)
echo -e "${YELLOW}Copying C++ shared library...${NC}"
NDK_SYSROOT="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib"

cp "$NDK_SYSROOT/aarch64-linux-android/libc++_shared.so" \
   android/app/src/main/jniLibs/arm64-v8a/

cp "$NDK_SYSROOT/arm-linux-androideabi/libc++_shared.so" \
   android/app/src/main/jniLibs/armeabi-v7a/

cp "$NDK_SYSROOT/x86_64-linux-android/libc++_shared.so" \
   android/app/src/main/jniLibs/x86_64/

# Build APK with Gradle
echo -e "${YELLOW}Building APK with Gradle...${NC}"
cd android

if [ -f "./gradlew" ]; then
    ./gradlew assembleRelease
else
    echo -e "${YELLOW}Gradle wrapper not found, creating...${NC}"
    gradle wrapper
    ./gradlew assembleRelease
fi

cd ..

# Report success
APK_PATH="android/app/build/outputs/apk/release/app-release-unsigned.apk"
if [ -f "$APK_PATH" ]; then
    echo -e "${GREEN}=== Build Successful! ===${NC}"
    echo -e "APK location: ${YELLOW}$APK_PATH${NC}"
    echo ""
    echo "To sign the APK for release, run:"
    echo "  apksigner sign --ks your-keystore.jks $APK_PATH"
    echo ""
    echo "To install on a connected device:"
    echo "  adb install $APK_PATH"
else
    echo -e "${RED}Build may have failed - APK not found at expected location${NC}"
    exit 1
fi

# Android Build Instructions for Whirled Peas Visualiser

This document explains how to build Whirled Peas Visualiser as an Android APK.

## Prerequisites

### 1. Android SDK and NDK

Install Android Studio or the command-line tools:
- **Android SDK** (API level 34 recommended)
- **Android NDK** (r25+ recommended, r26 or later preferred)
- **Android SDK Build-Tools**
- **Android SDK Platform-Tools**

Set environment variables:
```bash
export ANDROID_SDK_ROOT=$HOME/Android/Sdk
export ANDROID_NDK_HOME=$ANDROID_SDK_ROOT/ndk/26.1.10909125  # adjust version
export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
```

### 2. Rust Android Targets

Install the Rust cross-compilation targets:
```bash
rustup target add aarch64-linux-android    # ARM64 (most devices)
rustup target add armv7-linux-androideabi  # ARM32 (older devices)
rustup target add x86_64-linux-android     # x86_64 (emulators)
```

### 3. Java Development Kit

Android builds require JDK 17 or later:
```bash
# Ubuntu/Debian
sudo apt install openjdk-17-jdk

# Or use Android Studio's bundled JDK
export JAVA_HOME=/path/to/android-studio/jbr
```

## Building

### Option A: Using the Build Script (Recommended)

```bash
# Make sure environment variables are set, then:
./build-android.sh
```

This will:
1. Build the native Rust library for all Android architectures
2. Copy the .so files to the Android project
3. Build the APK using Gradle

The unsigned APK will be at:
`android/app/build/outputs/apk/release/app-release-unsigned.apk`

### Option B: Manual Build

#### Step 1: Build Native Libraries

```bash
# ARM64 (most modern devices)
cargo build --target aarch64-linux-android --release

# ARM32 (older devices)
cargo build --target armv7-linux-androideabi --release

# x86_64 (emulators)
cargo build --target x86_64-linux-android --release
```

#### Step 2: Copy Libraries

```bash
mkdir -p android/app/src/main/jniLibs/{arm64-v8a,armeabi-v7a,x86_64}

cp target/aarch64-linux-android/release/libwhirled_peas.so \
   android/app/src/main/jniLibs/arm64-v8a/

cp target/armv7-linux-androideabi/release/libwhirled_peas.so \
   android/app/src/main/jniLibs/armeabi-v7a/

cp target/x86_64-linux-android/release/libwhirled_peas.so \
   android/app/src/main/jniLibs/x86_64/
```

#### Step 3: Build APK

```bash
cd android
./gradlew assembleRelease
# Or for debug: ./gradlew assembleDebug
```

### Option C: Using Android Studio

1. Open the `android/` folder in Android Studio
2. Let it sync Gradle
3. Build > Build Bundle(s) / APK(s) > Build APK(s)

**Note:** You still need to build the native libraries first (Step 1 above).

## Signing the APK

### Debug Signing (for testing)

Debug builds are automatically signed with a debug key.

### Release Signing

1. Create a keystore (one-time):
```bash
keytool -genkey -v -keystore whirled-peas.jks -keyalg RSA -keysize 2048 -validity 10000 -alias whirled_peas
```

2. Sign the APK:
```bash
apksigner sign --ks whirled-peas.jks \
  android/app/build/outputs/apk/release/app-release-unsigned.apk
```

3. Or configure signing in `android/app/build.gradle`:
```gradle
android {
    signingConfigs {
        release {
            storeFile file('path/to/whirled-peas.jks')
            storePassword 'your-store-password'
            keyAlias 'whirled_peas'
            keyPassword 'your-key-password'
        }
    }
    buildTypes {
        release {
            signingConfig signingConfigs.release
        }
    }
}
```

## Installing

```bash
# Install on connected device
adb install android/app/build/outputs/apk/release/app-release.apk

# Or for debug builds
adb install android/app/build/outputs/apk/debug/app-debug.apk
```

## App Details

- **App Name:** Whirled Peas
- **Package ID:** `commercial_and_residential_evil.whirled_peas`
- **Native Library:** `libwhirled_peas.so`

## Troubleshooting

### Linker Errors

If you see linker errors, ensure the NDK toolchain is in your PATH:
```bash
export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
```

### Missing `libwhirled_peas.so`

Build the Rust library first before running Gradle:
```bash
cargo build --target aarch64-linux-android --release
```

### Graphics/Vulkan Issues

Whirled Peas Visualiser uses wgpu which supports Vulkan on Android. Most devices with Android 7.0+ support Vulkan. If you encounter issues:

1. Check Vulkan support: `adb shell cmd gpu vkjson`
2. The app requires OpenGL ES 3.0 as a fallback

### Touch Input

The app is designed for touch input on Android. Touch acts like mouse movement, and tap acts like left-click (pea explosion!).

## Project Structure

```
whirled_peas/
├── android/                    # Android project
│   ├── app/
│   │   ├── src/main/
│   │   │   ├── AndroidManifest.xml
│   │   │   ├── java/.../MainActivity.kt
│   │   │   ├── jniLibs/        # Native .so files go here
│   │   │   └── res/            # Android resources
│   │   └── build.gradle
│   ├── build.gradle
│   └── settings.gradle
├── assets/                     # Shared assets (copied to APK)
├── src/                        # Rust source code
├── Cargo.toml
├── build-android.sh            # Build script
└── ANDROID_BUILD.md            # This file
```

## Performance Tips

- ARM64 builds perform best on modern devices
- The release profile uses LTO for smaller binaries
- Pea particle count may need adjustment for lower-end devices

# Android Installation

armybox can be installed on Android devices in several ways:

## Method 1: Termux (Recommended - No Root)

Termux provides a Linux environment on Android without requiring root.

### Install via Termux Package

```bash
# Add armybox repository (once available)
pkg install armybox

# Or build from source
pkg install rust
git clone https://github.com/PegasusHeavyIndustries/armybox
cd armybox
cargo build --release
cp target/release/armybox $PREFIX/bin/
armybox --install $PREFIX/bin
```

### Quick Install Script

```bash
curl -fsSL https://raw.githubusercontent.com/PegasusHeavyIndustries/armybox/main/packaging/android/install-termux.sh | bash
```

## Method 2: ADB Sideload (Root Required)

Push the binary directly to your device via ADB.

### Prerequisites
- USB debugging enabled
- ADB installed on your computer
- Root access on device (for system installation)

### Installation

```bash
# Build for Android
./scripts/build-android.sh

# Push to device
adb push target/aarch64-linux-android/release/armybox /data/local/tmp/

# Install (with root)
adb shell su -c "mount -o rw,remount /system"
adb shell su -c "cp /data/local/tmp/armybox /system/bin/"
adb shell su -c "chmod 755 /system/bin/armybox"
adb shell su -c "armybox --install /system/bin"
adb shell su -c "mount -o ro,remount /system"
```

## Method 3: Magisk Module (Root Required)

Install as a Magisk module for systemless modification.

### Building the Module

```bash
./packaging/android/build-magisk-module.sh
```

### Installation

1. Copy `armybox-magisk-v0.1.0.zip` to your device
2. Open Magisk Manager
3. Go to Modules → Install from storage
4. Select the zip file
5. Reboot

## Supported Architectures

| Architecture | Target | Devices |
|-------------|--------|---------|
| ARM64 | `aarch64-linux-android` | Most modern phones (2016+) |
| ARM32 | `armv7-linux-androideabi` | Older phones, some tablets |
| x86_64 | `x86_64-linux-android` | Emulators, Chromebooks |
| x86 | `i686-linux-android` | Old emulators |

## Building from Source

### Prerequisites

```bash
# Install Android NDK
# Via Android Studio or:
wget https://dl.google.com/android/repository/android-ndk-r26b-linux.zip
unzip android-ndk-r26b-linux.zip
export ANDROID_NDK_HOME=$PWD/android-ndk-r26b

# Add Rust targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android

# Install cargo-ndk
cargo install cargo-ndk
```

### Build

```bash
# Build for ARM64 (most common)
cargo ndk -t arm64-v8a build --release

# Build for all architectures
./scripts/build-android.sh
```

## Troubleshooting

### "Permission denied"
- Ensure the binary has execute permission: `chmod +x armybox`
- For system paths, root access is required

### "Not executable: 64-bit ELF"
- You're trying to run an ARM64 binary on a 32-bit device
- Build for `armv7-linux-androideabi` instead

### SELinux Issues
- Try: `su -c "setenforce 0"` (temporary)
- Or relabel: `su -c "chcon u:object_r:system_file:s0 /system/bin/armybox"`

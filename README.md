# AndroidChecker

![Version](https://img.shields.io/badge/version-3.4.72-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)
![Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)

**AndroidChecker** is a high-performance Android device security audit and comprehensive management tool developed by **SmartDolphin Team** using Rust. It integrates robust command-line tools with a modern GUI monitoring interface, providing a one-stop Android device management solution for developers and security researchers.

## Download & Installation

You can download the latest installer directly from the [Releases](https://github.com/Astraeuszhao/AndroidChecker/releases) page.

- **Windows**: Download `AndroidChecker_Setup.exe`

## Core Features

### 1. Deep Security Audit
- **Root Detection**: Intelligently identifies `su` binaries, Magisk, SuperSU, and other Root management tools and their remnants.
- **Bootloader Status**: Checks Bootloader unlock status and integrity verification state (`verifiedbootstate`).
- **System Integrity**: Verifies Android security patch levels, SELinux status, and build fingerprints.

### 2. Backup & Restore
- **Full/Selective Backup**: Supports backup of app lists, user files (`/sdcard`), and app data (requires Root).
- **Standard Format**: Uses universal `tar.gz` packaging (`.adbbackup`) for easy manual extraction and cross-platform migration.
- **One-Click Restore**: Quickly restores data to the device from backup files.

### 3. Real-Time System Monitoring (GUI)
- **Resource Overview**: Visualizes CPU, memory, disk I/O, and network traffic in real-time.
- **Process Management**: View running process details, filter by name, and force kill processes.
- **App Management**: View installed app details, permissions, and signature information.

### 4. Stress Testing
- **ADB Stability Test**: Tests ADB connection stability through high-frequency concurrent commands.

## Quick Start

### Prerequisites
- Windows / macOS / Linux
- [ADB](https://developer.android.com/studio/command-line/adb) installed (the program will automatically try to use the built-in vendor directory).

### Build Guide

If you wish to build from source:

```bash
# Clone the repository
git clone https://github.com/Astraeuszhao/AndroidChecker.git
cd AndroidChecker

# Build Release version
cargo build --release

# Run
cargo run
```

## Important Notes

This project is currently the stable **v3.4.72** release, but please note:
*   Some advanced features (like app data backup) require Root access on the device.
*   The GUI works best on Windows; Linux/macOS may require additional font configuration.

## License

This project is licensed under the [MIT License](LICENSE.txt).
Copyright (c) 2025 **SmartDolphin Team**.

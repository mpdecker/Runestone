# Runestone Mobile Build Guide

## Prerequisites

### iOS (macOS only)
- macOS with Xcode 15+
- `tauri ios init` — generates Xcode project in `src-tauri/gen/apple/`
- Code signing certificates configured in Xcode

### Android (Windows, macOS, Linux)
- Android SDK with NDK installed
- `tauri android init` — generates Gradle project in `src-tauri/gen/android/`
- Set `ANDROID_HOME` environment variable

## Initial Setup

```bash
# Install dependencies
pnpm install

# Initialize iOS project (macOS only)
pnpm tauri ios init

# Initialize Android project
pnpm tauri android init
```

## Development

```bash
# iOS development (macOS only)
pnpm tauri:ios

# Android development
pnpm tauri:android
```

## Production Build

```bash
# iOS build (macOS only)
pnpm tauri:ios:build

# Android build
pnpm tauri:android:build
```

## Architecture

The mobile app connects to a remote Runestone server over HTTP. It does not run PostgreSQL or Neo4j locally. On first launch, the Connection screen prompts for the server URL. The server configuration is saved to localStorage for subsequent launches.

### Server Requirements

The remote server must expose a Runestone API endpoint. Configure the server URL in the Connection screen (e.g., `https://runestone.example.com`).

### Platform Detection

The app detects its platform at runtime using the `get_platform` Tauri command and renders the appropriate UI:
- iOS/Android: `MobileApp` with bottom tab navigation
- Desktop (Windows/macOS/Linux): `DesktopApp` with three-panel layout

## CI/CD

For automated mobile builds:
- iOS: macOS runner with Xcode + code signing certificates
- Android: Linux/macOS runner with Android SDK

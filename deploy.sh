#!/usr/bin/env bash
set -e

pnpm tauri android build --apk --aab

APK=src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
AAB=src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab

# Install on a connected device if one is present (AABs cannot be installed via adb)
if adb devices | grep -q $'\tdevice$'; then
    adb install -r "$APK"
    echo "Installed."
else
    echo "No device connected — skipped adb install."
fi

echo "AAB ready for Play Console upload: $AAB"

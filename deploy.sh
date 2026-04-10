#!/usr/bin/env bash
set -e
pnpm tauri android build
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
echo "Installed."

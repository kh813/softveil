#!/bin/bash
set -e

APP_NAME="Softveil"
TARGET_DIR="target/release"
APP_BUNDLE="${TARGET_DIR}/${APP_NAME}.app"

echo "Building Softveil in release mode..."
cargo build --release

echo "Creating .app bundle..."
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"

cp "target/release/softveil" "${APP_BUNDLE}/Contents/MacOS/"
cp "package/macos/Info.plist" "${APP_BUNDLE}/Contents/"
cp "assets/icon_macos.icns" "${APP_BUNDLE}/Contents/Resources/"

echo "Bundle created at ${APP_BUNDLE}"

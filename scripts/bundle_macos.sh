#!/bin/bash
set -e

APP_NAME="Softveil"
TARGET_DIR="target/aarch64-apple-darwin/release"
APP_BUNDLE="target/release/${APP_NAME}.app"

echo "Creating .app bundle from ${TARGET_DIR}..."
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"

cp "${TARGET_DIR}/softveil" "${APP_BUNDLE}/Contents/MacOS/"
cp "package/macos/Info.plist" "${APP_BUNDLE}/Contents/"
cp "assets/icon_macos.icns" "${APP_BUNDLE}/Contents/Resources/"
cp "assets/face_detector.onnx" "${APP_BUNDLE}/Contents/Resources/"

echo "Bundle created at ${APP_BUNDLE}"

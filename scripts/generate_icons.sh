#!/bin/bash
set -e

# Configuration
SOURCE_SVG="assets/softveil_icon.svg"
OUTPUT_DIR="assets/generated_icons"
mkdir -p "$OUTPUT_DIR"

echo "🎨 Generating icons from $SOURCE_SVG..."

# 1. Generate PNGs of various sizes
SIZES=(16 32 48 64 128 256 512 1024)
for size in "${SIZES[@]}"; do
    magick -background none "$SOURCE_SVG" -resize "${size}x${size}" "$OUTPUT_DIR/icon_${size}.png"
done

# 2. Create Windows .ico (16, 32, 48, 64, 256)
echo "🪟 Creating assets/icon_windows.ico..."
magick "$OUTPUT_DIR/icon_16.png" "$OUTPUT_DIR/icon_32.png" "$OUTPUT_DIR/icon_48.png" "$OUTPUT_DIR/icon_64.png" "$OUTPUT_DIR/icon_256.png" "assets/icon_windows.ico"

# 3. Create macOS .icns
echo "🍎 Creating assets/icon_macos.icns..."
ICONSET="$OUTPUT_DIR/icon.iconset"
mkdir -p "$ICONSET"

cp "$OUTPUT_DIR/icon_16.png"   "$ICONSET/icon_16x16.png"
cp "$OUTPUT_DIR/icon_32.png"   "$ICONSET/icon_16x16@2x.png"
cp "$OUTPUT_DIR/icon_32.png"   "$ICONSET/icon_32x32.png"
cp "$OUTPUT_DIR/icon_64.png"   "$ICONSET/icon_32x32@2x.png"
cp "$OUTPUT_DIR/icon_128.png"  "$ICONSET/icon_128x128.png"
cp "$OUTPUT_DIR/icon_256.png"  "$ICONSET/icon_128x128@2x.png"
cp "$OUTPUT_DIR/icon_256.png"  "$ICONSET/icon_256x256.png"
cp "$OUTPUT_DIR/icon_512.png"  "$ICONSET/icon_256x256@2x.png"
cp "$OUTPUT_DIR/icon_512.png"  "$ICONSET/icon_512x512.png"
cp "$OUTPUT_DIR/icon_1024.png" "$ICONSET/icon_512x512@2x.png"

iconutil -c icns "$ICONSET" -o "assets/icon_macos.icns"

# 4. Generate a 22x22 template icon for macOS menu bar (Template)
echo "🍎 Creating assets/icon_macos_template.png..."
# Standard macOS template icon: Black shape on transparent background.
# System will automatically tint it white in Dark Mode.
TEMPLATE_SVG="assets/temp_template.svg"
# Remove background and make sure all fills/strokes are black for the template
sed -e 's/<rect x="0" y="0" width="1024" height="1024" rx="224" fill="#f2f2f2"\/>//' \
    -e 's/fill="#[^"]*"/fill="#000000"/g' \
    -e 's/stroke="#[^"]*"/stroke="#000000"/g' \
    "$SOURCE_SVG" > "$TEMPLATE_SVG"

magick -background none "$TEMPLATE_SVG" -resize 22x22 "assets/icon_macos_template.png"
magick -background none "$TEMPLATE_SVG" -resize 44x44 "assets/icon_macos_template@2x.png"

rm "$TEMPLATE_SVG"

echo "✅ Icon generation complete!"
rm -rf "$OUTPUT_DIR"

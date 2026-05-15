#!/bin/bash
set -e

# Configuration
SOURCE_SVG="assets/softveil_icon.svg"
OUTPUT_DIR="assets/generated_icons"
mkdir -p "$OUTPUT_DIR"

echo "🎨 Generating icons from $SOURCE_SVG..."

# 1. Prepare base PNG
# If assets/softveil_icon.png exists, use it as the source for the app icon.
# Otherwise, generate it from SVG.
if [ -f "assets/softveil_icon.png" ]; then
    echo "📸 Using existing assets/softveil_icon.png as source..."
    # Clean up white corners if they exist (make everything outside the rounded rect transparent)
    # We use the SVG's clip path logic: rounded rect with rx=224
    magick "assets/softveil_icon.png" \
        \( +clone -fill black -colorize 100 -fill white -draw "roundrectangle 0,0 1024,1024 224,224" \) \
        -alpha off -compose CopyOpacity -composite "assets/icon_base.png"
else
    echo "🎨 Generating base PNG from $SOURCE_SVG..."
    magick -background none "$SOURCE_SVG" -resize 1024x1024 "assets/icon_base.png"
fi

# 1.5 Generate PNGs of various sizes from base
SIZES=(16 32 48 64 128 256 512 1024)
for size in "${SIZES[@]}"; do
    magick "assets/icon_base.png" -resize "${size}x${size}" "$OUTPUT_DIR/icon_${size}.png"
done

# 2. Create Windows .ico
echo "🪟 Creating assets/icon_windows.ico..."
magick "$OUTPUT_DIR/icon_16.png" "$OUTPUT_DIR/icon_32.png" "$OUTPUT_DIR/icon_48.png" "$OUTPUT_DIR/icon_64.png" "$OUTPUT_DIR/icon_256.png" "assets/icon_windows.ico"

# 3. Create macOS .icns
echo "🍎 Creating assets/icon_macos.icns..."
ICONSET="$OUTPUT_DIR/icon.iconset"
mkdir -p "$ICONSET"
# (cp commands ...)
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
# For the template, we want ONLY the monitor shape, no background.
# We'll use the SVG but render only the monitor parts.
TEMPLATE_SVG="assets/temp_template.svg"
# Remove the background rect and the clip-path (which is also a rounded rect)
sed -e 's/<rect x="0" y="0" width="1024" height="1024" rx="224" fill="#f2f2f2"\/>//' \
    -e 's/clip-path="url(#icon-clip)"//' \
    -e 's/fill="#[^"]*"/fill="#000000"/g' \
    -e 's/stroke="#[^"]*"/stroke="#000000"/g' \
    -e 's/stroke-width="42"/stroke-width="60"/g' \
    -e 's/stroke-width="38"/stroke-width="50"/g' \
    -e 's/fill-opacity="[^"]*"/fill-opacity="1.0"/g' \
    -e 's/stroke-opacity="[^"]*"/stroke-opacity="1.0"/g' \
    "$SOURCE_SVG" > "$TEMPLATE_SVG"

# Render to a larger size first to keep quality, then resize.
# Also force RGBA and make sure it's black on transparent.
magick -background none "$TEMPLATE_SVG" -resize 88x88 "assets/icon_macos_template_large.png"
magick "assets/icon_macos_template_large.png" -resize 22x22 -type truecoloralpha "assets/icon_macos_template.png"
magick "assets/icon_macos_template_large.png" -resize 44x44 -type truecoloralpha "assets/icon_macos_template@2x.png"

rm "$TEMPLATE_SVG" "assets/icon_macos_template_large.png" "assets/icon_base.png"

echo "✅ Icon generation complete!"
rm -rf "$OUTPUT_DIR"

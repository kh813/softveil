.PHONY: all mac win win-gnu win-msvc clean help

# Default target
all: mac win

help:
	@echo "Softveil Build Automation"
	@echo ""
	@echo "Usage:"
	@echo "  make mac       Build macOS release binary and create .app bundle"
	@echo "  make win-gnu   Build Windows release binary (MinGW/GNU - Good for cross-compiling)"
	@echo "  make win-msvc  Build Windows release binary (MSVC - Best for native Windows/CI)"
	@echo "  make win       Alias for win-gnu"
	@echo "  make all       Build both macOS and Windows (win-gnu)"
	@echo "  make clean     Remove build artifacts"

mac:
	@echo "Building for macOS (Apple Silicon)..."
	cargo build --release --target aarch64-apple-darwin
	./scripts/bundle_macos.sh

win-gnu:
	@echo "Building for Windows (GNU)..."
	@# Note: This requires x86_64-pc-windows-gnu target installed via rustup
	cargo build --release --target x86_64-pc-windows-gnu

win-msvc:
	@echo "Building for Windows (MSVC)..."
	@# Note: This requires x86_64-pc-windows-msvc target and MSVC toolchain (Windows only)
	cargo build --release --target x86_64-pc-windows-msvc

win: win-gnu

clean:
	@echo "Cleaning up..."
	cargo clean
	rm -rf target/release/Softveil.app

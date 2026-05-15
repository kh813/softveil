.PHONY: all mac win clean help

# Default target
all: mac win

help:
	@echo "Softveil Build Automation"
	@echo ""
	@echo "Usage:"
	@echo "  make mac     Build macOS release binary and create .app bundle"
	@echo "  make win     Build Windows release binary (requires mingw-w64 or cross)"
	@echo "  make all     Build both macOS and Windows"
	@echo "  make clean   Remove build artifacts"

mac:
	@echo "Building for macOS (Apple Silicon)..."
	cargo build --release --target aarch64-apple-darwin
	./scripts/bundle_macos.sh


win:
	@echo "Building for Windows..."
	@# Note: This requires x86_64-pc-windows-gnu target installed via rustup
	@# and mingw-w64 toolchain if building on macOS.
	cargo build --release --target x86_64-pc-windows-gnu

clean:
	@echo "Cleaning up..."
	cargo clean
	rm -rf target/release/Softveil.app

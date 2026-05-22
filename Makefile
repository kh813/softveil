.PHONY: all mac win msvc win-gnu win-msvc clean help

# OS detection
ifeq ($(OS),Windows_NT)
    HOST_OS := Windows
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Darwin)
        HOST_OS := macOS
    else
        HOST_OS := Linux
    endif
endif

# Default target based on host OS
ifeq ($(HOST_OS),Windows)
all: msvc mac
else
all: mac win
endif

help:
	@echo "Softveil Build Automation (Host: $(HOST_OS))"
	@echo ""
	@echo "Usage:"
	@echo "  make mac       Build macOS release binary and create .app bundle"
	@echo "  make win       Build Windows release binary (GNU - Good for cross-compiling)"
	@echo "  make msvc      Build Windows release binary (MSVC - Best for native Windows)"
	@echo "  make all       Build both macOS and Windows (Smart selection based on host)"
	@echo "  make clean     Remove build artifacts"

mac:
	@echo "Building for macOS (Apple Silicon)..."
	cargo build --release --target aarch64-apple-darwin
	@if [ "$(HOST_OS)" = "macOS" ]; then \
		./scripts/bundle_macos.sh; \
	else \
		echo "Skipping .app bundling (only supported on macOS host)"; \
	fi

win-gnu:
	@echo "Building for Windows (GNU)..."
	@# Note: This requires x86_64-pc-windows-gnu target installed via rustup
	cargo build --release --target x86_64-pc-windows-gnu

win-msvc:
	@echo "Building for Windows (MSVC)..."
	@# Note: This requires x86_64-pc-windows-msvc target and MSVC toolchain (Windows only)
	cargo build --release --target x86_64-pc-windows-msvc

win: win-gnu
msvc: win-msvc

clean:
	@echo "Cleaning up..."
	cargo clean
	@if [ -d target/release/Softveil.app ]; then rm -rf target/release/Softveil.app; fi


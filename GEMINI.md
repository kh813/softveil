# Softveil Project Instructions

## Release & Tagging Protocol

Before committing a version bump or pushing a new tag (e.g., `v*`), the following conditions MUST be met:

1. **Cross-Platform Compilation**:
   - Must compile successfully on macOS (native).
   - Must compile successfully for Windows. Use `make win` or `cargo check --target x86_64-pc-windows-gnu` to verify.
   
2. **Warning-Free Build**:
   - No unnecessary compiler warnings (e.g., `dead_code`, `unused`). 
   - If a feature is for future use, use `#[allow(dead_code)]` or `#[cfg]` appropriately.

3. **Mandatory Human Verification**:
   - **Gemini MUST NOT** create a version tag or push a release commit without explicit user confirmation.
   - Before tagging, Gemini must provide a summary of the current state and ask: "Has the behavior been manually verified on the target devices? May I proceed with the version tag?"

## Development Workflow

- **Platform Parity**: When adding features to `src/platform/macos.rs`, always ensure corresponding stubs or implementations exist in `src/platform/windows.rs`.
- **UI & Aesthetics**: Since this is a visual privacy tool, any shader or overlay changes must be visually verified.

[日本語版](MANUAL.md)

# Softveil User Manual

Welcome to Softveil! This manual explains how to use Softveil, the next-generation software privacy filter, and how to configure it effectively.

---

## 1. Quick Start

1. **Launch Softveil**: The Softveil icon will appear in the system tray (menu bar on macOS, taskbar on Windows).
2. **Run Benchmark**: Click the tray icon and select **Optimization (Benchmark) ▶ Optimize All Screens**.
   * *This is a crucial step!* Softveil analyzes your displays and automatically generates the best privacy filter profile (such as "Transit (Maximum)") based on the physical characteristics (OLED/LCD, PPI) of your screens.
3. **Toggle Global Filter**: Press `Cmd + Shift + P` (macOS) or `Ctrl + Shift + P` (Windows) to instantly toggle the privacy filter on all screens.

---

## 2. Menu Structure

Click the Softveil tray icon to open the menu.

```text
├── Filter: All On
├── [Monitor 1 Name] ▶
│   ├── Display Type: Notebook FHD (PPI: 140)
│   ├── Enable Filter
│   ├── Filter Mode ▶ (SPD Protect, Stealth Dark, Standard Louver, OCR Jammer (Anti-Camera), etc.)
│   ├── Filter Alpha ▶ (0% - 100%)
│   ├── Filter Intensity ▶ (Highest, High, Standard, Low, Lowest)
│   ├── Presets ▶ (Apply Preset, Save Current Settings...)
│   ├── Reset to Recommended
│   └── Advanced Fine-Tuning ▶
│       └── Stripe Thickness, Cover Ratio, Scroll Speed, Change Panel Type
├── [Monitor 2 Name] ▶ ... (Same as above)
├── Optimization (Benchmark) ▶
│   ├── Optimize All Screens
│   └── Optimize [Monitor Name]
├── AI Peep Prevention ▶ [Current Status]
│   ├── Off
│   ├── Vigilance (Deploy on Detect)
│   └── Enhanced Filter Always On
├── Start at Login [ ]
└── Quit Softveil
```

### 🖥️ Display Settings
You can fine-tune the filter settings for each connected monitor individually.
*   **Filter Mode**: Select the rendering algorithm. For example, `SPD Protect` is best for general use, while `Stealth Dark (LLCC)` offers extreme privacy by utilizing OS dark mode.
*   **Filter Alpha**: Adjust the transparency of the filter.
*   **Presets**: Save your favorite settings and apply them easily.

### 👁️ AI Peep Prevention (Camera-Based Detection)
*   **Off**: Disables camera detection.
*   **Vigilance (Deploy on Detect)**: The screen remains 100% clear (0% alpha) during normal use. The moment the camera detects someone behind you, it automatically and instantly deploys the privacy filter.
*   **Enhanced Filter Always On**: The filter is always active at your preferred level. When peeping is detected, the alpha level jumps to 80% for maximum protection.

---

## 3. Filter Modes Explained

| Mode | Recommended Use | Features |
|:---|:---|:---|
| **SPD Protect ✦** | Outdoor/Cafe | Uses high-frequency interference. Best balance of visibility and privacy. |
| **Stealth Dark (LLCC)** | High Security | Forces OS Dark Mode and drastically lowers brightness. |
| **Stealth Light (Subpixel)**| Office/Document | Focuses on destroying text contrast without darkening the screen too much. |
| **OCR Jammer (Anti-Camera)** | Highly Confidential | Superimposes a special pattern to make it difficult for smartphone cameras or AI text recognition (OCR) to read the screen. |
| **Standard Louver** | Legacy | Simple vertical stripes. |

---

## 4. Troubleshooting

*   **Filter is flickering**: Change the **Filter Intensity** to a lower setting, or use **Advanced Fine-Tuning** to reduce the **Scroll Speed**.
*   **Mac OS asks for Screen Recording permission again**: This is required for the Benchmark feature. Please allow it in System Settings > Privacy & Security > Screen Recording.


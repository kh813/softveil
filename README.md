# Softveil

[日本語 (Japanese)](#japanese) | [English](#english)

Softveil は、macOS および Windows 向けの次世代ソフトウェア・プライバシーフィルターです。  
Softveil is a next-generation software privacy filter for macOS and Windows.

---

<a id="japanese"></a>
## 🇯🇵 日本語 (Japanese)

物理的なプライバシーフィルター（覗き見防止シート）をソフトウェアで再現し、さらにディスプレイの物理特性や AI 視覚検知を組み合わせることで、従来の「ただ画面を暗くするだけ」のアプリとは一線を画す秘匿性能を提供します。

### 📚 ドキュメント

*   📖 **[ユーザーマニュアル (使い方と詳細設定)](MANUAL.md)**
*   📝 **[基本仕様書](Softveil_specs_v1_6.md)**
*   📋 **[開発 ToDo リスト](Softveil_ToDo_v1_3.md)**
*   📓 **[開発ログ](DEVLOG.md)**

### 🌟 主な機能

- **SPD (Software Defined Privacy Display) テクノロジー**: 
    - ディスプレイのパネル種別（OLED / LCD IPS）に合わせた最適な干渉パターンを生成。
    - 斜め方向からのコントラストを強制的に消失させ、文字の判読を困難にします。
- **AI 覗き見検知 & 監視モード (Phase 3 / Phase 15)**:
    - 内蔵カメラを使用して背後の人物をリアルタイム検知。覗き見されている間だけ自動的にフィルターを強化します。
    - **AI 監視 (Vigilance) モード**: 通常時は画面の視認性を100%維持（完全透明化）し、背後に覗き見者を検知した瞬間のみ自動的にプライバシーフィルターを展開する、極めて快適でインテリジェントな防衛モードも搭載。
- **ステルス・ダークモード (Phase 6)**:
    - フィルターの有効化と同時に、OSのテーマ変更（ダークモード化）と輝度抑制をアトミックに実行。
- **マルチディスプレイ & ホットプラグ対応**:
    - ディスプレイの接続・切断を自動検知し、個別の設定（PPI、パネル種別）を自動適用・復元します。
- **ハイブリッド SPD 制御**:
    - AIによる自動推奨設定をベースに、縞の太さやスクロール速度をユーザーが微調整可能。フリッカー（チラつき）を抑えつつ秘匿性を最大化できます。

### 🚀 インストール

[リリースページ](https://github.com/kh813/softveil/releases) から最新のバイナリをダウンロードしてください。

**macOS**
1. `Softveil-macOS.zip` を展開し、`Softveil.app` を `/Applications` フォルダへ移動します。
2. 初回起動時、システム設定で「アクセシビリティ」および「画面収録」の権限を許可してください。

**Windows**
1. `Softveil-Windows.zip` を任意のフォルダに展開し、`softveil.exe` を実行します。
> 💡 **ヒント**: AIモデル（ONNX）はバイナリに内蔵されているため、スタンドアロンで動作します。

### ⌨️ ショートカット

| 機能 | macOS | Windows |
|:---|:---|:---|
| フィルター 全体 ON/OFF | `Cmd + Shift + P` | `Ctrl + Shift + P` |

### 🛠 開発者向け情報

ビルド方法や技術スタックについては、[開発ログ](DEVLOG.md) や [基本仕様書](Softveil_specs_v1_6.md) をご参照ください。

### 🆕 リリース履歴

*   **v0.1.23**: メニューUIの改善（AI検知モードの集約）、日英バイリンガル対応の強化、「OCR妨害」への名称統一。

---

<a id="english"></a>
## 🇺🇸 English

Softveil replicates a physical privacy filter (peep-prevention sheet) in software. By combining the physical characteristics of the display with AI visual detection, it offers a level of concealment that sets it apart from conventional "screen dimming" apps.

### 📚 Documentation

*   📖 **[User Manual (How to Use & Settings)](MANUAL_en.md)**
*   📝 **[Specifications](Softveil_specs_v1_6_en.md)**
*   📋 **[Development ToDo List](Softveil_ToDo_v1_3_en.md)**
*   📓 **[Development Log](DEVLOG_en.md)**

### 🌟 Key Features

- **SPD (Software Defined Privacy Display) Technology**: 
    - Generates optimal interference patterns tailored to the display panel type (OLED / LCD IPS).
    - Forcibly eliminates contrast from diagonal viewing angles, making text difficult to read.
- **AI Peep Detection & Vigilance Mode**:
    - Uses the built-in camera to detect people behind you in real time. It automatically enhances the filter only when it detects peeping.
    - **AI Vigilance Mode**: Maintains 100% screen visibility (completely transparent) during normal use, and automatically deploys the privacy filter only the moment it detects a shoulder surfer.
- **Stealth Dark Mode**:
    - Atomically executes an OS theme change (switching to Dark Mode) and suppresses brightness simultaneously with enabling the filter.
- **Multi-display & Hotplug Support**:
    - Automatically detects when a display is connected or disconnected, and applies/restores individual settings (PPI, panel type) accordingly.
- **Hybrid SPD Control**:
    - Users can fine-tune stripe thickness and scroll speed based on AI recommendations to maximize concealment while minimizing flicker.

### 🚀 Installation

Please download the latest binary from the [Releases page](https://github.com/kh813/softveil/releases).

**macOS**
1. Extract `Softveil-macOS.zip` and move `Softveil.app` to the `/Applications` folder.
2. Upon the first launch, please allow "Accessibility" and "Screen Recording" permissions in System Settings.

**Windows**
1. Extract `Softveil-Windows.zip` to any folder and run `softveil.exe`.
> 💡 **Tip**: The AI model (ONNX) is embedded within the binary, so it runs standalone.

### ⌨️ Shortcuts

| Function | macOS | Windows |
|:---|:---|:---|
| Toggle Global Filter | `Cmd + Shift + P` | `Ctrl + Shift + P` |

### 🛠 For Developers

For build instructions and the technology stack, please refer to the [Development Log](DEVLOG_en.md) and [Specifications](Softveil_specs_v1_6_en.md).

### 🆕 Release History

*   **v0.1.23**: Improved Menu UI (consolidated AI detection modes), enhanced Japanese/English bilingual support, and unified terminology to "OCR Jammer".

# Softveil

**Softveil** は、macOS および Windows 向けの次世代ソフトウェア・プライバシーフィルターです。  
物理的なプライバシーフィルター（覗き見防止シート）をソフトウェアで再現し、さらにディスプレイの物理特性や AI 視覚検知を組み合わせることで、従来の「ただ画面を暗くするだけ」のアプリとは一線を画す秘匿性能を提供します。

---

## 🌟 主な機能

- **SPD (Software Defined Privacy Display) テクノロジー**: 
    - ディスプレイのパネル種別（OLED / LCD IPS）に合わせた最適な干渉パターンを生成。
    - 斜め方向からのコントラストを強制的に消失させ、文字の判読を困難にします。
- **AI 覗き見検知 (Phase 3)**:
    - 内蔵カメラを使用して背後の人物をリアルタイム検知。
    - 覗き見されている間だけ自動的にフィルターを強化します。
- **ステルス・ダークモード (Phase 6)**:
    - フィルターの有効化と同時に、OSのテーマ変更（ダークモード化）と輝度抑制をアトミックに実行。
- **マルチディスプレイ & ホットプラグ対応**:
    - ディスプレイの接続・切断を自動検知し、個別の設定（PPI、パネル種別）を自動適用・復元します。
- **ハイブリッド SPD 制御**:
    - AIによる自動推奨設定をベースに、縞の太さやスクロール速度をユーザーが微調整可能。フリッカー（チラつき）を抑えつつ秘匿性を最大化できます。

---

## 🚀 インストール

[リリースページ](https://github.com/kh813/softveil/releases) から最新のバイナリをダウンロードしてください。

### macOS
1. `Softveil-macOS.zip` を展開します。
2. `Softveil.app` を `/Applications` フォルダへ移動します。
3. 初回起動時、システム設定で「アクセシビリティ」および「画面収録」の権限を許可してください。

### Windows
1. `Softveil-Windows.zip` を任意のフォルダに展開します。
2. `softveil.exe` を実行します。
3. AI 覗き見検知を利用する場合、`face_detector.onnx` が実行ファイルと同じ階層にあることを確認してください。

---

## ⌨️ ショートカット

| 機能 | macOS | Windows |
|:---|:---|:---|
| フィルター 全体 ON/OFF | `Cmd + Shift + P` | `Ctrl + Shift + P` |

---

## 🛠 開発者向け情報

### ビルド方法

**必要な環境:**
- Rust ツールチェーン (`rustup`)
- (macOS) Xcode Command Line Tools
- (Windows クロスビルド用) `x86_64-pc-windows-gnu` ターゲット

```bash
# macOS ネイティブビルド
make mac

# Windows 向けクロスビルド
make win

# 全プラットフォーム一括ビルド
make all
```

### 技術スタック
- **Language**: Rust
- **Windowing**: [tao](https://github.com/tauri-apps/tao)
- **Rendering**: [wgpu](https://github.com/gfx-rs/wgpu) (WGSL Shader)
- **UI (Tray)**: [tray-icon](https://github.com/tauri-apps/tray-icon), [muda](https://github.com/tauri-apps/muda)
- **AI Inference**: [tract](https://github.com/snipslab/tract) (ONNX)

---

## 📜 ライセンス

本プロジェクトのライセンスについては `LICENSE` ファイル（準備中）を参照してください。

---

## ⚠️ 免責事項

Softveil はソフトウェアによる視覚的難読化を提供するものであり、物理的な出射角制限を行うプライバシーフィルターとは原理が異なります。最高レベルのセキュリティが必要な場合は、市販の物理フィルターとの併用を推奨します。また、長時間使用による眼精疲労を防ぐため、適宜「高度な微調整」メニューからスクロール速度を下げるなどの調整を行ってください。

import re

with open('src/tray.rs', 'r', encoding='utf-8') as f:
    content = f.read()

replacements = {
    '"フィルター：すべてオン"': 't("フィルター：すべてオン", "Filter: All On")',
    '"全画面を最適化する"': 't("全画面を最適化する", "Optimize All Screens")',
    '"この画面を個別で最適化"': 't("この画面を個別で最適化", "Optimize This Screen")',
    '"フィルターを有効"': 't("フィルターを有効", "Enable Filter")',
    '"フィルター形式"': 't("フィルター形式", "Filter Mode")',
    '"SPD プロテクト ✦ (推奨)"': 't("SPD プロテクト ✦ (推奨)", "SPD Protect ✦ (Rec.)")',
    '"ステルス・ダーク (LLCC)"': 't("ステルス・ダーク (LLCC)", "Stealth Dark (LLCC)")',
    '"ステルス・ライト (Subpixel)"': 't("ステルス・ライト (Subpixel)", "Stealth Light (Subpixel)")',
    '"標準ルーバー"': 't("標準ルーバー", "Standard Louver")',
    '"OCR妨害 (カメラ撮影対策)"': 't("OCR妨害 (カメラ撮影対策)", "OCR Jammer (Anti-Camera)")',
    '"単色（輝度を抑える）"': 't("単色（輝度を抑える）", "Solid Color (Dim)")',
    '"フィルター濃度"': 't("フィルター濃度", "Filter Alpha")',
    '"フィルター強度"': 't("フィルター強度", "Filter Intensity")',
    '"最高 (密度高)"': 't("最高 (密度高)", "Highest (High Density)")',
    '"高"': 't("高", "High")',
    '"標準"': 't("標準", "Standard")',
    '"低"': 't("低", "Low")',
    '"最低 (密度低)"': 't("最低 (密度低)", "Lowest (Low Density)")',
    '"設定プリセット"': 't("設定プリセット", "Presets")',
    '"(プリセットなし)"': 't("(プリセットなし)", "(No Presets)")',
    '"プリセット適用"': 't("プリセット適用", "Apply Preset")',
    '"プリセット削除"': 't("プリセット削除", "Delete Preset")',
    '"現在の設定を保存..."': 't("現在の設定を保存...", "Save Current Settings...")',
    '"おすすめ設定に戻す"': 't("おすすめ設定に戻す", "Reset to Recommended")',
    '"高度な微調整"': 't("高度な微調整", "Advanced Fine-Tuning")',
    '"縞の太さ (Period)"': 't("縞の太さ (Period)", "Stripe Thickness (Period)")',
    '"自動 (推奨)"': 't("自動 (推奨)", "Auto (Recommended)")',
    '"細い (0.8mm)"': 't("細い (0.8mm)", "Thin (0.8mm)")',
    '"標準 (1.2mm)"': 't("標準 (1.2mm)", "Standard (1.2mm)")',
    '"太い (1.8mm)"': 't("太い (1.8mm)", "Thick (1.8mm)")',
    '"極太 (2.5mm)"': 't("極太 (2.5mm)", "Very Thick (2.5mm)")',
    '"遮蔽率 (Cover Ratio)"': 't("遮蔽率 (Cover Ratio)", "Cover Ratio")',
    '"低 (50%)"': 't("低 (50%)", "Low (50%)")',
    '"標準 (70%)"': 't("標準 (70%)", "Standard (70%)")',
    '"高 (85%)"': 't("高 (85%)", "High (85%)")',
    '"最高 (95%)"': 't("最高 (95%)", "Highest (95%)")',
    '"スクロール速度"': 't("スクロール速度", "Scroll Speed")',
    '"静止 (0mm/s)"': 't("静止 (0mm/s)", "Static (0mm/s)")',
    '"極低速 (5mm/s)"': 't("極低速 (5mm/s)", "Very Slow (5mm/s)")',
    '"低速 (20mm/s)"': 't("低速 (20mm/s)", "Slow (20mm/s)")',
    '"標準 (50mm/s)"': 't("標準 (50mm/s)", "Standard (50mm/s)")',
    '"画面タイプを変更"': 't("画面タイプを変更", "Change Display Type")',
    '"ノートPC FHD"': 't("ノートPC FHD", "Notebook FHD")',
    '"ノートPC 高解像度"': 't("ノートPC 高解像度", "Notebook HiDPI")',
    '"外付け大型 4K"': 't("外付け大型 4K", "External Large 4K")',
    '"外付け 標準"': 't("外付け 標準", "External Standard")',
    '"不明"': 't("不明", "Unknown")',
    '"最適化 (ベンチマーク)"': 't("最適化 (ベンチマーク)", "Optimization (Benchmark)")',
    '"オフ"': 't("オフ", "Off")',
    '"Vigilance（検知時のみ展開）"': 't("Vigilance（検知時のみ展開）", "Vigilance (Deploy on Detect)")',
    '"常時フィルター強化"': 't("常時フィルター強化", "Enhanced Filter Always On")',
    '"ログイン時に起動"': 't("ログイン時に起動", "Start at Login")',
    '"Softveil を終了"': 't("Softveil を終了", "Quit Softveil")',
}

for k, v in replacements.items():
    content = content.replace(k, v)

# 動的なフォーマット部分の置換
content = re.sub(r'format!\("パネル種別を変更 \(\{\}\)", (.*?)\)', r'format!("{} ({})", t("パネル種別を変更", "Change Panel Type"), \1)', content)
content = re.sub(r'format!\("画面タイプ: \{\} \(PPI: \{:\.0\}\)", (.*?), (.*?)\)', r'format!("{}: {} (PPI: {:.0})", t("画面タイプ", "Display Type"), \1, \2)', content)
content = re.sub(r'format!\("全画面を最適化中 \(\{:?\.0\}%\) \.\.\.", (.*?)\)', r'format!("{} ({:.0}%) ...", t("全画面を最適化中", "Optimizing all screens"), \1)', content)
content = re.sub(r'format!\("\{\} を個別で最適化", (.*?)\)', r'format!("{} {}", \1, t("を個別で最適化", "Optimize (Individual)"))', content)
content = re.sub(r'format!\("AI 覗き見検知 ▶ \[\{\}\]", (.*?)\)', r'format!("{} ▶ [{}]", t("AI 覗き見検知", "AI Peep Prevention"), \1)', content)

# ベンチマーク状態等の動的変数
content = content.replace('("✓ Vigilance", false, true, false)', '(t("✓ Vigilance", "✓ Vigilance"), false, true, false)')
content = content.replace('("✓ 常時フィルター強化", false, false, true)', '(t("✓ 常時フィルター強化", "✓ Enhanced Filter"), false, false, true)')
content = content.replace('("オフ", true, false, false)', '(t("オフ", "Off"), true, false, false)')


with open('src/tray.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("Patch applied to src/tray.rs")

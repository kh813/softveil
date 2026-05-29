import re

with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# import 追加
if "use crate::i18n::t;" not in content:
    content = content.replace('use std::sync::mpsc;', 'use std::sync::mpsc;\nuse crate::i18n::t;')

# GPUエラー
gpu_err_jp = 'GPUの初期化に失敗しました。グラフィックドライバーが最新であることを確認してください。'
gpu_err_en = 'Failed to initialize GPU. Please ensure your graphics drivers are up to date.'
content = content.replace(
    'let msg = "GPUの初期化に失敗しました。グラフィックドライバーが最新であることを確認してください。\\nFailed to initialize GPU. Please ensure your graphics drivers are up to date.";',
    f'let msg = t("{gpu_err_jp}", "{gpu_err_en}");'
)
content = content.replace(
    'platform::show_error_dialog("Softveil Error", msg);',
    'platform::show_error_dialog(t("Softveil エラー", "Softveil Error"), msg);'
)

# ホットキー警告
hotkey_warn_jp = 'グローバルショートカット（Ctrl+Shift+P）の登録に失敗しました。他のアプリと競合している可能性があります。'
hotkey_warn_en = 'Failed to register global hotkey (Ctrl+Shift+P). It might be in use by another application.'
content = content.replace(
    'platform::show_error_dialog(\n                "Softveil Warning",\n                "グローバルショートカット（Ctrl+Shift+P）の登録に失敗しました。他のアプリと競合している可能性があります。\\nFailed to register global hotkey (Ctrl+Shift+P). It might be in use by another application."\n            );',
    f'platform::show_error_dialog(\n                t("Softveil 警告", "Softveil Warning"),\n                t("{hotkey_warn_jp}", "{hotkey_warn_en}")\n            );'
)

# 画面収録許可
screen_cap_jp_title = '「画面収録」の許可が必要です'
screen_cap_en_title = 'Screen Recording Permission Required'
screen_cap_jp_body = 'ベンチマーク機能には画面収録の権限が必要です。\\n\\nシステム設定 > プライバシーとセキュリティ > 画面収録\\nで Softveil を許可してから再試行してください。'
screen_cap_en_body = 'Benchmark requires Screen Recording permission.\\n\\nPlease allow Softveil in System Settings > Privacy & Security > Screen Recording, and try again.'
content = content.replace(
    'platform::show_error_dialog(\n                                "「画面収録」の許可が必要です",\n                                "ベンチマーク機能には画面収録の権限が必要です。\\n\\nシステム設定 > プライバシーとセキュリティ > 画面収録\\nで Softveil を許可してから再試行してください。",\n                            );',
    f'platform::show_error_dialog(\n                                t("{screen_cap_jp_title}", "{screen_cap_en_title}"),\n                                t("{screen_cap_jp_body}", "{screen_cap_en_body}"),\n                            );'
)

# 通知: ベンチマーク開始
content = content.replace(
    'platform::send_notification("Softveil", "ベンチマーク開始", "画面の最適化測定を開始しました。完了までしばらくお待ちください。");',
    'platform::send_notification("Softveil", t("ベンチマーク開始", "Benchmark Started"), t("画面の最適化測定を開始しました。完了までしばらくお待ちください。", "Optimization process started. Please wait."));'
)

# 通知: 最適化進行中
content = content.replace(
    'platform::send_notification("Softveil", "最適化進行中", &format!("進捗: {:.0}%", progress * 100.0));',
    'platform::send_notification("Softveil", t("最適化進行中", "Optimizing"), &format!("{}: {:.0}%", t("進捗", "Progress"), progress * 100.0));'
)

# 通知: 最適化完了
content = content.replace(
    'platform::send_notification("Softveil", "最適化完了", "性能測定が完了しました。");',
    'platform::send_notification("Softveil", t("最適化完了", "Optimization Complete"), t("性能測定が完了しました。", "Performance benchmark completed."));'
)

# ダイアログ: 最適化完了
dialog_title_jp = '最適化完了'
dialog_title_en = 'Optimization Complete'
dialog_body_jp = '性能測定と最適化が完了しました。\\n\\n【結果の要約】\\n{}\\n\\n外出先での利用に最適な「Transit (Maximum)」プロファイルを全画面に自動適用しました。'
dialog_body_en = 'Benchmark and optimization complete.\\n\\n[Summary]\\n{}\\n\\nThe "Transit (Maximum)" profile, ideal for outdoor use, has been automatically applied to all screens.'
content = content.replace(
    'crate::platform::show_info_dialog(\n                    "最適化完了 / Optimization Complete",\n                    &format!("性能測定と最適化が完了しました。\\n\\n【結果の要約】\\n{}\\n\\n外出先での利用に最適な「Transit (Maximum)」プロファイルを全画面に自動適用しました。", summary)\n                );',
    f'crate::platform::show_info_dialog(\n                    t("{dialog_title_jp}", "{dialog_title_en}"),\n                    &format!(t("{dialog_body_jp}", "{dialog_body_en}"), summary)\n                );'
)

# format で t() を利用する際にコンパイルエラーになるケースを避けるため、format! に t の戻り値を渡すようにする
# Rustのformat!マクロの第一引数はリテラルでなければならない
# なので、&format!(t("{dialog_body_jp}", "{dialog_body_en}"), summary) はコンパイルエラーになる。
# 代わりに、format!() のプレースホルダを一つ持つようにするか、
# format!("{}", t("...", "...").replace("{}", summary)) は面倒。
# 日本語と英語で分けて format! を呼ぶ方が簡単。
# => パッチ修正

content = content.replace(
    f'crate::platform::show_info_dialog(\n                    t("{dialog_title_jp}", "{dialog_title_en}"),\n                    &format!(t("{dialog_body_jp}", "{dialog_body_en}"), summary)\n                );',
    f'''let body = if crate::i18n::get_language() == crate::i18n::Language::Ja {{
                    format!("{dialog_body_jp}", summary)
                }} else {{
                    format!("{dialog_body_en}", summary)
                }};
                crate::platform::show_info_dialog(t("{dialog_title_jp}", "{dialog_title_en}"), &body);'''
)

with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("Patch applied to src/main.rs")

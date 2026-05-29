use std::sync::OnceLock;
use sys_locale::get_locale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Ja,
    En,
}

static LANGUAGE: OnceLock<Language> = OnceLock::new();

pub fn get_language() -> Language {
    *LANGUAGE.get_or_init(|| {
        let locale = get_locale().unwrap_or_else(|| String::from("en-US"));
        // 日本語ロケール("ja", "ja-JP"など)を判定。それ以外は英語をデフォルトとする。
        if locale.starts_with("ja") {
            Language::Ja
        } else {
            Language::En
        }
    })
}

/// 日本語と英語の文字列を引数に取り、現在のシステムロケールに応じた文字列を返します。
pub fn t<'a>(ja: &'a str, en: &'a str) -> &'a str {
    match get_language() {
        Language::Ja => ja,
        Language::En => en,
    }
}

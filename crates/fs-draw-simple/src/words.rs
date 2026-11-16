//! The words of the window, English and German (ADR 0065: nothing is shown in one language only). The choice follows
//! `LC_ALL`, `LC_MESSAGES` and `LANG`; `--lang de|en` overrides it.
//!
//! Pattern: none — one function per sentence, no formatting machinery for the basics.

/// Whether the environment asks for German.
#[must_use]
pub fn env_is_german() -> bool {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
        .is_some_and(|v| v.to_ascii_lowercase().starts_with("de"))
}

/// `en` or `de`, as asked.
#[must_use]
pub fn pick(german: bool, en: &'static str, de: &'static str) -> &'static str {
    if german { de } else { en }
}

/// The name of the program, shown in the title and the heading.
pub const NAME: &str = "FreeSynergy Draw";

//! Minimal runtime internationalisation.
//!
//! English is the default; French is opt-in via `language: fr` in the config.
//! The active language is a process-wide setting applied once at startup from
//! the loaded config, so every thread (CLI or daemon task) renders alike.

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

/// The languages Joséphine can speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    En,
    Fr,
}

static CURRENT: AtomicU8 = AtomicU8::new(Lang::En as u8);

/// Apply the active language process-wide (called once at startup).
pub fn set_lang(lang: Lang) {
    CURRENT.store(lang as u8, Ordering::Relaxed);
}

/// The active language (English until [`set_lang`] is called).
pub fn lang() -> Lang {
    if CURRENT.load(Ordering::Relaxed) == Lang::Fr as u8 {
        Lang::Fr
    } else {
        Lang::En
    }
}

/// Choose between an English and a French string literal for the active language.
pub fn t(en: &'static str, fr: &'static str) -> &'static str {
    match lang() {
        Lang::En => en,
        Lang::Fr => fr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_uses_lowercase_codes() {
        assert_eq!(serde_yaml::to_string(&Lang::Fr).unwrap().trim(), "fr");
        assert_eq!(serde_yaml::from_str::<Lang>("en").unwrap(), Lang::En);
    }

    #[test]
    fn defaults_to_english() {
        // No test sets the global language, so it stays at its English default.
        assert_eq!(Lang::default(), Lang::En);
        assert_eq!(t("hello", "bonjour"), "hello");
    }
}

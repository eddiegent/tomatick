//! Translations for the text the Rust side produces: the tray menu, the
//! notifications and the window titles (SPEC.md section 14).
//!
//! The text comes from the same files as the interface: `/locales/*.json`,
//! compiled in. This module does not depend on Tauri.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::Value;

use crate::engine::AlertEvent;

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

/// The setting value that means "use the language of the system".
pub const SYSTEM: &str = "system";
/// The language used when nothing else matches, and for missing keys.
pub const FALLBACK: &str = "en";

fn catalogs() -> &'static HashMap<&'static str, Value> {
    static CATALOGS: OnceLock<HashMap<&'static str, Value>> = OnceLock::new();
    CATALOGS.get_or_init(|| {
        LOCALES
            .iter()
            .filter_map(|(code, text)| {
                serde_json::from_str(text)
                    .map_err(|error| eprintln!("Tomatick: locale {code} is not valid: {error}"))
                    .ok()
                    .map(|value| (*code, value))
            })
            .collect()
    })
}

/// The code of a language that ships with the app, if `code` is one.
fn known(code: &str) -> Option<&'static str> {
    LOCALES
        .iter()
        .map(|(known, _)| *known)
        .find(|known| *known == code)
}

pub fn is_supported(code: &str) -> bool {
    known(code).is_some()
}

/// The language to use for a setting, "system" or a language code. For
/// "system", `system_locale` is the locale of the operating system, such as
/// `fr-FR` or `fr_FR.UTF-8`.
pub fn resolve(setting: &str, system_locale: Option<&str>) -> &'static str {
    if setting != SYSTEM
        && let Some(code) = known(setting)
    {
        return code;
    }
    system_locale
        .and_then(|locale| {
            let primary = locale
                .split(['-', '_', '.', '@'])
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            known(&primary)
        })
        .unwrap_or(FALLBACK)
}

fn lookup<'a>(catalog: &'a Value, key: &str) -> Option<&'a str> {
    key.split('.')
        .try_fold(catalog, |node, part| node.get(part))?
        .as_str()
}

/// The text for a key such as `tray.quit`. A key missing from the language
/// falls back to English, and then to the key itself.
pub fn translate(language: &str, key: &str) -> String {
    let catalogs = catalogs();
    [language, FALLBACK]
        .iter()
        .find_map(|code| catalogs.get(code).and_then(|catalog| lookup(catalog, key)))
        .unwrap_or(key)
        .to_string()
}

/// Like [`translate`], replacing each `{{name}}` by its value.
pub fn translate_with(language: &str, key: &str, values: &[(&str, &str)]) -> String {
    let mut text = translate(language, key);
    for (name, value) in values {
        text = text.replace(&format!("{{{{{name}}}}}"), value);
    }
    text
}

/// The notification text of an event. `label` is the exercise of the break
/// that starts, when there is one.
pub fn notification_text(
    language: &str,
    event: AlertEvent,
    label: Option<&str>,
) -> (String, String) {
    let name = match event {
        AlertEvent::WorkEnd => "workEnd",
        AlertEvent::ShortBreakEnd => "shortBreakEnd",
        AlertEvent::LongBreakEnd => "longBreakEnd",
        AlertEvent::CountdownEnd => "countdownEnd",
    };
    let title = translate(language, &format!("notification.{name}.title"));
    let body = match label {
        Some(label) if event == AlertEvent::WorkEnd => label.to_string(),
        _ => translate(language, &format!("notification.{name}.body")),
    };
    (title, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every key path of a catalog, and the `{{names}}` its text uses.
    fn leaves(value: &Value, path: &str, out: &mut Vec<(String, Vec<String>)>) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    let next = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    leaves(child, &next, out);
                }
            }
            Value::String(text) => {
                let mut names: Vec<String> = text
                    .split("{{")
                    .skip(1)
                    .filter_map(|rest| rest.split("}}").next())
                    .map(str::to_string)
                    .collect();
                names.sort();
                out.push((path.to_string(), names));
            }
            _ => panic!("{path} is neither text nor a group"),
        }
    }

    /// i18next adds a plural suffix to a key. The base key is what English
    /// must have too.
    fn base_key(key: &str) -> &str {
        for suffix in ["_zero", "_one", "_two", "_few", "_many", "_other"] {
            if let Some(base) = key.strip_suffix(suffix) {
                return base;
            }
        }
        key
    }

    #[test]
    fn english_and_french_ship_with_the_app() {
        assert!(is_supported("en"));
        assert!(is_supported("fr"));
        assert!(!is_supported("xx"));
    }

    #[test]
    fn every_language_has_the_keys_of_english_with_the_same_placeholders() {
        let english = catalogs().get(FALLBACK).expect("English ships");
        let mut reference = Vec::new();
        leaves(english, "", &mut reference);
        for (code, catalog) in catalogs() {
            let mut found = Vec::new();
            leaves(catalog, "", &mut found);
            for (key, names) in &reference {
                if base_key(key) != key {
                    continue;
                }
                let theirs = found
                    .iter()
                    .find(|(candidate, _)| candidate == key)
                    .unwrap_or_else(|| panic!("{code} lacks {key}"));
                assert_eq!(&theirs.1, names, "{code}: placeholders of {key}");
            }
            for (key, _) in &found {
                assert!(
                    reference.iter().any(|(candidate, _)| candidate == key)
                        || reference
                            .iter()
                            .any(|(candidate, _)| base_key(candidate) == base_key(key)),
                    "{code} has {key}, which English lacks"
                );
            }
            let name = lookup(catalog, "language.name");
            assert!(
                name.is_some_and(|name| !name.is_empty()),
                "{code} has no name"
            );
        }
    }

    #[test]
    fn the_language_follows_the_setting_then_the_system() {
        assert_eq!(resolve("fr", Some("en-US")), "fr");
        assert_eq!(resolve("system", Some("fr-FR")), "fr");
        assert_eq!(resolve("system", Some("fr_CA.UTF-8")), "fr");
        assert_eq!(resolve("system", Some("de-DE")), "en");
        assert_eq!(resolve("system", None), "en");
        assert_eq!(resolve("xx", Some("fr-FR")), "fr");
    }

    #[test]
    fn missing_keys_fall_back_to_english_and_then_to_the_key() {
        assert_eq!(translate("fr", "tray.quit"), "Quitter Tomatick");
        assert_eq!(translate("xx", "tray.quit"), "Quit Tomatick");
        assert_eq!(translate("fr", "no.such.key"), "no.such.key");
        assert_eq!(translate("fr", "tray"), "tray", "a group is not text");
    }

    #[test]
    fn placeholders_are_replaced() {
        assert_eq!(
            translate_with("en", "overlay.part", &[("index", "2"), ("count", "3")]),
            "Part 2 of 3"
        );
    }

    #[test]
    fn the_notification_carries_the_break_label() {
        let (title, body) = notification_text("en", AlertEvent::WorkEnd, Some("Planks (30 s)"));
        assert_eq!(title, "Time for a break");
        assert_eq!(body, "Planks (30 s)");

        let (_, body) = notification_text("en", AlertEvent::WorkEnd, None);
        assert_eq!(body, "Your work session is over.");

        let (title, _) = notification_text("fr", AlertEvent::CountdownEnd, None);
        assert_eq!(title, "Temps écoulé");

        // Only the end of work carries an exercise.
        let (_, body) = notification_text("en", AlertEvent::ShortBreakEnd, Some("x"));
        assert_eq!(body, "Back to work.");
    }
}

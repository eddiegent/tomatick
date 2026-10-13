//! Global shortcut settings (SPEC.md section 13.3).
//!
//! This module only checks and normalizes the key combinations. Registering
//! them with the system is done by the host. A combination is written as
//! modifiers and one key joined by `+`, for example `CommandOrControl+Alt+P`.
//! `CommandOrControl` is Ctrl on Windows and Linux and Cmd on macOS.

use std::fmt;

use serde::{Deserialize, Serialize};

/// What a shortcut does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutAction {
    StartPause,
    Skip,
    ToggleWidget,
}

/// The three shortcuts. An empty text means the shortcut is not bound.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutSettings {
    pub start_pause: String,
    pub skip: String,
    pub toggle_widget: String,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            start_pause: "CommandOrControl+Alt+P".to_string(),
            skip: "CommandOrControl+Alt+N".to_string(),
            toggle_widget: "CommandOrControl+Alt+T".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ShortcutError {
    /// No key besides the modifiers.
    NoKey(String),
    /// A global shortcut without a modifier would swallow a normal key.
    NoModifier(String),
    /// More than one key, a repeated modifier, or an unknown key name.
    Invalid(String),
    /// Two actions share the same combination.
    Duplicate(String),
}

impl fmt::Display for ShortcutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShortcutError::NoKey(text) => write!(f, "shortcut {text:?} has no key"),
            ShortcutError::NoModifier(text) => {
                write!(f, "shortcut {text:?} needs at least one modifier key")
            }
            ShortcutError::Invalid(text) => write!(f, "shortcut {text:?} is not valid"),
            ShortcutError::Duplicate(text) => {
                write!(f, "shortcut {text:?} is used for more than one action")
            }
        }
    }
}

impl std::error::Error for ShortcutError {}

/// The modifiers in the order they are written.
const MODIFIERS: [&str; 5] = ["CommandOrControl", "Control", "Super", "Alt", "Shift"];

fn canonical_modifier(token: &str) -> Option<&'static str> {
    match token.to_ascii_lowercase().as_str() {
        "commandorcontrol" | "cmdorctrl" | "commandorctrl" | "cmdorcontrol" => {
            Some("CommandOrControl")
        }
        "control" | "ctrl" => Some("Control"),
        "alt" | "option" => Some("Alt"),
        "shift" => Some("Shift"),
        "super" | "meta" | "command" | "cmd" | "win" => Some("Super"),
        _ => None,
    }
}

fn canonical_key(token: &str) -> Option<String> {
    let mut chars = token.chars();
    let first = chars.next()?;
    if chars.next().is_none() {
        return (first.is_ascii_alphanumeric()).then(|| first.to_ascii_uppercase().to_string());
    }
    let lower = token.to_ascii_lowercase();
    if let Some(number) = lower.strip_prefix('f')
        && let Ok(number) = number.parse::<u8>()
        && (1..=24).contains(&number)
    {
        return Some(format!("F{number}"));
    }
    let named = match lower.as_str() {
        "space" => "Space",
        "enter" | "return" => "Enter",
        "tab" => "Tab",
        "up" | "arrowup" => "Up",
        "down" | "arrowdown" => "Down",
        "left" | "arrowleft" => "Left",
        "right" | "arrowright" => "Right",
        "home" => "Home",
        "end" => "End",
        "pageup" => "PageUp",
        "pagedown" => "PageDown",
        "insert" => "Insert",
        "delete" => "Delete",
        _ => return None,
    };
    Some(named.to_string())
}

/// Checks one combination and returns it in its canonical spelling, so two
/// spellings of the same combination compare equal. Empty stays empty.
pub fn normalize(text: &str) -> Result<String, ShortcutError> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(String::new());
    }
    let mut modifiers: Vec<&'static str> = Vec::new();
    let mut key: Option<String> = None;
    for token in text.split('+').map(str::trim) {
        if let Some(modifier) = canonical_modifier(token) {
            if modifiers.contains(&modifier) {
                return Err(ShortcutError::Invalid(text.to_string()));
            }
            modifiers.push(modifier);
        } else if key.is_none() {
            key =
                Some(canonical_key(token).ok_or_else(|| ShortcutError::Invalid(text.to_string()))?);
        } else {
            return Err(ShortcutError::Invalid(text.to_string()));
        }
    }
    let key = key.ok_or_else(|| ShortcutError::NoKey(text.to_string()))?;
    if modifiers.is_empty() {
        return Err(ShortcutError::NoModifier(text.to_string()));
    }
    modifiers.sort_by_key(|modifier| MODIFIERS.iter().position(|m| m == modifier));
    let mut parts: Vec<&str> = modifiers;
    parts.push(&key);
    Ok(parts.join("+"))
}

impl ShortcutSettings {
    /// The three combinations with their actions.
    pub fn bindings(&self) -> [(ShortcutAction, &str); 3] {
        [
            (ShortcutAction::StartPause, &self.start_pause),
            (ShortcutAction::Skip, &self.skip),
            (ShortcutAction::ToggleWidget, &self.toggle_widget),
        ]
    }

    /// The settings with every combination normalized. Fails if one is not
    /// valid or if two actions share a combination.
    pub fn normalized(&self) -> Result<Self, ShortcutError> {
        let result = Self {
            start_pause: normalize(&self.start_pause)?,
            skip: normalize(&self.skip)?,
            toggle_widget: normalize(&self.toggle_widget)?,
        };
        let bound: Vec<&str> = result
            .bindings()
            .iter()
            .map(|(_, text)| *text)
            .filter(|text| !text.is_empty())
            .collect();
        for (index, text) in bound.iter().enumerate() {
            if bound[..index].contains(text) {
                return Err(ShortcutError::Duplicate((*text).to_string()));
            }
        }
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), ShortcutError> {
        self.normalized().map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_valid_and_already_canonical() {
        let defaults = ShortcutSettings::default();
        assert_eq!(defaults.normalized().unwrap(), defaults);
    }

    #[test]
    fn spellings_are_made_canonical() {
        assert_eq!(normalize("ctrl+alt+p").unwrap(), "Control+Alt+P");
        assert_eq!(normalize("Alt + Shift + f5").unwrap(), "Alt+Shift+F5");
        assert_eq!(
            normalize("Shift+Alt+Ctrl+Space").unwrap(),
            "Control+Alt+Shift+Space"
        );
        assert_eq!(normalize("cmd+arrowup").unwrap(), "Super+Up");
        assert_eq!(normalize("CmdOrCtrl+1").unwrap(), "CommandOrControl+1");
    }

    #[test]
    fn empty_means_not_bound() {
        assert_eq!(normalize("").unwrap(), "");
        assert_eq!(normalize("   ").unwrap(), "");
    }

    #[test]
    fn bad_combinations_are_refused() {
        assert!(matches!(
            normalize("Ctrl+Alt"),
            Err(ShortcutError::NoKey(_))
        ));
        assert!(matches!(normalize("P"), Err(ShortcutError::NoModifier(_))));
        assert!(matches!(
            normalize("Ctrl+P+Q"),
            Err(ShortcutError::Invalid(_))
        ));
        assert!(matches!(
            normalize("Ctrl+Ctrl+P"),
            Err(ShortcutError::Invalid(_))
        ));
        assert!(matches!(
            normalize("Ctrl+Banana"),
            Err(ShortcutError::Invalid(_))
        ));
        assert!(matches!(
            normalize("Ctrl+F25"),
            Err(ShortcutError::Invalid(_))
        ));
        assert!(matches!(
            normalize("Ctrl++"),
            Err(ShortcutError::Invalid(_))
        ));
    }

    #[test]
    fn two_actions_cannot_share_a_combination() {
        let settings = ShortcutSettings {
            skip: "ctrl+alt+p".to_string(),
            start_pause: "Alt+Control+P".to_string(),
            ..ShortcutSettings::default()
        };
        assert!(matches!(
            settings.normalized(),
            Err(ShortcutError::Duplicate(_))
        ));
    }

    #[test]
    fn several_unbound_actions_are_fine() {
        let settings = ShortcutSettings {
            start_pause: String::new(),
            skip: String::new(),
            toggle_widget: "Ctrl+Alt+T".to_string(),
        };
        assert!(settings.validate().is_ok());
    }
}

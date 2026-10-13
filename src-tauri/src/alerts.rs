//! Alert settings and custom sound files (SPEC.md section 8).
//!
//! This module does not depend on Tauri, so it is easy to test. What happens
//! when an alert fires (notification, bring to front) lives in `lib.rs`; the
//! sound itself is played by the widget window.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::AlertEvent;

/// The sounds that ship with the app. The audio files are in
/// `public/sounds/<id>.wav`, made by `tools/generate_sounds.py`.
pub const BUILTIN_SOUNDS: &[&str] = &["chime", "bell", "beep", "marimba"];

/// Audio formats a user-supplied sound can have.
pub const SOUND_EXTENSIONS: &[&str] = &["mp3", "wav", "ogg"];

/// Largest custom sound that is accepted, in bytes.
pub const MAX_SOUND_BYTES: u64 = 5 * 1024 * 1024;

/// Longest file name kept for a custom sound, extension included.
const MAX_SOUND_NAME_CHARS: usize = 48;

/// Folder, inside the app data folder, that holds the custom sounds.
pub const SOUNDS_DIR: &str = "sounds";

pub const DEFAULT_VOLUME: u8 = 70;

#[derive(Debug, PartialEq, Eq)]
pub enum AlertError {
    UnknownBuiltin(String),
    /// A custom sound name that is not a plain file name.
    BadSoundName(String),
    BadExtension,
    TooLarge,
    VolumeOutOfRange(u8),
    Io(String),
}

impl fmt::Display for AlertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertError::UnknownBuiltin(id) => write!(f, "unknown built-in sound \"{id}\""),
            AlertError::BadSoundName(name) => write!(f, "invalid sound file name \"{name}\""),
            AlertError::BadExtension => {
                write!(f, "the sound must be an MP3, WAV or OGG file")
            }
            AlertError::TooLarge => write!(
                f,
                "the sound file is larger than {} MB",
                MAX_SOUND_BYTES / (1024 * 1024)
            ),
            AlertError::VolumeOutOfRange(value) => {
                write!(f, "volume must be between 0 and 100, got {value}")
            }
            AlertError::Io(message) => write!(f, "cannot read or copy the sound: {message}"),
        }
    }
}

impl std::error::Error for AlertError {}

/// Which sound an event plays.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SoundChoice {
    /// One of [`BUILTIN_SOUNDS`].
    Builtin(String),
    /// A file name inside the custom sounds folder.
    Custom(String),
}

impl SoundChoice {
    pub fn validate(&self) -> Result<(), AlertError> {
        match self {
            SoundChoice::Builtin(id) => {
                if BUILTIN_SOUNDS.contains(&id.as_str()) {
                    Ok(())
                } else {
                    Err(AlertError::UnknownBuiltin(id.clone()))
                }
            }
            SoundChoice::Custom(name) => check_sound_name(name),
        }
    }
}

/// What happens for one alert event. Each type is independent (section 8.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EventAlerts {
    /// A native OS notification, with the break label when there is one.
    pub notification: bool,
    pub sound_enabled: bool,
    pub sound: SoundChoice,
    /// The full-screen overlay. Stored now, shown from the overlay milestone.
    pub overlay: bool,
    /// Raise and focus the widget window.
    pub bring_to_front: bool,
}

impl EventAlerts {
    fn new(sound: &str, overlay: bool) -> Self {
        Self {
            notification: true,
            sound_enabled: true,
            sound: SoundChoice::Builtin(sound.to_string()),
            overlay,
            bring_to_front: false,
        }
    }
}

impl Default for EventAlerts {
    fn default() -> Self {
        Self::new("chime", false)
    }
}

/// The alert settings of the whole app.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertSettings {
    /// A work phase ended, so a break starts.
    pub work_end: EventAlerts,
    /// A short break ended, so work starts.
    pub short_break_end: EventAlerts,
    /// A long break ended, so work starts.
    pub long_break_end: EventAlerts,
    pub countdown_end: EventAlerts,
    /// Volume of every alert sound, 0 to 100. Independent of the system volume.
    pub volume: u8,
    /// Silences every alert sound without losing the volume.
    pub muted: bool,
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            // The overlay is on where a break starts and for the countdown.
            work_end: EventAlerts::new("bell", true),
            short_break_end: EventAlerts::new("chime", false),
            long_break_end: EventAlerts::new("marimba", false),
            countdown_end: EventAlerts::new("beep", true),
            volume: DEFAULT_VOLUME,
            muted: false,
        }
    }
}

impl AlertSettings {
    pub fn validate(&self) -> Result<(), AlertError> {
        if self.volume > 100 {
            return Err(AlertError::VolumeOutOfRange(self.volume));
        }
        for alerts in [
            &self.work_end,
            &self.short_break_end,
            &self.long_break_end,
            &self.countdown_end,
        ] {
            alerts.sound.validate()?;
        }
        Ok(())
    }

    pub fn for_event(&self, event: AlertEvent) -> &EventAlerts {
        match event {
            AlertEvent::WorkEnd => &self.work_end,
            AlertEvent::ShortBreakEnd => &self.short_break_end,
            AlertEvent::LongBreakEnd => &self.long_break_end,
            AlertEvent::CountdownEnd => &self.countdown_end,
        }
    }
}

fn extension_of(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_lowercase)
}

/// A custom sound is referred to by a plain file name, never a path, so a
/// hand-edited database cannot point outside the sounds folder.
pub fn check_sound_name(name: &str) -> Result<(), AlertError> {
    let plain = !name.is_empty()
        && name.chars().count() <= MAX_SOUND_NAME_CHARS
        && !name.starts_with('.')
        && !name.contains(['/', '\\', ':', '\0'])
        && name != "."
        && name != "..";
    if !plain {
        return Err(AlertError::BadSoundName(name.to_string()));
    }
    match extension_of(name) {
        Some(extension) if SOUND_EXTENSIONS.contains(&extension.as_str()) => Ok(()),
        _ => Err(AlertError::BadExtension),
    }
}

/// The MIME type of a custom sound, for the player.
pub fn sound_mime(name: &str) -> &'static str {
    match extension_of(name).as_deref() {
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        _ => "audio/wav",
    }
}

/// A safe file name made from the user's file: letters, digits, `-`, `_` and
/// spaces are kept, the rest becomes `_`. The extension is lower case.
fn clean_name(source: &Path, extension: &str) -> String {
    let stem: String = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("sound")
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = stem.trim().trim_start_matches('.');
    let budget = MAX_SOUND_NAME_CHARS - extension.chars().count() - 1;
    let stem: String = stem.chars().take(budget.saturating_sub(4)).collect();
    let stem = if stem.is_empty() { "sound" } else { &stem };
    format!("{stem}.{extension}")
}

/// Copies a user-supplied audio file into `dir` and returns the name it was
/// stored under. The original can then move or disappear.
pub fn import_sound(source: &Path, dir: &Path) -> Result<String, AlertError> {
    let io_error = |error: std::io::Error| AlertError::Io(error.to_string());

    let extension = source
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(extension_of)
        .filter(|extension| SOUND_EXTENSIONS.contains(&extension.as_str()))
        .ok_or(AlertError::BadExtension)?;

    let metadata = fs::metadata(source).map_err(io_error)?;
    if !metadata.is_file() {
        return Err(AlertError::Io("not a file".to_string()));
    }
    if metadata.len() > MAX_SOUND_BYTES {
        return Err(AlertError::TooLarge);
    }

    fs::create_dir_all(dir).map_err(io_error)?;

    // Never overwrite a sound that is already stored: add a number instead.
    let name = clean_name(source, &extension);
    let (stem, _) = name.rsplit_once('.').unwrap_or((&name, ""));
    let mut candidate = name.clone();
    let mut number = 2;
    while dir.join(&candidate).exists() {
        candidate = format!("{stem} {number}.{extension}");
        number += 1;
    }

    fs::copy(source, dir.join(&candidate)).map_err(io_error)?;
    check_sound_name(&candidate)?;
    Ok(candidate)
}

/// The path of a stored custom sound, after checking the name.
pub fn sound_path(dir: &Path, name: &str) -> Result<PathBuf, AlertError> {
    check_sound_name(name)?;
    Ok(dir.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh empty folder under the system temp folder.
    fn scratch(test: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tomatick-{}-{test}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn defaults_are_valid_and_follow_the_spec() {
        let settings = AlertSettings::default();
        assert_eq!(settings.validate(), Ok(()));
        for event in [
            AlertEvent::WorkEnd,
            AlertEvent::ShortBreakEnd,
            AlertEvent::LongBreakEnd,
            AlertEvent::CountdownEnd,
        ] {
            let alerts = settings.for_event(event);
            assert!(alerts.notification && alerts.sound_enabled);
            assert!(!alerts.bring_to_front);
        }
        // The overlay is for the start of a break and for the countdown.
        assert!(settings.work_end.overlay);
        assert!(settings.countdown_end.overlay);
        assert!(!settings.short_break_end.overlay);
        assert!(!settings.long_break_end.overlay);
        assert_eq!(settings.volume, DEFAULT_VOLUME);
        assert!(!settings.muted);
    }

    #[test]
    fn every_builtin_sound_is_known() {
        for id in BUILTIN_SOUNDS {
            assert_eq!(SoundChoice::Builtin((*id).to_string()).validate(), Ok(()));
        }
        assert_eq!(
            SoundChoice::Builtin("gong".to_string()).validate(),
            Err(AlertError::UnknownBuiltin("gong".to_string()))
        );
    }

    #[test]
    fn volume_above_100_is_refused() {
        let settings = AlertSettings {
            volume: 101,
            ..AlertSettings::default()
        };
        assert_eq!(settings.validate(), Err(AlertError::VolumeOutOfRange(101)));
    }

    #[test]
    fn older_settings_without_alerts_still_load() {
        let settings: AlertSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings, AlertSettings::default());

        // A partly filled block keeps the defaults for the rest.
        let settings: AlertSettings =
            serde_json::from_str(r#"{"volume": 30, "work_end": {"notification": false}}"#).unwrap();
        assert_eq!(settings.volume, 30);
        assert!(!settings.work_end.notification);
        assert!(settings.work_end.sound_enabled);
        assert_eq!(
            settings.short_break_end,
            AlertSettings::default().short_break_end
        );
    }

    #[test]
    fn the_json_shape_matches_the_frontend() {
        let json = serde_json::to_value(SoundChoice::Builtin("bell".to_string())).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "kind": "builtin", "value": "bell" })
        );
        let json = serde_json::to_value(SoundChoice::Custom("gong.mp3".to_string())).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "kind": "custom", "value": "gong.mp3" })
        );
    }

    #[test]
    fn custom_sound_names_must_be_plain_audio_files() {
        assert_eq!(check_sound_name("gong.mp3"), Ok(()));
        assert_eq!(check_sound_name("My Bell.OGG"), Ok(()));
        for bad in [
            "../gong.mp3",
            "dir/gong.mp3",
            "dir\\gong.mp3",
            ".hidden.mp3",
            "",
        ] {
            assert!(
                matches!(check_sound_name(bad), Err(AlertError::BadSoundName(_))),
                "{bad:?} should be refused"
            );
        }
        assert_eq!(check_sound_name("gong.exe"), Err(AlertError::BadExtension));
        assert_eq!(check_sound_name("gong"), Err(AlertError::BadExtension));
    }

    #[test]
    fn a_bad_custom_choice_fails_the_whole_validation() {
        let mut settings = AlertSettings::default();
        settings.work_end.sound = SoundChoice::Custom("../../etc/passwd.mp3".to_string());
        assert!(matches!(
            settings.validate(),
            Err(AlertError::BadSoundName(_))
        ));
    }

    #[test]
    fn importing_copies_the_file() {
        let root = scratch("import");
        let source = root.join("My gong!.MP3");
        fs::write(&source, b"fake audio").unwrap();
        let dir = root.join("sounds");

        let name = import_sound(&source, &dir).unwrap();
        assert_eq!(name, "My gong_.mp3");
        assert_eq!(fs::read(dir.join(&name)).unwrap(), b"fake audio");

        // The original can go away.
        fs::remove_file(&source).unwrap();
        assert!(dir.join(&name).exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn importing_never_overwrites_a_stored_sound() {
        let root = scratch("no-overwrite");
        let source = root.join("gong.wav");
        let dir = root.join("sounds");
        fs::write(&source, b"first").unwrap();
        let first = import_sound(&source, &dir).unwrap();
        fs::write(&source, b"second").unwrap();
        let second = import_sound(&source, &dir).unwrap();

        assert_eq!(first, "gong.wav");
        assert_eq!(second, "gong 2.wav");
        assert_eq!(fs::read(dir.join(&first)).unwrap(), b"first");
        assert_eq!(fs::read(dir.join(&second)).unwrap(), b"second");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn importing_refuses_other_formats_and_big_files() {
        let root = scratch("refuse");
        let dir = root.join("sounds");

        let text = root.join("notes.txt");
        fs::write(&text, b"hello").unwrap();
        assert_eq!(import_sound(&text, &dir), Err(AlertError::BadExtension));

        let big = root.join("big.wav");
        let file = fs::File::create(&big).unwrap();
        file.set_len(MAX_SOUND_BYTES + 1).unwrap();
        assert_eq!(import_sound(&big, &dir), Err(AlertError::TooLarge));

        assert!(matches!(
            import_sound(&root.join("missing.mp3"), &dir),
            Err(AlertError::Io(_))
        ));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn long_names_are_shortened_to_a_valid_name() {
        let root = scratch("long");
        let source = root.join(format!("{}.wav", "a".repeat(200)));
        fs::write(&source, b"x").unwrap();
        let name = import_sound(&source, &root.join("sounds")).unwrap();
        assert!(name.chars().count() <= MAX_SOUND_NAME_CHARS);
        assert_eq!(check_sound_name(&name), Ok(()));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn mime_types_follow_the_extension() {
        assert_eq!(sound_mime("a.mp3"), "audio/mpeg");
        assert_eq!(sound_mime("a.OGG"), "audio/ogg");
        assert_eq!(sound_mime("a.wav"), "audio/wav");
    }
}

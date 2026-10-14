//! The update check (SPEC.md section 16.2).
//!
//! Asks the GitHub Releases API for the latest release and tells the user
//! when it is newer than the running version. It never downloads or installs
//! anything, and it is the only network request the app makes. This module
//! does not depend on Tauri: the host (`lib.rs`) decides when to check and
//! what to do with the answer.

use std::fmt;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use semver::Version;
use serde::{Deserialize, Serialize};

/// The API address of the latest published release.
pub const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/eddiegent/tomatick/releases/latest";

/// The releases page, opened when the user wants the new version.
pub const RELEASES_PAGE: &str = "https://github.com/eddiegent/tomatick/releases";

/// How often the app checks while it runs.
pub const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// How long the app waits after launch before the first check, so that the
/// start of the app is not slowed down and the network has time to come up.
pub const FIRST_CHECK_DELAY: Duration = Duration::from_secs(20);

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateError {
    /// The request failed, or GitHub answered with an error.
    Network(String),
    /// The answer was not what the API documents.
    Answer(String),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateError::Network(message) => write!(f, "cannot reach GitHub: {message}"),
            UpdateError::Answer(message) => write!(f, "unexpected answer from GitHub: {message}"),
        }
    }
}

impl std::error::Error for UpdateError {}

/// A release that is newer than the running version.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Available {
    /// The version without the leading `v`, for example `0.2.0`.
    pub version: String,
    /// The page of that release.
    pub url: String,
}

/// What the app knows about updates, as shown in the settings and the tray.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UpdateStatus {
    /// The running version.
    pub current: String,
    pub available: Option<Available>,
    pub checking: bool,
    /// Unix seconds of the last check that got an answer.
    pub last_checked: Option<u64>,
    /// Why the last check failed. `None` after a check that worked.
    pub error: Option<String>,
}

/// The fields of the API answer that matter here.
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// The version in a tag such as `v0.2.0`. `None` if it is not a version.
pub fn parse_version(tag: &str) -> Option<Version> {
    Version::parse(tag.trim().trim_start_matches(['v', 'V'])).ok()
}

/// Reads the answer of the API. `Ok(None)` when the running version is the
/// latest or newer, and for a draft or a pre-release.
///
/// The link comes from the network, so it is only used when it points into
/// the project's releases. Otherwise the releases page is used.
pub fn evaluate(current: &str, body: &str) -> Result<Option<Available>, UpdateError> {
    let release: Release =
        serde_json::from_str(body).map_err(|error| UpdateError::Answer(error.to_string()))?;
    if release.draft || release.prerelease {
        return Ok(None);
    }
    let latest = parse_version(&release.tag_name)
        .ok_or_else(|| UpdateError::Answer(format!("`{}` is not a version", release.tag_name)))?;
    let running = parse_version(current).ok_or_else(|| {
        UpdateError::Answer(format!("the running version `{current}` is invalid"))
    })?;
    if latest <= running {
        return Ok(None);
    }
    let url = release
        .html_url
        .filter(|url| url == RELEASES_PAGE || url.starts_with(&format!("{RELEASES_PAGE}/")))
        .unwrap_or_else(|| RELEASES_PAGE.to_string());
    Ok(Some(Available {
        version: latest.to_string(),
        url,
    }))
}

/// Asks GitHub for the latest release and returns the text of its answer.
/// Blocks, so call it from a thread of its own.
///
/// A project with no published release answers 404. That is not an error: it
/// is reported as version 0.0.0, which is older than any running version.
pub fn fetch_latest() -> Result<String, UpdateError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .into();
    let result = agent
        .get(LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .header(
            "User-Agent",
            concat!("Tomatick/", env!("CARGO_PKG_VERSION")),
        )
        .call();
    match result {
        Ok(mut response) => response
            .body_mut()
            .read_to_string()
            .map_err(|error| UpdateError::Network(error.to_string())),
        Err(ureq::Error::StatusCode(404)) => Ok(r#"{"tag_name": "v0.0.0"}"#.to_string()),
        Err(error) => Err(UpdateError::Network(error.to_string())),
    }
}

/// The shared state of the update check.
pub struct Updater {
    status: Mutex<UpdateStatus>,
}

impl Updater {
    pub fn new(current: &str) -> Self {
        Self {
            status: Mutex::new(UpdateStatus {
                current: current.to_string(),
                available: None,
                checking: false,
                last_checked: None,
                error: None,
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, UpdateStatus> {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn status(&self) -> UpdateStatus {
        self.lock().clone()
    }

    /// Forgets a found update, for when the user turns the check off.
    pub fn clear(&self) -> UpdateStatus {
        let mut status = self.lock();
        status.available = None;
        status.error = None;
        status.clone()
    }

    /// Runs one check on the calling thread. `fetch` gets the answer of the
    /// API, and `now` is the current time in Unix seconds. Returns `None`,
    /// without doing anything, when a check is already running.
    pub fn check(
        &self,
        fetch: impl FnOnce() -> Result<String, UpdateError>,
        now: u64,
    ) -> Option<UpdateStatus> {
        let current = {
            let mut status = self.lock();
            if status.checking {
                return None;
            }
            status.checking = true;
            status.current.clone()
        };
        // The lock is not held while waiting for the network.
        let outcome = fetch().and_then(|body| evaluate(&current, &body));
        let mut status = self.lock();
        status.checking = false;
        match outcome {
            Ok(available) => {
                status.available = available;
                status.last_checked = Some(now);
                status.error = None;
            }
            // A failed check keeps what was known before.
            Err(error) => status.error = Some(error.to_string()),
        }
        Some(status.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, url: &str) -> String {
        format!(
            r#"{{"tag_name": "{tag}", "html_url": "{url}", "draft": false, "prerelease": false}}"#
        )
    }

    const PAGE_020: &str = "https://github.com/eddiegent/tomatick/releases/tag/v0.2.0";

    #[test]
    fn versions_are_read_with_or_without_the_v() {
        assert_eq!(parse_version("v0.2.0"), Some(Version::new(0, 2, 0)));
        assert_eq!(parse_version("0.2.0"), Some(Version::new(0, 2, 0)));
        assert_eq!(parse_version(" v1.10.3 "), Some(Version::new(1, 10, 3)));
        assert_eq!(parse_version("latest"), None);
        assert_eq!(parse_version("v1.2"), None);
    }

    #[test]
    fn a_newer_release_is_reported() {
        let found = evaluate("0.1.0", &release("v0.2.0", PAGE_020)).unwrap();
        assert_eq!(
            found,
            Some(Available {
                version: "0.2.0".to_string(),
                url: PAGE_020.to_string(),
            })
        );
    }

    #[test]
    fn versions_are_compared_as_numbers_not_text() {
        assert!(
            evaluate("0.9.0", &release("v0.10.0", PAGE_020))
                .unwrap()
                .is_some()
        );
        assert!(
            evaluate("0.10.0", &release("v0.9.0", PAGE_020))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn the_same_or_an_older_release_is_not_reported() {
        assert_eq!(
            evaluate("0.2.0", &release("v0.2.0", PAGE_020)).unwrap(),
            None
        );
        assert_eq!(
            evaluate("0.3.0", &release("v0.2.0", PAGE_020)).unwrap(),
            None
        );
    }

    #[test]
    fn a_draft_or_a_pre_release_is_ignored() {
        let draft = r#"{"tag_name": "v9.0.0", "draft": true}"#;
        let pre = r#"{"tag_name": "v9.0.0", "prerelease": true}"#;
        assert_eq!(evaluate("0.1.0", draft).unwrap(), None);
        assert_eq!(evaluate("0.1.0", pre).unwrap(), None);
    }

    #[test]
    fn a_link_outside_the_project_is_replaced_by_the_releases_page() {
        for url in [
            "https://evil.example/releases/tag/v0.2.0",
            "https://github.com/someone-else/tomatick/releases/tag/v0.2.0",
            "https://github.com/eddiegent/tomatick/releases-evil",
            "javascript:alert(1)",
        ] {
            let found = evaluate("0.1.0", &release("v0.2.0", url)).unwrap().unwrap();
            assert_eq!(found.url, RELEASES_PAGE, "for {url}");
        }
        let missing = r#"{"tag_name": "v0.2.0"}"#;
        assert_eq!(
            evaluate("0.1.0", missing).unwrap().unwrap().url,
            RELEASES_PAGE
        );
    }

    #[test]
    fn an_answer_that_makes_no_sense_is_an_error() {
        assert!(matches!(
            evaluate("0.1.0", "not json"),
            Err(UpdateError::Answer(_))
        ));
        assert!(matches!(
            evaluate("0.1.0", "{}"),
            Err(UpdateError::Answer(_))
        ));
        assert!(matches!(
            evaluate("0.1.0", r#"{"tag_name": "nightly"}"#),
            Err(UpdateError::Answer(_))
        ));
        assert!(matches!(
            evaluate("dev", &release("v0.2.0", PAGE_020)),
            Err(UpdateError::Answer(_))
        ));
    }

    #[test]
    fn a_project_with_no_release_is_up_to_date() {
        assert_eq!(
            evaluate("0.1.0", r#"{"tag_name": "v0.0.0"}"#).unwrap(),
            None
        );
    }

    #[test]
    fn a_check_records_what_it_found() {
        let updater = Updater::new("0.1.0");
        let status = updater
            .check(|| Ok(release("v0.2.0", PAGE_020)), 1_000)
            .unwrap();
        assert_eq!(
            status.available.as_ref().map(|a| a.version.as_str()),
            Some("0.2.0")
        );
        assert_eq!(status.last_checked, Some(1_000));
        assert!(!status.checking);
        assert_eq!(status.error, None);
        assert_eq!(updater.status(), status);
    }

    #[test]
    fn a_failed_check_keeps_the_earlier_answer() {
        let updater = Updater::new("0.1.0");
        updater.check(|| Ok(release("v0.2.0", PAGE_020)), 1_000);
        let status = updater
            .check(|| Err(UpdateError::Network("offline".to_string())), 2_000)
            .unwrap();
        assert!(status.available.is_some());
        assert_eq!(status.last_checked, Some(1_000));
        assert!(status.error.unwrap().contains("offline"));
    }

    #[test]
    fn a_later_check_can_clear_the_notice() {
        let updater = Updater::new("0.2.0");
        updater.check(|| Ok(release("v0.3.0", PAGE_020)), 1);
        let status = updater
            .check(|| Ok(release("v0.2.0", PAGE_020)), 2)
            .unwrap();
        assert_eq!(status.available, None);
    }

    #[test]
    fn a_check_does_not_start_while_another_runs() {
        let updater = Updater::new("0.1.0");
        let inner = updater.check(
            || {
                // The nested call stands for a second thread asking meanwhile.
                assert!(updater.status().checking);
                assert_eq!(updater.check(|| unreachable!(), 5), None);
                Ok(release("v0.1.0", PAGE_020))
            },
            10,
        );
        assert!(inner.is_some());
        assert!(!updater.status().checking);
    }

    #[test]
    fn turning_the_check_off_forgets_the_notice() {
        let updater = Updater::new("0.1.0");
        updater.check(|| Ok(release("v0.2.0", PAGE_020)), 1);
        let status = updater.clear();
        assert_eq!(status.available, None);
        assert_eq!(status.last_checked, Some(1));
    }
}

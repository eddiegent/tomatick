//! The full-screen overlay: which alerts open it, and what page it loads.
//!
//! The window itself is built in `lib.rs`. This module holds the parts that
//! need no window, so they can be tested.

use serde::{Deserialize, Serialize};

use crate::engine::AlertEvent;

/// Label of the overlay window.
pub const OVERLAY_WINDOW: &str = "overlay";

/// What the overlay shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayKind {
    /// A break has just started or is about to: Start, Skip, Extend, Dismiss.
    Break,
    /// The countdown reached zero: Restart, Dismiss.
    Countdown,
}

impl OverlayKind {
    /// The overlay that goes with an alert, if the alert has one. Only the end
    /// of a work phase and the end of the countdown do: the end of a break
    /// needs no full-screen reminder.
    pub fn for_event(event: AlertEvent) -> Option<Self> {
        match event {
            AlertEvent::WorkEnd => Some(Self::Break),
            AlertEvent::CountdownEnd => Some(Self::Countdown),
            AlertEvent::ShortBreakEnd | AlertEvent::LongBreakEnd => None,
        }
    }

    fn query_value(self) -> &'static str {
        match self {
            Self::Break => "break",
            Self::Countdown => "countdown",
        }
    }
}

/// The page the overlay window loads. The kind and the preview flag travel in
/// the query string so the page knows what to show from its first render.
pub fn overlay_url(kind: OverlayKind, preview: bool) -> String {
    let mut url = format!("index.html?overlay={}", kind.query_value());
    if preview {
        url.push_str("&preview=1");
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_work_end_and_countdown_end_have_an_overlay() {
        assert_eq!(
            OverlayKind::for_event(AlertEvent::WorkEnd),
            Some(OverlayKind::Break)
        );
        assert_eq!(
            OverlayKind::for_event(AlertEvent::CountdownEnd),
            Some(OverlayKind::Countdown)
        );
        assert_eq!(OverlayKind::for_event(AlertEvent::ShortBreakEnd), None);
        assert_eq!(OverlayKind::for_event(AlertEvent::LongBreakEnd), None);
    }

    #[test]
    fn the_url_carries_the_kind() {
        assert_eq!(
            overlay_url(OverlayKind::Break, false),
            "index.html?overlay=break"
        );
        assert_eq!(
            overlay_url(OverlayKind::Countdown, false),
            "index.html?overlay=countdown"
        );
    }

    #[test]
    fn a_preview_is_flagged() {
        assert_eq!(
            overlay_url(OverlayKind::Break, true),
            "index.html?overlay=break&preview=1"
        );
    }

    #[test]
    fn the_kind_uses_snake_case_on_the_wire() {
        assert_eq!(
            serde_json::to_string(&OverlayKind::Countdown).unwrap(),
            "\"countdown\""
        );
        let kind: OverlayKind = serde_json::from_str("\"break\"").unwrap();
        assert_eq!(kind, OverlayKind::Break);
    }
}

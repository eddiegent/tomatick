//! Keeping the widget reachable when a screen goes away.
//!
//! Pure geometry, in physical pixels, with no dependency on Tauri.

/// A rectangle on the desktop: top-left corner and size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn right(&self) -> i64 {
        i64::from(self.x) + i64::from(self.width)
    }

    fn bottom(&self) -> i64 {
        i64::from(self.y) + i64::from(self.height)
    }

    /// The center, doubled so it stays an integer.
    fn center2(&self) -> (i64, i64) {
        (
            i64::from(self.x) * 2 + i64::from(self.width),
            i64::from(self.y) * 2 + i64::from(self.height),
        )
    }

    fn contains_center_of(&self, other: &Rect) -> bool {
        let (cx2, cy2) = other.center2();
        cx2 >= i64::from(self.x) * 2
            && cx2 < self.right() * 2
            && cy2 >= i64::from(self.y) * 2
            && cy2 < self.bottom() * 2
    }
}

/// Where the widget has to move to be on a screen again, or `None` if it
/// already is. The widget counts as on a screen when its center is on one,
/// so it can still be grabbed. Otherwise it goes to the nearest screen,
/// moved as little as possible, and fully inside it when it fits.
pub fn reposition(widget: Rect, screens: &[Rect]) -> Option<(i32, i32)> {
    if screens.is_empty() || screens.iter().any(|s| s.contains_center_of(&widget)) {
        return None;
    }
    let (wx2, wy2) = widget.center2();
    let nearest = screens.iter().min_by_key(|screen| {
        let (sx2, sy2) = screen.center2();
        let (dx, dy) = (sx2 - wx2, sy2 - wy2);
        dx * dx + dy * dy
    })?;
    Some((
        clamp_axis(widget.x, widget.width, nearest.x, nearest.width),
        clamp_axis(widget.y, widget.height, nearest.y, nearest.height),
    ))
}

/// Moves `start` the least that puts `[start, start + length)` inside
/// `[low, low + span)`. If it does not fit, the start edges line up.
fn clamp_axis(start: i32, length: u32, low: i32, span: u32) -> i32 {
    let high_start = i64::from(low) + i64::from(span) - i64::from(length);
    let clamped = i64::from(start).min(high_start).max(i64::from(low));
    i32::try_from(clamped).unwrap_or(low)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAIN: Rect = Rect {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };
    const RIGHT: Rect = Rect {
        x: 1920,
        y: 0,
        width: 1920,
        height: 1080,
    };

    #[test]
    fn a_widget_on_a_screen_stays_where_it_is() {
        assert_eq!(reposition(Rect::new(100, 100, 200, 200), &[MAIN]), None);
        assert_eq!(reposition(Rect::new(1800, 100, 200, 200), &[MAIN]), None);
        assert_eq!(
            reposition(Rect::new(1800, 100, 200, 200), &[MAIN, RIGHT]),
            None
        );
    }

    #[test]
    fn a_widget_with_its_center_on_a_screen_can_still_be_grabbed() {
        // A fifth of it hangs off the bottom edge.
        assert_eq!(reposition(Rect::new(100, 900, 200, 200), &[MAIN]), None);
    }

    #[test]
    fn a_widget_on_an_unplugged_screen_moves_to_the_nearest_one() {
        // It sat on the right screen, which is gone.
        let moved = reposition(Rect::new(2500, 300, 200, 200), &[MAIN]);
        assert_eq!(moved, Some((1720, 300)));
    }

    #[test]
    fn it_goes_to_the_nearest_of_several_screens() {
        let left = Rect::new(-1920, 0, 1920, 1080);
        let far_up_right = Rect::new(5000, -400, 200, 200);
        assert_eq!(
            reposition(far_up_right, &[left, MAIN, RIGHT]),
            Some((3640, 0))
        );
        let far_left = Rect::new(-5000, 100, 200, 200);
        assert_eq!(
            reposition(far_left, &[left, MAIN, RIGHT]),
            Some((-1920, 100))
        );
    }

    #[test]
    fn a_widget_far_below_comes_back_inside() {
        let moved = reposition(Rect::new(300, 4000, 200, 200), &[MAIN]);
        assert_eq!(moved, Some((300, 880)));
    }

    #[test]
    fn a_screen_smaller_than_the_widget_lines_up_the_corner() {
        let tiny = Rect::new(0, 0, 100, 100);
        assert_eq!(
            reposition(Rect::new(900, 900, 200, 200), &[tiny]),
            Some((0, 0))
        );
    }

    #[test]
    fn no_screens_means_nothing_to_do() {
        assert_eq!(reposition(Rect::new(5000, 5000, 200, 200), &[]), None);
    }
}

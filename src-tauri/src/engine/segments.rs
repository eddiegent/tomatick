/// Splits a long break into short-break-sized segments.
///
/// The number of segments is `long / short` (at least 1) and any remainder is
/// added to the last segment. For example 20 min with 5 min segments gives
/// `[300, 300, 300, 300]`, and 17 min gives `[300, 300, 420]`.
pub fn segment_lengths_secs(long_secs: u32, short_secs: u32) -> Vec<u32> {
    if short_secs == 0 {
        return vec![long_secs];
    }
    let count = (long_secs / short_secs).max(1);
    let mut lengths = vec![short_secs; count as usize];
    if let Some(last) = lengths.last_mut() {
        *last = long_secs.saturating_sub(short_secs * (count - 1));
    }
    lengths
}

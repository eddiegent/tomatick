//! Phase history and statistics (SPEC.md sections 11 and 12).
//!
//! Every finished phase is stored in the `phases` table by [`Store::record_phase`].
//! The statistics are grouped queries over that table, in the user's local
//! time: timestamps are stored in UTC, and SQLite converts each one with the
//! `localtime` modifier, so a change of daylight saving time is handled for
//! every date on its own.
//!
//! This module does not depend on Tauri. The queries take the time zone
//! modifier and today's date as arguments so the tests do not depend on the
//! machine they run on.

use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::engine::{PhaseKind, PhaseRecord, PhaseStatus};
use crate::storage::{StorageError, Store};

/// Key of the phase that is running, in the `settings` table. It is written
/// when a phase starts and removed when the phase is recorded, so a phase
/// that was running when the app crashed can be recorded at the next start.
const OPEN_PHASE_KEY: &str = "open_phase";

/// The SQLite modifier that turns a UTC timestamp into the local time.
pub const LOCAL_TIME: &str = "localtime";

/// Hours in a day, the bars of the day view.
const HOURS: usize = 24;

/// A phase that has started and not yet been recorded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenPhase {
    pub kind: PhaseKind,
    pub started_unix_ms: u64,
    pub planned_secs: u32,
}

fn kind_name(kind: PhaseKind) -> &'static str {
    match kind {
        PhaseKind::Work => "work",
        PhaseKind::ShortBreak => "short_break",
        PhaseKind::LongBreak => "long_break",
    }
}

fn status_name(status: PhaseStatus) -> &'static str {
    match status {
        PhaseStatus::Completed => "completed",
        PhaseStatus::Skipped => "skipped",
        PhaseStatus::Interrupted => "interrupted",
    }
}

const SQL_UTC_TIMESTAMP: &str = "strftime('%Y-%m-%dT%H:%M:%SZ', ?, 'unixepoch')";

// ----- dates -----
//
// Days are counted from 1970-01-01. The conversions are the well known
// civil-calendar algorithms, so no date crate is needed.

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = (if year >= 0 { year } else { year - 399 }) / 400;
    let year_of_era = year - era * 400;
    let month_index = (i64::from(month) + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    (year, month as u32, day as u32)
}

fn parse_date(text: &str) -> Option<i64> {
    let mut parts = text.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) {
        return None;
    }
    if day < 1 || day > days_in_month(year, month) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

fn format_date(days: i64) -> String {
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// 0 is Sunday, 6 is Saturday.
fn weekday(days: i64) -> u32 {
    (days + 4).rem_euclid(7) as u32
}

fn days_in_month(year: i64, month: u32) -> u32 {
    let next = if month == 12 {
        days_from_civil(year + 1, 1, 1)
    } else {
        days_from_civil(year, month + 1, 1)
    };
    (next - days_from_civil(year, month, 1)) as u32
}

fn add_months(year: i64, month: u32, delta: i32) -> (i64, u32) {
    let index = year * 12 + i64::from(month) - 1 + i64::from(delta);
    (index.div_euclid(12), index.rem_euclid(12) as u32 + 1)
}

// ----- queries and answers -----

/// What the statistics window looks at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    Day,
    Week,
    Month,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct StatsQuery {
    pub period: Period,
    /// 0 is the current period, -1 the one before, and so on.
    #[serde(default)]
    pub offset: i32,
    /// First day of the week, 0 for Sunday to 6 for Saturday.
    pub week_start: u8,
}

/// The first and last day (inclusive) of a period, as day numbers.
fn range_for(query: &StatsQuery, today: i64) -> (i64, i64) {
    match query.period {
        Period::Day => {
            let day = today + i64::from(query.offset);
            (day, day)
        }
        Period::Week => {
            let back = (i64::from(weekday(today)) + 7 - i64::from(query.week_start % 7)) % 7;
            let start = today - back + 7 * i64::from(query.offset);
            (start, start + 6)
        }
        Period::Month => {
            let (year, month, _) = civil_from_days(today);
            let (year, month) = add_months(year, month, query.offset);
            let start = days_from_civil(year, month, 1);
            (start, start + i64::from(days_in_month(year, month)) - 1)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TodayGlance {
    pub completed: u32,
    pub focus_secs: u64,
    pub goal: u32,
}

/// One bar of the chart: an hour for a day, a date for a week or a month.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Bucket {
    /// `"00"` to `"23"` for a day, `YYYY-MM-DD` otherwise.
    pub key: String,
    pub completed: u32,
    pub focus_secs: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Streak {
    pub current: u32,
    pub longest: u32,
}

/// Work phases of the period by how they ended.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Completion {
    pub completed: u32,
    pub skipped: u32,
    pub interrupted: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Totals {
    pub completed: u32,
    pub focus_secs: u64,
    /// Days with at least one completed pomodoro.
    pub active_days: u32,
    /// Focus time divided by the active days, 0 when there are none.
    pub average_focus_secs: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Stats {
    pub period: Period,
    /// First and last day of the period, `YYYY-MM-DD`.
    pub start: String,
    pub end: String,
    pub today: TodayGlance,
    pub buckets: Vec<Bucket>,
    pub streak: Streak,
    pub completion: Completion,
    /// Completed pomodoros of the period by the hour they started.
    pub hours: Vec<u32>,
    pub totals: Totals,
}

/// Longest run of consecutive days, and the run that ends today (or
/// yesterday, so a streak is not lost before the first pomodoro of the day).
fn streaks(days: &[i64], today: i64) -> Streak {
    let mut longest = 0;
    let mut run = 0;
    let mut previous: Option<i64> = None;
    for &day in days {
        run = if previous == Some(day - 1) {
            run + 1
        } else {
            1
        };
        longest = longest.max(run);
        previous = Some(day);
    }
    let mut current = 0;
    let mut expected = if days.contains(&today) {
        today
    } else {
        today - 1
    };
    while days.contains(&expected) {
        current += 1;
        expected -= 1;
    }
    Streak { current, longest }
}

impl Store {
    /// Stores a finished phase.
    pub fn record_phase(&self, record: &PhaseRecord) -> Result<(), StorageError> {
        let labels = serde_json::to_string(&record.labels)?;
        let sql = format!(
            "INSERT INTO phases (kind, started_at, ended_at, planned_seconds, actual_seconds,
                                 paused_seconds, extended_seconds, status, labels)
             VALUES (?1, {started}, {ended}, ?4, ?5, ?6, ?7, ?8, ?9)",
            started = SQL_UTC_TIMESTAMP.replace('?', "?2"),
            ended = SQL_UTC_TIMESTAMP.replace('?', "?3"),
        );
        self.connection.execute(
            &sql,
            params![
                kind_name(record.kind),
                (record.started_unix_ms / 1000) as i64,
                (record.ended_unix_ms / 1000) as i64,
                record.planned_secs,
                record.actual_secs,
                record.paused_secs,
                record.extended_secs,
                status_name(record.status),
                labels,
            ],
        )?;
        Ok(())
    }

    /// Notes that a phase has started. See [`OPEN_PHASE_KEY`].
    pub fn set_open_phase(&self, phase: &OpenPhase) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![OPEN_PHASE_KEY, serde_json::to_string(phase)?],
        )?;
        Ok(())
    }

    /// Forgets the open phase, once it has been recorded.
    pub fn clear_open_phase(&self) -> Result<(), StorageError> {
        self.connection.execute(
            "DELETE FROM settings WHERE key = ?1",
            params![OPEN_PHASE_KEY],
        )?;
        Ok(())
    }

    /// Records a phase that was running when the app stopped without a clean
    /// exit as `interrupted` (SPEC 5.5). The end and the time actually spent
    /// are not known, so the row has no end and no focus time. Returns
    /// whether there was one.
    pub fn recover_open_phase(&self) -> Result<bool, StorageError> {
        let stored: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![OPEN_PHASE_KEY],
                |row| row.get(0),
            )
            .optional()?;
        let Some(json) = stored else {
            return Ok(false);
        };
        self.clear_open_phase()?;
        let Ok(phase) = serde_json::from_str::<OpenPhase>(&json) else {
            return Ok(false);
        };
        let sql = format!(
            "INSERT INTO phases (kind, started_at, ended_at, planned_seconds, actual_seconds,
                                 paused_seconds, extended_seconds, status, labels)
             VALUES (?1, {started}, NULL, ?3, 0, 0, 0, 'interrupted', '[]')",
            started = SQL_UTC_TIMESTAMP.replace('?', "?2"),
        );
        self.connection.execute(
            &sql,
            params![
                kind_name(phase.kind),
                (phase.started_unix_ms / 1000) as i64,
                phase.planned_secs,
            ],
        )?;
        Ok(true)
    }

    /// Today's date in the local time zone, `YYYY-MM-DD`.
    pub fn local_today(&self) -> Result<String, StorageError> {
        Ok(self
            .connection
            .query_row("SELECT date('now', ?1)", params![LOCAL_TIME], |row| {
                row.get(0)
            })?)
    }

    /// The statistics of a period. `today` is the local date and `zone` the
    /// SQLite modifier for the local time ([`LOCAL_TIME`]).
    pub fn stats(
        &self,
        query: &StatsQuery,
        goal: u32,
        today: &str,
        zone: &str,
    ) -> Result<Stats, StorageError> {
        let today_days =
            parse_date(today).ok_or_else(|| StorageError::BadDate(today.to_string()))?;
        let (first, last) = range_for(query, today_days);
        let (start, end) = (format_date(first), format_date(last));

        let day_totals = |date: &str| -> Result<(u32, u64), StorageError> {
            Ok(self.connection.query_row(
                "SELECT COALESCE(SUM(status = 'completed'), 0), COALESCE(SUM(actual_seconds), 0)
                 FROM phases WHERE kind = 'work' AND date(started_at, ?1) = ?2",
                params![zone, date],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?)
        };
        let (today_completed, today_focus) = day_totals(today)?;

        // One bar per hour for a day, one per date otherwise.
        let mut buckets: Vec<Bucket>;
        if query.period == Period::Day {
            buckets = (0..HOURS)
                .map(|hour| Bucket {
                    key: format!("{hour:02}"),
                    completed: 0,
                    focus_secs: 0,
                })
                .collect();
            let mut statement = self.connection.prepare(
                "SELECT CAST(strftime('%H', started_at, ?1) AS INTEGER),
                        COALESCE(SUM(status = 'completed'), 0), COALESCE(SUM(actual_seconds), 0)
                 FROM phases WHERE kind = 'work' AND date(started_at, ?1) = ?2
                 GROUP BY 1",
            )?;
            let rows = statement.query_map(params![zone, start], |row| {
                Ok((row.get::<_, usize>(0)?, row.get(1)?, row.get(2)?))
            })?;
            for row in rows {
                let (hour, completed, focus_secs) = row?;
                if let Some(bucket) = buckets.get_mut(hour) {
                    bucket.completed = completed;
                    bucket.focus_secs = focus_secs;
                }
            }
        } else {
            buckets = (first..=last)
                .map(|day| Bucket {
                    key: format_date(day),
                    completed: 0,
                    focus_secs: 0,
                })
                .collect();
            let mut statement = self.connection.prepare(
                "SELECT date(started_at, ?1) AS day,
                        COALESCE(SUM(status = 'completed'), 0), COALESCE(SUM(actual_seconds), 0)
                 FROM phases
                 WHERE kind = 'work' AND date(started_at, ?1) BETWEEN ?2 AND ?3
                 GROUP BY day",
            )?;
            let rows = statement.query_map(params![zone, start, end], |row| {
                Ok((row.get::<_, String>(0)?, row.get(1)?, row.get(2)?))
            })?;
            for row in rows {
                let (day, completed, focus_secs) = row?;
                if let Some(bucket) = buckets.iter_mut().find(|bucket| bucket.key == day) {
                    bucket.completed = completed;
                    bucket.focus_secs = focus_secs;
                }
            }
        }

        let mut completion = Completion::default();
        {
            let mut statement = self.connection.prepare(
                "SELECT status, COUNT(*) FROM phases
                 WHERE kind = 'work' AND date(started_at, ?1) BETWEEN ?2 AND ?3
                 GROUP BY status",
            )?;
            let rows = statement.query_map(params![zone, start, end], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })?;
            for row in rows {
                let (status, count) = row?;
                match status.as_str() {
                    "completed" => completion.completed = count,
                    "skipped" => completion.skipped = count,
                    _ => completion.interrupted = count,
                }
            }
        }

        let mut hours = vec![0u32; HOURS];
        {
            let mut statement = self.connection.prepare(
                "SELECT CAST(strftime('%H', started_at, ?1) AS INTEGER), COUNT(*)
                 FROM phases
                 WHERE kind = 'work' AND status = 'completed'
                   AND date(started_at, ?1) BETWEEN ?2 AND ?3
                 GROUP BY 1",
            )?;
            let rows = statement.query_map(params![zone, start, end], |row| {
                Ok((row.get::<_, usize>(0)?, row.get::<_, u32>(1)?))
            })?;
            for row in rows {
                let (hour, count) = row?;
                if let Some(slot) = hours.get_mut(hour) {
                    *slot = count;
                }
            }
        }

        let active_days = if query.period == Period::Day {
            u32::from(buckets.iter().any(|bucket| bucket.completed > 0))
        } else {
            buckets.iter().filter(|bucket| bucket.completed > 0).count() as u32
        };
        let completed = buckets.iter().map(|bucket| bucket.completed).sum();
        let focus_secs: u64 = buckets.iter().map(|bucket| bucket.focus_secs).sum();
        let totals = Totals {
            completed,
            focus_secs,
            active_days,
            average_focus_secs: if active_days == 0 {
                0
            } else {
                focus_secs / u64::from(active_days)
            },
        };

        let streak = {
            let mut statement = self.connection.prepare(
                "SELECT DISTINCT date(started_at, ?1) AS day FROM phases
                 WHERE kind = 'work' AND status = 'completed' ORDER BY day",
            )?;
            let rows = statement.query_map(params![zone], |row| row.get::<_, String>(0))?;
            let mut days = Vec::new();
            for row in rows {
                if let Some(day) = parse_date(&row?) {
                    days.push(day);
                }
            }
            streaks(&days, today_days)
        };

        Ok(Stats {
            period: query.period,
            start,
            end,
            today: TodayGlance {
                completed: today_completed,
                focus_secs: today_focus,
                goal,
            },
            buckets,
            streak,
            completion,
            hours,
            totals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UTC, so the tests give the same answer on every machine.
    const UTC: &str = "+0 minutes";

    fn at(date: &str, hour: u64, minute: u64) -> u64 {
        let days = parse_date(date).unwrap() as u64;
        ((days * 24 + hour) * 60 + minute) * 60 * 1000
    }

    fn record(kind: PhaseKind, status: PhaseStatus, started: u64, secs: u32) -> PhaseRecord {
        PhaseRecord {
            kind,
            started_unix_ms: started,
            ended_unix_ms: started + u64::from(secs) * 1000,
            planned_secs: 1500,
            actual_secs: secs,
            paused_secs: 0,
            extended_secs: 0,
            status,
            labels: Vec::new(),
        }
    }

    fn work(date: &str, hour: u64, status: PhaseStatus, secs: u32) -> PhaseRecord {
        record(PhaseKind::Work, status, at(date, hour, 0), secs)
    }

    fn query(period: Period, offset: i32) -> StatsQuery {
        StatsQuery {
            period,
            offset,
            week_start: 1,
        }
    }

    fn store_with(records: &[PhaseRecord]) -> Store {
        let store = Store::open_in_memory().unwrap();
        for record in records {
            store.record_phase(record).unwrap();
        }
        store
    }

    #[test]
    fn dates_convert_both_ways() {
        assert_eq!(parse_date("1970-01-01"), Some(0));
        assert_eq!(format_date(0), "1970-01-01");
        for text in ["2000-03-01", "2024-02-29", "2026-10-13", "1999-12-31"] {
            assert_eq!(format_date(parse_date(text).unwrap()), text);
        }
        assert_eq!(
            parse_date("2026-10-14").unwrap() - parse_date("2026-10-13").unwrap(),
            1
        );
        assert_eq!(parse_date("2026-02-29"), None, "2026 is not a leap year");
        assert_eq!(parse_date("2026-13-01"), None);
        assert_eq!(parse_date("tomorrow"), None);
    }

    #[test]
    fn weekdays_and_month_lengths() {
        assert_eq!(weekday(parse_date("1970-01-01").unwrap()), 4, "a Thursday");
        assert_eq!(weekday(parse_date("2026-10-13").unwrap()), 2, "a Tuesday");
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2026, 12), 31);
        assert_eq!(add_months(2026, 1, -1), (2025, 12));
        assert_eq!(add_months(2026, 12, 1), (2027, 1));
        assert_eq!(add_months(2026, 3, -15), (2024, 12));
    }

    #[test]
    fn periods_cover_the_right_days() {
        let today = parse_date("2026-10-13").unwrap(); // a Tuesday
        let range = |period, offset, week_start| {
            let (a, b) = range_for(
                &StatsQuery {
                    period,
                    offset,
                    week_start,
                },
                today,
            );
            (format_date(a), format_date(b))
        };
        assert_eq!(
            range(Period::Day, 0, 1),
            ("2026-10-13".into(), "2026-10-13".into())
        );
        assert_eq!(
            range(Period::Day, -1, 1),
            ("2026-10-12".into(), "2026-10-12".into())
        );
        assert_eq!(
            range(Period::Week, 0, 1),
            ("2026-10-12".into(), "2026-10-18".into())
        );
        assert_eq!(
            range(Period::Week, 0, 0),
            ("2026-10-11".into(), "2026-10-17".into())
        );
        assert_eq!(
            range(Period::Week, -1, 1),
            ("2026-10-05".into(), "2026-10-11".into())
        );
        assert_eq!(
            range(Period::Month, 0, 1),
            ("2026-10-01".into(), "2026-10-31".into())
        );
        assert_eq!(
            range(Period::Month, -10, 1),
            ("2025-12-01".into(), "2025-12-31".into())
        );
        assert_eq!(
            range(Period::Month, -8, 1),
            ("2026-02-01".into(), "2026-02-28".into())
        );
    }

    #[test]
    fn streaks_count_consecutive_days() {
        let days = |list: &[&str]| -> Vec<i64> {
            list.iter().map(|text| parse_date(text).unwrap()).collect()
        };
        let today = parse_date("2026-10-13").unwrap();
        let none = streaks(&[], today);
        assert_eq!((none.current, none.longest), (0, 0));

        let alive = streaks(
            &days(&[
                "2026-10-01",
                "2026-10-02",
                "2026-10-03",
                "2026-10-12",
                "2026-10-13",
            ]),
            today,
        );
        assert_eq!((alive.current, alive.longest), (2, 3));

        let waiting = streaks(&days(&["2026-10-11", "2026-10-12"]), today);
        assert_eq!(
            waiting.current, 2,
            "not lost before the first pomodoro of the day"
        );

        let lost = streaks(&days(&["2026-10-09", "2026-10-10"]), today);
        assert_eq!((lost.current, lost.longest), (0, 2));
    }

    #[test]
    fn a_recorded_phase_is_stored_in_utc() {
        let store = store_with(&[work("2026-10-13", 9, PhaseStatus::Completed, 1500)]);
        let (started, ended, kind, status): (String, String, String, String) = store
            .connection
            .query_row(
                "SELECT started_at, ended_at, kind, status FROM phases",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(started, "2026-10-13T09:00:00Z");
        assert_eq!(ended, "2026-10-13T09:25:00Z");
        assert_eq!((kind.as_str(), status.as_str()), ("work", "completed"));
    }

    #[test]
    fn the_day_view_has_one_bucket_per_hour() {
        let store = store_with(&[
            work("2026-10-13", 9, PhaseStatus::Completed, 1500),
            work("2026-10-13", 9, PhaseStatus::Completed, 1500),
            work("2026-10-13", 14, PhaseStatus::Skipped, 300),
            work("2026-10-12", 9, PhaseStatus::Completed, 1500),
        ]);
        let stats = store
            .stats(&query(Period::Day, 0), 8, "2026-10-13", UTC)
            .unwrap();
        assert_eq!(stats.buckets.len(), 24);
        assert_eq!(stats.buckets[9].completed, 2);
        assert_eq!(stats.buckets[9].focus_secs, 3000);
        assert_eq!(stats.buckets[14].completed, 0);
        assert_eq!(
            stats.buckets[14].focus_secs, 300,
            "a skipped phase still took time"
        );
        assert_eq!(
            (stats.start.as_str(), stats.end.as_str()),
            ("2026-10-13", "2026-10-13")
        );
        assert_eq!(stats.totals.completed, 2);
        assert_eq!(stats.totals.focus_secs, 3300);
        assert_eq!(stats.totals.active_days, 1);
        assert_eq!(stats.totals.average_focus_secs, 3300);
    }

    #[test]
    fn the_week_view_groups_by_day_and_leaves_other_weeks_out() {
        let store = store_with(&[
            work("2026-10-12", 9, PhaseStatus::Completed, 1500), // Monday
            work("2026-10-13", 9, PhaseStatus::Completed, 1500),
            work("2026-10-13", 10, PhaseStatus::Completed, 1500),
            work("2026-10-11", 9, PhaseStatus::Completed, 1500), // the Sunday before
        ]);
        let stats = store
            .stats(&query(Period::Week, 0), 8, "2026-10-13", UTC)
            .unwrap();
        assert_eq!(stats.buckets.len(), 7);
        assert_eq!(stats.buckets[0].key, "2026-10-12");
        assert_eq!(stats.buckets[0].completed, 1);
        assert_eq!(stats.buckets[1].completed, 2);
        assert_eq!(stats.totals.completed, 3);
        assert_eq!(stats.totals.active_days, 2);
        assert_eq!(stats.totals.average_focus_secs, 2250);

        let before = store
            .stats(&query(Period::Week, -1), 8, "2026-10-13", UTC)
            .unwrap();
        assert_eq!(before.totals.completed, 1);
    }

    #[test]
    fn the_month_view_covers_every_day_of_the_month() {
        let store = store_with(&[work("2026-02-10", 9, PhaseStatus::Completed, 1500)]);
        let stats = store
            .stats(&query(Period::Month, -8), 8, "2026-10-13", UTC)
            .unwrap();
        assert_eq!(stats.buckets.len(), 28);
        assert_eq!(stats.buckets[9].key, "2026-02-10");
        assert_eq!(stats.buckets[9].completed, 1);
    }

    #[test]
    fn breaks_do_not_count_and_completion_compares_the_work_phases() {
        let store = store_with(&[
            work("2026-10-13", 9, PhaseStatus::Completed, 1500),
            work("2026-10-13", 10, PhaseStatus::Completed, 1500),
            work("2026-10-13", 11, PhaseStatus::Skipped, 200),
            work("2026-10-13", 12, PhaseStatus::Interrupted, 100),
            record(
                PhaseKind::ShortBreak,
                PhaseStatus::Completed,
                at("2026-10-13", 9, 30),
                300,
            ),
        ]);
        let stats = store
            .stats(&query(Period::Day, 0), 8, "2026-10-13", UTC)
            .unwrap();
        assert_eq!(
            stats.completion,
            Completion {
                completed: 2,
                skipped: 1,
                interrupted: 1
            }
        );
        assert_eq!(stats.totals.completed, 2);
        assert_eq!(stats.totals.focus_secs, 1500 + 1500 + 200 + 100);
        assert_eq!(stats.today.completed, 2);
        assert_eq!(stats.today.goal, 8);
    }

    #[test]
    fn the_best_time_of_day_counts_completed_pomodoros_by_hour() {
        let store = store_with(&[
            work("2026-10-12", 9, PhaseStatus::Completed, 1500),
            work("2026-10-13", 9, PhaseStatus::Completed, 1500),
            work("2026-10-13", 15, PhaseStatus::Completed, 1500),
            work("2026-10-13", 16, PhaseStatus::Skipped, 100),
        ]);
        let stats = store
            .stats(&query(Period::Week, 0), 8, "2026-10-13", UTC)
            .unwrap();
        assert_eq!(stats.hours[9], 2);
        assert_eq!(stats.hours[15], 1);
        assert_eq!(stats.hours[16], 0, "a skipped pomodoro is not counted");
        assert_eq!(stats.hours.iter().sum::<u32>(), 3);
    }

    #[test]
    fn days_follow_the_local_time_zone() {
        // 23:30 UTC is already the next day two hours east of Greenwich.
        let store = store_with(&[work("2026-10-13", 23, PhaseStatus::Completed, 1500)]);
        let utc = store
            .stats(&query(Period::Day, 0), 8, "2026-10-13", UTC)
            .unwrap();
        assert_eq!(utc.totals.completed, 1);
        let east = store
            .stats(&query(Period::Day, 0), 8, "2026-10-13", "+120 minutes")
            .unwrap();
        assert_eq!(east.totals.completed, 0);
        let next = store
            .stats(&query(Period::Day, 0), 8, "2026-10-14", "+120 minutes")
            .unwrap();
        assert_eq!(next.totals.completed, 1);
        assert_eq!(next.buckets[1].completed, 1, "01:00 local time");
    }

    #[test]
    fn the_streak_comes_from_the_whole_history() {
        let store = store_with(&[
            work("2026-09-01", 9, PhaseStatus::Completed, 1500),
            work("2026-09-02", 9, PhaseStatus::Completed, 1500),
            work("2026-09-03", 9, PhaseStatus::Completed, 1500),
            work("2026-10-12", 9, PhaseStatus::Completed, 1500),
            work("2026-10-13", 9, PhaseStatus::Skipped, 100),
        ]);
        let stats = store
            .stats(&query(Period::Day, 0), 8, "2026-10-13", UTC)
            .unwrap();
        assert_eq!(
            stats.streak,
            Streak {
                current: 1,
                longest: 3
            }
        );
    }

    #[test]
    fn a_bad_date_is_refused() {
        let store = Store::open_in_memory().unwrap();
        assert!(store.stats(&query(Period::Day, 0), 8, "soon", UTC).is_err());
    }

    #[test]
    fn local_today_is_a_date() {
        let store = Store::open_in_memory().unwrap();
        assert!(parse_date(&store.local_today().unwrap()).is_some());
    }

    #[test]
    fn a_phase_open_at_a_crash_is_recorded_as_interrupted() {
        let store = Store::open_in_memory().unwrap();
        assert!(!store.recover_open_phase().unwrap(), "nothing was open");

        store
            .set_open_phase(&OpenPhase {
                kind: PhaseKind::Work,
                started_unix_ms: at("2026-10-13", 9, 0),
                planned_secs: 1500,
            })
            .unwrap();
        assert!(store.recover_open_phase().unwrap());
        assert!(!store.recover_open_phase().unwrap(), "only once");

        let (status, ended, actual): (String, Option<String>, u32) = store
            .connection
            .query_row(
                "SELECT status, ended_at, actual_seconds FROM phases",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(status, "interrupted");
        assert_eq!(ended, None);
        assert_eq!(actual, 0);
    }

    #[test]
    fn a_recorded_phase_clears_the_open_one() {
        let store = Store::open_in_memory().unwrap();
        store
            .set_open_phase(&OpenPhase {
                kind: PhaseKind::Work,
                started_unix_ms: at("2026-10-13", 9, 0),
                planned_secs: 1500,
            })
            .unwrap();
        store.clear_open_phase().unwrap();
        assert!(!store.recover_open_phase().unwrap());
    }
}

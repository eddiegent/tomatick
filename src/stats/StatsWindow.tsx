import { useEffect, useState } from "react";
import "./stats.css";
import { useStats, type Period, type Stats } from "./api";

// English only for now. The text moves to the translation files in
// milestone 10.
const TEXT = {
  title: "Statistics",
  day: "Day",
  week: "Week",
  month: "Month",
  previous: "Previous",
  next: "Next",
  today: "Today",
  goalOf: (done: number, goal: number) => `${done} of ${goal} pomodoros`,
  goalReached: "Goal reached",
  focusTime: "Focus time",
  pomodoros: "Pomodoros",
  chartBy: "Show",
  streak: "Streak",
  currentStreak: "Current",
  longestStreak: "Longest",
  days: (n: number) => (n === 1 ? "1 day" : `${n} days`),
  completionRate: "Completion rate",
  completionNone: "No work phases yet",
  completed: "completed",
  skipped: "skipped",
  interrupted: "interrupted",
  totals: "Totals",
  totalPomodoros: "Pomodoros",
  totalFocus: "Focus time",
  activeDays: "Active days",
  averageFocus: "Average per active day",
  bestTime: "Best time of day",
  bestTimeHelp: "Completed pomodoros by the hour they started.",
  empty: "No pomodoros in this period yet.",
  loading: "Loading",
  hour: (hour: number) => `${hour}:00`,
  chartLabel: (what: string, title: string) => `${what}, ${title}`,
};

type Metric = "completed" | "focus";

/** `YYYY-MM-DD` as a date in the local time zone. */
function parseDay(text: string): Date {
  const [year, month, day] = text.split("-").map(Number);
  return new Date(year, month - 1, day);
}

function periodTitle(stats: Stats): string {
  const start = parseDay(stats.start);
  const end = parseDay(stats.end);
  switch (stats.period) {
    case "day":
      return start.toLocaleDateString(undefined, {
        weekday: "long",
        day: "numeric",
        month: "long",
        year: "numeric",
      });
    case "week": {
      const short = { day: "numeric", month: "short" } as const;
      return `${start.toLocaleDateString(undefined, short)} to ${end.toLocaleDateString(undefined, { ...short, year: "numeric" })}`;
    }
    case "month":
      return start.toLocaleDateString(undefined, { month: "long", year: "numeric" });
  }
}

/** `2 h 05 min`, `45 min`, or `0 min`. */
export function formatFocus(seconds: number): string {
  const minutes = Math.round(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) {
    return `${rest} min`;
  }
  return `${hours} h ${String(rest).padStart(2, "0")} min`;
}

function percent(part: number, whole: number): number {
  return whole === 0 ? 0 : Math.round((part / whole) * 100);
}

/** The label under a bar, or an empty string for the bars that go without. */
function barLabel(period: Period, key: string, index: number): string {
  switch (period) {
    case "day":
      return index % 3 === 0 ? String(Number(key)) : "";
    case "week":
      return parseDay(key).toLocaleDateString(undefined, { weekday: "short" });
    case "month": {
      const day = Number(key.slice(8));
      return day === 1 || day % 5 === 0 ? String(day) : "";
    }
  }
}

function barTitle(period: Period, key: string): string {
  if (period === "day") {
    return TEXT.hour(Number(key));
  }
  return parseDay(key).toLocaleDateString(undefined, {
    weekday: "short",
    day: "numeric",
    month: "short",
  });
}

interface Bar {
  label: string;
  title: string;
  value: number;
  text: string;
}

/** A small bar chart. Each bar has a hover text, and the whole chart a label. */
function BarChart({ bars, label, compact }: { bars: Bar[]; label: string; compact?: boolean }) {
  const width = 560;
  const height = compact ? 110 : 170;
  const top = 8;
  const bottom = 22;
  const max = Math.max(1, ...bars.map((bar) => bar.value));
  const slot = width / Math.max(1, bars.length);
  const barWidth = Math.max(2, slot * 0.64);
  const plot = height - top - bottom;

  return (
    <svg
      className="chart"
      viewBox={`0 0 ${width} ${height}`}
      role="img"
      aria-label={label}
      preserveAspectRatio="none"
    >
      <line className="axis" x1="0" x2={width} y1={height - bottom} y2={height - bottom} />
      {bars.map((bar, index) => {
        const barHeight = bar.value === 0 ? 0 : Math.max(2, (bar.value / max) * plot);
        const x = index * slot + (slot - barWidth) / 2;
        return (
          <g key={index}>
            <title>{`${bar.title}: ${bar.text}`}</title>
            {/* A taller invisible bar so a short one is easy to hover. */}
            <rect x={index * slot} y={top} width={slot} height={plot} fill="transparent" />
            <rect
              className="bar"
              x={x}
              y={height - bottom - barHeight}
              width={barWidth}
              height={barHeight}
              rx={Math.min(3, barWidth / 2)}
            />
            {bar.label !== "" && (
              <text className="tick" x={x + barWidth / 2} y={height - 6} textAnchor="middle">
                {bar.label}
              </text>
            )}
          </g>
        );
      })}
    </svg>
  );
}

function Today({ stats }: { stats: Stats }) {
  const { completed, goal, focus_secs } = stats.today;
  const ratio = Math.min(1, completed / Math.max(1, goal));
  return (
    <section className="card today">
      <h2>{TEXT.today}</h2>
      <p className="big">{TEXT.goalOf(completed, goal)}</p>
      <div
        className="goal"
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={goal}
        aria-valuenow={Math.min(completed, goal)}
      >
        <div className="goal-fill" style={{ width: `${ratio * 100}%` }} />
      </div>
      <p className="muted">
        {TEXT.focusTime}: {formatFocus(focus_secs)}
        {completed >= goal ? ` · ${TEXT.goalReached}` : ""}
      </p>
    </section>
  );
}

/** The statistics window, opened from the chart button of the widget. */
export function StatsWindow() {
  const [period, setPeriod] = useState<Period>("week");
  const [offset, setOffset] = useState(0);
  const [metric, setMetric] = useState<Metric>("completed");
  const { stats, error } = useStats(period, offset);

  useEffect(() => {
    document.title = TEXT.title;
  }, []);

  const choose = (next: Period) => {
    setPeriod(next);
    setOffset(0);
  };

  if (!stats) {
    return (
      <div className="stats">
        <p className={error ? "error" : "muted"}>{error ?? TEXT.loading}</p>
      </div>
    );
  }

  const title = periodTitle(stats);
  const bars: Bar[] = stats.buckets.map((bucket, index) => ({
    label: barLabel(stats.period, bucket.key, index),
    title: barTitle(stats.period, bucket.key),
    value: metric === "completed" ? bucket.completed : bucket.focus_secs,
    text:
      metric === "completed"
        ? String(bucket.completed)
        : formatFocus(bucket.focus_secs),
  }));
  const works = stats.completion.completed + stats.completion.skipped + stats.completion.interrupted;
  const bestHours: Bar[] = stats.hours.map((count, hour) => ({
    label: hour % 3 === 0 ? String(hour) : "",
    title: TEXT.hour(hour),
    value: count,
    text: String(count),
  }));
  const empty = stats.totals.completed === 0 && stats.totals.focus_secs === 0;

  return (
    <div className="stats">
      <header>
        <div className="segmented" role="radiogroup" aria-label={TEXT.title}>
          {(["day", "week", "month"] as const).map((value) => (
            <button
              key={value}
              type="button"
              role="radio"
              aria-checked={period === value}
              className={period === value ? "on" : undefined}
              onClick={() => choose(value)}
            >
              {TEXT[value]}
            </button>
          ))}
        </div>
        <div className="nav">
          <button
            type="button"
            className="icon"
            title={TEXT.previous}
            aria-label={TEXT.previous}
            onClick={() => setOffset(offset - 1)}
          >
            {"‹"}
          </button>
          <span className="range" aria-live="polite">
            {title}
          </span>
          <button
            type="button"
            className="icon"
            title={TEXT.next}
            aria-label={TEXT.next}
            disabled={offset >= 0}
            onClick={() => setOffset(offset + 1)}
          >
            {"›"}
          </button>
        </div>
      </header>

      <Today stats={stats} />

      <section className="card">
        <div className="card-head">
          <h2>{metric === "completed" ? TEXT.pomodoros : TEXT.focusTime}</h2>
          <div className="segmented small" role="radiogroup" aria-label={TEXT.chartBy}>
            {(["completed", "focus"] as const).map((value) => (
              <button
                key={value}
                type="button"
                role="radio"
                aria-checked={metric === value}
                className={metric === value ? "on" : undefined}
                onClick={() => setMetric(value)}
              >
                {value === "completed" ? TEXT.pomodoros : TEXT.focusTime}
              </button>
            ))}
          </div>
        </div>
        <BarChart
          bars={bars}
          label={TEXT.chartLabel(metric === "completed" ? TEXT.pomodoros : TEXT.focusTime, title)}
        />
        {empty && <p className="muted">{TEXT.empty}</p>}
      </section>

      <div className="grid">
        <section className="card">
          <h2>{TEXT.streak}</h2>
          <dl>
            <dt>{TEXT.currentStreak}</dt>
            <dd>{TEXT.days(stats.streak.current)}</dd>
            <dt>{TEXT.longestStreak}</dt>
            <dd>{TEXT.days(stats.streak.longest)}</dd>
          </dl>
        </section>

        <section className="card">
          <h2>{TEXT.completionRate}</h2>
          {works === 0 ? (
            <p className="muted">{TEXT.completionNone}</p>
          ) : (
            <>
              <p className="big">{percent(stats.completion.completed, works)}%</p>
              <p className="muted">
                {stats.completion.completed} {TEXT.completed}, {stats.completion.skipped}{" "}
                {TEXT.skipped}, {stats.completion.interrupted} {TEXT.interrupted}
              </p>
            </>
          )}
        </section>

        <section className="card wide">
          <h2>{TEXT.totals}</h2>
          <dl className="columns">
            <div>
              <dt>{TEXT.totalPomodoros}</dt>
              <dd>{stats.totals.completed}</dd>
            </div>
            <div>
              <dt>{TEXT.totalFocus}</dt>
              <dd>{formatFocus(stats.totals.focus_secs)}</dd>
            </div>
            <div>
              <dt>{TEXT.activeDays}</dt>
              <dd>{stats.totals.active_days}</dd>
            </div>
            <div>
              <dt>{TEXT.averageFocus}</dt>
              <dd>{formatFocus(stats.totals.average_focus_secs)}</dd>
            </div>
          </dl>
        </section>
      </div>

      <section className="card">
        <h2>{TEXT.bestTime}</h2>
        <p className="muted">{TEXT.bestTimeHelp}</p>
        <BarChart bars={bestHours} label={`${TEXT.bestTime}, ${title}`} compact />
      </section>
    </div>
  );
}

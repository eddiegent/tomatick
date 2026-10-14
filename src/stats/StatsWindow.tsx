import { useEffect, useState } from "react";
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import { formatLocale } from "../i18n";
import "./stats.css";
import { useStats, type Period, type Stats } from "./api";

type Metric = "completed" | "focus";

/** `YYYY-MM-DD` as a date in the local time zone. */
function parseDay(text: string): Date {
  const [year, month, day] = text.split("-").map(Number);
  return new Date(year, month - 1, day);
}

function periodTitle(stats: Stats, locale: string, t: TFunction): string {
  const start = parseDay(stats.start);
  const end = parseDay(stats.end);
  switch (stats.period) {
    case "day":
      return start.toLocaleDateString(locale, {
        weekday: "long",
        day: "numeric",
        month: "long",
        year: "numeric",
      });
    case "week": {
      const short = { day: "numeric", month: "short" } as const;
      return t("stats.range", {
        start: start.toLocaleDateString(locale, short),
        end: end.toLocaleDateString(locale, { ...short, year: "numeric" }),
      });
    }
    case "month":
      return start.toLocaleDateString(locale, { month: "long", year: "numeric" });
  }
}

/** `2 h 05 min`, `45 min`, or `0 min`. */
export function formatFocus(seconds: number, t: TFunction): string {
  const minutes = Math.round(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) {
    return t("stats.focusM", { minutes: rest });
  }
  return t("stats.focusHm", { hours, minutes: String(rest).padStart(2, "0") });
}

/** `73%`, or `73 %` where the language writes it that way. */
function percent(part: number, whole: number, locale: string): string {
  return new Intl.NumberFormat(locale, { style: "percent", maximumFractionDigits: 0 }).format(
    whole === 0 ? 0 : part / whole,
  );
}

/** The label under a bar, or an empty string for the bars that go without. */
function barLabel(period: Period, key: string, index: number, locale: string): string {
  switch (period) {
    case "day":
      return index % 3 === 0 ? String(Number(key)) : "";
    case "week":
      return parseDay(key).toLocaleDateString(locale, { weekday: "short" });
    case "month": {
      const day = Number(key.slice(8));
      return day === 1 || day % 5 === 0 ? String(day) : "";
    }
  }
}

function barTitle(period: Period, key: string, locale: string, t: TFunction): string {
  if (period === "day") {
    return t("stats.hour", { hour: Number(key) });
  }
  return parseDay(key).toLocaleDateString(locale, {
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
  const { t } = useTranslation();
  const { completed, goal, focus_secs } = stats.today;
  const ratio = Math.min(1, completed / Math.max(1, goal));
  return (
    <section className="card today">
      <h2>{t("stats.today")}</h2>
      <p className="big">{t("stats.goalOf", { done: completed, goal })}</p>
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
        {t("stats.focusTime")}: {formatFocus(focus_secs, t)}
        {completed >= goal ? ` · ${t("stats.goalReached")}` : ""}
      </p>
    </section>
  );
}

/** The statistics window, opened from the chart button of the widget. */
export function StatsWindow() {
  const { t, i18n } = useTranslation();
  // Dates and numbers follow the regional format of the system.
  const locale = formatLocale(i18n.language);
  const [period, setPeriod] = useState<Period>("week");
  const [offset, setOffset] = useState(0);
  const [metric, setMetric] = useState<Metric>("completed");
  const { stats, error } = useStats(period, offset, locale);

  useEffect(() => {
    document.title = t("stats.title");
  }, [t]);

  const choose = (next: Period) => {
    setPeriod(next);
    setOffset(0);
  };

  if (!stats) {
    return (
      <div className="stats">
        <p className={error ? "error" : "muted"}>{error ?? t("stats.loading")}</p>
      </div>
    );
  }

  const title = periodTitle(stats, locale, t);
  const bars: Bar[] = stats.buckets.map((bucket, index) => ({
    label: barLabel(stats.period, bucket.key, index, locale),
    title: barTitle(stats.period, bucket.key, locale, t),
    value: metric === "completed" ? bucket.completed : bucket.focus_secs,
    text:
      metric === "completed"
        ? String(bucket.completed)
        : formatFocus(bucket.focus_secs, t),
  }));
  const works = stats.completion.completed + stats.completion.skipped + stats.completion.interrupted;
  const bestHours: Bar[] = stats.hours.map((count, hour) => ({
    label: hour % 3 === 0 ? String(hour) : "",
    title: t("stats.hour", { hour }),
    value: count,
    text: String(count),
  }));
  const empty = stats.totals.completed === 0 && stats.totals.focus_secs === 0;

  return (
    <div className="stats">
      <header>
        <div className="segmented" role="radiogroup" aria-label={t("stats.title")}>
          {(["day", "week", "month"] as const).map((value) => (
            <button
              key={value}
              type="button"
              role="radio"
              aria-checked={period === value}
              className={period === value ? "on" : undefined}
              onClick={() => choose(value)}
            >
              {t(`stats.${value}`)}
            </button>
          ))}
        </div>
        <div className="nav">
          <button
            type="button"
            className="icon"
            title={t("stats.previous")}
            aria-label={t("stats.previous")}
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
            title={t("stats.next")}
            aria-label={t("stats.next")}
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
          <h2>{metric === "completed" ? t("stats.pomodoros") : t("stats.focusTime")}</h2>
          <div className="segmented small" role="radiogroup" aria-label={t("stats.chartBy")}>
            {(["completed", "focus"] as const).map((value) => (
              <button
                key={value}
                type="button"
                role="radio"
                aria-checked={metric === value}
                className={metric === value ? "on" : undefined}
                onClick={() => setMetric(value)}
              >
                {value === "completed" ? t("stats.pomodoros") : t("stats.focusTime")}
              </button>
            ))}
          </div>
        </div>
        <BarChart
          bars={bars}
          label={t("stats.chartLabel", {
            what: metric === "completed" ? t("stats.pomodoros") : t("stats.focusTime"),
            title,
          })}
        />
        {empty && <p className="muted">{t("stats.empty")}</p>}
      </section>

      <div className="grid">
        <section className="card">
          <h2>{t("stats.streak")}</h2>
          <dl>
            <dt>{t("stats.currentStreak")}</dt>
            <dd>{t("stats.days", { count: stats.streak.current })}</dd>
            <dt>{t("stats.longestStreak")}</dt>
            <dd>{t("stats.days", { count: stats.streak.longest })}</dd>
          </dl>
        </section>

        <section className="card">
          <h2>{t("stats.completionRate")}</h2>
          {works === 0 ? (
            <p className="muted">{t("stats.completionNone")}</p>
          ) : (
            <>
              <p className="big">{percent(stats.completion.completed, works, locale)}</p>
              <p className="muted">
                {t("stats.completionDetail", {
                  completed: stats.completion.completed,
                  skipped: stats.completion.skipped,
                  interrupted: stats.completion.interrupted,
                })}
              </p>
            </>
          )}
        </section>

        <section className="card wide">
          <h2>{t("stats.totals")}</h2>
          <dl className="columns">
            <div>
              <dt>{t("stats.totalPomodoros")}</dt>
              <dd>{stats.totals.completed}</dd>
            </div>
            <div>
              <dt>{t("stats.totalFocus")}</dt>
              <dd>{formatFocus(stats.totals.focus_secs, t)}</dd>
            </div>
            <div>
              <dt>{t("stats.activeDays")}</dt>
              <dd>{stats.totals.active_days}</dd>
            </div>
            <div>
              <dt>{t("stats.averageFocus")}</dt>
              <dd>{formatFocus(stats.totals.average_focus_secs, t)}</dd>
            </div>
          </dl>
        </section>
      </div>

      <section className="card">
        <h2>{t("stats.bestTime")}</h2>
        <p className="muted">{t("stats.bestTimeHelp")}</p>
        <BarChart bars={bestHours} label={`${t("stats.bestTime")}, ${title}`} compact />
      </section>
    </div>
  );
}

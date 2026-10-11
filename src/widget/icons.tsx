// Small line icons for the widget controls. They inherit the text color.

const common = {
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 2,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
  "aria-hidden": true,
};

export function PlayIcon() {
  return (
    <svg {...common}>
      <path d="M8 5.5v13l10.5-6.5z" fill="currentColor" />
    </svg>
  );
}

export function PauseIcon() {
  return (
    <svg {...common}>
      <path d="M8.5 5.5v13M15.5 5.5v13" strokeWidth={3.2} />
    </svg>
  );
}

export function SkipIcon() {
  return (
    <svg {...common}>
      <path d="M6 5.5v13l9-6.5z" fill="currentColor" />
      <path d="M18 5.5v13" />
    </svg>
  );
}

export function ResetIcon() {
  return (
    <svg {...common}>
      <path d="M4.5 12a7.5 7.5 0 1 0 2.4-5.5" />
      <path d="M4 4.5v4.5h4.5" />
    </svg>
  );
}

export function SwapIcon() {
  return (
    <svg {...common}>
      <path d="M6 8h12l-3-3.5M18 16H6l3 3.5" />
    </svg>
  );
}

export function ChartIcon() {
  return (
    <svg {...common}>
      <path d="M5.5 19v-6M12 19V5M18.5 19v-9" strokeWidth={3} />
    </svg>
  );
}

export function CogIcon() {
  return (
    <svg {...common}>
      <circle cx="12" cy="12" r="3.2" />
      <path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M5.6 18.4l2.1-2.1M16.3 7.7l2.1-2.1" />
    </svg>
  );
}

export function MoonIcon() {
  return (
    <svg {...common}>
      <path d="M20 14.5A8.5 8.5 0 0 1 9.5 4 8.5 8.5 0 1 0 20 14.5z" fill="currentColor" />
    </svg>
  );
}

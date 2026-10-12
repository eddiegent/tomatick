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
      <path d="M10.7 5.1 L10.7 2.5 L13.3 2.5 L13.3 5.1 L15.9 6.2 L17.8 4.4 L19.6 6.2 L17.8 8.1 L18.9 10.7 L21.5 10.7 L21.5 13.3 L18.9 13.3 L17.8 15.9 L19.6 17.8 L17.8 19.6 L15.9 17.8 L13.3 18.9 L13.3 21.5 L10.7 21.5 L10.7 18.9 L8.1 17.8 L6.2 19.6 L4.4 17.8 L6.2 15.9 L5.1 13.3 L2.5 13.3 L2.5 10.7 L5.1 10.7 L6.2 8.1 L4.4 6.2 L6.2 4.4 L8.1 6.2 Z" />
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

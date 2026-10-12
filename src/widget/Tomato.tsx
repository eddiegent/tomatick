const LEAF_ANGLES = [-72, -36, 0, 36, 72];

/**
 * The tomato, its calyx and the progress ring, drawn in a 100 by 100 box.
 * Colors come from CSS variables so the phase theme can restyle it.
 */
export function Tomato({ progress }: { progress: number }) {
  const arc = {
    cx: 50,
    cy: 50,
    r: 45,
    pathLength: 100,
    strokeDasharray: `${(progress * 100).toFixed(2)} 100`,
    transform: "rotate(-90 50 50)",
  };

  return (
    <svg className="tomato" viewBox="0 0 100 100" aria-hidden="true">
      <defs>
        <radialGradient id="tomato-body" cx="38%" cy="32%" r="78%">
          <stop offset="0%" style={{ stopColor: "var(--tomato-light)" }} />
          <stop offset="100%" style={{ stopColor: "var(--tomato)" }} />
        </radialGradient>
      </defs>

      <circle className="ring-track" cx="50" cy="50" r="45" />
      {progress > 0 && (
        <>
          {/* The white halo keeps the arc readable on any desktop color. */}
          <circle className="ring-halo" {...arc} />
          <circle className="ring-progress" {...arc} />
        </>
      )}

      <circle cx="50" cy="53" r="38" fill="url(#tomato-body)" />
      <ellipse
        cx="36"
        cy="39"
        rx="10"
        ry="5.5"
        fill="#ffffff"
        opacity="0.22"
        transform="rotate(-32 36 39)"
      />

      <g
        transform="translate(50 18.5) scale(1.15)"
        style={{ fill: "var(--leaf)" }}
      >
        {LEAF_ANGLES.map((angle) => (
          <ellipse
            key={angle}
            cx="0"
            cy="-3.5"
            rx="2.8"
            ry="7"
            transform={`rotate(${angle})`}
          />
        ))}
        <rect x="-1.3" y="-9.5" width="2.6" height="6" rx="1.3" />
      </g>
    </svg>
  );
}

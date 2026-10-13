const LEAF_ANGLES = [-72, -36, 0, 36, 72];

// The tomato body, and the ring that hugs it.
const BODY = { cx: 50, cy: 53, r: 38 };
/** Must match `.ring-track` in widget.css. */
const TRACK_WIDTH = 5;
/**
 * The ring has the same center as the body. Its inner edge sits a hair under
 * the body edge, which is drawn on top, so no gap shows between them.
 */
const RING_RADIUS = BODY.r + TRACK_WIDTH / 2 - 0.3;

/**
 * The tomato, its calyx and the progress ring, drawn in a 100 by 100 box.
 * Colors come from CSS variables so the phase theme can restyle it.
 */
export function Tomato({ progress }: { progress: number }) {
  const arc = {
    cx: BODY.cx,
    cy: BODY.cy,
    r: RING_RADIUS,
    pathLength: 100,
    strokeDasharray: `${(progress * 100).toFixed(2)} 100`,
    transform: `rotate(-90 ${BODY.cx} ${BODY.cy})`,
  };

  return (
    <svg className="tomato" viewBox="0 0 100 100" aria-hidden="true">
      <defs>
        <radialGradient id="tomato-body" cx="38%" cy="32%" r="78%">
          <stop offset="0%" style={{ stopColor: "var(--tomato-light)" }} />
          <stop offset="100%" style={{ stopColor: "var(--tomato)" }} />
        </radialGradient>
      </defs>

      <circle
        className="ring-track"
        cx={BODY.cx}
        cy={BODY.cy}
        r={RING_RADIUS}
      />
      {progress > 0 && (
        <>
          {/* The white halo keeps the arc readable on any desktop color. */}
          <circle className="ring-halo" {...arc} />
          <circle className="ring-progress" {...arc} />
        </>
      )}

      <circle cx={BODY.cx} cy={BODY.cy} r={BODY.r} fill="url(#tomato-body)" />
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

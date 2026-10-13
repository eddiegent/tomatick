const LEAF_ANGLES = [-72, -36, 0, 36, 72];

// The tomato body, and the ring that hugs it.
const BODY = { cx: 50, cy: 53, r: 38 };
/**
 * Width of the ring including its black edges. Must match `.ring-track` and
 * `.ring-edge` in widget.css; `.ring-progress` is this less both edges.
 */
const RING_WIDTH = 4;
/** How far the ring overlaps the edge of the body, so no gap or seam shows. */
const RING_OVERLAP = 0.3;
/**
 * The ring has the same center as the body. Its inner black edge sits
 * `RING_OVERLAP` inside the body edge.
 */
const RING_RADIUS = BODY.r + RING_WIDTH / 2 - RING_OVERLAP;
/**
 * The ring is masked to what lies outside this circle, so its round ends and
 * the track never paint over the body.
 */
const RING_MASK_RADIUS = BODY.r - RING_OVERLAP;
/**
 * The ring breaks at the top where the leaves cross it: this many degrees
 * either side of 12 o'clock. The leaf tips reach about 17 degrees out; the
 * rest leaves room for the round ends and a clear gap.
 */
const LEAF_GAP_DEGREES = 24;
/** Share of the full circle the ring covers, out of a `pathLength` of 100. */
const RING_SPAN = (100 * (360 - 2 * LEAF_GAP_DEGREES)) / 360;

/**
 * The tomato, its calyx and the progress ring, drawn in a 100 by 100 box.
 * Colors come from CSS variables so the phase theme can restyle it.
 */
export function Tomato({ progress }: { progress: number }) {
  // Every part of the ring starts at the right end of the leaf gap and runs
  // clockwise, so the progress fills the ring and stops short of the leaves.
  const ring = {
    cx: BODY.cx,
    cy: BODY.cy,
    r: RING_RADIUS,
    pathLength: 100,
    transform: `rotate(${-90 + LEAF_GAP_DEGREES} ${BODY.cx} ${BODY.cy})`,
  };
  const arc = {
    ...ring,
    strokeDasharray: `${(progress * RING_SPAN).toFixed(2)} 100`,
  };

  return (
    <svg className="tomato" viewBox="0 0 100 100" aria-hidden="true">
      <defs>
        <radialGradient id="tomato-body" cx="38%" cy="32%" r="78%">
          <stop offset="0%" style={{ stopColor: "var(--tomato-light)" }} />
          <stop offset="100%" style={{ stopColor: "var(--tomato)" }} />
        </radialGradient>
        <mask id="tomato-ring-mask">
          <rect width="100" height="100" fill="#ffffff" />
          <circle
            cx={BODY.cx}
            cy={BODY.cy}
            r={RING_MASK_RADIUS}
            fill="#000000"
          />
        </mask>
      </defs>

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

      <g mask="url(#tomato-ring-mask)">
        <circle
          className="ring-track"
          {...ring}
          strokeDasharray={`${RING_SPAN.toFixed(2)} 100`}
        />
        {progress > 0 && (
          <>
            {/* Drawn wider underneath, it shows as thin black edges. */}
            <circle className="ring-edge" {...arc} />
            <circle className="ring-progress" {...arc} />
          </>
        )}
      </g>
    </svg>
  );
}

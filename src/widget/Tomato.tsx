const LEAF_ANGLES = [-72, -36, 0, 36, 72];

// The tomato body, and the ring that hugs it.
const BODY = { cx: 50, cy: 53, r: 38 };
/** Must match `.ring-progress` and `.ring-halo` in widget.css. */
const PROGRESS_WIDTH = 4.4;
const HALO_WIDTH = 5.4;
/** How far the ring overlaps the edge of the body, so no gap or seam shows. */
const RING_OVERLAP = 0.3;
/**
 * The ring has the same center as the body. The inner edge of the progress
 * arc sits `RING_OVERLAP` inside the body edge.
 */
const RING_RADIUS = BODY.r + PROGRESS_WIDTH / 2 - RING_OVERLAP;
/**
 * The ring is drawn last, in front of the leaves. Its white halo and the wide
 * track would also paint over the body, so the ring is masked to what lies
 * outside this circle, which is where the progress arc starts.
 */
const RING_MASK_RADIUS = BODY.r - RING_OVERLAP;
/**
 * The white halo only shows outside the progress arc, so its center is moved
 * out: its outer edge is 1.4 beyond the arc, its inner edge is just inside the
 * arc's, and no white shows on the tomato.
 */
const HALO_RADIUS = RING_RADIUS + (HALO_WIDTH - PROGRESS_WIDTH) / 2 + 0.4;

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

      {/* In front of the leaves. The track is see-through, so they show. */}
      <g mask="url(#tomato-ring-mask)">
        <circle
          className="ring-track"
          cx={BODY.cx}
          cy={BODY.cy}
          r={RING_RADIUS}
        />
        {progress > 0 && (
          <>
            {/* The white halo keeps the arc readable on any desktop color. */}
            <circle className="ring-halo" {...arc} r={HALO_RADIUS} />
            <circle className="ring-progress" {...arc} />
          </>
        )}
      </g>
    </svg>
  );
}

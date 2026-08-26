type Axis = { label: string; value: number; max?: number };

type Props = {
  data: Axis[];
  size?: number;
  caption?: string;
  rings?: number;
};

/** Editorial radar (spider) chart in pure SVG — bronze fill, ink stroke. */
export function RadarChart({ data, size = 240, caption, rings = 4 }: Props) {
  const n = data.length;
  if (n < 3) return null;
  const cx = size / 2;
  const cy = size / 2;
  const pad = 48;
  const r = size / 2 - pad;

  const angle = (i: number) => -Math.PI / 2 + (i / n) * Math.PI * 2;
  const max = Math.max(...data.map((d) => (d.max ?? 100)));

  // Axis points
  const axisPts = data.map((d, i) => {
    const a = angle(i);
    return { x: cx + r * Math.cos(a), y: cy + r * Math.sin(a), a, d };
  });

  // Value polygon
  const valuePts = data.map((d, i) => {
    const a = angle(i);
    const ratio = Math.min(1, d.value / (d.max ?? max));
    return { x: cx + r * ratio * Math.cos(a), y: cy + r * ratio * Math.sin(a) };
  });
  const valuePath = valuePts.map((p, i) => (i === 0 ? `M ${p.x} ${p.y}` : `L ${p.x} ${p.y}`)).join(" ") + " Z";

  // Rings
  const ringPaths: string[] = [];
  for (let k = 1; k <= rings; k++) {
    const rr = (r * k) / rings;
    const pts = Array.from({ length: n }, (_, i) => {
      const a = angle(i);
      return `${cx + rr * Math.cos(a)} ${cy + rr * Math.sin(a)}`;
    });
    ringPaths.push("M " + pts.join(" L ") + " Z");
  }

  return (
    <figure
      className="no-break"
      style={{ margin: "6mm 0", display: "flex", gap: "12mm", alignItems: "center" }}
    >
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} style={{ flexShrink: 0 }}>
        {/* Rings */}
        {ringPaths.map((p, i) => (
          <path
            key={`ring-${i}`}
            d={p}
            fill="none"
            stroke="var(--hairline)"
            strokeWidth={0.4}
          />
        ))}
        {/* Axes */}
        {axisPts.map((p, i) => (
          <line
            key={`axis-${i}`}
            x1={cx}
            y1={cy}
            x2={p.x}
            y2={p.y}
            stroke="var(--hairline)"
            strokeWidth={0.4}
          />
        ))}
        {/* Value area */}
        <path
          d={valuePath}
          fill="var(--accent)"
          fillOpacity={0.18}
          stroke="var(--accent)"
          strokeWidth={1.4}
          strokeLinejoin="round"
        />
        {/* Value points */}
        {valuePts.map((p, i) => (
          <circle key={`pt-${i}`} cx={p.x} cy={p.y} r={2.2} fill="var(--accent)" />
        ))}
        {/* Axis labels */}
        {axisPts.map((p, i) => {
          const a = p.a;
          const labelR = r + 18;
          const lx = cx + labelR * Math.cos(a);
          const ly = cy + labelR * Math.sin(a);
          const anchor = Math.cos(a) > 0.2 ? "start" : Math.cos(a) < -0.2 ? "end" : "middle";
          // Truncate long labels
          const text = p.d.label.length > 14 ? p.d.label.slice(0, 12) + "…" : p.d.label;
          return (
            <text
              key={`lbl-${i}`}
              x={lx}
              y={ly}
              textAnchor={anchor}
              dominantBaseline="middle"
              style={{
                fontFamily: "var(--mono)",
                fontSize: 6.5,
                letterSpacing: "0.1em",
                textTransform: "uppercase",
                fill: "var(--ink)"
              }}
            >
              {text}
            </text>
          );
        })}
      </svg>
      <div style={{ flex: 1 }}>
        <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "9pt" }}>
          <tbody>
            {data.map((d, i) => (
              <tr
                key={i}
                style={{ borderBottom: "0.4pt solid var(--hairline)" }}
              >
                <td
                  style={{
                    padding: "3pt 0",
                    fontFamily: "var(--sans)",
                    fontWeight: 600,
                    color: "var(--ink)"
                  }}
                >
                  {d.label}
                </td>
                <td
                  className="num"
                  style={{
                    padding: "3pt 0",
                    fontFamily: "var(--mono)",
                    fontSize: "8.5pt",
                    textAlign: "right",
                    color: "var(--ink)"
                  }}
                >
                  {d.value}
                  {d.max != null ? `/${d.max}` : ""}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {caption && (
          <p
            className="caption"
            style={{ marginTop: "3mm", fontSize: "7.5pt", color: "var(--mute)" }}
          >
            {caption}
          </p>
        )}
      </div>
    </figure>
  );
}

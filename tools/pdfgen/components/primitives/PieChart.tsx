type Slice = {
  label: string;
  value: number;
  note?: string;
};

type Props = {
  data: Slice[];
  caption?: string;
  size?: number;
  /** Inner radius ratio for donut style (0 = full pie) */
  donut?: number;
};

/**
 * Editorial pie/donut chart in pure SVG.
 * Monochrome — each slice differentiated by shade (no decorative colors).
 */
export function PieChart({ data, caption, size = 200, donut = 0 }: Props) {
  const total = data.reduce((s, d) => s + d.value, 0);
  const cx = size / 2;
  const cy = size / 2;
  const r = size / 2 - 8;
  const rInner = r * donut;

  // Build slice paths
  let angle = -Math.PI / 2; // start at top
  const slices = data.map((d, i) => {
    const sweep = (d.value / total) * Math.PI * 2;
    const a0 = angle;
    const a1 = angle + sweep;
    angle = a1;
    const x0 = cx + r * Math.cos(a0);
    const y0 = cy + r * Math.sin(a0);
    const x1 = cx + r * Math.cos(a1);
    const y1 = cy + r * Math.sin(a1);
    const large = sweep > Math.PI ? 1 : 0;

    let path: string;
    if (donut > 0) {
      const xi0 = cx + rInner * Math.cos(a0);
      const yi0 = cy + rInner * Math.sin(a0);
      const xi1 = cx + rInner * Math.cos(a1);
      const yi1 = cy + rInner * Math.sin(a1);
      path = [
        `M ${x0} ${y0}`,
        `A ${r} ${r} 0 ${large} 1 ${x1} ${y1}`,
        `L ${xi1} ${yi1}`,
        `A ${rInner} ${rInner} 0 ${large} 0 ${xi0} ${yi0}`,
        "Z"
      ].join(" ");
    } else {
      path = [
        `M ${cx} ${cy}`,
        `L ${x0} ${y0}`,
        `A ${r} ${r} 0 ${large} 1 ${x1} ${y1}`,
        "Z"
      ].join(" ");
    }

    // Monochrome shades: black → mid → light grey
    const shades = ["#000000", "#555555", "#AAAAAA", "#DDDDDD"];
    const fill = shades[i % shades.length];

    // Label position at mid-angle
    const aMid = (a0 + a1) / 2;
    const labelR = r + 18;
    const lx = cx + labelR * Math.cos(aMid);
    const ly = cy + labelR * Math.sin(aMid);
    const anchor = Math.cos(aMid) > 0.2 ? "start" : Math.cos(aMid) < -0.2 ? "end" : "middle";

    return { path, fill, label: d.label, value: d.value, lx, ly, anchor, pct: (d.value / total) * 100 };
  });

  return (
    <figure
      className="no-break"
      style={{ display: "flex", gap: "10mm", alignItems: "center", margin: "6mm 0" }}
    >
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} style={{ flexShrink: 0 }}>
        {slices.map((s, i) => (
          <g key={i}>
            <path d={s.path} fill={s.fill} stroke="#FFFFFF" strokeWidth={1.5} />
          </g>
        ))}
      </svg>
      {/* Legend */}
      <div style={{ flex: 1 }}>
        <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "9pt" }}>
          <tbody>
            {slices.map((s, i) => (
              <tr key={i}>
                <td style={{ padding: "3pt 0", width: "10mm" }}>
                  <span
                    style={{
                      display: "inline-block",
                      width: 8,
                      height: 8,
                      background: s.fill,
                      border: s.fill === "#FFFFFF" || s.fill === "#DDDDDD" ? "0.5pt solid var(--ink)" : "none"
                    }}
                  />
                </td>
                <td
                  style={{
                    padding: "3pt 6pt",
                    fontFamily: "var(--sans)",
                    fontWeight: 600,
                    color: "var(--ink)"
                  }}
                >
                  {s.label}
                </td>
                <td
                  className="num"
                  style={{
                    padding: "3pt 0",
                    fontFamily: "var(--mono)",
                    fontSize: "9pt",
                    textAlign: "right",
                    color: "var(--ink)"
                  }}
                >
                  {s.value}%
                </td>
              </tr>
            ))}
            <tr style={{ borderTop: "0.75pt solid var(--ink)" }}>
              <td />
              <td
                style={{
                  padding: "3pt 6pt",
                  fontFamily: "var(--mono)",
                  fontSize: "7.5pt",
                  letterSpacing: "0.08em",
                  textTransform: "uppercase",
                  color: "var(--mute)"
                }}
              >
                Total
              </td>
              <td
                className="num"
                style={{
                  padding: "3pt 0",
                  fontFamily: "var(--mono)",
                  fontSize: "9pt",
                  textAlign: "right",
                  color: "var(--ink)"
                }}
              >
                {total}%
              </td>
            </tr>
          </tbody>
        </table>
        {caption && (
          <p
            className="caption"
            style={{ marginTop: "3mm", color: "var(--mute)", fontSize: "7.5pt" }}
          >
            {caption}
          </p>
        )}
      </div>
    </figure>
  );
}

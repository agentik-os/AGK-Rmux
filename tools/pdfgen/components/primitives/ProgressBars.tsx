type Bar = {
  label: string;
  value: number;     // 0-100
  note?: string;     // optional secondary text
};

type Props = {
  data: Bar[];
  caption?: string;
  /** Bar height in pt */
  thickness?: number;
};

/** Editorial horizontal bar matrix — replaces ASCII [====----] patterns. */
export function ProgressBars({ data, caption, thickness = 5 }: Props) {
  const maxValue = Math.max(...data.map((d) => d.value), 100);
  return (
    <figure
      className="no-break"
      style={{ margin: "5mm 0", maxWidth: "165mm" }}
    >
      <table style={{ width: "100%", borderCollapse: "collapse" }}>
        <tbody>
          {data.map((b, i) => (
            <tr key={i}>
              <td
                style={{
                  width: "26mm",
                  padding: "3pt 6pt 3pt 0",
                  fontFamily: "var(--sans)",
                  fontWeight: 600,
                  fontSize: "9pt",
                  color: "var(--ink)",
                  verticalAlign: "middle",
                  letterSpacing: "0.01em"
                }}
              >
                {b.label}
              </td>
              <td
                style={{
                  padding: "3pt 6pt",
                  verticalAlign: "middle"
                }}
              >
                <div
                  style={{
                    position: "relative",
                    height: `${thickness * 1.5}pt`,
                    background: "var(--paper-alt)",
                    border: "0.4pt solid var(--hairline)"
                  }}
                >
                  <div
                    style={{
                      position: "absolute",
                      left: 0,
                      top: 0,
                      bottom: 0,
                      width: `${(b.value / maxValue) * 100}%`,
                      background: "var(--ink)"
                    }}
                  />
                </div>
              </td>
              <td
                style={{
                  width: "18mm",
                  padding: "3pt 0 3pt 6pt",
                  fontFamily: "var(--mono)",
                  fontSize: "9pt",
                  textAlign: "right",
                  color: "var(--ink)",
                  verticalAlign: "middle"
                }}
                className="num"
              >
                {b.value}%
              </td>
              {data.some((d) => d.note) && (
                <td
                  style={{
                    width: "55mm",
                    padding: "3pt 0 3pt 8pt",
                    fontFamily: "var(--sans)",
                    fontSize: "8pt",
                    color: "var(--mute)",
                    verticalAlign: "middle"
                  }}
                >
                  {b.note || ""}
                </td>
              )}
            </tr>
          ))}
        </tbody>
      </table>
      {caption && (
        <figcaption
          className="caption"
          style={{
            marginTop: "3mm",
            fontFamily: "var(--mono)",
            fontSize: "7.5pt",
            color: "var(--mute)"
          }}
        >
          {caption}
        </figcaption>
      )}
    </figure>
  );
}

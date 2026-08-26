type Stat = {
  label: string;
  value: string | number;
  unit?: string;
  note?: string;
};

type Props = {
  items: Stat[];
  /** 2, 3 or 4 columns */
  columns?: 2 | 3 | 4;
};

/** Editorial stat grid — large numerals, monospace labels above. */
export function StatCard({ items, columns }: Props) {
  const cols = columns || Math.min(items.length, 3);
  return (
    <div
      className="no-break"
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
        gap: 0,
        margin: "6mm 0",
        borderTop: "1pt solid var(--ink)",
        borderBottom: "0.5pt solid var(--hairline)"
      }}
    >
      {items.map((s, i) => (
        <div
          key={i}
          style={{
            padding: "5mm 6mm",
            borderRight: i < items.length - 1 ? "0.4pt solid var(--hairline)" : "none",
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
            gap: "2mm"
          }}
        >
          <div
            className="t-eyebrow"
            style={{ fontSize: "6.5pt", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}
          >
            {s.label}
          </div>
          <div
            style={{
              display: "flex",
              alignItems: "baseline",
              gap: "3pt"
            }}
          >
            <span
              className="num"
              style={{
                fontFamily: "var(--sans)",
                fontWeight: 700,
                fontSize: "26pt",
                lineHeight: 0.95,
                letterSpacing: "-0.028em",
                color: "var(--ink)"
              }}
            >
              {s.value}
            </span>
            {s.unit && (
              <span
                style={{
                  fontFamily: "var(--mono)",
                  fontSize: "8.5pt",
                  color: "var(--mute)",
                  letterSpacing: "0.04em"
                }}
              >
                {s.unit}
              </span>
            )}
          </div>
          {s.note && (
            <div
              style={{
                fontFamily: "var(--sans)",
                fontSize: "7.5pt",
                lineHeight: 1.3,
                color: "var(--slate)",
                overflow: "hidden"
              }}
            >
              {s.note}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

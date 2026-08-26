type Item = {
  label: string;
  value: string | number;
  unit?: string;
  delta?: string;
  deltaTone?: "up" | "down" | "neutral";
};

type Props = {
  items: Item[];
};

export function KPIGrid({ items }: Props) {
  const cols = Math.min(items.length, 4);
  return (
    <div
      className="no-break"
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
        gap: 0,
        margin: "8mm 0",
        borderTop: "1px solid var(--ink)",
        borderBottom: "1px solid var(--hairline)"
      }}
    >
      {items.map((k, i) => (
        <div
          key={i}
          style={{
            padding: "10pt 12pt",
            borderRight: i < items.length - 1 ? "1px solid var(--hairline)" : "none"
          }}
        >
          <div
            className="t-eyebrow"
            style={{ fontSize: "7pt", marginBottom: "6pt" }}
          >
            {k.label}
          </div>
          <div style={{ display: "flex", alignItems: "baseline", gap: "6pt" }}>
            <span
              className="num"
              style={{
                fontFamily: "var(--sans)",
                fontWeight: 700,
                fontSize: "26pt",
                lineHeight: 1,
                letterSpacing: "-0.025em",
                color: "var(--ink)"
              }}
            >
              {k.value}
            </span>
            {k.unit && (
              <span
                className="caption"
                style={{ color: "var(--mute)", fontSize: "8.5pt" }}
              >
                {k.unit}
              </span>
            )}
          </div>
          {k.delta && (
            <div
              className="caption num"
              style={{
                marginTop: "3mm",
                color:
                  k.deltaTone === "up" ? "var(--accent)" :
                  k.deltaTone === "down" ? "#9A2A2A" :
                  "var(--mute)"
              }}
            >
              {k.delta}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

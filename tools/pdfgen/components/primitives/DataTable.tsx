type Col = {
  key: string;
  label: string;
  align?: "left" | "right" | "center";
  width?: string;
  mono?: boolean;
};

type Props = {
  columns: Col[];
  rows: Array<Record<string, string | number>>;
  caption?: string;
};

export function DataTable({ columns, rows, caption }: Props) {
  return (
    <figure className="no-break my-6">
      <table
        className="w-full"
        style={{
          borderCollapse: "collapse",
          fontSize: "10pt",
          tableLayout: "fixed"
        }}
      >
        <thead>
          <tr>
            {columns.map((c) => (
              <th
                key={c.key}
                style={{
                  textAlign: c.align || "left",
                  width: c.width,
                  padding: "6pt 8pt",
                  borderBottom: "1px solid var(--ink)",
                  fontFamily: "var(--sans)",
                  fontWeight: 600,
                  fontSize: "8.5pt",
                  letterSpacing: "0.06em",
                  textTransform: "uppercase",
                  color: "var(--ink)"
                }}
              >
                {c.label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((r, i) => (
            <tr
              key={i}
              style={{
                borderBottom: "1px solid var(--hairline)",
                background: i % 2 === 1 ? "var(--paper-alt)" : "transparent"
              }}
            >
              {columns.map((c) => (
                <td
                  key={c.key}
                  style={{
                    textAlign: c.align || "left",
                    padding: "7pt 8pt",
                    fontFamily: c.mono || c.align === "right" ? "var(--mono)" : "var(--sans)",
                    fontVariantNumeric: "tabular-nums lining-nums",
                    color: "var(--ink-soft)"
                  }}
                >
                  {r[c.key]}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {caption && (
        <figcaption className="caption mt-2" style={{ color: "var(--mute)" }}>
          {caption}
        </figcaption>
      )}
    </figure>
  );
}

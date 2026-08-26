type Props = {
  index?: string;
  eyebrow?: string;
  title: string;
  lead?: string;
};

export function SectionTitle({ index, eyebrow, title, lead }: Props) {
  return (
    <header className="no-break" style={{ marginBottom: "8mm" }}>
      {(eyebrow || index) && (
        <div
          style={{
            display: "flex",
            alignItems: "baseline",
            gap: "6mm",
            marginBottom: "3mm"
          }}
        >
          {index && (
            <span
              className="caption num"
              style={{
                fontFamily: "var(--mono)",
                fontWeight: 500,
                fontSize: "8pt",
                color: "var(--accent)",
                letterSpacing: "0.02em"
              }}
            >
              {index}
            </span>
          )}
          {eyebrow && <span className="t-eyebrow">{eyebrow}</span>}
        </div>
      )}
      <h2 className="t-1" style={{ maxWidth: "165mm" }}>
        {title}
      </h2>
      {lead && (
        <p
          className="body--lead"
          style={{
            marginTop: "4mm",
            marginBottom: 0,
            maxWidth: "150mm"
          }}
        >
          {lead}
        </p>
      )}
      <div
        style={{
          borderTop: "0.5pt solid var(--ink)",
          marginTop: "5mm",
          width: "100%"
        }}
      />
    </header>
  );
}

type Props = {
  number: string;
  label?: string;
  title: string;
  subtitle?: string;
};

/** Full-page section divider — dedicated CSS class, absolute positioning, no flex. */
export function PartDivider({ number, label = "PARTIE", title, subtitle }: Props) {
  return (
    <section className="pdf-divider">
      {/* Top: bronze bar + label */}
      <div style={{ position: "absolute", top: "20mm", left: "22mm", right: "22mm" }}>
        <div
          style={{
            width: "20mm",
            height: "2pt",
            background: "var(--accent)",
            marginBottom: "5mm"
          }}
        />
        <div className="t-eyebrow">{label}</div>
      </div>

      {/* Massive numeral — vertically anchored ~32% from top */}
      <div
        style={{
          position: "absolute",
          top: "90mm",
          left: "22mm",
          right: "22mm"
        }}
      >
        <div
          className="num"
          style={{
            fontFamily: "var(--sans)",
            fontWeight: 800,
            fontSize: "160pt",
            lineHeight: 0.85,
            letterSpacing: "-0.055em",
            color: "var(--ink)"
          }}
        >
          {number}
        </div>
      </div>

      {/* Title block — fixed position below numeral */}
      <div
        style={{
          position: "absolute",
          top: "175mm",
          left: "22mm",
          right: "22mm"
        }}
      >
        <div
          style={{
            borderTop: "1pt solid var(--ink)",
            paddingTop: "6mm",
            maxWidth: "165mm"
          }}
        >
          <h1
            style={{
              fontFamily: "var(--sans)",
              fontWeight: 700,
              fontSize: "30pt",
              lineHeight: 1.05,
              letterSpacing: "-0.022em",
              color: "var(--ink)",
              margin: 0
            }}
          >
            {title}
          </h1>
          {subtitle && (
            <p
              style={{
                marginTop: "5mm",
                marginBottom: 0,
                fontFamily: "var(--serif)",
                fontStyle: "italic",
                fontWeight: 400,
                fontSize: "13pt",
                lineHeight: 1.4,
                color: "var(--slate)",
                maxWidth: "140mm"
              }}
            >
              {subtitle}
            </p>
          )}
        </div>
      </div>

      {/* Bottom brand */}
      <div
        style={{
          position: "absolute",
          bottom: "20mm",
          left: "22mm",
          fontFamily: "var(--mono)",
          fontSize: "7.5pt",
          color: "var(--mute)",
          letterSpacing: "0.1em",
          textTransform: "uppercase"
        }}
      >
        OmegaOS · WHITEPAPER
      </div>
    </section>
  );
}

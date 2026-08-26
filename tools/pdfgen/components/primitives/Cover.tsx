import { Page } from "./Page";

type Props = {
  eyebrow?: string;
  title: string;
  subtitle?: string;
  author?: string;
  date?: string;
  docId?: string;
  brand?: string;
};

export function Cover({ eyebrow, title, subtitle, author, date, docId, brand = "OmegaOS" }: Props) {
  return (
    <Page variant="cover">
      <div
        style={{
          height: "100%",
          width: "100%",
          padding: "var(--gutter-top) var(--gutter) var(--gutter-bottom) var(--gutter)",
          display: "flex",
          flexDirection: "column",
          position: "relative"
        }}
      >
        {/* Top: bronze bar + eyebrow */}
        <div>
          <div
            style={{
              width: "20mm",
              height: "2pt",
              background: "var(--accent)",
              marginBottom: "5mm"
            }}
          />
          <div className="t-eyebrow">{eyebrow || brand}</div>
        </div>

        {/* Vertical brand mark */}
        <div
          className="caption"
          style={{
            position: "absolute",
            top: "50%",
            right: "calc(var(--gutter) - 4mm)",
            transform: "rotate(90deg) translateX(50%)",
            transformOrigin: "right",
            letterSpacing: "0.42em",
            color: "var(--mute)"
          }}
        >
          {brand}
        </div>

        {/* Title block */}
        <div style={{ marginTop: "auto", marginBottom: "20mm" }}>
          <h1 className="t-display" style={{ maxWidth: "175mm" }}>
            {title}
          </h1>
          {subtitle && (
            <p
              style={{
                marginTop: "8mm",
                marginBottom: 0,
                fontFamily: "var(--serif)",
                fontStyle: "italic",
                fontWeight: 400,
                fontSize: "13pt",
                lineHeight: 1.45,
                color: "var(--slate)",
                maxWidth: "165mm"
              }}
            >
              {subtitle}
            </p>
          )}
          <div
            style={{
              borderTop: "0.5pt solid var(--ink)",
              marginTop: "10mm",
              paddingTop: "4mm",
              display: "flex",
              alignItems: "baseline",
              gap: "8mm",
              fontFamily: "var(--mono)",
              fontSize: "8pt",
              color: "var(--mute)",
              letterSpacing: "0.04em"
            }}
          >
            {date && <span>{date}</span>}
            {docId && (
              <>
                <span aria-hidden>·</span>
                <span>{docId}</span>
              </>
            )}
            {author && (
              <>
                <span aria-hidden>·</span>
                <span>{author}</span>
              </>
            )}
          </div>
        </div>

        {/* Bottom brand */}
        <div
          style={{
            fontFamily: "var(--mono)",
            fontSize: "7.5pt",
            color: "var(--mute)",
            letterSpacing: "0.1em",
            textTransform: "uppercase"
          }}
        >
          {brand}
        </div>
      </div>
    </Page>
  );
}

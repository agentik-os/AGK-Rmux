type Props = {
  text: string;
  cite?: string;
};

/** Full-width editorial pull quote — large serif italic, hairline above and below. */
export function PullQuote({ text, cite }: Props) {
  return (
    <figure
      className="no-break"
      style={{
        margin: "12mm 0 10mm 0",
        maxWidth: "165mm",
        borderTop: "1pt solid var(--ink)",
        borderBottom: "0.5pt solid var(--hairline)",
        paddingTop: "8mm",
        paddingBottom: "8mm"
      }}
    >
      <blockquote
        style={{
          margin: 0,
          fontFamily: "var(--serif)",
          fontWeight: 400,
          fontStyle: "italic",
          fontSize: "18pt",
          lineHeight: 1.3,
          letterSpacing: "-0.005em",
          color: "var(--ink)"
        }}
      >
        <span
          aria-hidden
          style={{
            fontFamily: "var(--serif)",
            fontStyle: "normal",
            fontSize: "22pt",
            verticalAlign: "-3pt",
            marginRight: "1pt",
            color: "var(--ink)"
          }}
        >
          “
        </span>
        {text}
        <span
          aria-hidden
          style={{
            fontFamily: "var(--serif)",
            fontStyle: "normal",
            fontSize: "22pt",
            verticalAlign: "-3pt",
            marginLeft: "1pt",
            color: "var(--ink)"
          }}
        >
          ”
        </span>
      </blockquote>
      {cite && (
        <figcaption
          className="caption"
          style={{
            marginTop: "5mm",
            fontFamily: "var(--mono)",
            fontSize: "7.5pt",
            letterSpacing: "0.06em",
            color: "var(--mute)"
          }}
        >
          — {cite}
        </figcaption>
      )}
    </figure>
  );
}

type Props = {
  text: string;
  cite?: string;
};

export function Quote({ text, cite }: Props) {
  return (
    <blockquote
      className="no-break"
      style={{
        margin: "8mm 0",
        maxWidth: "160mm",
        borderLeft: "3px solid var(--accent)",
        paddingLeft: "8mm"
      }}
    >
      <p
        style={{
          margin: 0,
          fontFamily: "var(--sans)",
          fontWeight: 500,
          fontSize: "16pt",
          lineHeight: 1.4,
          letterSpacing: "-0.012em",
          color: "var(--ink)"
        }}
      >
        {text}
      </p>
      {cite && (
        <footer
          className="caption"
          style={{ marginTop: "3mm", color: "var(--mute)" }}
        >
          — {cite}
        </footer>
      )}
    </blockquote>
  );
}

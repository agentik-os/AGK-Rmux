import { ReactNode } from "react";

type Props = {
  label?: string;
  children: ReactNode;
  tone?: "neutral" | "accent";
};

export function Callout({ label = "Note", children, tone = "neutral" }: Props) {
  const isAccent = tone === "accent";
  return (
    <aside
      className="no-break"
      style={{
        margin: "6mm 0",
        display: "grid",
        gridTemplateColumns: "20mm 1fr",
        gap: "6mm"
      }}
    >
      <div
        className="t-eyebrow"
        style={{
          color: isAccent ? "var(--accent)" : "var(--mute)",
          paddingTop: "1mm"
        }}
      >
        {label}
      </div>
      <div
        className="body"
        style={{
          borderLeft: `2px solid ${isAccent ? "var(--accent)" : "var(--hairline)"}`,
          paddingLeft: "6mm",
          color: "var(--ink-soft)",
          margin: 0
        }}
      >
        {children}
      </div>
    </aside>
  );
}

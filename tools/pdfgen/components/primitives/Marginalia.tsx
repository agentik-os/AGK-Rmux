import { ReactNode } from "react";

type Props = {
  label?: string;
  children: ReactNode;
};

/** Editorial side-note — bronze stripe + small mono label + body sans. */
export function Marginalia({ label = "Note", children }: Props) {
  return (
    <aside
      className="no-break"
      style={{
        display: "grid",
        gridTemplateColumns: "32mm 1fr",
        gap: "6mm",
        margin: "6mm 0",
        paddingTop: "3mm",
        borderTop: "0.4pt solid var(--hairline)",
        borderBottom: "0.4pt solid var(--hairline)",
        paddingBottom: "3mm"
      }}
    >
      <div>
        <div
          className="t-eyebrow"
          style={{ marginBottom: "2mm", fontSize: "7pt" }}
        >
          {label}
        </div>
        <div
          style={{
            width: "10mm",
            height: "2pt",
            background: "var(--accent)"
          }}
        />
      </div>
      <div
        style={{
          fontFamily: "var(--sans)",
          fontSize: "9pt",
          lineHeight: 1.55,
          color: "var(--ink-soft)"
        }}
      >
        {children}
      </div>
    </aside>
  );
}

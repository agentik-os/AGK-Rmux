type Props = {
  name: string;
  role: string;
  age?: string;
  income?: string;
  pains: string[];
  goals: string[];
  channels?: string[];
};

export function PersonaCard({ name, role, age, income, pains, goals, channels }: Props) {
  return (
    <article
      className="no-break"
      style={{
        border: "1px solid var(--hairline)",
        padding: "10mm",
        breakInside: "avoid",
        background: "var(--paper)"
      }}
    >
      <header
        style={{
          borderBottom: "1px solid var(--ink)",
          paddingBottom: "5mm",
          marginBottom: "6mm"
        }}
      >
        <div className="t-eyebrow" style={{ color: "var(--accent)", marginBottom: "3mm" }}>
          {role}
        </div>
        <h3 className="t-1" style={{ fontSize: "22pt", margin: 0 }}>
          {name}
        </h3>
        {(age || income) && (
          <div
            className="caption"
            style={{ marginTop: "3mm", display: "flex", gap: "8mm" }}
          >
            {age && <span>{age}</span>}
            {income && <span>{income}</span>}
          </div>
        )}
      </header>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: "8mm"
        }}
      >
        <div>
          <div className="t-eyebrow" style={{ marginBottom: "3mm" }}>Pains</div>
          <ul className="body" style={{ listStyle: "none", padding: 0, margin: 0 }}>
            {pains.map((p, i) => (
              <li
                key={i}
                style={{
                  paddingLeft: "4mm",
                  textIndent: "-4mm",
                  marginBottom: "2mm"
                }}
              >
                <span style={{ color: "var(--accent)" }}>— </span>
                {p}
              </li>
            ))}
          </ul>
        </div>
        <div>
          <div className="t-eyebrow" style={{ marginBottom: "3mm" }}>Goals</div>
          <ul className="body" style={{ listStyle: "none", padding: 0, margin: 0 }}>
            {goals.map((g, i) => (
              <li
                key={i}
                style={{
                  paddingLeft: "4mm",
                  textIndent: "-4mm",
                  marginBottom: "2mm"
                }}
              >
                <span style={{ color: "var(--accent)" }}>— </span>
                {g}
              </li>
            ))}
          </ul>
        </div>
      </div>
      {channels && channels.length > 0 && (
        <footer
          className="caption"
          style={{
            marginTop: "6mm",
            paddingTop: "4mm",
            borderTop: "1px solid var(--hairline)",
            color: "var(--mute)"
          }}
        >
          CHANNELS · {channels.join(" · ")}
        </footer>
      )}
    </article>
  );
}

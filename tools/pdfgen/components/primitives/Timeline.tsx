type Milestone = {
  when: string;
  title: string;
  desc?: string;
};

type Props = {
  milestones: Milestone[];
};

export function Timeline({ milestones }: Props) {
  return (
    <ol style={{ listStyle: "none", padding: 0, margin: "8mm 0 0 0" }}>
      {milestones.map((m, i) => (
        <li
          key={i}
          className="no-break"
          style={{
            display: "grid",
            gridTemplateColumns: "26mm 6mm 1fr",
            columnGap: 0,
            paddingBottom: i < milestones.length - 1 ? "10mm" : 0,
            position: "relative"
          }}
        >
          <div className="caption num" style={{ color: "var(--accent)", paddingTop: "1mm" }}>
            {m.when}
          </div>
          <div style={{ position: "relative" }}>
            <span
              style={{
                position: "absolute",
                left: 0,
                top: "2mm",
                width: 6,
                height: 6,
                borderRadius: 9999,
                background: "var(--accent)"
              }}
            />
            <span
              style={{
                position: "absolute",
                left: "2.5px",
                top: "8mm",
                bottom: 0,
                width: 1,
                background: i < milestones.length - 1 ? "var(--hairline)" : "transparent"
              }}
            />
          </div>
          <div style={{ paddingLeft: "0", minWidth: 0 }}>
            <h3 className="t-2" style={{ margin: 0 }}>
              {m.title}
            </h3>
            {m.desc && (
              <p
                className="body"
                style={{ margin: "2mm 0 0 0", maxWidth: "130mm" }}
              >
                {m.desc}
              </p>
            )}
          </div>
        </li>
      ))}
    </ol>
  );
}

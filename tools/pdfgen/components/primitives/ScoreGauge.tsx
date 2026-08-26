type Props = {
  score: number;
  label?: string;
  size?: number;
};

export function ScoreGauge({ score, label = "Score", size = 140 }: Props) {
  const clamped = Math.max(0, Math.min(100, score));
  const r = size / 2 - 8;
  const c = 2 * Math.PI * r;
  const dash = (clamped / 100) * c;

  return (
    <figure
      className="no-break"
      style={{ display: "inline-block", width: size, margin: 0 }}
    >
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke="var(--hairline)"
          strokeWidth={3}
        />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke="var(--accent)"
          strokeWidth={4}
          strokeDasharray={`${dash} ${c - dash}`}
          strokeDashoffset={c / 4}
          strokeLinecap="butt"
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
        />
        <text
          x="50%"
          y="49%"
          textAnchor="middle"
          dominantBaseline="middle"
          style={{
            fontFamily: "var(--sans)",
            fontWeight: 700,
            fontSize: size * 0.32,
            fill: "var(--ink)",
            letterSpacing: "-0.025em"
          }}
        >
          {clamped}
        </text>
        <text
          x="50%"
          y="63%"
          textAnchor="middle"
          style={{
            fontFamily: "var(--mono)",
            fontSize: size * 0.07,
            fill: "var(--mute)",
            letterSpacing: "0.14em",
            textTransform: "uppercase"
          }}
        >
          /100
        </text>
      </svg>
      {label && (
        <figcaption
          className="caption"
          style={{ textAlign: "center", marginTop: "2mm", color: "var(--mute)" }}
        >
          {label}
        </figcaption>
      )}
    </figure>
  );
}

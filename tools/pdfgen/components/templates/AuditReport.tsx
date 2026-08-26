import {
  Cover,
  Page,
  SectionTitle,
  FooterMeta,
  ScoreGauge,
  KPIGrid,
  BarChart,
  Timeline,
  Callout,
  DataTable
} from "../primitives";
import type { AuditData } from "../../lib/schemas";

const SEV_LABEL: Record<string, string> = {
  critical: "Critical",
  high: "High",
  medium: "Medium",
  low: "Low"
};

export function AuditReport({ data }: { data: AuditData }) {
  const brand = data.brand || "OmegaOS";
  return (
    <>
      <Cover
        eyebrow={data.eyebrow || "AUDIT REPORT"}
        title={data.title}
        subtitle={data.subtitle}
        author={data.author}
        date={data.date}
        docId={data.docId}
        brand={brand}
      />

      <Page>
        <SectionTitle index="01" eyebrow="Executive Summary" title="Overall verdict" />
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr 60mm",
            gap: "12mm",
            alignItems: "start",
            marginTop: "2mm"
          }}
        >
          <p
            className="body--lead"
            style={{ margin: 0, color: "var(--ink)", maxWidth: "120mm" }}
          >
            {data.verdict}
          </p>
          <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end" }}>
            <ScoreGauge score={data.overallScore} label="Overall" size={150} />
          </div>
        </div>

        {data.kpis && data.kpis.length > 0 && <KPIGrid items={data.kpis} />}

        <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
          <FooterMeta left={brand} center="Audit / Summary" right="02" />
        </div>
      </Page>

      <Page>
        <SectionTitle index="02" eyebrow="Breakdown" title="Domain scores" />
        <BarChart
          data={data.domains.map((d) => ({ label: d.label, value: d.score }))}
          caption="Score per domain — 0 to 100, higher is better."
          height={260}
        />
        <DataTable
          columns={[
            { key: "label", label: "Domain" },
            { key: "weight", label: "Weight", align: "right", mono: true, width: "24mm" },
            { key: "score", label: "Score", align: "right", mono: true, width: "24mm" }
          ]}
          rows={data.domains.map((d) => ({
            label: d.label,
            weight: d.weight != null ? `${d.weight}%` : "—",
            score: `${d.score}`
          }))}
        />
        <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
          <FooterMeta left={brand} center="Audit / Breakdown" right="03" />
        </div>
      </Page>

      <Page>
        <SectionTitle
          index="03"
          eyebrow="Findings"
          title="What we found"
          lead={`${data.findings.length} issues identified, ordered by severity.`}
        />
        <ol style={{ listStyle: "none", padding: 0, margin: 0 }}>
          {data.findings.map((f, i) => (
            <li
              key={i}
              className="no-break"
              style={{
                borderBottom: "1px solid var(--hairline)",
                padding: "6mm 0"
              }}
            >
              <div style={{ display: "flex", alignItems: "baseline", gap: "6mm" }}>
                <span
                  className="caption num"
                  style={{ minWidth: "12mm", color: "var(--accent)" }}
                >
                  {String(i + 1).padStart(2, "0")}
                </span>
                <span
                  className="t-eyebrow"
                  style={{
                    minWidth: "20mm",
                    color:
                      f.severity === "critical" ? "#9A2A2A" :
                      f.severity === "high" ? "var(--ink)" :
                      f.severity === "medium" ? "var(--slate)" :
                      "var(--mute)"
                  }}
                >
                  {SEV_LABEL[f.severity]}
                </span>
                <h3 className="t-2" style={{ flex: 1, margin: 0 }}>
                  {f.title}
                </h3>
              </div>
              <div
                style={{
                  marginTop: "3mm",
                  display: "grid",
                  gridTemplateColumns: "38mm 1fr",
                  gap: 0
                }}
              >
                <span />
                <div>
                  <p className="body" style={{ margin: 0 }}>{f.description}</p>
                  {f.recommendation && (
                    <Callout label="Fix" tone="accent">{f.recommendation}</Callout>
                  )}
                  {f.evidence && (
                    <pre
                      style={{
                        fontFamily: "var(--mono)",
                        fontSize: "8.5pt",
                        background: "var(--paper-alt)",
                        padding: "4mm",
                        marginTop: "3mm",
                        whiteSpace: "pre-wrap",
                        wordBreak: "break-word"
                      }}
                    >
                      {f.evidence}
                    </pre>
                  )}
                </div>
              </div>
            </li>
          ))}
        </ol>
        <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
          <FooterMeta left={brand} center="Audit / Findings" right="04" />
        </div>
      </Page>

      <Page>
        <SectionTitle
          index="04"
          eyebrow="Roadmap"
          title="Action plan"
          lead="Prioritized timeline to close every finding above."
        />
        <Timeline milestones={data.actionPlan} />
        <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
          <FooterMeta left={brand} center="Audit / Roadmap" right="05" />
        </div>
      </Page>
    </>
  );
}

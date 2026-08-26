import {
  Cover,
  Page,
  SectionTitle,
  FooterMeta,
  KPIGrid,
  BarChart,
  PersonaCard,
  Callout
} from "../primitives";
import type { MarketingData } from "../../lib/schemas";

export function MarketingReport({ data }: { data: MarketingData }) {
  const brand = data.brand || "OmegaOS";

  return (
    <>
      <Cover
        eyebrow={data.eyebrow || "MARKETING REPORT"}
        title={data.title}
        subtitle={data.subtitle}
        author={data.author}
        date={data.date}
        docId={data.docId}
        brand={brand}
      />

      <Page>
        <SectionTitle index="01" eyebrow="Executive Summary" title="State of the market" />
        <p
          className="body--lead"
          style={{ margin: 0, color: "var(--ink)", maxWidth: "150mm" }}
        >
          {data.executiveSummary}
        </p>
        <KPIGrid items={data.kpis} />
        <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
          <FooterMeta left={brand} center="Marketing / Summary" right="02" />
        </div>
      </Page>

      <Page>
        <SectionTitle
          index="02"
          eyebrow="Allocation"
          title="Budget split"
          lead="How we propose to deploy budget across the next quarter."
        />
        <BarChart
          data={data.budgetAllocation}
          caption="Budget allocation by channel (percentage)."
          height={220}
        />
        <div style={{ marginTop: "8mm" }}>
          <SectionTitle index="03" eyebrow="Performance" title="Channel performance" />
        </div>
        <BarChart
          data={data.channelPerformance}
          caption="Conversion rate by channel (relative index)."
          height={220}
        />
        <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
          <FooterMeta left={brand} center="Marketing / Channels" right="03" />
        </div>
      </Page>

      {data.personas.map((p, i) => (
        <Page key={i}>
          {i === 0 && (
            <SectionTitle index="04" eyebrow="Personas" title="Who we are talking to" />
          )}
          <PersonaCard {...p} />
          <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
            <FooterMeta
              left={brand}
              center={`Persona — ${p.name}`}
              right={String(4 + i).padStart(2, "0")}
            />
          </div>
        </Page>
      ))}

      <Page>
        <SectionTitle
          index="05"
          eyebrow="Next Steps"
          title="Recommendations"
          lead="Concrete moves, in priority order."
        />
        <ol style={{ listStyle: "none", padding: 0, margin: 0 }}>
          {data.recommendations.map((r, i) => (
            <li
              key={i}
              className="no-break"
              style={{
                display: "grid",
                gridTemplateColumns: "14mm 1fr",
                gap: "4mm",
                padding: "5mm 0",
                borderBottom: "1px solid var(--hairline)"
              }}
            >
              <span className="caption num" style={{ color: "var(--accent)" }}>
                R{String(i + 1).padStart(2, "0")}
              </span>
              <span className="body" style={{ margin: 0, color: "var(--ink)" }}>
                {r}
              </span>
            </li>
          ))}
        </ol>
        <Callout label="Note">
          Recommendations are scored by impact × ease of execution. Run the action plan in
          this order unless capacity or dependencies dictate otherwise.
        </Callout>
        <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
          <FooterMeta left={brand} center="Marketing / Roadmap" right="—" />
        </div>
      </Page>
    </>
  );
}

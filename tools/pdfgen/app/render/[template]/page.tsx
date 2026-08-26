import { promises as fs } from "node:fs";
import path from "node:path";
import { Whitepaper } from "../../../components/templates/Whitepaper";
import { AuditReport } from "../../../components/templates/AuditReport";
import { MarketingReport } from "../../../components/templates/MarketingReport";
import { GenericDoc } from "../../../components/templates/GenericDoc";
import {
  sampleWhitepaper,
  sampleAudit,
  sampleMarketing,
  sampleDoc
} from "../../../lib/samples";
import type { AnyData } from "../../../lib/schemas";

const SAMPLES: Record<string, AnyData> = {
  whitepaper: sampleWhitepaper,
  audit: sampleAudit,
  marketing: sampleMarketing,
  doc: sampleDoc
};

async function loadData(template: string, dataFile: string | null, demo: boolean): Promise<AnyData> {
  if (demo) return SAMPLES[template];
  if (dataFile) {
    const raw = await fs.readFile(path.resolve(dataFile), "utf-8");
    return JSON.parse(raw) as AnyData;
  }
  return SAMPLES[template];
}

const THEMES = ["agentik"] as const;

export default async function RenderPage({
  params,
  searchParams
}: {
  params: Promise<{ template: string }>;
  searchParams: Promise<{ demo?: string; data?: string; theme?: string }>;
}) {
  const { template } = await params;
  const sp = await searchParams;
  const demo = sp.demo === "1" || sp.demo === "true";
  const dataFile = sp.data || null;
  const data = await loadData(template, dataFile, demo);

  const requested = sp.theme ?? (data as { theme?: string }).theme;
  const theme = THEMES.includes(requested as (typeof THEMES)[number])
    ? (requested as (typeof THEMES)[number])
    : "agentik";

  // Fall back to the URL param (the `--template` CLI flag) when the data file
  // omits the `template` discriminant — previously that silently rendered
  // "Unknown template". Assigning keeps the discriminated-union narrowing below.
  if (!(data as { template?: string }).template) {
    (data as { template?: string }).template = template;
  }
  let body: React.ReactNode;
  if (data.template === "whitepaper") body = <Whitepaper data={data} />;
  else if (data.template === "audit") body = <AuditReport data={data} />;
  else if (data.template === "marketing") body = <MarketingReport data={data} />;
  else if (data.template === "doc") body = <GenericDoc data={data} />;
  else body = <div style={{ padding: 40 }}>Unknown template: {template}</div>;

  return <div className={`theme-${theme}`}>{body}</div>;
}

export const dynamic = "force-dynamic";

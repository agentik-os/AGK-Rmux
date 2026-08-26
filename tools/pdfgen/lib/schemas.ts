/**
 * Strict input shapes for each template.
 * The CLI validates against these before rendering.
 */

export type Theme = "agentik";

export type CommonMeta = {
  title: string;
  subtitle?: string;
  eyebrow?: string;
  author?: string;
  date?: string;
  docId?: string;
  brand?: string;
  theme?: Theme;
};

// ----- Whitepaper -----
export type WhitepaperSection = {
  index: string;            // "01"
  eyebrow?: string;
  title: string;
  lead?: string;
  /** Markdown body for the section */
  body: string;
};

export type WhitepaperData = CommonMeta & {
  template: "whitepaper";
  abstract?: string;
  toc?: Array<{ index: string; title: string; page: number; depth?: 1 | 2 }>;
  sections: WhitepaperSection[];
};

// ----- AuditReport -----
export type AuditFinding = {
  severity: "critical" | "high" | "medium" | "low";
  title: string;
  description: string;
  recommendation?: string;
  evidence?: string;
};

export type AuditScoreDomain = {
  label: string;
  score: number;          // 0-100
  weight?: number;
};

export type AuditData = CommonMeta & {
  template: "audit";
  overallScore: number;       // 0-100
  verdict: string;            // one-paragraph executive summary
  domains: AuditScoreDomain[];
  kpis?: Array<{ label: string; value: string | number; unit?: string; delta?: string; deltaTone?: "up" | "down" | "neutral" }>;
  findings: AuditFinding[];
  actionPlan: Array<{ when: string; title: string; desc?: string }>;
};

// ----- MarketingReport -----
export type Persona = {
  name: string;
  role: string;
  age?: string;
  income?: string;
  pains: string[];
  goals: string[];
  channels?: string[];
};

export type MarketingData = CommonMeta & {
  template: "marketing";
  executiveSummary: string;
  kpis: Array<{ label: string; value: string | number; unit?: string; delta?: string; deltaTone?: "up" | "down" | "neutral" }>;
  budgetAllocation: Array<{ label: string; value: number }>;        // % or $
  channelPerformance: Array<{ label: string; value: number }>;
  personas: Persona[];
  recommendations: string[];
};

// ----- GenericDoc (markdown → editorial PDF) -----
export type DocData = CommonMeta & {
  template: "doc";
  /** Raw markdown */
  body: string;
};

export type AnyData = WhitepaperData | AuditData | MarketingData | DocData;

"use client";

import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { PieChart } from "./PieChart";
import { RadarChart } from "./RadarChart";
import { ProgressBars } from "./ProgressBars";
import { StatCard } from "./StatCard";
import { PullQuote } from "./PullQuote";
import { PartDivider } from "./PartDivider";
import { Marginalia } from "./Marginalia";

type Props = {
  source: string;
};

/** Deep-flatten React children into plain text. */
function flattenText(node: any): string {
  if (node == null || typeof node === "boolean") return "";
  if (typeof node === "string" || typeof node === "number") return String(node);
  if (Array.isArray(node)) return node.map(flattenText).join("");
  if (typeof node === "object" && node.props) return flattenText(node.props.children);
  return "";
}

/** Split a multi-section ASCII bar block into groups separated by ESTIMATION/header lines. */
function detectAsciiBarBlocks(text: string): Array<{
  title?: string;
  bars: Array<{ label: string; value: number; note?: string }>;
}> | null {
  const lines = text.split("\n");
  const blocks: Array<{ title?: string; bars: Array<{ label: string; value: number; note?: string }> }> = [];
  let current: { title?: string; bars: Array<{ label: string; value: number; note?: string }> } = { bars: [] };
  const barRx = /^\s*\|?\s*([A-Za-zÀ-ÿ][A-Za-zÀ-ÿ0-9 _+\-]*?)\s+\[[=+\-\s]+\]\s+(\d+)%\s*(?:\(([^)]+)\))?\s*\|?\s*$/;
  const titleRx = /^\s*\|?\s*(ESTIMATION[^|]*|REPARTITION[^|]*|CONSTITUTION[^|]*|VIKRITI[^|]*|PRAKRITI[^|]*)\|?\s*$/i;
  for (const raw of lines) {
    const line = raw.replace(/\s+$/, "");
    const tm = line.match(titleRx);
    if (tm) {
      // New section
      if (current.bars.length) blocks.push(current);
      current = { title: tm[1].trim(), bars: [] };
      continue;
    }
    const bm = line.match(barRx);
    if (bm) {
      current.bars.push({
        label: bm[1].trim(),
        value: parseInt(bm[2], 10),
        note: bm[3]?.trim()
      });
    }
  }
  if (current.bars.length) blocks.push(current);
  const hasAny = blocks.some((b) => b.bars.length >= 2);
  return hasAny ? blocks : null;
}

/* ----- Pull-quote heuristic -----
   A blockquote starting with **bold text** AND > 80 chars total = candidate for PullQuote */
function isPullQuoteCandidate(text: string): boolean {
  const trimmed = text.trim();
  return trimmed.length > 80 && trimmed.length < 280 && /^[\*_]{0,2}[A-Za-zÀ-ÿ]/.test(trimmed);
}

export function Markdown({ source }: Props) {
  return (
    <div className="markdown body">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          /* ----- Headings ----- */
          h1: (props) => (
            <h1
              className="t-1"
              style={{ marginTop: "12mm", marginBottom: "4mm" }}
              {...props}
            />
          ),
          h2: (props) => (
            <h2
              className="t-2"
              style={{
                marginTop: "8mm",
                marginBottom: "3mm",
                fontSize: "13.5pt"
              }}
              {...props}
            />
          ),
          h3: (props) => (
            <h3
              className="t-3"
              style={{
                marginTop: "6mm",
                marginBottom: "2mm",
                fontSize: "11.5pt"
              }}
              {...props}
            />
          ),
          h4: (props) => (
            <h4
              style={{
                fontFamily: "var(--sans)",
                fontWeight: 600,
                fontSize: "10.5pt",
                margin: "4mm 0 1.5mm 0",
                color: "var(--ink)"
              }}
              {...props}
            />
          ),
          p: (props) => (
            <p
              className="body"
              style={{ margin: "0 0 3mm 0", maxWidth: "165mm" }}
              {...props}
            />
          ),
          a: (props) => (
            <a
              {...props}
              style={{
                color: "var(--accent)",
                textDecoration: "underline",
                textUnderlineOffset: "2px",
                textDecorationThickness: "0.5pt"
              }}
            />
          ),
          ul: (props) => (
            <ul
              {...props}
              style={{
                listStyle: "none",
                padding: 0,
                margin: "0 0 4mm 0",
                maxWidth: "165mm"
              }}
            />
          ),
          ol: (props) => (
            <ol
              {...props}
              style={{
                padding: 0,
                margin: "0 0 4mm 4mm",
                maxWidth: "165mm"
              }}
            />
          ),
          li: ({ children, ...rest }) => (
            <li
              {...rest}
              className="body"
              style={{
                paddingLeft: "5mm",
                textIndent: "-5mm",
                marginBottom: "1.5mm"
              }}
            >
              <span style={{ color: "var(--accent)", fontFamily: "var(--mono)" }}>·  </span>
              {children}
            </li>
          ),
          blockquote: ({ children }) => {
            // Extract text content to decide quote vs pullquote
            const text = (() => {
              try {
                const flatten = (node: any): string => {
                  if (typeof node === "string") return node;
                  if (Array.isArray(node)) return node.map(flatten).join("");
                  if (node?.props?.children) return flatten(node.props.children);
                  return "";
                };
                return flatten(children).trim();
              } catch {
                return "";
              }
            })();
            if (isPullQuoteCandidate(text)) {
              return <PullQuote text={text.replace(/^\*\*|\*\*$/g, "")} />;
            }
            return (
              <blockquote
                className="no-break"
                style={{
                  margin: "5mm 0",
                  maxWidth: "162mm",
                  borderLeft: "2pt solid var(--accent)",
                  paddingLeft: "5mm",
                  fontFamily: "var(--serif)",
                  fontStyle: "italic",
                  fontWeight: 400,
                  fontSize: "12pt",
                  lineHeight: 1.5,
                  color: "var(--slate)"
                }}
              >
                {children}
              </blockquote>
            );
          },
          code: ({ children, className, ...rest }) => {
            const text = flattenText(children);
            const hasNewline = text.includes("\n");
            const isInline = !hasNewline && !className;
            const lang = (className || "").replace("language-", "");

            // ----- Custom fenced blocks -----
            if (lang === "piechart") {
              try {
                const parsed = JSON.parse(String(children).trim());
                const items = Array.isArray(parsed) ? parsed : parsed.data;
                return <PieChart data={items} caption={parsed.caption} donut={parsed.donut ?? 0} />;
              } catch { return <pre style={{ color: "#9A2A2A", fontSize: "8pt" }}>Invalid piechart JSON</pre>; }
            }
            if (lang === "radar") {
              try {
                const parsed = JSON.parse(String(children).trim());
                const items = Array.isArray(parsed) ? parsed : parsed.data;
                return <RadarChart data={items} caption={parsed.caption} />;
              } catch { return <pre style={{ color: "#9A2A2A", fontSize: "8pt" }}>Invalid radar JSON</pre>; }
            }
            if (lang === "bars" || lang === "progressbars") {
              try {
                const parsed = JSON.parse(String(children).trim());
                const items = Array.isArray(parsed) ? parsed : parsed.data;
                return <ProgressBars data={items} caption={parsed.caption} />;
              } catch { return <pre style={{ color: "#9A2A2A", fontSize: "8pt" }}>Invalid bars JSON</pre>; }
            }
            if (lang === "stat" || lang === "stats") {
              try {
                const parsed = JSON.parse(String(children).trim());
                const items = Array.isArray(parsed) ? parsed : parsed.data;
                return <StatCard items={items} columns={parsed.columns} />;
              } catch { return <pre style={{ color: "#9A2A2A", fontSize: "8pt" }}>Invalid stat JSON</pre>; }
            }
            if (lang === "quote" || lang === "pullquote") {
              try {
                const parsed = JSON.parse(String(children).trim());
                return <PullQuote text={parsed.text} cite={parsed.cite} />;
              } catch { return <pre style={{ color: "#9A2A2A", fontSize: "8pt" }}>Invalid quote JSON</pre>; }
            }
            if (lang === "note" || lang === "marginalia") {
              try {
                const parsed = JSON.parse(String(children).trim());
                return <Marginalia label={parsed.label}>{parsed.body}</Marginalia>;
              } catch { return <pre style={{ color: "#9A2A2A", fontSize: "8pt" }}>Invalid note JSON</pre>; }
            }

            // ----- Auto-detect ASCII bar pattern in plain code blocks -----
            if (!isInline) {
              const blocks = detectAsciiBarBlocks(text);
              if (blocks) {
                return (
                  <div className="no-break" style={{ margin: "5mm 0" }}>
                    {blocks.map((b, i) => (
                      <div key={i} style={{ marginTop: i > 0 ? "8mm" : 0 }}>
                        {b.title && (
                          <div
                            className="t-eyebrow"
                            style={{ marginBottom: "3mm", color: "var(--mute)" }}
                          >
                            {b.title}
                          </div>
                        )}
                        <ProgressBars data={b.bars} />
                      </div>
                    ))}
                  </div>
                );
              }
            }

            // ----- Inline code -----
            if (isInline) {
              return (
                <code
                  {...rest}
                  style={{
                    fontFamily: "var(--mono)",
                    fontSize: "8.5pt",
                    background: "var(--paper-alt)",
                    padding: "0.5pt 3pt",
                    borderRadius: 2
                  }}
                >
                  {children}
                </code>
              );
            }

            // ----- Plain code block -----
            return (
              <pre
                className="no-break"
                style={{
                  fontFamily: "var(--mono)",
                  fontSize: "8pt",
                  lineHeight: 1.5,
                  background: "var(--paper-alt)",
                  padding: "5mm",
                  border: "0.4pt solid var(--hairline)",
                  overflow: "hidden",
                  whiteSpace: "pre-wrap",
                  wordBreak: "break-word",
                  margin: "3mm 0",
                  color: "var(--ink-soft)"
                }}
              >
                <code>{children}</code>
              </pre>
            );
          },
          hr: () => (
            <hr
              style={{
                border: 0,
                borderTop: "0.5pt solid var(--hairline)",
                margin: "8mm 0"
              }}
            />
          ),
          table: ({ children }) => (
            <div className="no-break" style={{ margin: "4mm 0" }}>
              <table
                style={{
                  width: "100%",
                  borderCollapse: "collapse",
                  fontSize: "10pt"
                }}
              >
                {children}
              </table>
            </div>
          ),
          thead: (props) => <thead {...props} />,
          tbody: (props) => <tbody {...props} />,
          tr: (props) => (
            <tr
              {...props}
              style={{ borderBottom: "0.4pt solid var(--hairline)" }}
            />
          ),
          th: (props) => (
            <th
              {...props}
              style={{
                textAlign: "left",
                padding: "5pt 7pt",
                borderBottom: "0.75pt solid var(--ink)",
                fontFamily: "var(--sans)",
                fontWeight: 600,
                fontSize: "8pt",
                letterSpacing: "0.08em",
                textTransform: "uppercase",
                color: "var(--ink)"
              }}
            />
          ),
          td: (props) => (
            <td
              {...props}
              style={{
                padding: "5.5pt 7pt",
                fontFamily: "var(--sans)",
                fontSize: "10pt",
                lineHeight: 1.4,
                color: "var(--ink-soft)",
                verticalAlign: "top"
              }}
            />
          ),
          strong: (props) => (
            <strong
              {...props}
              style={{ fontWeight: 600, color: "var(--ink)" }}
            />
          ),
          em: (props) => <em {...props} style={{ fontStyle: "italic" }} />
        }}
      >
        {source}
      </ReactMarkdown>
    </div>
  );
}

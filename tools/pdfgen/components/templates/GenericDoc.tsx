import { Cover, Page, Markdown, PartDivider } from "../primitives";
import type { DocData } from "../../lib/schemas";

type Part = {
  divider?: { number: string; title: string; label?: string };
  body: string;
};

/** Split markdown on "# Volet N -- Title" / "# Partie N -- Title" boundaries.
 *  Each part starts with an optional divider page + body content. */
function splitOnVolets(md: string): Part[] {
  const lines = md.split("\n");
  const parts: Part[] = [];
  let current: Part = { body: "" };
  const rx = /^#\s+(?:Volet|Partie|Part)\s+(\d+)\s*(?:--|—|:|-)\s*(.+)$/i;

  for (const line of lines) {
    const m = line.match(rx);
    if (m) {
      // Flush current
      if (current.body.trim() || current.divider) parts.push(current);
      // Start new part
      current = {
        divider: {
          number: String(m[1]).padStart(2, "0"),
          title: m[2].trim(),
          label: "VOLET"
        },
        body: ""
      };
    } else {
      current.body += line + "\n";
    }
  }
  if (current.body.trim() || current.divider) parts.push(current);
  return parts;
}

export function GenericDoc({ data }: { data: DocData }) {
  const parts = splitOnVolets(data.body);
  return (
    <>
      <Cover
        eyebrow={data.eyebrow || "DOCUMENT"}
        title={data.title}
        subtitle={data.subtitle}
        author={data.author}
        date={data.date}
        docId={data.docId}
        brand={data.brand}
      />
      {parts.map((p, i) => (
        <div key={i}>
          {p.divider && (
            <PartDivider
              number={p.divider.number}
              label={p.divider.label}
              title={p.divider.title}
            />
          )}
          {p.body.trim() && (
            <Page variant="flow">
              <Markdown source={p.body} />
            </Page>
          )}
        </div>
      ))}
    </>
  );
}

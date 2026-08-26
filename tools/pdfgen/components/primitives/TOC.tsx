import { Page } from "./Page";
import { SectionTitle } from "./SectionTitle";
import { FooterMeta } from "./FooterMeta";

type Entry = {
  index: string;
  title: string;
  page: number;
  depth?: 1 | 2;
};

type Props = {
  entries: Entry[];
  title?: string;
  brand?: string;
  pageNo?: string;
};

export function TOC({ entries, title = "Contents", brand = "OmegaOS", pageNo }: Props) {
  return (
    <Page>
      <SectionTitle eyebrow="Index" title={title} />
      <ol style={{ listStyle: "none", padding: 0, margin: 0 }}>
        {entries.map((e, i) => (
          <li
            key={i}
            className="no-break"
            style={{ marginBottom: "4mm" }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "baseline",
                gap: "6mm",
                paddingLeft: e.depth === 2 ? "10mm" : 0
              }}
            >
              <span
                className="caption num"
                style={{ minWidth: "14mm", color: "var(--mute)" }}
              >
                {e.index}
              </span>
              <span
                className="body"
                style={{
                  flex: 1,
                  color: e.depth === 2 ? "var(--slate)" : "var(--ink)",
                  fontWeight: e.depth === 2 ? 400 : 500,
                  marginBottom: 0
                }}
              >
                {e.title}
              </span>
              <span
                aria-hidden
                style={{
                  flex: 1,
                  borderBottom: "1px dotted var(--hairline)",
                  height: 1,
                  alignSelf: "flex-end",
                  marginBottom: "2mm"
                }}
              />
              <span
                className="caption num"
                style={{ minWidth: "8mm", textAlign: "right" }}
              >
                {String(e.page).padStart(2, "0")}
              </span>
            </div>
          </li>
        ))}
      </ol>
      <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
        <FooterMeta left={brand} center="Index" right={pageNo} />
      </div>
    </Page>
  );
}

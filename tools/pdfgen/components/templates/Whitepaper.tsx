import {
  Cover,
  Page,
  SectionTitle,
  TOC,
  FooterMeta,
  Markdown
} from "../primitives";
import type { WhitepaperData } from "../../lib/schemas";

export function Whitepaper({ data }: { data: WhitepaperData }) {
  const brand = data.brand || "OmegaOS";
  let pageNum = 2;

  return (
    <>
      <Cover
        eyebrow={data.eyebrow}
        title={data.title}
        subtitle={data.subtitle}
        author={data.author}
        date={data.date}
        docId={data.docId}
        brand={brand}
      />

      {data.abstract && (
        <Page>
          <SectionTitle index="00" eyebrow="Abstract" title="Synopsis" />
          <div style={{ maxWidth: "150mm" }}>
            <p className="body--lead" style={{ margin: 0 }}>
              {data.abstract}
            </p>
          </div>
          <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
            <FooterMeta
              left={brand}
              center={data.docId}
              right={String(pageNum++).padStart(2, "0")}
            />
          </div>
        </Page>
      )}

      {data.toc && data.toc.length > 0 && (
        <TOC entries={data.toc} brand={brand} pageNo={String(pageNum++).padStart(2, "0")} />
      )}

      {data.sections.map((s, i) => (
        <Page key={i}>
          <SectionTitle
            index={s.index}
            eyebrow={s.eyebrow}
            title={s.title}
            lead={s.lead}
          />
          <div style={{ flex: 1, overflow: "hidden" }}>
            <Markdown source={s.body} />
          </div>
          <div style={{ marginTop: "auto", paddingTop: "10mm" }}>
            <FooterMeta
              left={brand}
              center={s.title}
              right={String(pageNum++).padStart(2, "0")}
            />
          </div>
        </Page>
      ))}
    </>
  );
}

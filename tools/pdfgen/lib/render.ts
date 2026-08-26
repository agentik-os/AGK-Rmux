import { chromium, Browser } from "playwright";
import { promises as fs } from "node:fs";
import path from "node:path";

export type RenderOptions = {
  template: "whitepaper" | "audit" | "marketing" | "doc";
  /** Path to JSON file matching the template's schema, OR demo:true */
  dataFile?: string;
  demo?: boolean;
  theme?: "agentik";
  out: string;             // output PDF path
  baseUrl: string;         // e.g. http://localhost:4317
  /** Wait for selector before printing (defaults: body) */
  waitForSelector?: string;
};

export async function renderPdf(opts: RenderOptions): Promise<string> {
  const url = new URL(`/render/${opts.template}`, opts.baseUrl);
  if (opts.demo) url.searchParams.set("demo", "1");
  if (opts.dataFile) url.searchParams.set("data", path.resolve(opts.dataFile));
  if (opts.theme) url.searchParams.set("theme", opts.theme);

  const browser: Browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({
      viewport: { width: 794, height: 1123 }   // A4 @ 96 DPI
    });
    await page.goto(url.toString(), { waitUntil: "networkidle", timeout: 60_000 });
    await page.evaluate(() => (document as any).fonts && (document as any).fonts.ready);
    if (opts.waitForSelector) {
      await page.waitForSelector(opts.waitForSelector, { timeout: 10_000 });
    }
    await page.emulateMedia({ media: "print" });

    await fs.mkdir(path.dirname(path.resolve(opts.out)), { recursive: true });
    await page.pdf({
      path: path.resolve(opts.out),
      format: "A4",
      printBackground: true,
      preferCSSPageSize: true,
      margin: { top: 0, right: 0, bottom: 0, left: 0 }
    });
    return path.resolve(opts.out);
  } finally {
    await browser.close();
  }
}

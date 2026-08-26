#!/usr/bin/env -S npx tsx
/**
 * Smoke test — renders all four templates with demo data and reports paths.
 */
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";
import { renderPdf } from "../lib/render";
import { startServer, stopServer } from "../lib/server";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const OUT_DIR = path.join(ROOT, "output");

async function ensureBuilt() {
  const fs = await import("node:fs/promises");
  try {
    await fs.access(path.join(ROOT, ".next/BUILD_ID"));
    return;
  } catch {
    console.error("[smoke] building Next app…");
    await new Promise<void>((resolve, reject) => {
      const p = spawn("npx", ["next", "build"], { cwd: ROOT, stdio: "inherit" });
      p.on("exit", (code) => (code === 0 ? resolve() : reject(new Error(`build exited ${code}`))));
    });
  }
}

(async () => {
  await ensureBuilt();
  const base = await startServer(ROOT);
  const templates = ["whitepaper", "audit", "marketing", "doc"] as const;
  try {
    for (const t of templates) {
      const out = path.join(OUT_DIR, `${t}.pdf`);
      console.error(`[smoke] rendering ${t} →`);
      const p = await renderPdf({
        template: t,
        demo: true,
        out,
        baseUrl: base
      });
      console.log(`  ${t}: ${p}`);
    }
  } finally {
    await stopServer();
  }
})().catch((err) => {
  console.error(err);
  process.exit(1);
});

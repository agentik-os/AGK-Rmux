#!/usr/bin/env -S npx tsx
/**
 * pdfgen — unified PDF generator CLI
 *
 * Examples:
 *   pdfgen --template=whitepaper --demo --out=/tmp/wp.pdf
 *   pdfgen --template=audit --data=./audit.json --theme=agentik --out=./audit.pdf
 *   pdfgen --template=doc --data=./doc.json --send-dm
 *   pdfgen --template=marketing --data=./mkt.json --send-topic=32 --send-group=-1003587170167
 */

import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { renderPdf } from "../lib/render";
import { startServer, stopServer } from "../lib/server";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

type CLI = {
  template?: string;
  data?: string;
  demo?: boolean;
  theme?: string;
  out?: string;
  sendDm?: boolean;
  sendTopic?: string;
  sendGroup?: string;
  caption?: string;
};

function parseArgs(argv: string[]): CLI {
  const out: CLI = {};
  for (const a of argv) {
    const [k, v] = a.includes("=") ? a.split("=") : [a, "true"];
    const key = k.replace(/^--/, "");
    switch (key) {
      case "template": out.template = v; break;
      case "data": out.data = v; break;
      case "demo": out.demo = v === "true" || v === "1"; break;
      case "theme": out.theme = v; break;
      case "out": out.out = v; break;
      case "send-dm": out.sendDm = v === "true" || v === "1"; break;
      case "send-topic": out.sendTopic = v; break;
      case "send-group": out.sendGroup = v; break;
      case "caption": out.caption = v; break;
    }
  }
  return out;
}

async function sendTelegram(pdfPath: string, opts: { dm?: boolean; topic?: string; group?: string; caption?: string }) {
  const HOME = process.env.HOME ?? "";
  // Cross-user: env override wins, then ~/.omega, then the legacy ~/.claude path.
  const telegramSh = process.env.OMEGA_TELEGRAM_SCRIPT
    ?? `${HOME}/.omega/telegram.sh`;
  const filename = path.basename(pdfPath);
  if (opts.topic && opts.group) {
    const fs = await import("node:fs/promises");
    const CONFIG_FILE = process.env.OMEGA_TELEGRAM_CONFIG
      ?? `${HOME}/.claude/config/telegram.json`;
    const token = JSON.parse(await fs.readFile(CONFIG_FILE, "utf-8")).bot.token;
    const api = `https://api.telegram.org/bot${token}/sendDocument`;
    const { execFileSync } = await import("node:child_process");
    execFileSync("curl", [
      "-s", "-X", "POST", api,
      "-F", `chat_id=${opts.group}`,
      "-F", `message_thread_id=${opts.topic}`,
      "-F", `caption=${opts.caption || filename}`,
      "-F", `document=@${pdfPath}`
    ]);
    console.log(`[ok] sent to topic ${opts.topic} of group ${opts.group}`);
    return;
  }
  if (opts.dm) {
    const { execFileSync } = await import("node:child_process");
    const chatId = process.env.OMEGA_TELEGRAM_CHAT_ID;
    if (!chatId || !/^-?\d+$/.test(chatId)) {
      throw new Error(
        "OMEGA_TELEGRAM_CHAT_ID is required for raw pdfgen --send-dm. Prefer `omega pdf --send`, which reads the allow-listed OmegaOS Telegram configuration."
      );
    }
    execFileSync(telegramSh, ["file", chatId, pdfPath, opts.caption || filename]);
    console.log(`[ok] sent to DM`);
  }
}

async function ensureBuilt() {
  const fs = await import("node:fs/promises");
  try {
    await fs.access(path.join(ROOT, ".next/BUILD_ID"));
    return;
  } catch {
    console.error("[pdfgen] building Next app (one-time, ~30s)…");
    await new Promise<void>((resolve, reject) => {
      const p = spawn("npx", ["next", "build"], { cwd: ROOT, stdio: "inherit" });
      p.on("exit", (code) => (code === 0 ? resolve() : reject(new Error(`build exited ${code}`))));
    });
  }
}

(async () => {
  const args = parseArgs(process.argv.slice(2));
  if (!args.template) {
    console.error("Usage: pdfgen --template=<whitepaper|audit|marketing|doc> [--demo] [--data=path.json] [--theme=...] --out=path.pdf [--send-dm] [--send-topic=ID --send-group=ID]");
    process.exit(2);
  }
  if (!args.out) {
    args.out = `/tmp/agentik-pdf-${Date.now()}-${args.template}.pdf`;
  }

  await ensureBuilt();
  const base = await startServer(ROOT);

  try {
    console.error(`[pdfgen] rendering ${args.template} → ${args.out}`);
    const out = await renderPdf({
      template: args.template as any,
      dataFile: args.data,
      demo: args.demo,
      theme: args.theme as any,
      out: args.out,
      baseUrl: base
    });
    console.log(out);

    if (args.sendDm || (args.sendTopic && args.sendGroup)) {
      await sendTelegram(out, {
        dm: args.sendDm,
        topic: args.sendTopic,
        group: args.sendGroup,
        caption: args.caption
      });
    }
  } finally {
    await stopServer();
  }
})().catch((err) => {
  console.error(err);
  process.exit(1);
});

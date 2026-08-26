// OmegaOS Growth Engine — X engagement executor (Playwright, bounded, human-paced).
// Drives a LOGGED-IN @Agentik_os session (storageState) to post genuine replies + likes.
// Reads a JSONL action queue; emits a JSONL result per action to stdout.
//
// Usage:
//   bun playwright-engage.mjs --session <storageState.json> --queue <queue.jsonl> [--check]
//                             [--replies-cap N] [--likes-cap N] [--run-dir DIR] [--headed]
//
// SAFETY: bounded by caps, randomized human pacing, skips already-liked, aborts if the
// session is not logged in. This is assistive automation on the operator's OWN account,
// kept low-volume on purpose. Mass automation would get the account suspended.
import { chromium } from "playwright";
import { readFileSync, existsSync, mkdirSync } from "fs";

const args = process.argv.slice(2);
const opt = (k, d) => { const i = args.indexOf(k); return i >= 0 ? args[i + 1] : d; };
const has = (k) => args.includes(k);

const SESSION = opt("--session", `${process.env.HOME}/.omega/secrets/x-session.json`);
const QUEUE = opt("--queue", "");
const CHECK = has("--check");
const REPLIES_CAP = parseInt(opt("--replies-cap", "6"), 10);
const LIKES_CAP = parseInt(opt("--likes-cap", "15"), 10);
const RUN_DIR = opt("--run-dir", "/tmp");
const HEADED = has("--headed");

const emit = (o) => process.stdout.write(JSON.stringify(o) + "\n");
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const jitter = (min, max) => Math.floor(min + Math.random() * (max - min));

if (!existsSync(SESSION)) {
  emit({ fatal: "no_session", msg: `session file missing: ${SESSION}` });
  process.exit(2);
}
try { mkdirSync(RUN_DIR, { recursive: true }); } catch {}

const browser = await chromium.launch({ headless: !HEADED });
const ctx = await browser.newContext({
  storageState: SESSION,
  viewport: { width: 1280, height: 900 },
  userAgent:
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0 Safari/537.36",
});
const page = await ctx.newPage();

async function loggedIn() {
  await page.goto("https://x.com/home", { waitUntil: "domcontentloaded", timeout: 45000 });
  await sleep(2500);
  const url = page.url();
  if (url.includes("/login") || url.includes("/i/flow/login")) return false;
  const compose = await page.$('[data-testid="SideNav_NewTweet_Button"]');
  return !!compose;
}

const ok = await loggedIn();
if (!ok) {
  emit({ fatal: "not_logged_in", msg: "session invalid or expired, no engagement performed" });
  await browser.close();
  process.exit(3);
}
emit({ status: "logged_in" });

if (CHECK) { emit({ status: "check_ok" }); await browser.close(); process.exit(0); }

async function doReply(url, text) {
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 45000 });
  await sleep(jitter(2000, 4500));
  await page.locator('[data-testid="reply"]').first().click({ timeout: 15000 });
  const box = page.locator('[data-testid="tweetTextarea_0"]').first();
  await box.click({ timeout: 15000 });
  // type with small per-char delay to look human
  await box.type(text, { delay: jitter(18, 55) });
  await sleep(jitter(800, 1800));
  await page.locator('[data-testid="tweetButton"]').first().click({ timeout: 15000 });
  await sleep(jitter(2500, 4500));
}

async function doLike(url) {
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 45000 });
  await sleep(jitter(1500, 3500));
  const already = await page.$('[data-testid="unlike"]');
  if (already) return "already_liked";
  await page.locator('[data-testid="like"]').first().click({ timeout: 12000 });
  await sleep(jitter(700, 1600));
  return "liked";
}

const queue = QUEUE && existsSync(QUEUE)
  ? readFileSync(QUEUE, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l))
  : [];
let replies = 0, likes = 0;
for (const a of queue) {
  try {
    if (a.type === "reply") {
      if (replies >= REPLIES_CAP) { emit({ skip: "replies_cap", fingerprint: a.fingerprint }); continue; }
      await doReply(a.url, a.text);
      replies++;
      emit({ action: "reply", ok: true, fingerprint: a.fingerprint, url: a.url });
    } else if (a.type === "like") {
      if (likes >= LIKES_CAP) { emit({ skip: "likes_cap", fingerprint: a.fingerprint }); continue; }
      const r = await doLike(a.url);
      likes++;
      emit({ action: "like", ok: true, result: r, fingerprint: a.fingerprint, url: a.url });
    }
    await sleep(jitter(22000, 75000)); // human pacing between actions
  } catch (e) {
    try { await page.screenshot({ path: `${RUN_DIR}/fail-${a.fingerprint}.png` }); } catch {}
    emit({ action: a.type, ok: false, fingerprint: a.fingerprint, error: String(e).slice(0, 200) });
  }
}
emit({ status: "done", replies, likes });
await browser.close();

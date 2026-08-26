#!/usr/bin/env node
/**
 * OmegaOS acceptance sweep — generic browser truth for any scaffolded app.
 *
 * Proves "it WORKS", not "it builds": every route renders 200, the console is
 * clean, no network request fails, and the authenticated golden path completes
 * with a real persisted write. Exits non-zero with a JSON failure report so the
 * engine (and the /omg-acceptance heal loop) can act on it.
 *
 * Config via env:
 *   BASE_URL              (required) e.g. http://127.0.0.1:3100  — SECURE CONTEXT only
 *   ROUTES                comma list of extra routes to force-check (besides crawl)
 *   PROTECTED_ROUTES      comma list of routes that require auth (swept after login)
 *   CLERK_PUBLISHABLE_KEY + CLERK_SECRET_KEY   enable Clerk programmatic login
 *   TEST_IDENTIFIER + TEST_PASSWORD            the e2e user (…+clerk_test@example.com)
 *   GOLDEN_PATH           e.g. /chat  — page where the core authed action runs
 *   GOLDEN_INPUT          selector for the core-action input (default: textarea,input[type=text])
 *   GOLDEN_TEXT           text to submit (default: "omega acceptance <ts>")
 *   IGNORE_CONSOLE        regex of console noise to ignore (3rd-party)
 */
const path = require("path");
const APP = process.cwd();
const { chromium } = require(path.join(APP, "node_modules/playwright"));

const BASE = (process.env.BASE_URL || "http://127.0.0.1:3100").replace(/\/$/, "");
const EXTRA = (process.env.ROUTES || "").split(",").map(s => s.trim()).filter(Boolean);
const PROTECTED = (process.env.PROTECTED_ROUTES || "").split(",").map(s => s.trim()).filter(Boolean);
const GOLDEN_PATH = process.env.GOLDEN_PATH || "";
const GOLDEN_INPUT = process.env.GOLDEN_INPUT || 'textarea, input[type="text"]';
const GOLDEN_TEXT = process.env.GOLDEN_TEXT || `omega acceptance ${Date.now()}`;
const IGNORE = new RegExp(process.env.IGNORE_CONSOLE ||
  "evmAsk|Cannot redefine property: ethereum|development keys|clerk\\.com/.*telemetry|hydrat", "i");

const fail = [];
const addFail = (kind, where, detail) => fail.push({ kind, where, detail });

// A request is "ours" (app bug) vs third-party (Clerk/Convex infra we can't fix here).
// We still surface infra 4xx/5xx because the missing-JWT-template bug shows up as a
// 404 on clerk.com tokens/<template> — exactly the class we must catch.
function watch(page, label) {
  page.on("console", m => {
    if (m.type() === "error" && !IGNORE.test(m.text()))
      addFail("console", label, m.text().slice(0, 200));
  });
  page.on("pageerror", e => addFail("pageerror", label, String(e.message).slice(0, 200)));
  page.on("response", r => {
    const s = r.status();
    const u = r.url();
    if (s >= 400 && !/favicon|\.map$/.test(u) && !IGNORE.test(u))
      addFail("network", label, `${s} ${u.slice(0, 160)}`);
  });
}

async function checkRoute(page, route) {
  const url = BASE + route;
  let resp;
  try {
    resp = await page.goto(url, { waitUntil: "networkidle", timeout: 30000 });
  } catch (e) {
    addFail("nav", route, String(e.message).slice(0, 160));
    return;
  }
  // App Router returns a real HTTP 404 for notFound()/unmatched routes and 500 for a
  // render throw — status is the reliable signal. (Next 16 embeds the not-found copy in
  // every page's RSC payload, so body-text matching false-positives — don't use it.)
  const status = resp ? resp.status() : 0;
  if (status >= 400) addFail("route", route, `HTTP ${status}`);
}

async function crawl(page) {
  const seen = new Set(["/"]);
  await checkRoute(page, "/");
  // collect same-origin links from the landing
  const hrefs = await page.$$eval("a[href]", as => as.map(a => a.getAttribute("href")));
  for (const h of hrefs) {
    if (!h || h.startsWith("#") || h.startsWith("http") || h.startsWith("mailto")) continue;
    const r = h.split("?")[0].split("#")[0];
    if (r.startsWith("/") && !seen.has(r)) seen.add(r);
  }
  for (const r of EXTRA) seen.add(r);
  for (const r of seen) if (r !== "/") await checkRoute(page, r);
  return [...seen];
}

(async () => {
  const browser = await chromium.launch();
  const report = { base: BASE, publicRoutes: [], authedRoutes: [], golden: null, secureContext: null };

  // ---- Phase 1: anonymous sweep ----
  {
    const ctx = await browser.newContext();
    const page = await ctx.newPage();
    watch(page, "anon");
    await checkRoute(page, "/");
    report.secureContext = await page.evaluate(() => window.isSecureContext).catch(() => null);
    if (report.secureContext === false)
      addFail("secure-context", "/", "BASE_URL is not a secure context — Clerk/WebCrypto auth will fail. Use 127.0.0.1 or HTTPS.");
    report.publicRoutes = await crawl(page);
    await ctx.close();
  }

  // ---- Phase 2: authenticated golden path ----
  const wantAuth = process.env.CLERK_SECRET_KEY && process.env.TEST_IDENTIFIER && process.env.TEST_PASSWORD;
  if (wantAuth) {
    let clerk, clerkSetup;
    try { ({ clerk, clerkSetup } = require(path.join(APP, "node_modules/@clerk/testing/playwright"))); }
    catch (e) { addFail("auth-setup", "clerk-testing", "missing @clerk/testing: " + e.message); }
    if (clerk) {
      try { await clerkSetup(); } catch (e) { addFail("auth-setup", "clerkSetup", e.message); }
      const ctx = await browser.newContext();
      const page = await ctx.newPage();
      watch(page, "authed");
      try {
        const { setupClerkTestingToken } = require(path.join(APP, "node_modules/@clerk/testing/playwright"));
        await setupClerkTestingToken({ page });
      } catch {}
      await page.goto(BASE, { waitUntil: "networkidle", timeout: 30000 });
      try {
        await clerk.signIn({
          page,
          signInParams: { strategy: "password", identifier: process.env.TEST_IDENTIFIER, password: process.env.TEST_PASSWORD },
        });
      } catch (e) { addFail("auth", "signIn", String(e.message).slice(0, 200)); }

      const toSweep = PROTECTED.length ? PROTECTED : (GOLDEN_PATH ? [GOLDEN_PATH] : []);
      for (const r of toSweep) { await checkRoute(page, r); report.authedRoutes.push(r); }

      // golden action: type + submit + assert it persists across reload
      if (GOLDEN_PATH) {
        try {
          await page.goto(BASE + GOLDEN_PATH, { waitUntil: "networkidle", timeout: 30000 });
          const input = await page.waitForSelector(GOLDEN_INPUT, { timeout: 15000 });
          await input.fill(GOLDEN_TEXT);
          await page.keyboard.press("Enter");
          await page.waitForTimeout(4000);
          const after = await page.content();
          const persistedNow = after.includes(GOLDEN_TEXT);
          // reload — a real backend write survives a reload; optimistic-only UI does not
          await page.reload({ waitUntil: "networkidle", timeout: 30000 });
          await page.waitForTimeout(3000);
          const afterReload = await page.content();
          const persisted = afterReload.includes(GOLDEN_TEXT);
          report.golden = { submitted: true, visibleAfterSubmit: persistedNow, persistedAfterReload: persisted };
          if (!persistedNow) addFail("golden", GOLDEN_PATH, "core action submitted but never appeared (mutation likely failed)");
          else if (!persisted) addFail("golden", GOLDEN_PATH, "appeared but did NOT survive reload — not a real persisted write");
        } catch (e) { addFail("golden", GOLDEN_PATH, String(e.message).slice(0, 200)); }
      }
      await ctx.close();
    }
  } else {
    report.golden = "SKIPPED (no CLERK_SECRET_KEY + TEST_IDENTIFIER + TEST_PASSWORD) — authed path UNVERIFIED";
  }

  await browser.close();
  report.failures = fail;
  report.ok = fail.length === 0;
  console.log(JSON.stringify(report, null, 2));
  process.exit(fail.length === 0 ? 0 : 1);
})().catch(e => { console.error("SWEEP CRASH:", e.stack || e.message); process.exit(2); });

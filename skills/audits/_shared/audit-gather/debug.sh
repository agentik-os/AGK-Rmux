#!/usr/bin/env bash
# debug.sh — runtime bug gather. Playwright console capture, network errors, screenshot.
# Args: RAW_DIR PROJECT_PATH FILES URL
set -uo pipefail

RAW_DIR="${1:?missing raw_dir}"
PROJECT_PATH="${2:?missing project_path}"
FILES="${3:-}"
URL="${4:-}"
mkdir -p "$RAW_DIR"

# shellcheck disable=SC1091
source "$HOME/.omega/lib/audit-gather/_common.sh"

gather_header "debug"

if [ -z "$URL" ]; then
    log_skip "playwright" "URL empty — cannot capture runtime behavior"
    log_skip "screenshot" "URL empty"
    log_skip "console-capture" "URL empty"
    echo "debug: no URL provided, runtime checks skipped"
    exit 0
fi

# ─── 1. Playwright probe (console + network + screenshot) ───────
SCRIPT="$RAW_DIR/_pw-probe.cjs"
cat > "$SCRIPT" <<'JS'
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const URL = process.argv[2];
const OUT = process.argv[3];

(async () => {
  const consoleEvents = [];
  const networkErrors = [];
  const networkRequests = [];
  const pageErrors = [];

  let browser;
  try {
    browser = await chromium.launch({
      headless: true,
      args: ['--no-sandbox', '--disable-dev-shm-usage'],
    });
  } catch (e) {
    fs.writeFileSync(path.join(OUT, 'playwright-error.json'),
      JSON.stringify({error: 'launch failed', detail: String(e)}, null, 2));
    process.exit(2);
  }
  const ctx = await browser.newContext({
    userAgent: 'Mozilla/5.0 (compatible; AisbAuditBot/1.0)',
    viewport: { width: 1280, height: 800 },
  });
  const page = await ctx.newPage();

  page.on('console', (msg) => {
    consoleEvents.push({
      type: msg.type(),
      text: msg.text().slice(0, 500),
      location: msg.location(),
    });
  });
  page.on('pageerror', (err) => {
    pageErrors.push({ message: err.message.slice(0, 500), stack: (err.stack || '').slice(0, 1000) });
  });
  page.on('requestfailed', (req) => {
    networkErrors.push({
      url: req.url().slice(0, 300),
      method: req.method(),
      failure: req.failure() && req.failure().errorText,
      resource_type: req.resourceType(),
    });
  });
  page.on('response', (res) => {
    const status = res.status();
    if (status >= 400) {
      networkRequests.push({
        url: res.url().slice(0, 300),
        status: status,
        method: res.request().method(),
        resource_type: res.request().resourceType(),
      });
    }
  });

  let nav_error = null;
  try {
    await page.goto(URL, { waitUntil: 'networkidle', timeout: 45000 });
  } catch (e) {
    nav_error = String(e).slice(0, 500);
  }

  // Allow late console logs to flush
  await page.waitForTimeout(1500);

  // Screenshot
  try {
    await page.screenshot({ path: path.join(OUT, 'screenshot.png'), fullPage: false });
  } catch (e) { /* ignore */ }

  // Page metadata
  const meta = {};
  try {
    meta.title = await page.title();
    meta.url_final = page.url();
    meta.html_size = (await page.content()).length;
  } catch (_) { /* ignore */ }

  await browser.close();

  fs.writeFileSync(path.join(OUT, 'console.json'), JSON.stringify(consoleEvents, null, 2));
  fs.writeFileSync(path.join(OUT, 'page-errors.json'), JSON.stringify(pageErrors, null, 2));
  fs.writeFileSync(path.join(OUT, 'network-failures.json'), JSON.stringify(networkErrors, null, 2));
  fs.writeFileSync(path.join(OUT, 'network-4xx-5xx.json'), JSON.stringify(networkRequests, null, 2));
  fs.writeFileSync(path.join(OUT, 'page-meta.json'), JSON.stringify({...meta, nav_error}, null, 2));
})().catch((e) => {
  fs.writeFileSync(path.join(OUT, 'playwright-fatal.json'),
    JSON.stringify({error: String(e), stack: e.stack || ''}, null, 2));
  process.exit(1);
});
JS

# Find a globally-installed playwright (most likely path)
PW_NODE_PATH=""
for candidate in \
    "$HOME/.npm-global/lib/node_modules" \
    "/usr/local/lib/node_modules" \
    "/usr/lib/node_modules"; do
    if [ -d "$candidate/playwright" ]; then
        PW_NODE_PATH="$candidate"
        break
    fi
done

run_playwright_probe() {
    local node_path="$1"
    if [ -n "$node_path" ]; then
        NODE_PATH="$node_path" timeout 120 node "$SCRIPT" "$URL" "$RAW_DIR" 2>>"$RAW_DIR/_stderr.log"
    else
        timeout 120 node "$SCRIPT" "$URL" "$RAW_DIR" 2>>"$RAW_DIR/_stderr.log"
    fi
    return $?
}

if check_npx_pkg "playwright"; then
    pw_ok=false
    if [ -n "$PW_NODE_PATH" ]; then
        echo "debug: using playwright from $PW_NODE_PATH"
        if run_playwright_probe "$PW_NODE_PATH"; then
            pw_ok=true
        fi
    fi
    if [ "$pw_ok" = "false" ]; then
        # Try project-local
        if [ -f "$PROJECT_PATH/node_modules/playwright/package.json" ]; then
            echo "debug: using playwright from $PROJECT_PATH/node_modules"
            if run_playwright_probe "$PROJECT_PATH/node_modules"; then
                pw_ok=true
            fi
        fi
    fi

    if [ "$pw_ok" = "true" ]; then
        for f in console.json page-errors.json network-failures.json network-4xx-5xx.json page-meta.json; do
            [ -f "$RAW_DIR/$f" ] && log_run "playwright-$f" "$f" 0
        done
        [ -f "$RAW_DIR/screenshot.png" ] && log_run "playwright-screenshot" "screenshot.png" 0
    else
        log_skip "playwright" "playwright not resolvable (no global or project install)"
    fi
else
    log_skip "playwright" "npx not available"
fi

# ─── 2. Curl-based fallback (always run as a sanity check) ──────
HEAD_OUT="$RAW_DIR/curl-head.txt"
curl -sLI -A "Mozilla/5.0" --max-time 15 -w "\nFINAL_URL:%{url_effective}\nHTTP_CODE:%{http_code}\nTIME:%{time_total}\n" "$URL" -o "$HEAD_OUT" 2>>"$RAW_DIR/_stderr.log" || true
if [ -s "$HEAD_OUT" ]; then
    log_run "curl-head" "curl-head.txt" 0
fi

echo "debug gather done"

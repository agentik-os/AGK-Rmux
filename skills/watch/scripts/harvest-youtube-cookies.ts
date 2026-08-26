// Harvest anonymous YouTube visitor cookies with headless Chromium (no login).
// Fixes "Sign in to confirm you're not a bot" on servers/datacenter IPs when
// combined with the PoT provider stack (see SKILL.md "YouTube on servers").
// Usage:
//   CHROME_EXE=$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux*/chrome | tail -1) \
//   PROFILE_DIR=/tmp/yt-profile COOKIES_OUT=$HOME/.config/yt-dlp/youtube-cookies.txt \
//   bun run harvest-youtube-cookies.ts
// Then reference the jar from ~/.config/yt-dlp/config:  --cookies ~/.config/yt-dlp/youtube-cookies.txt
import { chromium } from 'playwright-core';
import { writeFileSync } from 'fs';

const exe = process.env.CHROME_EXE!;
const profile = process.env.PROFILE_DIR!;
const out = process.env.COOKIES_OUT!;
const url = process.env.TARGET_URL || 'https://www.youtube.com/watch?v=jNQXAC9IVRw';

const ctx = await chromium.launchPersistentContext(profile, {
  headless: true,
  executablePath: exe,
  viewport: { width: 1280, height: 800 },
  locale: 'en-US',
  args: ['--no-sandbox', '--disable-blink-features=AutomationControlled'],
});
await ctx.addCookies([
  { name: 'SOCS', value: 'CAI', domain: '.youtube.com', path: '/' },
  { name: 'CONSENT', value: 'PENDING+987', domain: '.youtube.com', path: '/' },
]);
const page = await ctx.newPage();
try {
  await page.goto('https://www.youtube.com', { waitUntil: 'domcontentloaded', timeout: 60000 });
  await page.waitForTimeout(3000);
  for (const sel of ['button[aria-label*="Accept all"]', 'button[aria-label*="Tout accepter"]', '[aria-label*="Accept the use of cookies"]']) {
    const b = page.locator(sel).first();
    if (await b.isVisible().catch(() => false)) { await b.click().catch(() => {}); await page.waitForTimeout(2000); break; }
  }
  await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 60000 });
  await page.waitForTimeout(6000);
} catch (e) {
  console.error('nav warning:', (e as Error).message);
}
const cookies = await ctx.cookies();
const lines = ['# Netscape HTTP Cookie File'];
for (const c of cookies) {
  const includeSub = c.domain.startsWith('.') ? 'TRUE' : 'FALSE';
  const secure = c.secure ? 'TRUE' : 'FALSE';
  const expiry = c.expires && c.expires > 0 ? Math.floor(c.expires) : Math.floor(Date.now() / 1000) + 3600 * 24 * 30;
  lines.push([c.domain, includeSub, c.path, secure, expiry, c.name, c.value].join('\t'));
}
writeFileSync(out, lines.join('\n') + '\n');
console.log(`wrote ${cookies.length} cookies -> ${out}`);
console.log('names:', cookies.map((c) => c.name).join(','));
await ctx.close();

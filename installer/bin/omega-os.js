#!/usr/bin/env node
'use strict';
/*
 * omega-os — one-command installer for OmegaOS.
 *   npx omega-os                install into ./OmegaOS (or $HOME/Station/OmegaOS if ~/Station exists)
 *   npx omega-os --dir DIR      install into DIR
 *   npx omega-os --plain        disable the animation (simple single-line bar)
 *   npx omega-os --no-telegram  skip the guided Telegram remote-control setup
 *   npx omega-os --help
 *
 * After the install succeeds (cooked input, normal screen restored), an
 * interactive wizard offers the Telegram remote: BotFather walkthrough, live
 * token validation (getMe), chat-id auto-detection (getUpdates after "send
 * your bot a message") — then wires it immediately via `omega telegram
 * setup`, against the fully-installed system. Never during the animation,
 * where prompts are invisible.
 *
 * Zero runtime deps (Node builtins only) so npx is fast + robust. Clones the
 * PUBLIC repo agentik-os/OmegaOS and runs its install.sh.
 *
 * While install.sh runs (~8 min on a fresh box) and we're on a real TTY, we
 * take over the screen with a full-screen experience: Matrix rain by default
 * (OmegaOS agents ARE Matrix characters): your keystrokes materialize as bright
 * glyph-drops, [space] fires a pulse, and each install phase name decodes out of
 * the katakana — with the OMEGA progress bar pinned at the bottom. install.sh
 * output is parsed for
 * phase markers (to advance the bar) and buffered (for error reporting) but
 * never printed onto the animated screen.
 *
 * Fallback: no TTY / --plain / CI / tiny terminal → the original simple
 * single-line progress bar (code path kept fully intact).
 */
const { spawn, spawnSync } = require('child_process');
const os = require('os');
const fs = require('fs');
const path = require('path');
const https = require('https');
const readline = require('readline');

const REPO = 'https://github.com/agentik-os/OmegaOS.git';

// install.sh runs behind the animation (TTY held), so any prompt would hang
// invisibly. Force every child tool non-interactive: git fails fast instead of
// asking for credentials, apt never prompts. (install.sh also exports these,
// belt-and-suspenders.) NOTE: CI here only affects install.sh's children — the
// animation decision already happened in main() against the real process.env.
const CHILD_ENV = Object.assign({}, process.env, {
  GIT_TERMINAL_PROMPT: '0',
  GIT_SSH_COMMAND: process.env.GIT_SSH_COMMAND || 'ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new',
  DEBIAN_FRONTEND: 'noninteractive',
  CI: process.env.CI || '1',
});
const tty = process.stdout.isTTY;
const C = (n, s) => (tty ? `\x1b[${n}m${s}\x1b[0m` : s);
const cyan = (s) => C('36', s), mag = (s) => C('35;1', s), grn = (s) => C('32', s),
      red = (s) => C('31', s), gray = (s) => C('90', s), bold = (s) => C('1', s), yel = (s) => C('33', s);

const OMEGA = [
  '    ██████╗ ███╗   ███╗███████╗ ██████╗  █████╗ ',
  '   ██╔═══██╗████╗ ████║██╔════╝██╔════╝ ██╔══██╗',
  '   ██║   ██║██╔████╔██║█████╗  ██║  ███╗███████║',
  '   ██║   ██║██║╚██╔╝██║██╔══╝  ██║   ██║██╔══██║',
  '   ╚██████╔╝██║ ╚═╝ ██║███████╗╚██████╔╝██║  ██║',
  '    ╚═════╝ ╚═╝     ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝',
];

// install.sh phase markers → ordered progress steps. The clone is step 0.
const STEPS = [
  { key: 'clone',  label: 'Fetching OmegaOS' },
  { re: /Phase 1: Environment/i,   label: 'Environment detection' },
  { re: /Phase 2:/i,               label: 'Dependencies (rust, deps, mosh, bun)' },
  { re: /Phase 3: Building rmux/i, label: 'Building rmux (multiplexer)' },
  { re: /Phase 4: Building OmegaOS/i, label: 'Building OmegaOS' },
  { re: /Phase 5: Configuring/i,   label: 'Configuring (~/.omega, skills, agents, rules)' },
  { re: /Phase 6: Shell/i,         label: 'Shell integration' },
  { re: /Phase 6\.5/i,             label: 'Self-containment (hooks, identity, service)' },
  { re: /Phase 6\.9:/i,            label: 'Companion tools (multi-LLM skills)' },
  { re: /Phase 6\.95/i,            label: 'Telegram interface (OmegaMC)' },
  { key: 'done',   label: 'Finalizing' },
];

// Matrix-agent lore tips rotated in the HUD. OmegaOS agents ARE Matrix chars.
const LORE = [
  'Neo is compiling — there is no spoon, only cargo.',
  'Morpheus offers two pipelines — you take the one that ships.',
  'Oracle routes the task to the right agent.',
  'Agent Smith is replicating across panes.',
  'Following the white rabbit into ~/.omega ...',
  'Seraph guards the quality gate.',
  'Keymaker is cutting keys for every provider.',
  'Trinity locked the daemon socket.',
  'Niobe is steering the fleet.',
  'Link keeps the operator console online.',
];

function banner() {
  process.stdout.write('\n');
  for (const l of OMEGA) process.stdout.write('  ' + mag(l) + '\n');
  process.stdout.write('\n  ' + bold(cyan('OmegaOS')) + gray('  ·  the agentic terminal OS  ·  rmux + AI orchestration') + '\n\n');
}

function bar(idx, total, label) {
  const pct = Math.min(100, Math.round((idx / (total - 1)) * 100));
  const w = 32;
  const filled = Math.round((pct / 100) * w);
  const b = '█'.repeat(filled) + gray('░'.repeat(w - filled));
  const line = `  ${mag('▸ OMEGA')}  ${b} ${bold(String(pct).padStart(3) + '%')}  ${cyan(label)}`;
  if (tty) process.stdout.write('\r\x1b[2K' + line);
  else process.stdout.write(line + '\n');
}

function die(msg) { process.stdout.write('\n  ' + red('✗ ' + msg) + '\n\n'); process.exit(1); }

function have(cmd) { return spawnSync(cmd, ['--version'], { stdio: 'ignore' }).status === 0; }

/* =========================================================================
 * TELEGRAM WIZARD — guided remote-control setup AFTER the install succeeds.
 *
 * Runs on the NORMAL screen with cooked input, once the animation has
 * released the TTY and the whole system exists (omega binary, bot service,
 * ~/.omega). Collect + wire happen back-to-back, so the closing "connected"
 * line reflects the bot's REAL state — message the bot right away and it
 * answers. Skipped silently when non-interactive (no TTY / CI /
 * --no-telegram) or when ~/.omega/telegram.toml already exists (re-install
 * keeps it).
 * ========================================================================= */

// Never rejects. Resolves the parsed Telegram body (ok:true/false), or
// { ok:false, _network:'<why>' } when the request itself failed (timeout,
// DNS, unparseable body, or a SYNCHRONOUS https.get throw — e.g.
// ERR_UNESCAPED_CHARACTERS from a malformed token, which previously
// unwound past the wizard's try/finally and was swallowed silently).
function tgApi(token, method, params) {
  return new Promise((resolve) => {
    const qs = params
      ? '?' + Object.entries(params).map(([k, v]) => k + '=' + encodeURIComponent(v)).join('&')
      : '';
    let req;
    try {
      req = https.get(
        { host: 'api.telegram.org', path: '/bot' + encodeURIComponent(token) + '/' + method + qs, timeout: 10000 },
        (res) => {
          let body = '';
          res.on('data', (d) => { body += d; });
          res.on('end', () => {
            try { resolve(JSON.parse(body)); } catch (e) { resolve({ ok: false, _network: 'unparseable response from Telegram' }); }
          });
        }
      );
    } catch (e) {
      resolve({ ok: false, _network: e && e.code === 'ERR_UNESCAPED_CHARACTERS' ? 'invalid characters in token' : ((e && e.message) || 'request failed') });
      return;
    }
    req.on('timeout', () => { req.destroy(); resolve({ ok: false, _network: 'timeout — Telegram unreachable' }); });
    req.on('error', (e) => resolve({ ok: false, _network: (e && e.message) || 'network error' }));
  });
}

// Queue-based prompt over readline 'line' events instead of rl.question():
// lines arriving while NO prompt is pending (multi-line paste, scripted stdin,
// a fast typist during the getMe network call) are buffered, not dropped.
// Resolves null once stdin closes (EOF / Ctrl-D) — callers treat null as
// "skip Telegram", so a dead stdin can never hang the wizard.
function makeAsker(rl) {
  const lines = [];
  const waiters = [];
  let closed = false;
  rl.on('line', (l) => {
    const w = waiters.shift();
    if (w) w(l.trim()); else lines.push(l.trim());
  });
  rl.on('close', () => {
    closed = true;
    while (waiters.length) waiters.shift()(null);
  });
  return (q) => {
    process.stdout.write(q);
    if (lines.length) { const l = lines.shift(); process.stdout.write(l + '\n'); return Promise.resolve(l); }
    if (closed) { process.stdout.write('\n'); return Promise.resolve(null); }
    return new Promise((resolve) => waiters.push(resolve));
  };
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function telegramWizard() {
  // Already configured (re-install) → keep it, stay quiet.
  const tomlPath = path.join(os.homedir(), '.omega', 'telegram.toml');
  if (fs.existsSync(tomlPath)) {
    process.stdout.write('  ' + grn('✓') + gray(' Telegram already configured (~/.omega/telegram.toml) — keeping it') + '\n\n');
    return null;
  }

  process.stdout.write('  ' + mag('▸ TELEGRAM REMOTE') + gray('  ·  pilot OmegaOS from your phone (recommended)') + '\n');
  process.stdout.write('  ' + gray('Dispatch missions, get reports, approve gates — from anywhere.') + '\n\n');

  const rl = readline.createInterface({ input: process.stdin });
  const ask = makeAsker(rl);
  try {
    const go = await ask('  Set it up now? ' + gray('[Y/n] '));
    if (go === null || /^n/i.test(go)) {
      process.stdout.write('  ' + gray('Skipped — later: OMEGA_TG_TOKEN=<BOT_TOKEN> omega telegram setup <YOUR_ID> --user-id <YOUR_ID>') + '\n\n');
      return null;
    }

    // --- Step 1/2: bot token (validated live against the Telegram API) ----
    process.stdout.write('\n  ' + bold('Step 1/2 — create your bot') + '\n');
    process.stdout.write('    1. Open Telegram and message ' + cyan('@BotFather') + '\n');
    process.stdout.write('    2. Send ' + cyan('/newbot') + ' and follow the two questions (name, username)\n');
    process.stdout.write('    3. Copy the ' + bold('HTTP API token') + ' it gives you\n\n');

    let token = null, botUser = null;
    for (let attempt = 0; attempt < 3 && !token; attempt++) {
      const t = await ask('  Paste the bot token ' + gray('(Enter to skip): '));
      if (!t) {
        process.stdout.write('  ' + gray('Skipped — later: OMEGA_TG_TOKEN=<BOT_TOKEN> omega telegram setup <YOUR_ID> --user-id <YOUR_ID>') + '\n\n');
        return null;
      }
      // A space / non-ASCII char in the token used to throw synchronously in
      // https.get and silently kill the whole wizard — catch it up front.
      if (/[^\x21-\x7e]/.test(t)) {
        process.stdout.write('  ' + red('✗') + ' Telegram rejected that token (invalid characters)' + gray(' — re-copy it from BotFather, no spaces or accents') + '\n');
        continue;
      }
      const me = await tgApi(t, 'getMe');
      if (me && me.ok && me.result && me.result.username) {
        token = t; botUser = me.result.username;
        process.stdout.write('  ' + grn('✓') + ' bot found: ' + bold('@' + botUser) + '\n');
      } else if (me && me._network === 'invalid characters in token') {
        process.stdout.write('  ' + red('✗') + ' Telegram rejected that token (invalid characters)' + gray(' — re-copy it from BotFather, no spaces or accents') + '\n');
      } else if (me && me._network) {
        process.stdout.write('  ' + red('✗') + ' could not reach Telegram (' + me._network + ')' + gray(' — check the network and retry') + '\n');
      } else {
        process.stdout.write('  ' + red('✗') + ' Telegram rejected that token' + gray(' — ' + ((me && me.description) || 'recheck the BotFather message')) + '\n');
      }
    }
    if (!token) {
      process.stdout.write('  ' + yel('Token never validated — skipping Telegram.') + gray(' Later: omega telegram setup …') + '\n\n');
      return null;
    }

    // --- Step 2/2: your chat id (auto-detected from your first message) ---
    process.stdout.write('\n  ' + bold('Step 2/2 — identify you') + '\n');
    process.stdout.write('    Open ' + cyan('t.me/' + botUser) + ' and send your bot any message (e.g. "hello").\n\n');

    let chatId = null, who = '';
    // getUpdates offset, tracked across attempts: every poll advances past the
    // updates it has seen, so NEW messages win — a stale backlog (stranger DMs,
    // group chatter from a reused bot) can no longer become the allow-list id.
    let nextOffset = 0;
    while (!chatId) {
      const a = await ask('  Press Enter AFTER sending it ' + gray('(or type your numeric id, or "skip"): '));
      if (a === null || /^skip$/i.test(a)) {
        process.stdout.write('  ' + gray('Skipped — later: OMEGA_TG_TOKEN=' + token.slice(0, 8) + '… omega telegram setup <YOUR_ID> --user-id <YOUR_ID>') + '\n\n');
        return null;
      }
      if (/^-?\d+$/.test(a)) {
        if (a.charAt(0) === '-') {
          process.stdout.write('  ' + yel('⚠ negative id = GROUP/CHANNEL chat — EVERY member of that chat will be able to control this machine.') + '\n');
        }
        if (a.replace('-', '').length < 5) {
          // Almost certainly a typo — require an explicit yes, default reject.
          process.stdout.write('  ' + yel('⚠ unusually short id — Telegram ids are normally 5+ digits, double-check it.') + '\n');
          const yn = await ask('  Use ' + a + ' anyway? ' + gray('[y/N] '));
          if (yn === null || !/^y/i.test(yn)) {
            process.stdout.write('  ' + gray('Rejected — re-enter your id, or send your bot a message and press Enter.') + '\n');
            continue;
          }
        }
        chatId = a; who = 'manual id'; break;
      }
      // Poll getUpdates a few times — the just-created bot has no other consumer.
      let pollErr = null;
      for (let i = 0; i < 5 && !chatId; i++) {
        const up = await tgApi(token, 'getUpdates', { limit: 10, offset: nextOffset });
        if (up && up.ok === false) {
          // Surface the real failure — an ok:false body (409 webhook conflict)
          // or a dead network is NOT the same as "no message yet".
          if (up._network) {
            pollErr = 'could not reach Telegram (' + up._network + ')';
          } else {
            pollErr = 'Telegram error: ' + (up.description || 'unknown');
            if (/webhook/i.test(up.description || '')) {
              pollErr += ' — a webhook is active on this bot; delete it (curl "https://api.telegram.org/bot<TOKEN>/deleteWebhook") or create a fresh bot';
            }
          }
          await sleep(1500);
          continue;
        }
        const msgs = up && up.ok && Array.isArray(up.result) ? up.result : [];
        for (const u of msgs) {
          if (typeof u.update_id === 'number' && u.update_id >= nextOffset) nextOffset = u.update_id + 1;
        }
        // Auto-detect accepts PRIVATE chats only — a group/channel id here would
        // hand `claude --dangerously-skip-permissions` to every member.
        let cand = null;
        for (let k = msgs.length - 1; k >= 0; k--) {
          const m = msgs[k].message;
          if (m && m.chat && m.chat.id && m.chat.type === 'private') {
            cand = {
              id: String(m.chat.id),
              name: (m.from && (m.from.first_name || m.from.username)) || 'unknown',
            };
            break;
          }
        }
        if (cand) {
          // Explicit operator confirmation — this id becomes BOTH the chat_id
          // and the --user-id allow-list. Never accept it on detection alone.
          const yn = await ask('  detected: ' + bold(cand.name) + ' (id ' + cand.id + ') — is that you? ' + gray('[y/N] '));
          if (yn !== null && /^y/i.test(yn)) { chatId = cand.id; who = cand.name; break; }
          process.stdout.write('  ' + gray('Ignored — send a NEW message from YOUR account, or type your numeric id.') + '\n');
        }
        if (!chatId) await sleep(1500);
      }
      if (!chatId) {
        process.stdout.write('  ' + yel(pollErr ? '✗ ' + pollErr : 'No message seen yet') + gray(' — send a NEW message to t.me/' + botUser + ' then press Enter again') + '\n');
      }
    }
    process.stdout.write('  ' + grn('✓') + ' detected: ' + bold(who) + gray(' (chat id ' + chatId + ')') + '\n\n');
    process.stdout.write('  ' + grn('Telegram ready') + gray(' — wiring it now\u2026') + '\n\n');
    // Acknowledge everything the wizard consumed: getUpdates only marks updates
    // as read on the NEXT call with offset > update_id. Without this final ack
    // the freshly started bot re-receives the user's handshake message ("hello")
    // and dispatches it to the Atlas brain as if it were a real instruction.
    if (nextOffset > 0) await tgApi(token, 'getUpdates', { offset: nextOffset, limit: 1 });
    return { token, chatId, botUser };
  } finally {
    rl.close();
  }
}

// The wizard's credentials must survive ANY interruption (Ctrl-C mid-build,
// install failure, crash) — never make the user redo the BotFather dance.
// Written right after the wizard succeeds, deleted only on confirmed success.
const PENDING_TG_PATH = path.join(os.homedir(), '.omega', 'pending-telegram.txt');
function savePendingTelegram(tg) {
  if (!tg) return null;
  const setupCmd = 'OMEGA_TG_TOKEN=' + tg.token + ' omega telegram setup ' + tg.chatId + ' --user-id ' + tg.chatId;
  try {
    fs.mkdirSync(path.dirname(PENDING_TG_PATH), { recursive: true });
    const note = '# Run once the install succeeds — the OMEGA_TG_TOKEN env prefix keeps\n'
      + '# the token out of `ps` output.\n';
    fs.writeFileSync(PENDING_TG_PATH, note + setupCmd + '\n', { mode: 0o600 });
    fs.chmodSync(PENDING_TG_PATH, 0o600); // mode option only applies on create
    return PENDING_TG_PATH;
  } catch (e) { return null; }
}
function clearPendingTelegram() { try { fs.unlinkSync(PENDING_TG_PATH); } catch (e) {} }

// Is the tg-bot service actually running? (systemd user unit on Linux —
// XDG_RUNTIME_DIR defaulted for bus-less shells; launchd LaunchAgent on macOS.)
// Mirrors crates/omega-core/src/service.rs. Returns true/false/null(unknown).
function tgBotServiceAlive() {
  try {
    if (process.platform === 'darwin') {
      const uid = process.getuid ? process.getuid() : 501;
      const r = spawnSync('launchctl', ['print', 'gui/' + uid + '/os.omega.tg-bot'], { encoding: 'utf8' });
      if (r.error) return null;
      return r.status === 0 && /state = running/.test(r.stdout || '');
    }
    const uid = process.getuid ? process.getuid() : null;
    const env = Object.assign({}, process.env);
    if (!env.XDG_RUNTIME_DIR && uid !== null && fs.existsSync('/run/user/' + uid)) {
      env.XDG_RUNTIME_DIR = '/run/user/' + uid;
    }
    const r = spawnSync('systemctl', ['--user', 'is-active', 'omega-tg-bot.service'], { encoding: 'utf8', env });
    if (r.error) return null;
    const s = (r.stdout || '').trim();
    return s === 'active' ? true : (s ? false : null);
  } catch (e) { return null; }
}

// Wire the collected credentials AFTER install.sh succeeded: the omega binary
// now exists at ~/.local/bin. Never throws — a failure prints the manual command.
function configureTelegram(tg) {
  if (!tg) return;
  const omegaBin = path.join(os.homedir(), '.local', 'bin', 'omega');
  const manual = 'OMEGA_TG_TOKEN=<token> omega telegram setup ' + tg.chatId + ' --user-id ' + tg.chatId;
  if (!fs.existsSync(omegaBin)) {
    process.stdout.write('  ' + yel('⚠ omega binary not found — finish Telegram manually: ') + cyan(manual) + '\n');
    process.stdout.write('    ' + gray('(your credentials are saved in ' + PENDING_TG_PATH + ')') + '\n\n');
    return;
  }
  // Token travels via env (OMEGA_TG_TOKEN, read by `omega telegram setup`),
  // never argv — a positional token is visible to every user in `ps` output
  // for the duration of the setup call. chat-id/--user-id stay positional.
  const r = spawnSync(omegaBin, ['telegram', 'setup', tg.chatId, '--user-id', tg.chatId], {
    encoding: 'utf8',
    env: Object.assign({}, process.env, { OMEGA_TG_TOKEN: tg.token }),
  });
  if (r.status !== 0) {
    process.stdout.write('  ' + yel('⚠ Telegram setup did not finish: ') + gray(((r.stderr || r.stdout || '').trim() || 'unknown error').split('\n')[0]) + '\n');
    process.stdout.write('    ' + gray('Run it manually: ') + cyan(manual) + '\n\n');
    return;
  }
  // The toml is written — but "connected" is only true if the bot service is
  // actually polling (it picks the token up within ~5s). On a headless Mac the
  // launchd bootstrap can fail (no GUI session) and the old unconditional
  // "✓ connected" was a lie the user discovered only when the bot stayed mute.
  clearPendingTelegram();
  const alive = tgBotServiceAlive();
  if (alive === false) {
    const startHint = process.platform === 'darwin'
      ? 'launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/os.omega.tg-bot.plist'
      : 'systemctl --user enable --now omega-tg-bot.service';
    process.stdout.write('  ' + yel('⚠ Telegram configured, but the bot service is NOT running.') + '\n');
    process.stdout.write('    ' + gray('Start it: ') + cyan(startHint) + gray('   (or run in foreground: omega telegram run)') + '\n\n');
  } else {
    const note = alive === true ? '' : gray('  (service state unknown — if the bot stays silent, run: omega telegram run)');
    process.stdout.write('  ' + grn('✓ Telegram connected') + ' — message ' + bold('@' + tg.botUser) + ' and try: ' + cyan('status') + note + '\n\n');
  }
}

// Post-install Telegram flow: wizard (BotFather walkthrough + live token
// validation + chat-id detection) THEN immediate wiring — the omega binary,
// the bot service and the config dir all exist at this point, so the
// "connected" verdict is real. Never throws; every exit path prints the
// manual command.
async function finishTelegram(enabled) {
  const tomlPath = path.join(os.homedir(), '.omega', 'telegram.toml');
  if (fs.existsSync(tomlPath)) {
    process.stdout.write('  ' + grn('✓') + gray(' Telegram already configured (~/.omega/telegram.toml) — keeping it') + '\n\n');
    return;
  }
  if (!enabled) {
    process.stdout.write('  ' + gray('Telegram remote not configured — later: OMEGA_TG_TOKEN=<BOT_TOKEN> omega telegram setup <YOUR_ID> --user-id <YOUR_ID>') + '\n\n');
    return;
  }
  let tg = null;
  try { tg = await telegramWizard(); } catch (e) { tg = null; }
  if (!tg) return; // wizard already printed the skip/manual hint
  // Persist first (survives a crash between wizard and wiring), then wire.
  savePendingTelegram(tg);
  configureTelegram(tg);
}

// --- success / failure summaries (normal screen) -------------------------

// Post-install health gate. "install.sh exited 0" only means the SCRIPT ran; it
// does not mean the doctrine got exported, the hooks got registered or the
// binary works. `omega doctor` answers that, and it is the difference between
// discovering a broken install now and discovering it three missions later.
// Advisory by design: a warning here never fails the install, it just refuses
// to let the operator walk away believing everything is green when it is not.
function runHealthCheck(dir) {
  const bin = path.join(process.env.HOME || '~', '.local/bin/omega');
  const exe = fs.existsSync(bin) ? bin : 'omega';
  const r = spawnSync(exe, ['doctor'], { encoding: 'utf8', timeout: 120000 });
  if (r.error || typeof r.stdout !== 'string') {
    process.stdout.write('  ' + yel('! could not run `omega doctor`') + gray(' — run it yourself after reloading your shell') + '\n\n');
    return;
  }
  const lines = r.stdout.split('\n');
  const bad = lines.filter((l) => /\[[!x]\]/.test(l));
  const ok = lines.filter((l) => /\[\+\]/.test(l)).length;
  if (!bad.length) {
    process.stdout.write('  ' + grn('✓ health check: ' + ok + ' checks passed') + '\n\n');
    return;
  }
  process.stdout.write('  ' + yel('! health check: ' + ok + ' passed, ' + bad.length + ' need attention') + '\n');
  for (const l of bad.slice(0, 8)) process.stdout.write('  ' + gray('│ ') + l.trim() + '\n');
  process.stdout.write('  ' + gray('Re-run `omega doctor` after `exec $SHELL -l`; most warnings are credentials you have not set up yet.') + '\n\n');
}

function printSuccess(dir) {
  process.stdout.write('\n  ' + grn('✓ OmegaOS installed') + '  →  ' + bold(dir) + '\n\n');
  runHealthCheck(dir);
  process.stdout.write('  Next:\n');
  process.stdout.write('    ' + cyan('exec $SHELL -l') + gray('   # reload your shell so `omega` is on PATH (bash & zsh)') + '\n');
  process.stdout.write('    ' + cyan('omega') + gray('            # launch the TUI') + '\n\n');
  process.stdout.write('  ' + gray('First run: open the TUI Account panel to log into Claude, then set up') + '\n');
  process.stdout.write('  ' + gray('Telegram / providers — Enter on any panel opens a guided wizard.') + '\n\n');
}

function printFailure(dir, code, lastLines, tg) {
  process.stdout.write('\n  ' + red('✗ install.sh exited ' + code) + '\n');
  process.stdout.write('  ' + gray('Re-run inside the clone to see full output:  cd ' + dir + ' && ./install.sh') + '\n');
  if (lastLines && lastLines.length) {
    process.stdout.write('\n  ' + gray('Last output lines:') + '\n');
    for (const l of lastLines) process.stdout.write('  ' + gray('│ ') + l + '\n');
  }
  // The wizard's credentials must survive an install failure — don't make the
  // user redo the BotFather dance after fixing the build.
  if (tg) {
    const setupCmd = 'OMEGA_TG_TOKEN=' + tg.token + ' omega telegram setup ' + tg.chatId + ' --user-id ' + tg.chatId;
    process.stdout.write('\n  ' + yel('⚠ Your Telegram credentials were collected but NOT applied (install failed).') + '\n');
    process.stdout.write('    ' + gray('Once the install succeeds, run: ') + cyan(setupCmd) + '\n');
    const saved = savePendingTelegram(tg);
    if (saved) process.stdout.write('    ' + gray('Saved to ' + saved + ' (chmod 600).') + '\n');
    else process.stdout.write('    ' + gray('(could not save it to ' + PENDING_TG_PATH + ')') + '\n');
  }
  process.stdout.write('\n');
}

// --- clone (shared by both paths) ----------------------------------------

function doClone(dir) {
  const exists = fs.existsSync(path.join(dir, '.git'));
  const cloneArgs = exists
    ? ['-C', dir, 'pull', '--ff-only']
    : ['clone', '--depth', '1', REPO, dir];
  const clone = spawnSync('git', cloneArgs, { encoding: 'utf8' });
  if (clone.status !== 0) die('git ' + (exists ? 'pull' : 'clone') + ' failed:\n  ' + gray((clone.stderr || '').trim()));
}

/* =========================================================================
 * PLAIN path — original simple single-line progress bar. Kept intact.
 * ========================================================================= */
function runPlain(dir, tgEnabled) {
  const total = STEPS.length;
  let step = 0;
  bar(step, total, STEPS[0].label);

  // 1) clone (or pull if present) — uses a spinner-less line here as before
  doClone(dir);

  // 2) run install.sh, parse phase markers to drive the bar
  const sh = path.join(dir, 'install.sh');
  if (!fs.existsSync(sh)) die('install.sh not found in ' + dir);
  step = 1; bar(step, total, STEPS[1].label);

  const child = spawn('bash', [sh], { cwd: dir, env: CHILD_ENV });
  let buf = '';
  const lastLines = [];
  const onData = (d) => {
    buf += d.toString();
    let nl;
    while ((nl = buf.indexOf('\n')) !== -1) {
      const line = buf.slice(0, nl); buf = buf.slice(nl + 1);
      lastLines.push(line); if (lastLines.length > 50) lastLines.shift();
      for (let i = step; i < STEPS.length; i++) {
        if (STEPS[i].re && STEPS[i].re.test(line)) { step = i; bar(step, total, STEPS[i].label); break; }
      }
    }
  };
  child.stdout.on('data', onData);
  child.stderr.on('data', onData);

  child.on('close', (code) => {
    if (code === 0) {
      step = total - 1; bar(step, total, 'Done');
      process.stdout.write('\n');
      finishTelegram(tgEnabled).then(() => printSuccess(dir));
    } else {
      process.stdout.write('\n');
      printFailure(dir, code, lastLines.slice(-40), null);
      process.exit(code || 1);
    }
  });
}

/* =========================================================================
 * ANIMATED path — full-screen interactive Matrix rain, OMEGA bar pinned bottom.
 * ========================================================================= */
function runAnimated(dir, tgEnabled) {
  const out = process.stdout;
  const inp = process.stdin;

  // --- clone first, on the NORMAL screen, with a tiny spinner -------------
  const exists = fs.existsSync(path.join(dir, '.git'));
  const spin = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
  let si = 0;
  out.write('  ' + mag('▸ OMEGA') + '  ' + cyan(exists ? 'Updating clone' : 'Fetching OmegaOS') + '  ' + spin[0]);
  const spinTimer = setInterval(() => {
    si = (si + 1) % spin.length;
    out.write('\r\x1b[2K  ' + mag('▸ OMEGA') + '  ' + cyan(exists ? 'Updating clone' : 'Fetching OmegaOS') + '  ' + spin[si]);
  }, 90);
  doClone(dir); // synchronous (spawnSync). die() exits cleanly if it fails.
  clearInterval(spinTimer);
  out.write('\r\x1b[2K  ' + mag('▸ OMEGA') + '  ' + grn('✓') + ' clone ready\n');

  const sh = path.join(dir, 'install.sh');
  if (!fs.existsSync(sh)) die('install.sh not found in ' + dir);

  // ---- terminal / scene state -------------------------------------------
  let W = out.columns || 80;
  let H = out.rows || 24;
  let rawEnabled = false;
  let cleaned = false;
  let child = null;
  const startTime = Date.now();
  // Stall watchdog: a hang must NEVER be invisible behind the animation. We
  // track the last time install.sh emitted ANY output; if it goes quiet for too
  // long the HUD surfaces a warning + the last line so the user knows it's
  // waiting (and can Ctrl-C) instead of staring at a frozen bar.
  let lastOutputAt = Date.now();
  const STALL_MS = 90 * 1000;

  // progress state (parsed from install.sh)
  const total = STEPS.length;
  let step = 1;                  // step 0 (clone) is done; now on Environment
  const lastLines = [];          // ~50 buffered lines for error reporting
  let parseBuf = '';

  // matrix columns
  let columns = [];
  // lore rotation
  let loreIdx = 0;
  let frame = 0;
  // phase-decode banner: when install.sh advances a phase, the new label
  // resolves out of random katakana, char by char, over ~1.2s (decodeDur frames).
  let decode = null;     // { text, start }
  let lastStep = -1;
  const decodeDur = 26;  // frames (~1.2s at 22fps)

  // ── Interactive layer ───────────────────────────────────────────────────
  // No game — YOU and the Matrix. Every printable key you press materializes
  // as a BRIGHT glyph-drop falling down a column (your input, rendered in the
  // rain). [space] fires a pulse: a band of columns flares white and fades.
  let userDrops = []; // { x, y(float), speed, glyphs:[], len }
  let pulses = [];    // { x, life }
  let injectCol = 3;  // round-robin column so successive keys spread out
  function injectGlyph(ch) {
    const x = ((injectCol % Math.max(1, W)) + W) % Math.max(1, W);
    injectCol += 11;
    const len = 6 + ((Math.random() * 9) | 0);
    const glyphs = [ch && ch.trim() ? ch : glyph()];
    for (let i = 1; i < len; i++) glyphs.push(glyph());
    userDrops.push({ x, y: 0, speed: 0.7 + Math.random() * 0.7, glyphs, len });
    if (userDrops.length > 160) userDrops.shift();
  }
  function firePulse() {
    const x = (Math.random() * W) | 0;
    for (let d = -3; d <= 3; d++) {
      const px = x + d;
      if (px >= 0 && px < W) pulses.push({ x: px, life: 14 - Math.abs(d) * 2 });
    }
    if (pulses.length > 400) pulses.splice(0, pulses.length - 400);
  }

  // matrix glyphs: katakana U+30A0..U+30FF + latin + digits
  const GLYPHS = (() => {
    const g = [];
    for (let c = 0x30a0; c <= 0x30ff; c++) g.push(String.fromCharCode(c));
    const extra = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$%&*<>=+';
    for (const ch of extra) g.push(ch);
    return g;
  })();
  const glyph = () => GLYPHS[(Math.random() * GLYPHS.length) | 0];

  function buildColumns() {
    columns = [];
    for (let x = 0; x < W; x++) {
      columns.push(makeColumn(x, true));
    }
  }
  function makeColumn(x, randomStart) {
    const len = 4 + ((Math.random() * Math.min(18, Math.max(4, H - 4))) | 0);
    const chars = [];
    for (let i = 0; i < len; i++) chars.push(glyph());
    return {
      x,
      y: randomStart ? -Math.random() * H : -(Math.random() * 6),
      speed: 0.2 + Math.random() * 0.8,
      len,
      chars,
    };
  }

  // ---- rendering ---------------------------------------------------------
  function pad(s, visibleLen) {
    // pad a (possibly ansi-colored) string to W columns using visibleLen
    const padN = Math.max(0, W - visibleLen);
    return s + ' '.repeat(padN);
  }

  function render() {
    frame++;
    // Install advanced a phase → kick off the decode banner for the new label.
    if (step !== lastStep) {
      lastStep = step;
      if (STEPS[step] && STEPS[step].label) decode = { text: STEPS[step].label, start: frame };
    }
    let s = '\x1b[H'; // home
    const playRows = H - 3; // rows 1..playRows are the rain; last 3 are HUD
    stepMatrix();
    stepInteractive(playRows);
    s += renderMatrix(playRows);
    s += renderDecodeBanner(playRows);
    s += renderHUD();
    out.write(s);
  }

  function stepInteractive(playRows) {
    for (const d of userDrops) d.y += d.speed;
    userDrops = userDrops.filter((d) => Math.floor(d.y) - d.len < playRows);
    for (const p of pulses) p.life--;
    pulses = pulses.filter((p) => p.life > 0);
  }

  function stepMatrix() {
    for (const col of columns) {
      col.y += col.speed;
      // occasional glyph mutation
      if ((Math.random() < 0.08)) {
        const i = (Math.random() * col.len) | 0;
        col.chars[i] = glyph();
      }
      const head = Math.floor(col.y);
      if (head - col.len > H) {
        // ran off the bottom → reset above the top
        const fresh = makeColumn(col.x, false);
        col.y = fresh.y; col.speed = fresh.speed; col.len = fresh.len; col.chars = fresh.chars;
      }
    }
  }

  function renderMatrix(playRows) {
    // build a grid of cells (char + color code) for the scene area
    const blank = ' ';
    // grid[y] = array of strings (already colored) for each x
    const grid = new Array(playRows);
    for (let y = 0; y < playRows; y++) {
      grid[y] = new Array(W).fill(null);
    }
    for (const col of columns) {
      const head = Math.floor(col.y);
      for (let k = 0; k < col.len; k++) {
        const y = head - k;
        if (y < 0 || y >= playRows) continue;
        const ch = col.chars[(head - k + col.len * 4) % col.len] || glyph();
        let colored;
        if (k === 0) {
          colored = '\x1b[97m' + ch + '\x1b[0m';           // bright near-white head
        } else if (k === 1) {
          colored = '\x1b[38;5;120m' + ch + '\x1b[0m';      // bright green
        } else if (k < col.len * 0.5) {
          colored = '\x1b[32m' + ch + '\x1b[0m';            // mid green
        } else {
          colored = '\x1b[38;5;28m' + ch + '\x1b[0m';       // dim green fade
        }
        if (col.x >= 0 && col.x < W) grid[y][col.x] = colored;
      }
    }
    // overlay [space] pulses: a column flares white→green and fades with life.
    for (const p of pulses) {
      if (p.x < 0 || p.x >= W) continue;
      const c = p.life > 9 ? '\x1b[97m' : p.life > 5 ? '\x1b[38;5;120m' : '\x1b[32m';
      for (let y = 0; y < playRows; y++) grid[y][p.x] = c + glyph() + '\x1b[0m';
    }
    // overlay user keystroke drops: bright white head, your typed glyph leading.
    for (const d of userDrops) {
      const head = Math.floor(d.y);
      for (let k = 0; k < d.len; k++) {
        const y = head - k;
        if (y < 0 || y >= playRows || d.x < 0 || d.x >= W) continue;
        const ch = d.glyphs[k] || glyph();
        let colored;
        if (k === 0) colored = '\x1b[97m' + ch + '\x1b[0m';          // white head (your key)
        else if (k <= 2) colored = '\x1b[38;5;231m' + ch + '\x1b[0m'; // bright
        else colored = '\x1b[38;5;120m' + ch + '\x1b[0m';            // bright green trail
        grid[y][d.x] = colored;
      }
    }
    // emit rows (rows are 1-indexed on screen; scene starts at row 1)
    let buf = '';
    for (let y = 0; y < playRows; y++) {
      buf += '\x1b[' + (y + 1) + ';1H';
      let line = '';
      for (let x = 0; x < W; x++) line += grid[y][x] || blank;
      buf += line + '\x1b[K';
    }
    return buf;
  }

  // The current phase label resolves out of random katakana, char by char.
  function renderDecodeBanner(playRows) {
    if (!decode) return '';
    const age = frame - decode.start;
    if (age >= decodeDur) { decode = null; return ''; }
    const text = decode.text;
    const revealed = (text.length * age) / decodeDur; // chars resolved so far
    let shown = '';
    for (let i = 0; i < text.length; i++) {
      shown += i < revealed
        ? '\x1b[38;5;120m' + text[i] + '\x1b[0m'   // resolved (bright green)
        : '\x1b[38;5;28m' + glyph() + '\x1b[0m';   // still scrambling (dim)
    }
    const visible = 2 + text.length; // "▸ " + text
    const row = Math.max(1, Math.floor(playRows / 2));
    const colStart = Math.max(1, Math.floor((W - visible) / 2));
    return '\x1b[' + row + ';' + colStart + 'H\x1b[97m▸ \x1b[0m' + shown;
  }

  function renderHUD() {
    const rowBar = H - 2;
    const rowMid = H - 1;
    const rowBot = H;

    // --- progress bar line ---
    const pct = Math.min(100, Math.round((step / (total - 1)) * 100));
    const label = STEPS[step] ? STEPS[step].label : 'Working';
    // bar width adapts to terminal width, leaving room for prefix/suffix
    const bw = Math.max(8, Math.min(40, W - 28));
    const filled = Math.round((pct / 100) * bw);
    const barStr = '\x1b[38;5;120m' + '█'.repeat(filled) + '\x1b[38;5;236m' + '░'.repeat(bw - filled) + '\x1b[0m';
    const pctStr = '\x1b[1m' + (String(pct) + '%').padStart(4) + '\x1b[0m';
    const barVisible = ('▸ OMEGA  [' + ' '.repeat(bw) + '] ' + (String(pct) + '%').padStart(4) + '  ' + label).length;
    const barColored = '\x1b[35;1m▸ OMEGA\x1b[0m  [' + barStr + '] ' + pctStr + '  \x1b[36m' + label + '\x1b[0m';
    let hud = '\x1b[' + rowBar + ';1H' + pad(barColored, barVisible) + '\x1b[K';

    // --- middle line: lore tip, OR a stall warning if install.sh went quiet ---
    const quietMs = Date.now() - lastOutputAt;
    let mid, midVisible;
    if (quietMs > STALL_MS) {
      const secs = Math.floor(quietMs / 1000);
      let last = (lastLines[lastLines.length - 1] || '').replace(/\x1b\[[0-9;]*m/g, '').trim();
      if (last.length > Math.max(10, W - 34)) last = last.slice(0, Math.max(10, W - 34)) + '…';
      const txt = '⚠ quiet ' + secs + 's — still working? last: ' + (last || '(no output)');
      mid = '\x1b[33m' + txt + '\x1b[0m';
      midVisible = txt.length;
    } else {
      const tip = LORE[loreIdx % LORE.length];
      // rotate lore roughly every ~4s (22fps * 4 ≈ 88 frames)
      if (frame % 88 === 0) loreIdx++;
      mid = '\x1b[2;32m' + tip + '\x1b[0m';
      midVisible = tip.length;
    }
    hud += '\x1b[' + rowMid + ';1H' + pad(mid, midVisible) + '\x1b[K';

    // --- bottom controls + timer ---
    const elapsed = Math.floor((Date.now() - startTime) / 1000);
    const mm = String(Math.floor(elapsed / 60)).padStart(2, '0');
    const ss = String(elapsed % 60).padStart(2, '0');
    const ctrlPlain = 'type to inject glyphs  ·  [space] pulse  ·  [Ctrl-C] cancel';
    const timerPlain = '⏱ ' + mm + ':' + ss;
    const gap = Math.max(2, W - ctrlPlain.length - timerPlain.length);
    const ctrlColored = '\x1b[90mtype to inject glyphs  ·  [\x1b[0m\x1b[1mspace\x1b[0m\x1b[90m] pulse  ·  [\x1b[0m\x1b[1mCtrl-C\x1b[0m\x1b[90m] cancel\x1b[0m';
    const timerColored = '\x1b[36m' + timerPlain + '\x1b[0m';
    const botVisible = ctrlPlain.length + gap + timerPlain.length;
    hud += '\x1b[' + rowBot + ';1H' + pad(ctrlColored + ' '.repeat(gap) + timerColored, botVisible) + '\x1b[K';

    return hud;
  }

  // ---- cleanup (bulletproof, idempotent) ---------------------------------
  function cleanup() {
    if (cleaned) return;
    cleaned = true;
    try { if (renderTimer) clearInterval(renderTimer); } catch (e) {}
    try {
      if (rawEnabled && inp.isTTY && typeof inp.setRawMode === 'function') {
        inp.setRawMode(false);
      }
    } catch (e) {}
    try { inp.pause(); } catch (e) {}
    try { inp.removeListener('data', onKey); } catch (e) {}
    try { out.write('\x1b[?25h'); } catch (e) {}      // show cursor
    try { out.write('\x1b[?1049l'); } catch (e) {}    // leave alt screen
  }

  // ---- input handling ----------------------------------------------------
  function onKey(data) {
    const str = data.toString();
    for (let i = 0; i < str.length; i++) {
      const ch = str[i];
      if (ch === '\x03') { // Ctrl-C
        try { if (child) child.kill('SIGTERM'); } catch (e) {}
        cleanup();
        process.stdout.write('\n  ' + yel('✗ cancelled') + gray('  — install.sh stopped. Re-run: ') + cyan('npx omega-os') + '\n\n');
        process.exit(130);
        return;
      }
      // Swallow escape sequences (arrow keys etc.) so they don't inject junk.
      if (ch === '\x1b') {
        if (str[i + 1] === '[') i += 2;
        continue;
      }
      if (ch === ' ') { firePulse(); continue; }        // [space] → pulse
      // Any other printable key materializes as a bright drop in the rain.
      const code = ch.charCodeAt(0);
      if (code >= 0x20 && code !== 0x7f) injectGlyph(ch);
    }
  }

  // ---- resize ------------------------------------------------------------
  function onResize() {
    W = out.columns || 80;
    H = out.rows || 24;
    buildColumns();
  }

  // ---- enter the experience ---------------------------------------------
  out.write('\x1b[?1049h'); // enter alternate screen buffer
  out.write('\x1b[?25l');   // hide cursor
  out.write('\x1b[2J');     // clear

  try {
    if (inp.isTTY && typeof inp.setRawMode === 'function') {
      inp.setRawMode(true);
      rawEnabled = true;
    }
  } catch (e) {}
  inp.resume();
  inp.on('data', onKey);
  out.on('resize', onResize);

  buildColumns();

  // register bulletproof cleanup on every exit path
  process.on('exit', cleanup);
  process.on('SIGINT', () => { try { if (child) child.kill('SIGTERM'); } catch (e) {} cleanup(); process.exit(130); });
  process.on('SIGTERM', () => { try { if (child) child.kill('SIGTERM'); } catch (e) {} cleanup(); process.exit(143); });
  process.on('uncaughtException', (err) => {
    cleanup();
    process.stderr.write('\n' + red('omega-os error: ') + (err && err.stack ? err.stack : String(err)) + '\n');
    try { if (child) child.kill('SIGTERM'); } catch (e) {}
    process.exit(1);
  });

  const renderTimer = setInterval(render, 45); // ~22fps

  // ---- run install.sh; parse markers, buffer output (never printed) ------
  child = spawn('bash', [sh], { cwd: dir, env: CHILD_ENV });
  const onData = (d) => {
    lastOutputAt = Date.now(); // feed the stall watchdog
    parseBuf += d.toString();
    let nl;
    while ((nl = parseBuf.indexOf('\n')) !== -1) {
      const line = parseBuf.slice(0, nl); parseBuf = parseBuf.slice(nl + 1);
      lastLines.push(line); if (lastLines.length > 50) lastLines.shift();
      for (let i = step; i < STEPS.length; i++) {
        if (STEPS[i].re && STEPS[i].re.test(line)) { step = i; break; }
      }
    }
  };
  child.stdout.on('data', onData);
  child.stderr.on('data', onData);

  child.on('close', (code) => {
    step = total - 1;            // finalize bar
    cleanup();                   // back to normal screen
    if (code === 0) {
      banner();
      finishTelegram(tgEnabled).then(() => printSuccess(dir));
    } else {
      banner();
      printFailure(dir, code, lastLines.slice(-40), null);
      process.exit(code || 1);
    }
  });
}

/* ========================================================================= */

function shouldAnimate(args) {
  if (args.includes('--plain')) return false;
  if (!process.stdout.isTTY) return false;
  if (process.env.CI) return false;
  const rows = process.stdout.rows || 0;
  const cols = process.stdout.columns || 0;
  if (rows < 12 || cols < 40) return false;
  return true;
}

async function main() {
  const args = process.argv.slice(2);
  if (args.includes('--help') || args.includes('-h')) {
    banner();
    process.stdout.write('  Usage: npx omega-os [--dir <path>] [--plain] [--no-telegram]\n\n');
    process.stdout.write('    --dir <path>   install location (default: ~/Station/OmegaOS if ~/Station exists, else ./OmegaOS)\n');
    process.stdout.write('    --plain        disable the full-screen animation (simple progress bar only)\n');
    process.stdout.write('    --no-telegram  skip the guided Telegram remote-control setup\n');
    process.stdout.write('    --help         this help\n\n');
    process.stdout.write('  Installs OmegaOS from ' + REPO + '\n  then runs its install.sh (builds rmux + omega, ~8 min on a fresh box).\n');
    process.stdout.write('  ' + gray('While it builds, the Matrix rain is interactive — type to inject glyphs, [space] to pulse.') + '\n\n');
    return;
  }

  banner();

  if (process.platform === 'win32') die('OmegaOS installs on Linux/macOS (rmux + bash). Use WSL or a Linux VPS.');
  if (!have('git')) die('git is required. Install git and re-run: npx omega-os');

  // target dir
  let dir = null;
  const di = args.indexOf('--dir');
  if (di !== -1 && args[di + 1]) dir = args[di + 1];
  const explicitDir = dir !== null;
  if (!dir) {
    const station = path.join(os.homedir(), 'Station');
    dir = fs.existsSync(station) ? path.join(station, 'OmegaOS') : path.join(process.cwd(), 'OmegaOS');
  }
  dir = path.resolve(dir);
  // The DEFAULT target's parent must be writable: npx gets run from / or other
  // read-only places (proven: "could not create work tree dir '/OmegaOS':
  // Read-only file system" — the clone died AFTER the Telegram wizard). Fall
  // back to ~/OmegaOS instead of dying; an explicit --dir is always respected.
  if (!explicitDir && !fs.existsSync(path.join(dir, '.git'))) {
    let parentWritable = true;
    try { fs.accessSync(path.dirname(dir), fs.constants.W_OK); } catch (e) { parentWritable = false; }
    if (!parentWritable) {
      const fallback = path.join(os.homedir(), 'OmegaOS');
      process.stdout.write('  ' + yel('⚠') + gray(' ' + path.dirname(dir) + ' is not writable — installing to ') + bold(fallback) + '\n\n');
      dir = fallback;
    }
  }

  // Telegram setup runs AFTER the install completes (operator request: the
  // early wizard confused users — credentials collected before anything
  // existed, then a 10-minute build, then the wiring. Now the whole dance
  // happens at the end, against a fully-installed system, and the success
  // line reflects the bot's REAL state). Interactive sessions only:
  // piped/CI runs skip silently, --no-telegram opts out.
  const interactive = process.stdin.isTTY && process.stdout.isTTY && !process.env.CI;
  const tgEnabled = interactive && !args.includes('--no-telegram');

  if (shouldAnimate(args)) runAnimated(dir, tgEnabled);
  else runPlain(dir, tgEnabled);
}

if (require.main === module) {
  main().catch((err) => die(err && err.message ? err.message : String(err)));
} else {
  // Exported for scripted smoke tests (https layer stubbed) — never used at runtime.
  module.exports = { tgApi, telegramWizard, configureTelegram, finishTelegram, printFailure };
}

#!/usr/bin/env bun
/**
 * omega-zernio — pure Bun/TS CLI over the Zernio REST API (raw fetch, ZERO deps).
 *
 * Social publishing modeled as ONE Zernio profile per OmegaOS project. The
 * project→profileId map is persisted at ~/.omega/zernio-profiles.json.
 *
 * Auth: Authorization: Bearer $ZERNIO_API_KEY. The key is read from the
 * environment, else parsed out of ~/.omega/secrets/integrations.env at runtime.
 * The key value is NEVER printed, logged, or written to any repo file (R-ENV/L0).
 */

import { readFileSync, writeFileSync, existsSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const API_BASE = "https://zernio.com/api/v1";
const HOME = homedir();
const SECRETS_FILE = join(HOME, ".omega", "secrets", "integrations.env");
const MAP_FILE = join(HOME, ".omega", "zernio-profiles.json");

const PLATFORMS = [
  "facebook", "instagram", "linkedin", "twitter", "tiktok", "youtube",
  "threads", "reddit", "pinterest", "bluesky", "googlebusiness", "telegram",
  "snapchat", "discord", "whatsapp",
];

// Media type inference by extension → mediaItem.type + presign contentType.
const MEDIA_EXT: Record<string, { type: string; contentType: string }> = {
  jpg: { type: "image", contentType: "image/jpeg" },
  jpeg: { type: "image", contentType: "image/jpeg" },
  png: { type: "image", contentType: "image/png" },
  webp: { type: "image", contentType: "image/webp" },
  gif: { type: "gif", contentType: "image/gif" },
  mp4: { type: "video", contentType: "video/mp4" },
  mov: { type: "video", contentType: "video/quicktime" },
  webm: { type: "video", contentType: "video/webm" },
  pdf: { type: "document", contentType: "application/pdf" },
};

// ── tiny helpers ────────────────────────────────────────────────────────────
function die(msg: string): never {
  process.stderr.write(`omega-zernio: ${msg}\n`);
  process.exit(1);
}

function normalize(s: string): string {
  return s.toLowerCase().replace(/[^a-z0-9]/g, "");
}

function extOf(p: string): string {
  const m = p.toLowerCase().match(/\.([a-z0-9]+)(?:\?|#|$)/);
  return m ? m[1] : "";
}

// Read the API key without ever exposing it. env first, then secrets file.
function loadKey(): string {
  const fromEnv = process.env.ZERNIO_API_KEY;
  if (fromEnv && fromEnv.trim()) return fromEnv.trim();
  if (existsSync(SECRETS_FILE)) {
    const txt = readFileSync(SECRETS_FILE, "utf8");
    for (const line of txt.split(/\r?\n/)) {
      const m = line.match(/^\s*(?:export\s+)?ZERNIO_API_KEY\s*=\s*(.*)$/);
      if (m) {
        let v = m[1].trim();
        if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'"))) {
          v = v.slice(1, -1);
        }
        if (v) return v;
      }
    }
  }
  die(
    `ZERNIO_API_KEY not found. Set it in the environment or add a line\n` +
    `  ZERNIO_API_KEY=...\nto ${SECRETS_FILE}`,
  );
}

function hasKey(): boolean {
  if (process.env.ZERNIO_API_KEY && process.env.ZERNIO_API_KEY.trim()) return true;
  if (!existsSync(SECRETS_FILE)) return false;
  const txt = readFileSync(SECRETS_FILE, "utf8");
  return txt.split(/\r?\n/).some((l) => /^\s*(?:export\s+)?ZERNIO_API_KEY\s*=\s*\S/.test(l));
}

// ── HTTP ────────────────────────────────────────────────────────────────────
let CACHED_KEY: string | null = null;
function key(): string {
  if (CACHED_KEY === null) CACHED_KEY = loadKey();
  return CACHED_KEY;
}

interface ApiResult<T = any> {
  ok: boolean;
  status: number;
  body: T;
}

async function api<T = any>(
  method: string,
  path: string,
  body?: unknown,
): Promise<ApiResult<T>> {
  const headers: Record<string, string> = {
    Authorization: `Bearer ${key()}`,
    Accept: "application/json",
  };
  const init: RequestInit = { method, headers };
  if (body !== undefined) {
    headers["Content-Type"] = "application/json";
    init.body = JSON.stringify(body);
  }
  let res: Response;
  try {
    res = await fetch(`${API_BASE}${path}`, init);
  } catch (e: any) {
    die(`network error calling ${method} ${path}: ${e?.message ?? e}`);
  }
  const text = await res.text();
  let parsed: any = text;
  try {
    parsed = text ? JSON.parse(text) : {};
  } catch {
    /* leave raw text */
  }
  return { ok: res.ok, status: res.status, body: parsed as T };
}

function apiErr(r: ApiResult, what: string): never {
  const b: any = r.body;
  const msg =
    (b && (b.message || b.error)) ||
    (typeof b === "string" ? b : JSON.stringify(b));
  die(`${what} failed (HTTP ${r.status}): ${msg}`);
}

// ── profile map ─────────────────────────────────────────────────────────────
function readMap(): Record<string, string> {
  if (!existsSync(MAP_FILE)) {
    writeFileSync(MAP_FILE, "{}\n");
    return {};
  }
  try {
    const o = JSON.parse(readFileSync(MAP_FILE, "utf8"));
    return o && typeof o === "object" ? o : {};
  } catch {
    return {};
  }
}

function writeMap(m: Record<string, string>): void {
  writeFileSync(MAP_FILE, JSON.stringify(m, null, 2) + "\n");
}

// ── API fetchers ────────────────────────────────────────────────────────────
interface Profile {
  _id: string;
  name: string;
  isDefault?: boolean;
  color?: string;
  accountUsernames?: string[];
}
interface Account {
  _id: string;
  platform: string;
  username?: string;
  displayName?: string;
  profileId?: { _id: string; name?: string } | string | null;
  platformStatus?: string;
  isActive?: boolean;
  enabled?: boolean;
}

async function getProfiles(): Promise<Profile[]> {
  const r = await api<{ profiles: Profile[] }>("GET", "/profiles");
  if (!r.ok) apiErr(r, "GET /profiles");
  return r.body.profiles ?? [];
}

async function getAccounts(): Promise<Account[]> {
  const r = await api<{ accounts: Account[] }>("GET", "/accounts");
  if (!r.ok) apiErr(r, "GET /accounts");
  return r.body.accounts ?? [];
}

function accProfileId(a: Account): string | null {
  if (!a.profileId) return null;
  return typeof a.profileId === "string" ? a.profileId : a.profileId._id ?? null;
}

// Resolve <project> → profileId. create=true → POST a profile when unmatched.
async function resolveProfile(
  project: string,
  opts: { create?: boolean } = {},
): Promise<{ id: string; profile?: Profile }> {
  const map = readMap();
  if (map[project]) return { id: map[project] };

  const profiles = await getProfiles();
  const want = normalize(project);
  const match = profiles.find((p) => normalize(p.name) === want);
  if (match) {
    map[project] = match._id;
    writeMap(map);
    return { id: match._id, profile: match };
  }

  if (opts.create) {
    const r = await api<{ profile: Profile }>("POST", "/profiles", { name: project });
    if (!r.ok) apiErr(r, "POST /profiles");
    const created = r.body.profile;
    map[project] = created._id;
    writeMap(map);
    return { id: created._id, profile: created };
  }

  die(
    `No Zernio profile for '${project}'. Run: omega-zernio connect ${project} <platform>`,
  );
}

// ── arg parsing ─────────────────────────────────────────────────────────────
interface Args {
  _: string[];
  flags: Record<string, string | boolean>;
}
function parseArgs(argv: string[]): Args {
  const _: string[] = [];
  const flags: Record<string, string | boolean> = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith("--")) {
      const eq = a.indexOf("=");
      if (eq !== -1) {
        flags[a.slice(2, eq)] = a.slice(eq + 1);
      } else {
        const name = a.slice(2);
        const next = argv[i + 1];
        if (next !== undefined && !next.startsWith("--")) {
          flags[name] = next;
          i++;
        } else {
          flags[name] = true;
        }
      }
    } else {
      _.push(a);
    }
  }
  return { _, flags };
}
const JSON_OUT = (f: Record<string, any>) => f.json === true;

function out(obj: unknown): void {
  process.stdout.write(JSON.stringify(obj, null, 2) + "\n");
}

// ── media ───────────────────────────────────────────────────────────────────
function mediaFromUrl(url: string): { type: string; url: string } {
  const info = MEDIA_EXT[extOf(url)];
  const type = info ? info.type : "image"; // default image for unknown URL ext
  return { type, url };
}

// Presign-upload a local file → returns the final public URL.
async function uploadLocal(path: string): Promise<{ type: string; url: string }> {
  if (!existsSync(path)) die(`media file not found: ${path}`);
  const ext = extOf(path);
  const info = MEDIA_EXT[ext];
  if (!info) {
    die(
      `unsupported media extension '.${ext}'. Supported: ${Object.keys(MEDIA_EXT).join(", ")}. ` +
      `Or pass a public http(s) URL instead.`,
    );
  }
  const filename = path.split("/").pop() || `upload.${ext}`;
  const size = statSync(path).size;
  const pres = await api<any>("POST", "/media/presign", {
    filename,
    contentType: info.contentType,
    size,
  });
  if (!pres.ok) {
    die(
      `media presign failed (HTTP ${pres.status}). Pass a public http(s) URL via --media instead.`,
    );
  }
  const p: any = pres.body || {};
  // Defensive: presign response shape isn't fully pinned. Inspect common keys.
  const uploadUrl =
    p.uploadUrl || p.presignedUrl || p.signedUrl || p.putUrl || (p.upload && p.upload.url);
  const finalUrl =
    p.publicUrl || p.mediaUrl || p.url || p.fileUrl || (p.upload && p.upload.publicUrl);
  if (!uploadUrl || !finalUrl) {
    die(
      `unexpected presign response (keys: ${Object.keys(p).join(", ") || "none"}). ` +
      `Pass a public http(s) URL via --media instead.`,
    );
  }
  const bytes = readFileSync(path);
  let put: Response;
  try {
    put = await fetch(uploadUrl, {
      method: "PUT",
      headers: { "Content-Type": info.contentType },
      body: bytes,
    });
  } catch (e: any) {
    die(`media upload PUT failed: ${e?.message ?? e}. Pass a public http(s) URL instead.`);
  }
  if (!put.ok) {
    die(`media upload PUT failed (HTTP ${put.status}). Pass a public http(s) URL instead.`);
  }
  return { type: info.type, url: finalUrl };
}

// ── subcommands ─────────────────────────────────────────────────────────────
async function cmdProfiles(a: Args): Promise<void> {
  const [profiles, accounts] = await Promise.all([getProfiles(), getAccounts()]);
  const countFor = (id: string) =>
    accounts.filter((ac) => accProfileId(ac) === id).length;
  const map = readMap();

  if (JSON_OUT(a.flags)) {
    out({
      profiles: profiles.map((p) => ({
        name: p.name,
        _id: p._id,
        isDefault: !!p.isDefault,
        accounts: countFor(p._id),
      })),
      mappedProjects: map,
    });
    return;
  }
  console.log("Profiles:");
  for (const p of profiles) {
    const def = p.isDefault ? " (default)" : "";
    console.log(`  ${p.name}${def}  —  ${countFor(p._id)} account(s)  [${p._id}]`);
  }
  const keys = Object.keys(map);
  console.log(`\nMapped projects (${MAP_FILE}):`);
  if (keys.length === 0) {
    console.log("  (none yet — run: omega-zernio connect <project> <platform>)");
  } else {
    for (const k of keys) console.log(`  ${k} → ${map[k]}`);
  }
}

async function cmdConnect(a: Args): Promise<void> {
  const project = a._[0];
  const platform = (a._[1] || "").toLowerCase();
  if (!project || !platform) {
    die("usage: omega-zernio connect <project> <platform>");
  }
  if (!PLATFORMS.includes(platform)) {
    die(`unknown platform '${platform}'. Valid: ${PLATFORMS.join(", ")}`);
  }
  const { id } = await resolveProfile(project, { create: true });
  const r = await api<{ authUrl: string; state?: string }>(
    "GET",
    `/connect/${platform}?profileId=${encodeURIComponent(id)}`,
  );
  if (!r.ok) apiErr(r, `GET /connect/${platform}`);
  const authUrl = r.body.authUrl;
  if (!authUrl) die(`connect response missing authUrl: ${JSON.stringify(r.body)}`);

  if (JSON_OUT(a.flags)) {
    out({ project, platform, profileId: id, authUrl });
    return;
  }
  console.log(`Authorize ${platform} for project '${project}' (profile ${id}):`);
  console.log(`\n  ${authUrl}\n`);
  console.log(
    `Open the URL above to authorize. The ${platform} account will attach to the '${project}' profile.`,
  );
}

async function cmdAccounts(a: Args): Promise<void> {
  const project = a._[0];
  const accounts = await getAccounts();
  let filtered = accounts;
  let profileName: string | undefined;
  if (project) {
    const { id } = await resolveProfile(project);
    const profiles = await getProfiles();
    profileName = profiles.find((p) => p._id === id)?.name;
    filtered = accounts.filter((ac) => accProfileId(ac) === id);
  }

  if (JSON_OUT(a.flags)) {
    out(filtered);
    return;
  }

  const profiles = await getProfiles();
  const pname = (id: string | null) =>
    (id && profiles.find((p) => p._id === id)?.name) || "—";

  if (filtered.length === 0) {
    console.log(project ? `No accounts for project '${project}'.` : "No connected accounts.");
    return;
  }
  const rows = filtered.map((ac) => {
    const pid = accProfileId(ac);
    const status = ac.platformStatus || (ac.isActive ? "active" : "inactive");
    return [
      ac.platform,
      ac.username ? `@${ac.username.replace(/^@/, "")}` : "—",
      pname(pid),
      status,
      ac._id,
    ];
  });
  const header = ["platform", "username", "profile", "status", "accountId"];
  const widths = header.map((h, i) =>
    Math.max(h.length, ...rows.map((r) => r[i].length)),
  );
  const fmt = (cols: string[]) =>
    cols.map((c, i) => c.padEnd(widths[i])).join("  ");
  console.log(fmt(header));
  console.log(widths.map((w) => "-".repeat(w)).join("  "));
  for (const r of rows) console.log(fmt(r));
}

async function cmdPost(a: Args): Promise<void> {
  const project = a._[0];
  if (!project) die("usage: omega-zernio post <project> --text \"…\" --platforms a,b,c [--media url|path] [--dry-run] [--schedule ISO]");
  const text = a.flags.text;
  if (typeof text !== "string" || !text.length) die("--text is required");
  const platStr = a.flags.platforms;
  if (typeof platStr !== "string" || !platStr.length) {
    die("--platforms is required (comma-separated, e.g. tiktok,twitter)");
  }
  const wanted = platStr.split(",").map((s) => s.trim().toLowerCase()).filter(Boolean);
  const bad = wanted.filter((p) => !PLATFORMS.includes(p));
  if (bad.length) die(`unknown platform(s): ${bad.join(", ")}. Valid: ${PLATFORMS.join(", ")}`);

  const dryRun = a.flags["dry-run"] === true;
  const schedule = typeof a.flags.schedule === "string" ? a.flags.schedule : undefined;

  // Resolve project profile + connected accounts for each requested platform.
  const { id: profileId } = await resolveProfile(project);
  const accounts = (await getAccounts()).filter((ac) => accProfileId(ac) === profileId);

  const platformEntries: { platform: string; accountId: string }[] = [];
  const missing: string[] = [];
  for (const plat of wanted) {
    const acc = accounts.find((ac) => ac.platform === plat);
    if (!acc) missing.push(plat);
    else platformEntries.push({ platform: plat, accountId: acc._id });
  }
  if (missing.length) {
    die(
      `project '${project}' has no connected account for: ${missing.join(", ")}.\n` +
      missing.map((p) => `  Run: omega-zernio connect ${project} ${p}`).join("\n"),
    );
  }

  // Media (optional).
  let mediaItems: { type: string; url: string }[] | undefined;
  const media = a.flags.media;
  if (typeof media === "string" && media.length) {
    const item = /^https?:\/\//i.test(media) ? mediaFromUrl(media) : await uploadLocal(media);
    mediaItems = [item];
  }

  // Build the /v1/posts body that WOULD be (or is) sent.
  const postBody: Record<string, any> = {
    content: text,
    platforms: platformEntries,
  };
  if (mediaItems) postBody.mediaItems = mediaItems;
  if (schedule) postBody.scheduledFor = schedule;
  else if (!dryRun) postBody.publishNow = true;

  if (dryRun) {
    // Validate via the dedicated preview endpoint — DOES NOT publish.
    const validateBody = {
      content: text,
      platforms: wanted.map((platform) => ({ platform })),
    };
    const v = await api<{
      valid: boolean;
      message?: string;
      errors?: { platform: string; error: string }[];
      warnings?: { platform: string; warning: string }[];
    }>("POST", "/tools/validate/post", validateBody);
    if (!v.ok) apiErr(v, "POST /tools/validate/post");

    // /v1/tools/validate/post body is only {content, platforms:[{platform}]} —
    // it never sees the --media we passed, so it flags "media required" even
    // when the real post WILL include media. When --media was provided, demote
    // those specific errors to informational notes (the would-send body already
    // carries mediaItems). Other real errors still block and still show valid.
    const MEDIA_REQ = /requires?.*media|media.*required/i;
    const allErrors = v.body.errors ?? [];
    const mediaNotes = mediaItems
      ? allErrors.filter((e) => MEDIA_REQ.test(e.error))
      : [];
    const realErrors = allErrors.filter((e) => !mediaNotes.includes(e));
    const effectiveValid = realErrors.length === 0 && (v.body.valid || mediaNotes.length > 0);

    if (JSON_OUT(a.flags)) {
      out({
        dryRun: true,
        validation: v.body,
        effectiveValid,
        mediaNotes,
        wouldSend: postBody,
      });
      return;
    }
    console.log("DRY RUN — nothing published.\n");
    console.log("Validation (/v1/tools/validate/post):");
    console.log(`  valid: ${effectiveValid}`);
    if (v.body.message) console.log(`  message: ${v.body.message}`);
    if (realErrors.length) {
      console.log("  errors:");
      for (const e of realErrors) console.log(`    - [${e.platform}] ${e.error}`);
    }
    if (mediaNotes.length) {
      console.log("  notes:");
      for (const e of mediaNotes) {
        console.log(
          `    - [${e.platform}] note: validator does not inspect media; you passed --media, the real post will include it`,
        );
      }
    }
    if (v.body.warnings?.length) {
      console.log("  warnings:");
      for (const w of v.body.warnings) console.log(`    - [${w.platform}] ${w.warning}`);
    }
    console.log("\nWould send → POST /v1/posts:");
    console.log(JSON.stringify(postBody, null, 2));
    return;
  }

  // Real publish / schedule.
  const r = await api<{ post: { _id: string; status?: string } }>("POST", "/posts", postBody);
  if (!r.ok) apiErr(r, "POST /posts");
  const post = r.body.post;
  if (JSON_OUT(a.flags)) {
    out({ posted: true, post });
    return;
  }
  console.log(`Posted. _id=${post?._id ?? "?"}${post?.status ? ` status=${post.status}` : ""}${schedule ? ` scheduledFor=${schedule}` : ""}`);
}

async function cmdStatus(a: Args): Promise<void> {
  const keyPresent = hasKey();
  if (!keyPresent) {
    if (JSON_OUT(a.flags)) {
      out({ keyPresent: false, reachable: false });
      return;
    }
    console.log("key present: false");
    console.log(`Add ZERNIO_API_KEY=... to ${SECRETS_FILE}`);
    process.exit(1);
  }
  const r = await api<{ profiles: Profile[] }>("GET", "/profiles");
  const reachable = r.ok;
  let accounts: Account[] = [];
  if (reachable) {
    try {
      accounts = await getAccounts();
    } catch {
      /* ignore */
    }
  }
  const byPlatform: Record<string, number> = {};
  for (const ac of accounts) byPlatform[ac.platform] = (byPlatform[ac.platform] || 0) + 1;
  const map = readMap();

  if (JSON_OUT(a.flags)) {
    out({
      keyPresent: true,
      reachable,
      httpStatus: r.status,
      profiles: reachable ? (r.body.profiles ?? []).length : 0,
      accounts: accounts.length,
      byPlatform,
      mappedProjects: map,
    });
    if (!reachable) process.exit(1);
    return;
  }

  console.log(`key present: true`);
  console.log(`API reachable: ${reachable} (GET /v1/profiles → HTTP ${r.status})`);
  if (reachable) {
    console.log(`profiles: ${(r.body.profiles ?? []).length}`);
    console.log(`accounts: ${accounts.length}`);
    const plats = Object.keys(byPlatform).sort();
    if (plats.length) {
      console.log("by platform:");
      for (const p of plats) console.log(`  ${p}: ${byPlatform[p]}`);
    }
  }
  const mk = Object.keys(map);
  console.log(`mapped projects: ${mk.length ? mk.join(", ") : "(none)"}`);
  if (!reachable) process.exit(1);
}

// ── help ────────────────────────────────────────────────────────────────────
const HELP = `omega-zernio — social publishing via the Zernio REST API (one profile per project)

Usage:
  omega-zernio status                                 Health: key present, API reachable, counts
  omega-zernio profiles                               List profiles + mapped projects
  omega-zernio connect <project> <platform>           Resolve/create profile, print hosted authUrl
  omega-zernio accounts [project]                     List connected accounts (optionally per project)
  omega-zernio post <project> --text "…" --platforms a,b,c [--media url|path] [--dry-run] [--schedule ISO]

Global flags:
  --json     Machine-readable output
  --help     This help

Platforms: ${PLATFORMS.join(", ")}

The key (ZERNIO_API_KEY) is read from the env or ~/.omega/secrets/integrations.env
and is never printed. --dry-run validates + previews the post body WITHOUT publishing.`;

// ── entry ───────────────────────────────────────────────────────────────────
async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  const a = parseArgs(argv);
  const cmd = a._.shift();

  if (!cmd || a.flags.help === true || cmd === "help") {
    console.log(HELP);
    process.exit(0);
  }

  switch (cmd) {
    case "profiles": return cmdProfiles(a);
    case "connect": return cmdConnect(a);
    case "accounts": return cmdAccounts(a);
    case "post": return cmdPost(a);
    case "status": return cmdStatus(a);
    default:
      die(`unknown subcommand '${cmd}'. Run: omega-zernio --help`);
  }
}

main().catch((e: any) => die(e?.message ?? String(e)));

import { afterAll, beforeAll, expect, test } from "bun:test";
import {
  chmodSync,
  existsSync,
  linkSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  rmdirSync,
  statSync,
  symlinkSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { pathToFileURL } from "node:url";

let root = "";
let api: typeof import("../../telegram-bot/omega-tg-bot");

beforeAll(async () => {
  root = mkdtempSync(`${tmpdir()}/omega-ecosystem-tg-`);
  mkdirSync(`${root}/rules`, { recursive: true });
  mkdirSync(`${root}/state`, { recursive: true });
  writeFileSync(`${root}/doctrine.txt`, "doctrine-v1\n");
  writeFileSync(`${root}/rules/R-TEST.md`, "v1\n");
  writeFileSync(`${root}/AGENTS.md`, "kernel-v1\n");
  const omega = `${root}/omega-mock`;
  writeFileSync(omega, `#!/bin/sh\ncat ${JSON.stringify(`${root}/doctrine.txt`)}\n`);
  chmodSync(omega, 0o700);
  process.env.OMEGA_DIR = root;
  process.env.OMEGA_BIN = omega;
  process.env.OMEGA_ECOSYSTEM_TEST = "1";
  process.env.OMEGA_STORE_LOCK_STALE_MS = "1";
  const moduleUrl = pathToFileURL(resolve(dirname(import.meta.path), "../../telegram-bot/omega-tg-bot.ts"));
  moduleUrl.searchParams.set("ecosystem", String(Date.now()));
  api = await import(moduleUrl.href);
});

afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });

test("groups use atomic 0600 writes and reconcile stale unrelated edits", () => {
  writeFileSync(`${root}/telegram-groups.json`, JSON.stringify({ hub: -1001, topics: { "1": "A" } }));
  const first = api.loadGroups();
  const second = api.loadGroups();
  first.topics!["2"] = "B";
  second.alerts_topic = 99;
  api.saveGroups(first);
  api.saveGroups(second);

  const saved = JSON.parse(readFileSync(`${root}/telegram-groups.json`, "utf8"));
  expect(saved.topics).toEqual({ "1": "A", "2": "B" });
  expect(saved.alerts_topic).toBe(99);
  expect(statSync(`${root}/telegram-groups.json`).mode & 0o777).toBe(0o600);
  expect(readdirSync(root).filter(name => name.includes(".tmp.") || name.endsWith(".lock"))).toEqual([]);
});

test("project mutation preserves fiche and unknown fields", () => {
  writeFileSync(`${root}/projects.json`, JSON.stringify({
    schema_version: 9,
    owner: "external",
    projects: [{
      name: "A", path: "/p/a", created_at: "x", repo: "https://example.invalid/a.git",
      telegram_topic_id: null, oracle_session: null, git_email: null,
      slug: "acme/a", default_branch: "trunk", future: { nested: true },
    }],
  }));
  api.updateRegistry(registry => { registry.projects[0].telegram = false; });
  const saved = JSON.parse(readFileSync(`${root}/projects.json`, "utf8"));
  expect(saved.schema_version).toBe(9);
  expect(saved.owner).toBe("external");
  expect(saved.projects[0].repo).toBe("https://example.invalid/a.git");
  expect(saved.projects[0].slug).toBe("acme/a");
  expect(saved.projects[0].default_branch).toBe("trunk");
  expect(saved.projects[0].future).toEqual({ nested: true });
  expect(saved.projects[0].telegram).toBe(false);
});

test("project mutations reject invalid output and conflicting logical identity", () => {
  const projects = `${root}/projects.json`;
  const original = JSON.stringify({ projects: [{
    name: "A", path: "/p/a", created_at: "x",
    telegram_topic_id: null, oracle_session: null, git_email: null,
  }] });
  writeFileSync(projects, original);
  expect(() => api.updateRegistry(registry => {
    registry.projects.push({
      name: "Broken", path: "", created_at: "x",
      telegram_topic_id: null, oracle_session: null, git_email: null,
    });
  })).toThrow("refusing invalid mutated JSON state shape");
  expect(readFileSync(projects, "utf8")).toBe(original);

  expect(() => api.recordProject("A", "/p/different")).toThrow("already registered at /p/a");
  expect(readFileSync(projects, "utf8")).toBe(original);
});

test("malformed and structurally invalid nested state fails closed", () => {
  const agentBots = `${root}/agent-bots.json`;
  writeFileSync(agentBots, "{not-json");
  expect(() => api.loadAgentBots()).toThrow("refusing malformed JSON state");
  writeFileSync(agentBots, JSON.stringify({ alpha: { token: "123:TEST", allow: ["1"], project: "Alpha" } }));
  expect(() => api.loadAgentBots()).toThrow("refusing invalid JSON state shape");
  writeFileSync(agentBots, JSON.stringify({ alpha: { token: "123:TEST", allow: [1], project: "Alpha", kind: "future-kind" } }));
  expect(() => api.loadAgentBots()).toThrow("refusing invalid JSON state shape");
  writeFileSync(agentBots, "{}");

  const groups = `${root}/telegram-groups.json`;
  writeFileSync(groups, JSON.stringify({ hub: 42, isForum: true, topics: {} }));
  expect(() => api.loadGroups()).toThrow("refusing invalid JSON state shape");
  writeFileSync(groups, JSON.stringify({ hub: -100123, isForum: true, topics: { nope: "Alpha" } }));
  expect(() => api.loadGroups()).toThrow("refusing invalid JSON state shape");
  writeFileSync(groups, JSON.stringify({ hub: -100123, isForum: true, topics: { "7": 42 } }));
  expect(() => api.loadGroups()).toThrow("refusing invalid JSON state shape");
  writeFileSync(groups, JSON.stringify({ hub: -100123, topics: { "7": "Alpha", "8": "alpha" } }));
  expect(() => api.loadGroups()).toThrow("refusing invalid JSON state shape");
  writeFileSync(groups, "{}");

  const projects = `${root}/projects.json`;
  writeFileSync(projects, JSON.stringify({ schema_version: 0, projects: [] }));
  expect(() => api.loadRegistry()).toThrow("refusing invalid JSON state shape");
  writeFileSync(projects, JSON.stringify({ projects: [{
    name: "A", path: "/p/a", created_at: "x", telegram_topic_id: 1.5,
    oracle_session: null, git_email: null,
  }] }));
  expect(() => api.loadRegistry()).toThrow("refusing invalid JSON state shape");
  writeFileSync(projects, JSON.stringify({ projects: [{ name: "A", path: "/p/a" }] }));
  expect(() => api.loadRegistry()).toThrow("refusing invalid JSON state shape");
  writeFileSync(projects, JSON.stringify({ projects: [] }));

  const pending = `${root}/state/tg-pending.json`;
  writeFileSync(pending, JSON.stringify([["main:42", { kind: "made-up", ts: Date.now() }]]));
  expect(() => api.setPending(42, "new-project", "Alpha")).toThrow("refusing invalid JSON state shape");
  writeFileSync(pending, JSON.stringify([["main:42", { kind: "new-project", ts: Date.now(), arg: 7 }]]));
  expect(() => api.setPending(42, "new-project", "Alpha")).toThrow("refusing invalid JSON state shape");
  writeFileSync(pending, "[]");
});

test("Telegram TOML security fields load strictly and fail closed", () => {
  const config = `${root}/telegram.toml`;
  writeFileSync(config, 'bot_token = "123:TEST"\nallow_user_ids = [7, 9]\nenabled = true\n');
  expect(api.loadTelegramRuntimeConfig(config)).toEqual({
    botToken: "123:TEST",
    allowUserIds: [7, 9],
    enabled: true,
  });

  writeFileSync(config, 'bot_token = "123:TEST"\nallow_user_ids = ["7"]\n');
  expect(() => api.loadTelegramRuntimeConfig(config)).toThrow("allow_user_ids must be an array of positive integers");
  writeFileSync(config, 'allow_user_ids = [7]\nallow_user_ids = [9]\n');
  expect(() => api.loadTelegramRuntimeConfig(config)).toThrow("duplicate Telegram config key");
  writeFileSync(config, "enabled = false\n");
  expect(api.loadTelegramRuntimeConfig(config).enabled).toBe(false);
});

test("stale lock takeover is owner-token exact and ABA safe", () => {
  const data = `${root}/aba-state.json`;
  const lock = `${data}.lock`;
  mkdirSync(lock, { mode: 0o700 });
  writeFileSync(`${lock}/owner.json`, JSON.stringify({
    schema_version: 1,
    token: "dead-generation",
    pid: 2_147_483_647,
    process_start: "definitely-not-live",
    created_ms: 1,
  }));
  chmodSync(`${lock}/owner.json`, 0o600);
  const old = new Date(Date.now() - 10_000);
  utimesSync(lock, old, old);

  const replacement = api.acquireJsonLock(data);
  const replacementOwner = JSON.parse(readFileSync(`${lock}/owner.json`, "utf8"));
  expect(replacementOwner.token).not.toBe("dead-generation");

  api.releaseJsonLock(lock, "dead-generation");
  expect(JSON.parse(readFileSync(`${lock}/owner.json`, "utf8")).token).toBe(replacementOwner.token);
  expect(() => rmdirSync(lock)).toThrow();
  api.releaseJsonLock(replacement.lock, replacement.token);
  expect(existsSync(lock)).toBe(false);
  expect(readdirSync(root).some(name => name.includes(".stale."))).toBe(false);
});

test("lock owner symlinks and hard links are never trusted as authority", () => {
  const lock = `${root}/aliased-state.json.lock`;
  const external = `${root}/external-owner.json`;
  mkdirSync(lock, { mode: 0o700 });
  writeFileSync(external, JSON.stringify({
    schema_version: 1,
    token: "forged-generation",
    pid: 2_147_483_647,
    process_start: "gone",
    created_ms: 1,
  }), { mode: 0o600 });

  symlinkSync(external, `${lock}/owner.json`);
  expect(() => api.releaseJsonLock(lock, "forged-generation")).toThrow();
  expect(existsSync(lock)).toBe(true);
  rmSync(`${lock}/owner.json`);

  linkSync(external, `${lock}/owner.json`);
  expect(() => api.releaseJsonLock(lock, "forged-generation")).toThrow();
  expect(existsSync(lock)).toBe(true);
});

test("agent-bot and pending writers are atomic, locked and mode 0600", () => {
  writeFileSync(`${root}/agent-bots.json`, "{}");
  api.updateAgentBots(bots => {
    bots.alpha = { token: "123:TEST", allow: [1], project: "Alpha" };
  });
  expect(JSON.parse(readFileSync(`${root}/agent-bots.json`, "utf8")).alpha.project).toBe("Alpha");
  expect(statSync(`${root}/agent-bots.json`).mode & 0o777).toBe(0o600);

  api.setPending(42, "new-project", "Alpha");
  let entries = JSON.parse(readFileSync(`${root}/state/tg-pending.json`, "utf8"));
  expect(entries).toHaveLength(1);
  expect(entries[0][0]).toBe("main:42");
  expect(entries[0][1].arg).toBe("Alpha");
  expect(statSync(`${root}/state/tg-pending.json`).mode & 0o777).toBe(0o600);
  api.clearPending(42);
  entries = JSON.parse(readFileSync(`${root}/state/tg-pending.json`, "utf8"));
  expect(entries).toEqual([]);
});

test("command collisions and the 100-command cap are explicit", () => {
  const base = Array.from({ length: 99 }, (_, i) => ({ command: `base_${i}`, description: "base" }));
  const project = (name: string) => ({
    name, path: `/p/${name}`, created_at: "x",
    telegram_topic_id: null, oracle_session: null, git_email: null,
  });
  const plan = api.planProjectCommands(base, [project("Foo Bar"), project("Foo-Bar"), project("Alpha"), project("Beta")]);
  expect(plan.collisions).toEqual([{ command: "foo_bar", projects: ["Foo Bar", "Foo-Bar"] }]);
  expect(plan.commands.map(command => command.command)).toEqual(["alpha"]);
  expect(plan.omitted).toEqual(["Beta"]);
});

test("doctrine cache invalidates when the installed rule surface changes", () => {
  expect(api.doctrineCached("worker")).toBe("doctrine-v1");
  writeFileSync(`${root}/doctrine.txt`, "doctrine-v2\n");
  writeFileSync(`${root}/rules/R-TEST.md`, "v2\n");
  const future = new Date(Date.now() + 2_000);
  utimesSync(`${root}/rules/R-TEST.md`, future, future);
  expect(api.doctrineCached("worker")).toBe("doctrine-v2");
});

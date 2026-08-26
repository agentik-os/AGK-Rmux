#!/usr/bin/env bun
import { createHash } from "node:crypto";
import { readdir, readFile, stat, writeFile } from "node:fs/promises";
import { join, relative, resolve, sep } from "node:path";

const PRODUCTS = [
  "mindset-os",
  "health-energy-os",
  "habit-tracker-os",
  "alignment-os",
  "strategy-portfolio-os",
  "brainstorm-os",
  "market-research-os",
  "blueprint-os",
  "design-os",
  "stepper-os",
  "builder-os",
  "quality-evaluation-release-os",
  "storyteller-os",
  "revenue-os",
  "delivery-customer-success-os",
  "relationship-network-os",
  "wealth-capital-os",
  "execution-os",
  "operations-automation-os",
  "review-governance-os",
  "context-memory-os",
  "ai-logic-os",
  "content-os",
  "books-os",
] as const;

const REQUIRED = [
  "README.md",
  "README_FR.md",
  "MANIFEST.json",
  "OMEGA_INTEGRATION.md",
  "SKILL.md",
] as const;
const README_PARITY_PRODUCTS = new Set([
  "ai-logic-os",
  "alignment-os",
  "content-os",
  "context-memory-os",
  "delivery-customer-success-os",
  "health-energy-os",
  "operations-automation-os",
  "quality-evaluation-release-os",
  "relationship-network-os",
  "revenue-os",
  "review-governance-os",
  "strategy-portfolio-os",
  "wealth-capital-os",
]);

type FileRecord = { path: string; sha256: string; bytes: number };
type Manifest = Record<string, unknown> & {
  slug?: string;
  counts?: Record<string, number>;
  files?: Array<string | FileRecord>;
  events?: EventContract;
};
type EventEdge = {
  name: string;
  consumed_by?: string | string[] | null;
  produced_by?: string | string[];
  payload: Record<string, unknown>;
};
type EventContract = {
  schema_status: "defined";
  produces: EventEdge[];
  consumes: EventEdge[];
};

const repoRoot = resolve(import.meta.dir, "..");
const osRoot = join(repoRoot, "OS");
const writeMode = process.argv.includes("--write");
const EXCLUDED_DIRS = new Set([
  ".git",
  ".pytest_cache",
  ".venv",
  "__pycache__",
  "ledger",
  "node_modules",
]);

async function walk(root: string, dir = root): Promise<string[]> {
  const found: string[] = [];
  const entries = await readdir(dir, { withFileTypes: true });
  entries.sort((a, b) => a.name.localeCompare(b.name));
  for (const entry of entries) {
    if (EXCLUDED_DIRS.has(entry.name) || entry.name.endsWith(".egg-info")) continue;
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      found.push(...(await walk(root, fullPath)));
    } else if (
      entry.isFile() &&
      entry.name !== "MANIFEST.json" &&
      entry.name !== "checksums.sha256" &&
      !entry.name.endsWith(".pyc")
    ) {
      found.push(relative(root, fullPath).split(sep).join("/"));
    }
  }
  return found;
}

async function record(root: string, path: string): Promise<FileRecord> {
  const content = await readFile(join(root, path));
  return {
    path,
    sha256: createHash("sha256").update(content).digest("hex"),
    bytes: content.byteLength,
  };
}

async function verifyMarkdownLinks(root: string, paths: string[], slug: string): Promise<void> {
  for (const path of paths.filter((candidate) => candidate.endsWith(".md"))) {
    const content = await readFile(join(root, path), "utf8");
    const links = content.matchAll(/\[[^\]]*\]\(([^)]+)\)/g);
    for (const match of links) {
      const rawTarget = match[1].trim().replace(/^<|>$/g, "");
      if (
        !rawTarget ||
        rawTarget.startsWith("#") ||
        rawTarget.startsWith("/") ||
        rawTarget.startsWith("~") ||
        /^[a-z][a-z0-9+.-]*:/i.test(rawTarget)
      ) {
        continue;
      }
      const target = decodeURIComponent(rawTarget.split("#", 1)[0]);
      const resolved = resolve(root, join(path, ".."), target);
      if (!resolved.startsWith(`${root}${sep}`) && resolved !== root) {
        failures.push(`${slug}: ${path} link escapes the pack (${rawTarget})`);
        continue;
      }
      try {
        await stat(resolved);
      } catch {
        failures.push(`${slug}: ${path} has broken link ${rawTarget}`);
      }
    }
  }
}

function countPrefix(files: FileRecord[], prefix: string): number {
  return files.filter((file) => file.path.startsWith(`${prefix}/`)).length;
}

function expectedCounts(files: FileRecord[]): Record<string, number> {
  const nestedSkills = countPrefix(files, "skills");
  return {
    agents: countPrefix(files, "agents"),
    skills: nestedSkills || (files.some((file) => file.path === "SKILL.md") ? 1 : 0),
    protocols: countPrefix(files, "protocols"),
    references: countPrefix(files, "references"),
    scripts: countPrefix(files, "scripts"),
    assets: countPrefix(files, "assets"),
    files: files.length,
  };
}

function stable(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function checksumFile(files: FileRecord[]): string {
  return files.map((file) => `${file.sha256}  ${file.path}`).join("\n") + "\n";
}

const failures: string[] = [];
let verifiedFiles = 0;
const manifests = new Map<string, Manifest>();

for (const slug of PRODUCTS) {
  const root = join(osRoot, slug);
  for (const required of REQUIRED) {
    try {
      const metadata = await stat(join(root, required));
      if (!metadata.isFile()) failures.push(`${slug}: ${required} is not a file`);
    } catch {
      failures.push(`${slug}: missing ${required}`);
    }
  }

  let manifest: Manifest;
  try {
    manifest = JSON.parse(await readFile(join(root, "MANIFEST.json"), "utf8"));
  } catch (error) {
    failures.push(`${slug}: MANIFEST.json is invalid (${String(error)})`);
    continue;
  }
  const productId = slug.replace(/-os$/, "");
  if (manifest.slug !== productId) {
    failures.push(`${slug}: manifest slug must be ${productId}, got ${String(manifest.slug)}`);
  }
  manifests.set(productId, manifest);

  const paths = await walk(root);
  const files = await Promise.all(paths.map((path) => record(root, path)));
  files.sort((a, b) => a.path.localeCompare(b.path));
  await verifyMarkdownLinks(root, paths, slug);
  if (README_PARITY_PRODUCTS.has(slug)) {
    const french = await readFile(join(root, "README_FR.md"), "utf8");
    for (const requiredSection of ["## Commandes", "## Installation"]) {
      if (!french.includes(requiredSection)) failures.push(`${slug}: README_FR.md lacks ${requiredSection}`);
    }
    if (!french.includes("## Handoffs") && !french.includes("## Passages de relais")) {
      failures.push(`${slug}: README_FR.md lacks a handoff section`);
    }
    if (!french.includes("## Ce que contient") && !french.includes("## Ce que cet OS contient")) {
      failures.push(`${slug}: README_FR.md lacks a pack-specific contents section`);
    }
  }
  const counts = expectedCounts(files);
  const expectedChecksum = checksumFile(files);
  const nextManifest: Manifest = {
    ...manifest,
    schema_version: 1,
    counts,
    files,
  };

  if (writeMode) {
    await writeFile(join(root, "MANIFEST.json"), stable(nextManifest));
    await writeFile(join(root, "checksums.sha256"), expectedChecksum);
  } else {
    if (stable(manifest) !== stable(nextManifest)) {
      failures.push(`${slug}: MANIFEST.json inventory or counts are stale`);
    }
    let actualChecksum = "";
    try {
      actualChecksum = await readFile(join(root, "checksums.sha256"), "utf8");
    } catch {
      failures.push(`${slug}: missing checksums.sha256`);
    }
    if (actualChecksum !== expectedChecksum) {
      failures.push(`${slug}: checksums.sha256 is stale`);
    }
  }
  verifiedFiles += files.length;
}

function names(value: string | string[] | null | undefined): string[] {
  if (value == null) return [];
  return Array.isArray(value) ? value : [value];
}

const MEMORY_BUS_EVENTS = new Set([
  "memory.context.compiled",
  "memory.record.staged",
  "memory.record.verified",
]);

for (const [productId, manifest] of manifests) {
  const events = manifest.events;
  if (
    !events ||
    events.schema_status !== "defined" ||
    !Array.isArray(events.produces) ||
    !Array.isArray(events.consumes)
  ) {
    failures.push(`${productId}: events must use the defined produces/consumes schema`);
    continue;
  }

  for (const [direction, edges] of [
    ["produces", events.produces],
    ["consumes", events.consumes],
  ] as const) {
    const seen = new Set<string>();
    for (const edge of edges) {
      if (!edge || typeof edge.name !== "string" || !edge.name.trim()) {
        failures.push(`${productId}: ${direction} contains an unnamed event`);
        continue;
      }
      if (edge.name.includes("*")) {
        failures.push(`${productId}: wildcard event is forbidden (${edge.name})`);
      }
      if (seen.has(edge.name)) failures.push(`${productId}: duplicate ${direction} ${edge.name}`);
      seen.add(edge.name);
      if (!edge.payload || typeof edge.payload !== "object" || Array.isArray(edge.payload)) {
        failures.push(`${productId}: ${edge.name} needs a machine-readable payload object`);
      }

      const peers = names(direction === "produces" ? edge.consumed_by : edge.produced_by);
      for (const peer of peers) {
        if (peer === "*" || peer.startsWith("none")) {
          failures.push(`${productId}: ${edge.name} has non-concrete peer ${peer}`);
        } else if (!manifests.has(peer)) {
          failures.push(`${productId}: ${edge.name} references unknown product ${peer}`);
        }
      }
    }
  }
}

for (const [producerId, manifest] of manifests) {
  const events = manifest.events;
  if (!events || !Array.isArray(events.produces)) continue;
  for (const produced of events.produces) {
    if (MEMORY_BUS_EVENTS.has(produced.name)) continue;
    for (const consumerId of names(produced.consumed_by)) {
      const consumer = manifests.get(consumerId)?.events;
      if (!consumer || !Array.isArray(consumer.consumes)) continue;
      const reciprocal = consumer.consumes.find(
        (edge) => edge.name === produced.name && names(edge.produced_by).includes(producerId),
      );
      if (!reciprocal) {
        failures.push(`${producerId}: ${produced.name} is not reciprocated by ${consumerId}`);
      }
    }
  }
}

for (const [consumerId, manifest] of manifests) {
  const events = manifest.events;
  if (!events || !Array.isArray(events.consumes)) continue;
  for (const consumed of events.consumes) {
    if (MEMORY_BUS_EVENTS.has(consumed.name)) continue;
    for (const producerId of names(consumed.produced_by)) {
      const producer = manifests.get(producerId)?.events;
      if (!producer || !Array.isArray(producer.produces)) continue;
      const reciprocal = producer.produces.find(
        (edge) => edge.name === consumed.name && names(edge.consumed_by).includes(consumerId),
      );
      if (!reciprocal) {
        failures.push(`${consumerId}: ${consumed.name} is not produced for it by ${producerId}`);
      }
    }
  }
}

if (failures.length) {
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}

console.log(
  `${writeMode ? "Wrote" : "Verified"} ${PRODUCTS.length} OS manifests covering ${verifiedFiles} content files.`,
);

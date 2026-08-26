#!/usr/bin/env bun
/**
 * Bun fallback for the OmegaOS planner engine — used ONLY when the `omega`
 * binary is not on PATH. The real driver (ready-set → spawn worker → Guardian
 * verify → advance) lives in Rust (`omega plan-run`); this fallback is a thin,
 * read-only safety net: it parses the SAME typed `.planner/tracker.json`,
 * computes the DAG ready-set, and reports it. It does NOT spawn workers — full
 * autonomous execution requires the engine (install it: `./install.sh`).
 *
 * Usage:
 *   bun plan.ts status [dir]    # progress + DAG (mirrors `omega plan-status`)
 *   bun plan.ts run [dir]       # show ready-set + direct to the engine
 */

import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";

type Step = {
  step_id: string;
  title: string;
  status: "pending" | "in_progress" | "done" | "blocked" | "failed";
  depends_on: string[];
  files_to_touch: string[];
  wave?: string | null;
};
type Tracker = {
  project: string;
  active_phase: number;
  total_phases: number;
  steps: Step[];
};

const ICON: Record<Step["status"], string> = {
  pending: "[ ]",
  in_progress: "[~]",
  done: "[x]",
  blocked: "[!]",
  failed: "[F]",
};

function load(dir: string): Tracker {
  const path = join(dir, ".planner", "tracker.json");
  if (!existsSync(path)) {
    console.error(`no .planner/tracker.json in ${dir} — run /planner first`);
    process.exit(1);
  }
  let t: Tracker;
  try {
    t = JSON.parse(readFileSync(path, "utf8")) as Tracker;
  } catch (e: any) {
    // Surface the REAL parse error — a malformed tracker is not "no tracker".
    console.error(`malformed ${path}: ${e?.message || e}`);
    process.exit(1);
  }
  validate(t);
  return t;
}

/** Mirror of the Rust gate's structural checks (validate() in planner.rs):
 *  dangling deps, dependency cycles, and empty/bare-directory files_to_touch.
 *  The fallback only REPORTS (it never spawns), so violations are loud
 *  warnings — the engine itself refuses to run such a plan. */
function validate(t: Tracker) {
  const ids = new Set(t.steps.map((s) => s.step_id));
  for (const s of t.steps) {
    for (const d of s.depends_on)
      if (!ids.has(d)) console.error(`WARN ${s.step_id}: depends_on '${d}' does not exist`);
    if (s.files_to_touch.length === 0)
      console.error(`WARN ${s.step_id}: files_to_touch is empty — scope-claim locking is impossible`);
    for (const f of s.files_to_touch)
      if (f.endsWith("/")) console.error(`WARN ${s.step_id}: bare directory scope '${f}' — claim files, not trees`);
  }
  // Cycle check: iteratively strip steps whose deps are all stripped; leftovers cycle.
  const stripped = new Set<string>();
  let advanced = true;
  while (advanced) {
    advanced = false;
    for (const s of t.steps) {
      if (stripped.has(s.step_id)) continue;
      if (s.depends_on.every((d) => stripped.has(d) || !ids.has(d))) {
        stripped.add(s.step_id);
        advanced = true;
      }
    }
  }
  const cyclic = t.steps.filter((s) => !stripped.has(s.step_id));
  if (cyclic.length)
    console.error(`WARN dependency cycle: ${cyclic.map((s) => s.step_id).join(" → ")} — these steps can never become ready`);
}

function depsSatisfied(t: Tracker, s: Step): boolean {
  return s.depends_on.every(
    (d) => t.steps.find((x) => x.step_id === d)?.status === "done",
  );
}

/** Pending steps whose deps are done and whose files are pairwise-disjoint. */
function readySteps(t: Tracker): Step[] {
  const out: Step[] = [];
  const claimed = new Set<string>();
  for (const s of t.steps) {
    if (s.status !== "pending" || !depsSatisfied(t, s)) continue;
    if (s.files_to_touch.some((f) => claimed.has(f))) continue;
    s.files_to_touch.forEach((f) => claimed.add(f));
    out.push(s);
  }
  return out;
}

function status(dir: string) {
  const t = load(dir);
  const done = t.steps.filter((s) => s.status === "done").length;
  const failed = t.steps.filter((s) => s.status === "failed").length;
  const ready = readySteps(t).length;
  const blocked = t.steps.filter(
    (s) => s.status === "pending" && !depsSatisfied(t, s),
  ).length;
  const pct = t.steps.length ? Math.round((done / t.steps.length) * 100) : 0;
  console.log(
    `Plan: ${t.project} | ${pct}% (${done}/${t.steps.length} done) | ready ${ready} | blocked ${blocked} | failed ${failed} | phase ${t.active_phase}/${t.total_phases}`,
  );
  for (const s of t.steps) console.log(`  ${ICON[s.status]} ${s.step_id} ${s.title}`);
}

function run(dir: string) {
  const t = load(dir);
  const ready = readySteps(t);
  console.log(
    "omega binary not found — the autonomous driver (spawn + Guardian verify) requires it.",
  );
  console.log("Install the engine:  ./install.sh   (then: omega plan-run .)");
  console.log("");
  if (ready.length === 0) {
    console.log("No ready steps (plan complete, or blocked behind failures).");
  } else {
    console.log(`Next ${ready.length} ready step(s) (deps satisfied, file-disjoint):`);
    for (const s of ready) console.log(`  → ${s.step_id} ${s.title}  [${s.files_to_touch.join(", ")}]`);
  }
  process.exit(ready.length > 0 ? 2 : 0);
}

const cmd = process.argv[2] ?? "status";
const dir = process.argv[3] ?? ".";
if (cmd === "status") status(dir);
else if (cmd === "run") run(dir);
else {
  console.error(`unknown command: ${cmd} (use: status | run)`);
  process.exit(1);
}

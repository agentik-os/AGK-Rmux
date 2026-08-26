#!/usr/bin/env node
'use strict';
/*
 * Tests for the omega-os npm installer package.
 *
 * Contract, non-negotiable (these are the reasons this file has no imports
 * beyond Node builtins):
 *   - ZERO dependencies. The published package declares none and must keep
 *     declaring none, so its own test suite may not smuggle one in.
 *   - OFFLINE. Nothing here reaches the network, installs anything, or spawns
 *     npm. The only child process is `node --check`, a pure parse.
 *   - NO ~/.omega. Nothing here reads or writes the user's OmegaOS home.
 *     bin/omega-os.js is safe to require() because of its
 *     `require.main === module` guard (bin/omega-os.js:1013) — importing it
 *     defines exports and runs no installer.
 *
 * Runs on `node:test` when the installed Node has it (>= 16.17 / >= 18), and
 * falls back to a built-in harness on older Node 16, which the package's
 * `engines.node: >=16` still admits. Both paths exit non-zero on failure.
 */

const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const PKG_DIR = path.resolve(__dirname, '..');
const PKG_JSON_PATH = path.join(PKG_DIR, 'package.json');
const BIN_PATH = path.join(PKG_DIR, 'bin', 'omega-os.js');
const README_PATH = path.join(PKG_DIR, 'README.md');
// The repo checkout, when the tests run from a clone rather than a tarball.
const REPO_ROOT = path.resolve(PKG_DIR, '..');
const INSTALL_SH = path.join(REPO_ROOT, 'install.sh');

const pkgRaw = fs.readFileSync(PKG_JSON_PATH, 'utf8');
const binSrc = fs.readFileSync(BIN_PATH, 'utf8');

// ── tiny harness ─────────────────────────────────────────────────────────────
// Cases are collected first, then dispatched to node:test or to the fallback.
// Every case is synchronous, so the fallback needs no scheduling logic.
const cases = [];
const t = (name, fn) => cases.push([name, fn]);
const skips = [];
function skip(name, reason) {
  // An honest skip PRINTS why. A check that cannot see its subject must never
  // look like a pass.
  skips.push(name + ' — SKIPPED: ' + reason);
}

// ── package.json ─────────────────────────────────────────────────────────────

t('package.json is valid JSON', () => {
  assert.doesNotThrow(() => JSON.parse(pkgRaw), 'package.json must parse');
});

const pkg = JSON.parse(pkgRaw);

t('package identity is intact', () => {
  assert.strictEqual(pkg.name, 'omega-os', 'the published name must not drift');
  assert.ok(typeof pkg.description === 'string' && pkg.description.length > 0,
    'a published package needs a description');
  assert.strictEqual(pkg.license, 'MIT OR Apache-2.0');
  assert.ok(pkg.repository && pkg.repository.directory === 'installer',
    'repository.directory must point at installer/ so npm links the right subdir');
});

t('version is valid semver', () => {
  // Official semver.org recommended regex.
  const SEMVER = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;
  assert.ok(typeof pkg.version === 'string', 'version must be a string');
  assert.match(pkg.version, SEMVER, 'version "' + pkg.version + '" is not valid semver');
});

t('the installer declares NO runtime dependencies', () => {
  // npx must stay fast and supply-chain-free. Any of these fields carrying an
  // entry means a fresh `npx omega-os` starts downloading third-party code.
  for (const field of ['dependencies', 'peerDependencies', 'optionalDependencies']) {
    const v = pkg[field];
    if (v === undefined) continue;
    assert.strictEqual(Object.keys(v).length, 0,
      field + ' must be empty — the installer is Node-builtins only, found: ' + Object.keys(v).join(', '));
  }
  const bundled = pkg.bundledDependencies || pkg.bundleDependencies;
  if (bundled !== undefined) {
    assert.strictEqual(bundled.length, 0, 'bundledDependencies must be empty');
  }
});

t('engines.node declares the supported floor', () => {
  assert.ok(pkg.engines && typeof pkg.engines.node === 'string', 'engines.node must be declared');
  assert.match(pkg.engines.node, /\d+/, 'engines.node must name a version');
  const floor = parseInt(pkg.engines.node.replace(/[^\d]*(\d+).*/, '$1'), 10);
  assert.ok(floor >= 16, 'engines.node floor must be >= 16, got ' + pkg.engines.node);
});

// ── the bin entrypoint ───────────────────────────────────────────────────────

t('bin declares omega-os and its target file EXISTS', () => {
  assert.ok(pkg.bin && typeof pkg.bin === 'object', 'bin must be an object');
  assert.ok(pkg.bin['omega-os'], 'bin must declare the omega-os command');
  for (const [cmd, rel] of Object.entries(pkg.bin)) {
    const abs = path.join(PKG_DIR, rel);
    assert.ok(fs.existsSync(abs), 'bin.' + cmd + ' points at a missing file: ' + rel);
    assert.ok(fs.statSync(abs).isFile(), 'bin.' + cmd + ' is not a file: ' + rel);
  }
});

t('the bin target is executable', () => {
  for (const rel of Object.values(pkg.bin)) {
    const mode = fs.statSync(path.join(PKG_DIR, rel)).mode;
    assert.ok((mode & 0o111) !== 0,
      rel + ' is not executable (mode ' + (mode & 0o777).toString(8) + ') — chmod +x it');
  }
});

t('the bin target carries a node shebang', () => {
  // npm creates the PATH shim from the shebang on POSIX. Lose it and the
  // installed `omega-os` command runs as a shell script and dies immediately.
  for (const rel of Object.values(pkg.bin)) {
    const first = fs.readFileSync(path.join(PKG_DIR, rel), 'utf8').split('\n')[0];
    assert.match(first, /^#!\/usr\/bin\/env node/, rel + ' lost its node shebang, got: ' + first);
  }
});

t('the files allowlist actually covers everything bin needs', () => {
  // This is the test that stops a packed tarball shipping without its own
  // entrypoint: `files` is an allowlist, so a bin/ path outside it is silently
  // dropped at publish time and every `npx omega-os` fails on a missing file.
  assert.ok(Array.isArray(pkg.files) && pkg.files.length > 0, 'files allowlist must be present');
  const covered = (target) => {
    const tgt = target.replace(/^\.\//, '');
    return pkg.files.some((f) => {
      const entry = f.replace(/^\.\//, '').replace(/\/+$/, '');
      return tgt === entry || tgt.startsWith(entry + '/');
    });
  };
  for (const [cmd, rel] of Object.entries(pkg.bin)) {
    assert.ok(covered(rel),
      'files allowlist does not cover bin.' + cmd + ' (' + rel + ') — the tarball would ship without its entrypoint');
  }
});

t('every files allowlist entry exists on disk', () => {
  // A dead allowlist entry is a promise the tarball cannot keep.
  for (const f of pkg.files) {
    const rel = f.replace(/^\.\//, '').replace(/\/+$/, '');
    assert.ok(fs.existsSync(path.join(PKG_DIR, rel)),
      'files entry "' + f + '" does not exist in the package directory');
  }
});

// ── bin/omega-os.js source ───────────────────────────────────────────────────

t('bin/omega-os.js parses (node --check)', () => {
  const r = spawnSync(process.execPath, ['--check', BIN_PATH], { encoding: 'utf8' });
  assert.strictEqual(r.status, 0,
    'node --check failed:\n' + ((r.stderr || '') + (r.stdout || '')).trim());
});

t('bin/omega-os.js requires ONLY Node builtins', () => {
  // package.json declaring no dependencies is not proof: the code could still
  // require one and blow up on a user's machine. Check the source itself.
  const builtins = new Set(require('node:module').builtinModules);
  const found = [];
  const re = /\brequire\(\s*['"]([^'"]+)['"]\s*\)/g;
  let m;
  while ((m = re.exec(binSrc)) !== null) found.push(m[1]);
  assert.ok(found.length > 0, 'expected the installer to require at least one module');
  for (const spec of found) {
    if (spec.startsWith('.') || spec.startsWith('/')) continue; // local file
    const bare = spec.replace(/^node:/, '');
    assert.ok(builtins.has(bare),
      'bin/omega-os.js requires the non-builtin module "' + spec + '" — the installer must stay dependency-free');
  }
});

t('requiring bin/omega-os.js does NOT run the installer', () => {
  // Guarded by `if (require.main === module)` at bin/omega-os.js:1013. If that
  // guard is ever removed, requiring the file here would clone a repo and run
  // install.sh — so this test both documents and protects the guard.
  assert.match(binSrc, /require\.main === module/,
    'the main guard is gone — importing the installer would execute it');
  const mod = require(BIN_PATH);
  assert.strictEqual(typeof mod, 'object', 'the guarded import must export an object');
  for (const fn of ['tgApi', 'telegramWizard', 'configureTelegram', 'finishTelegram', 'printFailure']) {
    assert.strictEqual(typeof mod[fn], 'function', 'expected exported function: ' + fn);
  }
});

t('the clone target is the public OmegaOS repo over https', () => {
  // A typo or a fork here silently redirects every `npx omega-os` user.
  const m = binSrc.match(/const REPO = '([^']+)'/);
  assert.ok(m, 'the REPO constant is gone from bin/omega-os.js');
  assert.strictEqual(m[1], 'https://github.com/agentik-os/OmegaOS.git',
    'REPO points somewhere unexpected: ' + m[1]);
});

t('every flag documented in --help is actually parsed', () => {
  // Documenting a flag the argument parser never reads is a lie the user only
  // discovers when the flag silently does nothing.
  const help = binSrc.slice(binSrc.indexOf("Usage: npx omega-os"));
  const documented = new Set((help.slice(0, 900).match(/--[a-z][a-z-]+/g) || []));
  assert.ok(documented.size >= 3, 'expected the help text to document several flags');
  for (const flag of documented) {
    const parsed = binSrc.includes("args.includes('" + flag + "')")
      || binSrc.includes("args.indexOf('" + flag + "')");
    assert.ok(parsed, flag + ' is documented in --help but never parsed from argv');
  }
});

t('README documents every flag the installer parses', () => {
  const readme = fs.readFileSync(README_PATH, 'utf8');
  const parsed = new Set();
  const re = /args\.(?:includes|indexOf)\(\s*'(--[a-z][a-z-]+)'\s*\)/g;
  let m;
  while ((m = re.exec(binSrc)) !== null) parsed.add(m[1]);
  assert.ok(parsed.size >= 3, 'expected the installer to parse several flags, found ' + parsed.size);
  for (const flag of parsed) {
    if (flag === '--help') continue; // -h/--help is self-evident and shown in the usage line
    assert.ok(readme.includes(flag), 'README.md never mentions the supported flag ' + flag);
  }
});

// ── cross-file parity with install.sh ────────────────────────────────────────
// The animated + plain progress bars advance by matching STEPS[].re against
// install.sh's own phase banners. Rename a phase in install.sh and the bar
// silently freezes for the whole install, with nothing to see. This is the one
// place the npm package depends on the shell installer, so it is the one place
// that dependency gets a test.
if (fs.existsSync(INSTALL_SH)) {
  const installSrc = fs.readFileSync(INSTALL_SH, 'utf8');
  // The phase banners the installer prints, as `step "Phase N: …"`.
  const banners = (installSrc.match(/^\s*step\s+"([^"]+)"/gm) || [])
    .map((l) => l.replace(/^\s*step\s+"/, '').replace(/"$/, ''));

  t('install.sh still prints phase banners', () => {
    assert.ok(banners.length >= 5,
      'expected install.sh to print several `step "Phase …"` banners, found ' + banners.length);
  });

  t('every STEPS regex still matches a real install.sh phase banner', () => {
    // Pull the literal regex sources out of the STEPS table rather than
    // requiring the module's internals, so this stays a source-level contract.
    const stepsBlock = binSrc.slice(binSrc.indexOf('const STEPS = ['), binSrc.indexOf('];', binSrc.indexOf('const STEPS = [')));
    const patterns = (stepsBlock.match(/re:\s*\/([^/]+)\/i/g) || [])
      .map((s) => s.replace(/^re:\s*\//, '').replace(/\/i$/, ''));
    assert.ok(patterns.length >= 5, 'expected several phase regexes in STEPS, found ' + patterns.length);
    for (const p of patterns) {
      const re = new RegExp(p, 'i');
      assert.ok(banners.some((b) => re.test(b)),
        'STEPS regex /' + p + '/i matches no install.sh phase banner — the progress bar would freeze there');
    }
  });
} else {
  skip('install.sh phase-marker parity',
    'install.sh not found at ' + INSTALL_SH + ' (running outside a repo checkout)');
}

// ── dispatch ─────────────────────────────────────────────────────────────────

for (const line of skips) process.stdout.write('# ' + line + '\n');

let nativeTest = null;
try {
  // Present on Node >= 18 and backported to >= 16.17. engines.node is >=16,
  // so an older 16.x is legal and must still be able to run this suite.
  nativeTest = require('node:test').test;
} catch (e) {
  nativeTest = null;
}

if (nativeTest) {
  for (const [name, fn] of cases) nativeTest(name, fn);
} else {
  process.stdout.write('# node:test unavailable on ' + process.version + ' — using the built-in fallback harness\n');
  let failed = 0;
  let n = 0;
  process.stdout.write('1..' + cases.length + '\n');
  for (const [name, fn] of cases) {
    n++;
    try {
      fn();
      process.stdout.write('ok ' + n + ' - ' + name + '\n');
    } catch (err) {
      failed++;
      process.stdout.write('not ok ' + n + ' - ' + name + '\n');
      process.stdout.write('  ' + String((err && err.message) || err).split('\n').join('\n  ') + '\n');
    }
  }
  process.stdout.write('# pass ' + (cases.length - failed) + '\n# fail ' + failed + '\n');
  if (failed > 0) process.exit(1);
}

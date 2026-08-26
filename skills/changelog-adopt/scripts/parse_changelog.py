#!/usr/bin/env python3
"""Parse the official Claude Code CHANGELOG.md and emit the entries NEWER than a
watermark version, as JSON.

Input : raw CHANGELOG.md on stdin (GitHub `anthropics/claude-code`).
Argv  : [last_version] [seed_versions]
  last_version  — the forward watermark; only versions strictly greater are new.
                  Empty/"-" ⇒ FIRST run: emit only the newest `seed_versions`
                  versions (default 1) instead of dumping the whole history.
  seed_versions — how many latest versions the first run treats as new (default 1).

Output (stdout JSON):
  {"latest": "2.1.202",
   "new_entries": [{"version": "2.1.202", "entry": "Added ...", "fingerprint": "ab12cd34"}, ...]}

Version headers look like `## 2.1.202`. Entries are the `- ` bullets under a header
(continuation lines folded in). A blank last_version is the seed case.
"""
import sys, re, json, hashlib


def parse_version(s):
    """'2.1.202' -> (2,1,202). Non-numeric segments sort as -1 so they never
    outrank a real release. Returns a tuple usable for ordering."""
    out = []
    for part in s.strip().split("."):
        m = re.match(r"(\d+)", part)
        out.append(int(m.group(1)) if m else -1)
    return tuple(out)


def cmp_versions(a, b):
    pa, pb = parse_version(a), parse_version(b)
    # pad to equal length
    n = max(len(pa), len(pb))
    pa = pa + (0,) * (n - len(pa))
    pb = pb + (0,) * (n - len(pb))
    return (pa > pb) - (pa < pb)


def fingerprint(version, entry):
    h = hashlib.sha1(f"{version}|{entry}".encode("utf-8")).hexdigest()
    return h[:8]


def main():
    last = sys.argv[1] if len(sys.argv) > 1 else ""
    if last in ("-", "none", "None"):
        last = ""
    seed_n = int(sys.argv[2]) if len(sys.argv) > 2 else 1

    text = sys.stdin.read()
    lines = text.splitlines()

    # Collect versions in file order (changelogs list newest first).
    versions = []          # [(version, [entry, ...])]
    cur_ver = None
    cur_entries = None
    cur_bullet = None

    def flush_bullet():
        nonlocal cur_bullet
        if cur_bullet is not None and cur_entries is not None:
            e = cur_bullet.strip()
            if e:
                cur_entries.append(e)
        cur_bullet = None

    ver_re = re.compile(r"^##+\s+v?(\d+(?:\.\d+)+)\s*$")
    for ln in lines:
        m = ver_re.match(ln.strip())
        if m:
            flush_bullet()
            cur_ver = m.group(1)
            cur_entries = []
            versions.append((cur_ver, cur_entries))
            continue
        if cur_entries is None:
            continue
        if re.match(r"^\s*[-*]\s+", ln):
            flush_bullet()
            cur_bullet = re.sub(r"^\s*[-*]\s+", "", ln)
        elif cur_bullet is not None and ln.strip():
            cur_bullet += " " + ln.strip()
        elif not ln.strip():
            flush_bullet()
    flush_bullet()

    if not versions:
        print(json.dumps({"latest": last or "", "new_entries": []}))
        return

    latest = versions[0][0]

    # Which versions are "new"?
    if not last:
        new_versions = [v for v, _ in versions[:seed_n]]
    else:
        new_versions = [v for v, _ in versions if cmp_versions(v, last) > 0]

    new_set = set(new_versions)
    new_entries = []
    for v, entries in versions:
        if v not in new_set:
            continue
        for e in entries:
            new_entries.append(
                {"version": v, "entry": e, "fingerprint": fingerprint(v, e)}
            )

    print(json.dumps({"latest": latest, "new_entries": new_entries}, ensure_ascii=False))


if __name__ == "__main__":
    main()

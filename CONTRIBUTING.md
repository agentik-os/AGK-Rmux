# Contributing to OmegaOS

Thanks for looking. OmegaOS is young (0.1.x — see [CHANGELOG.md](CHANGELOG.md)) and the surface moves, so a quick issue before a big PR saves everyone time.

## Build it

You need a Linux box and a Rust toolchain. The installer builds `rmux` and `omega` from source:

```
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

For day-to-day work you don't need the full installer. Build and test the workspace directly:

```
cargo build --release
cargo test
```

`cargo test` also compiles the examples, so it catches a stale example caller before CI does.

## The layout

Three crates:

- `omega-core` — orchestration, the rules registry, doctor, timeline, cleanup, patrol, file-scope locking. Most logic lives here.
- `omega-cli` — the `omega` binary (clap).
- `omega-tui` — the session manager (ratatui).

The doctrine (the 6 Laws and the named operational Rules — `omega rules list` prints the current set) is a typed registry in `crates/omega-core/src/rules.rs`. If you touch how agents are dispatched, that file and the funnel (`rules::agent_context_block`) are where the rules get injected. Read the "How a mission runs" section of the README first.

## Before you open a PR

- `cargo build --release` is clean. CI builds with `-D warnings`, so a warning fails the build.
- `cargo test` passes.
- If you added an installed asset (an agent, a command, a config, a cron, a template), update `install.sh` so a fresh `git clone && ./install.sh` reproduces it, then run `./scripts/verify-install.sh`. A feature that a fresh install wouldn't get is not done.
- No secrets in the tree. Tokens and keys live under `~/.omega/`, which is gitignored.
- A `feat:` commit ships with a `CHANGELOG.md` entry (under `## [Unreleased]`). A feature nobody can find in the changelog gets lost at release time.
- No emoji in terminal output. The TUI and CLI use ASCII glyphs (`[+]`, `[!]`, `[x]`); Telegram messages are the one place emoji are fine.

`clippy` and `rustfmt` aren't clean across the whole tree yet, so CI runs them as advisory rather than blocking. Don't add new clippy warnings, and if you're touching a file that's already rustfmt-clean, keep it that way.

## Commits

Write commit messages in English, present tense, explaining the why. Small, focused commits beat one big one. If two changes touch different files, they can be different commits.

## Questions

Open an issue. For anything security-related, see [SECURITY.md](SECURITY.md) instead of a public issue.

## What this changes


## Why


## Checklist
- [ ] `cargo build --release` is clean (CI builds with `-D warnings`, so a warning fails it)
- [ ] `cargo test` passes
- [ ] No new clippy warnings
- [ ] If I added an installed asset (agent, command, config, cron, template), `install.sh` ships it and `./scripts/verify-install.sh` passes
- [ ] No secrets in the diff, and no emoji in terminal output

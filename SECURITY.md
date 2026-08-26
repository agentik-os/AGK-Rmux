# Security Policy

## Reporting a vulnerability

Don't open a public issue for a security problem. Use GitHub's private reporting instead: go to the repository's **Security** tab and choose **Report a vulnerability** (Private Vulnerability Reporting). That opens a private thread with the maintainers.

Tell us what you found, how to reproduce it, and what an attacker could do with it. A proof of concept helps. We'll confirm we received it and keep you posted while we work on a fix.

## What counts

OmegaOS runs AI agents with real shell access on a machine you control, and it stores credentials under `~/.omega/`. The things we care most about:

- Credentials (`~/.omega/`, provisioning groups) leaking into logs, the repo, or another project's scope.
- An agent or worker escaping its declared file scope, or one session reading another's secrets.
- The Telegram bridge accepting commands from someone outside the configured chat or sender allow-list.
- Command injection through a dispatched mission, a session name, or a config value.

## What doesn't

The agent runtime executes code on purpose. Running a worker that modifies files in its working directory, or an oracle dispatching shell commands, is the tool doing its job, not a vulnerability. Authorized security testing (the `/hack` pipeline) is a feature.

## Supported versions

OmegaOS is pre-1.0. Only the latest commit on `main` is supported. There are no backports yet.

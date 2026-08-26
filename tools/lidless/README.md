# lidless — OmegaOS tool

**Lidless** is a macOS menu-bar toggle for lid-close sleep. One click decides
whether the Mac sleeps when you shut the screen, so a running agent keeps
working while the machine is closed and locked.

- **Upstream:** https://github.com/agentik-os/Lidless
- **Pinned commit:** `162e00132c34a87f74f49fe6ab8c40c93ad1ae1f`
- **License:** MIT
- **Platform:** macOS only (Apple Silicon or Intel), macOS 13+
- **Author:** first-party (Agentik OS), not a third-party vendor

## Why it exists

The operator dispatches an oracle or a worker, then has to leave: a meeting, a
coffee, the bathroom. Closing the lid suspends the machine and kills the run.
The underlying setting is `pmset disablesleep`, but flipping it means a terminal
and a sudo password every time, and leaving it permanently on means the Mac
never sleeps and cooks itself in a bag.

Lidless makes it a two-state menu-bar icon with a battery guard.

## Install (OPT-IN, never automatic)

```sh
bash tools/lidless/install-lidless.sh
```

`install.sh` does **not** run this. Two reasons, both deliberate:

1. It writes a sudoers rule and needs an admin password. Nothing in an
   unattended OmegaOS install should escalate privileges on its own.
2. It is macOS-only, so it would be dead weight on the Linux VPS path.

Same boundary as `tools/zernflow`, higgsfield and browser-use: the repo ships
the markdown plus the installer, the actual install stays an explicit act.

## What it does to the machine

| Change | Where | Reversible |
| --- | --- | --- |
| One sudoers rule | `/etc/sudoers.d/lidless` (`root:wheel`, `0440`) | `bash uninstall.sh` |
| The app | `/Applications/Lidless.app` | `bash uninstall.sh` |
| Login item | SMAppService, toggled in the menu | in the menu |

The sudoers rule grants exactly two commands and nothing else:

```
/usr/bin/pmset -a disablesleep 0
/usr/bin/pmset -a disablesleep 1
```

Verified on macOS 26.2: any other command, including other `pmset` arguments,
still demands a password. The file is validated with `visudo -c` before it is
installed, so a malformed rule can never lock sudo out.

## Design notes

- **State is never cached.** The app re-reads `pmset -g` on launch, on every
  menu open, after every action and on wake. A change made in the terminal
  shows up in the menu bar (L1: runtime is the only truth).
- **No privileged helper.** A signed SMAppService daemon was the alternative;
  the narrow sudoers rule reaches the same result with a fraction of the code
  and an obvious uninstall path.
- **Screen lock via `SACLockScreenImmediate`**, dlopen'd from login.framework.
  `CGSession -suspend` no longer exists on current macOS.
- **Battery guard.** Below 20% on battery with the block on, the app re-arms
  normal sleep and notifies. It is the only safety net: there is no timer.

## Operator caveat

Lid closed with no external display, the Mac runs without ventilating. It gets
hot. Never in a bag in that state. This is the same caveat recorded for the
Tailscale SSH setup, where `disablesleep` used to be left on permanently —
Lidless replaces that standing state with an on-demand one.

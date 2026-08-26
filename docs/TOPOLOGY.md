# Topology — logical profiles, modular isolation

We ship an opinionated default, not a locked architecture.

**Product model (stable):** Operator / Agentik / Mission / Private are
logical profiles. They are not Linux users.

**Infrastructure (chosen at install):** single-user, multi-user, container,
remote/VPS, or hybrid. Same monorepo. Same APIs. Same UI.

```
profile_id = mission
     ↓
Topology Manager
     ↓
single-user:  $HOME/workspace/mission/…
multi-user:   /home/mission/workspace/…
container:    /workspace/…
remote:       resolved on the target
```

The project keeps the same `project_id`. Paths are derived.

## Detection

`omega agk topology detect` looks only for Linux users named
`operator`, `agentik`, `mission`, and `private`. Extra Unix accounts
do not flip the mode.

- All four present → candidate `multi-user`
- Otherwise → candidate `single-user`

`~/.omega/topology.yaml` always wins. Detect never migrates. `plan` is
non-destructive.

```
omega agk topology status
omega agk topology detect
omega agk topology plan single-user
omega agk topology plan multi-user
omega agk topology init    # write only if missing
```

A host that already has those four users (the current VPS) is treated as
multi-user **unless** a topology file already says otherwise. Do not
`useradd`, merge homes, or move data.

## Inheritance

Runtime can override at:

Global → Profile → Client → Project → Mission / Run

More specific wins. Sensitive Mission clients may still require a
container or remote VPS even in single-user mode.

## Discord / Desktop / Web

Surfaces route with `profile_id`, never `linux_username`. See
[DISCORD-SURFACE.md](DISCORD-SURFACE.md).

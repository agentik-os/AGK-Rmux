# AGK scopes — four logical profiles, modular isolation

A virgin `./install.sh` prepares the control layer and wires it to
**official Hermes profiles** (the Nous repo / `omega install hermes`).
Do not fork Hermes: `hermes update` refreshes shared code once.

**Layer formula:** OmegaOS presents and controls. Agentik Core organizes
and governs. Hermes thinks and orchestrates. OS define how work should
be done. Agents execute specialized work. RMUX keeps execution alive.
Convex stores structured state and events. Discord / Desktop / Web / TUI
are only surfaces over the same canonical system.

Hermes session ≠ RMUX session ≠ project ≠ mission ≠ task ≠ run ≠ agent ≠ OS.

**OmegaOS must not duplicate Hermes or RMUX**, scrape `~/.hermes` JSON, or
become a second session backend.

Profiles are **logical**. Linux users / containers / remotes are
**deployment metadata**. See [TOPOLOGY.md](TOPOLOGY.md). We ship an
opinionated default, not a locked architecture.

## Default layout (single-user)

```
$HOME/workspace/
├── operator/{infrastructure,security,deployments,monitoring,automation,deposit,docs}
├── agentik/{projects,products,missions,research,content,growth,community,knowledge,artifacts}
├── mission/
│   ├── clients/<name>/{.client,projects,missions,knowledge,artifacts,infrastructure,automation,docs}
│   ├── internal/
│   └── shared/
└── private/{projects,journal,goals,learning,research,knowledge,artifacts}

$HOME/
├── .hermes/                  # official install root — hermes update
│   └── profiles/{operator,agentik,mission,private}
├── .omega/
│   ├── AGK.md
│   ├── topology.yaml         # explicit config wins
│   ├── scopes/<id>/secrets/  # 0700
│   └── state/{active-scope.json,scope.env}
└── .secrets/<id> → symlink
```

On a host that already has Linux users `operator`, `agentik`, `mission`,
and `private`, `omega agk topology init` writes a **multi-user** mapping
instead (`/home/<user>/workspace`, `/home/<user>/.hermes`). Same
`profile_id`. Different physical path. Existing homes are not moved.

Profile roles: **OPERATOR** operates the machine; **AGENTIK** the
organization; **MISSION** other organizations (clients); **PRIVATE** the
self. `agk switch` changes the complete context. Do not create a sample
client under `mission/clients/`.

`omega scope` remains the **file-claim** command (parallel-write locks).
The control layer is `agk` / `omega agk`.

## Flow

```
Surface (Discord / Desktop / Web / TUI)
   ↓
profile_id   (never linux_username)
   ↓
Topology Manager
   ↓
Hermes / RMUX / workspace / secrets
```

```
agk switch mission
agk
omega agk topology status
```

Canonical IDs (not paths): `profile_id`, `client_id`, `project_id`,
`session_id`, `mission_id`, `task_id`, `run_id`, `agent_id`,
`agent_run_id`, `runtime_id`, `artifact_id`.

## Isolation

Official Hermes profiles isolate **Hermes state**. They do **not**
sandbox the host filesystem.

Supported strategies (mixable):

1. Four Linux users — strongest host boundary
2. One user + bubblewrap sibling-hide for Mission and Private
3. Containers / remote VPS per profile or per client
4. Folders/profiles only — discouraged for YOLO mission + private

When isolation is `logical` and `bwrap` is on PATH, Mission/Private
agent panes hide sibling trees. When isolation is `linux-user`, the
Unix account is the boundary and bwrap is not required.

Do not start four Hermes gateways at install. Per-profile messaging:

```
hermes -p mission gateway setup
```

Use a **different** BotFather token than Atlas.

Discord: four bots = four profiles. BOT ≠ AGENT. See
[DISCORD-SURFACE.md](DISCORD-SURFACE.md).

## Commands

```
omega agk topology init|status|detect|plan
omega agk provision     # idempotent — install.sh runs this
omega agk list
omega agk show
agk switch operator|agentik|mission|private
agk                     # cd workspace + omega TUI
omega doctor --fix      # reprovisions missing trees
```

Existing `~/VibeCoding` (or `~/projects`, `~/code`, …) is left alone.
A virgin single-user machine has none of those, so after provision the
default projects root is `~/workspace/operator`.

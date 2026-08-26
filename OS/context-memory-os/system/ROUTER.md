# Intent Router

| Command | Mode | Purpose |
| --- | --- | --- |
| `/memory` | retrieve | Search or inspect memory |
| `/remember` | capture | Propose a memory write |
| `/ingest` | capture | Ingest a file or event |
| `/context` | compile | Compile a purpose-specific context pack |
| `/snapshot` | snapshot | Create a versioned snapshot |
| `/decision-log` | capture | Record a decision and rationale |
| `/contradiction` | resolve | Resolve conflicting records |
| `/memory-audit` | govern | Audit provenance and access |
| `/forget` | forget | Delete or archive authorized memory |
| `/export-memory` | govern | Create a user-readable export |

## Routing priority
1. Safety / legal / privacy boundary
2. Explicit command
3. User intent
4. Data/evidence availability
5. Cheapest reversible action
6. Handoff when another OS owns the next responsibility

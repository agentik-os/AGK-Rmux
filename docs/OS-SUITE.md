# AgentikOS OS suite

The canonical product registry lives in
`crates/omega-core/src/os_products.rs` (`OsProduct::all()`). The TUI **OS** tab
uses that registry for identity, order, group, declared commands, filesystem
path, and readiness evidence. Product directories live under `OS/<slug>/` and
are installed under `~/.omega/os/`.

## Canonical registry

The current registry contains 24 canonical products in four ordered groups.

### Personal

| OS | Canonical slug |
|---|---|
| Mindset OS | `mindset-os` |
| Health & Energy OS | `health-energy-os` |
| Habit Tracker OS | `habit-tracker-os` |
| Alignment OS | `alignment-os` |

### Build chain

| Step | OS | Canonical slug |
|---|---|---|
| 01 | Strategy & Portfolio OS | `strategy-portfolio-os` |
| 02 | Brainstorm OS | `brainstorm-os` |
| 03 | Market Research OS | `market-research-os` |
| 04 | Blueprint OS | `blueprint-os` |
| 05 | Design OS | `design-os` |
| 06 | Stepper OS | `stepper-os` |
| 07 | Builder OS | `builder-os` |
| 08 | Quality / Evaluation / Release OS | `quality-evaluation-release-os` |

### Growth

| OS | Canonical slug |
|---|---|
| Storyteller OS | `storyteller-os` |
| Revenue OS | `revenue-os` |
| Delivery & Customer Success OS | `delivery-customer-success-os` |
| Relationship & Network OS | `relationship-network-os` |
| Wealth & Capital OS | `wealth-capital-os` |

### Systems

| OS | Canonical slug |
|---|---|
| Execution OS | `execution-os` |
| Operations & Automation OS | `operations-automation-os` |
| Review & Governance OS | `review-governance-os` |
| Context & Memory OS | `context-memory-os` |
| AI Logic OS | `ai-logic-os` |
| Content OS | `content-os` |
| Books OS | `books-os` |

## Compatibility aliases

Five older slugs still resolve to canonical products, but new references and
state must use the canonical slug:

| Legacy slug | Canonical slug |
|---|---|
| `ideation-os` | `brainstorm-os` |
| `researcher-os` | `market-research-os` |
| `designer-os` | `design-os` |
| `habits-os` | `habit-tracker-os` |
| `storytelling-os` | `storyteller-os` |

Aliases do not add products to the registry. Legacy directories can remain on
disk for compatibility, which is why counting `OS/*/` is not a valid product
count.

## Readiness means filesystem evidence

`os_products::dir_status` reports one of four static evidence levels:

- **Scaffold:** a directory with a usable `MASTER.md` but no richer declared
  surface.
- **Reference:** documentation or pack material is present.
- **Runnable:** a declared command or runnable surface is present.
- **Testable:** runnable evidence plus tests or explicit verification assets is
  present.

These labels are intentionally fail-closed. Static presence is never reported
as a successful runtime test, deployment, or production verification. Inspect
the selected product in the TUI and execute its own documented smoke test before
calling it operational.

## Expected product layout

Products vary, but a mature product can provide:

```text
OS/<slug>/
├── README.md                 product-specific runbook and limitations
├── MASTER.md                 agent prompt launched by the TUI
├── pack/                     operator-supplied reference material
├── engine/ or runtime/       implementation, when applicable
├── bin/                      command wrappers, when applicable
├── commands/                 provider prompt surfaces, when applicable
├── tests/                    executable verification, when applicable
└── ledger/                   user-local runtime state (never committed)
```

The registry declares product capabilities; it does not require every product
to have every directory. Do not create placeholder files merely to raise a
readiness label.

## Adding or completing a product

1. Read its current README, prompt, pack, and executable surfaces.
2. If adding a product, add one canonical identity to `OsProduct::all()` in the
   correct group. Add an alias only for a real backwards-compatibility need.
3. Implement the smallest coherent runtime and preserve operator-supplied pack
   material verbatim.
4. Add provider surfaces only when they execute the same product contract.
5. Add tests that exercise the real entrypoint and a negative or recovery path.
6. Keep installer parity so a fresh source install reproduces the product.
7. Verify the product runtime, then inspect the TUI readiness evidence. The TUI
   label alone is not acceptance.
8. Update this registry reference only when the canonical roster or contract
   changes.

Before release, run the product-specific tests, the workspace gates, and the
installer verification required by [RELEASE.md](RELEASE.md).

# Researcher OS

AgentikOS build chain **#02** - **integrated** (Market Research {OS} v1.0.0).

The market-evidence and validation compiler placed before Blueprint: sources,
rights preflights, hypotheses, methods, samples, observations, auditable
models, falsifiable experiments, risks, traceability, gates, a BOUNDED
recommendation (GO / PIVOT / HOLD / NO-GO / INSUFFICIENT EVIDENCE), and a
frozen Blueprint input manifest. Payload source:
`Market-Research-OS-Complete-v1.0.0.zip` (Deposit, 2026-08-10).

Chain: `01 Ideation -> 02 Researcher -> 03 Blueprint -> 04 Designer ->
05 Stepper -> 06 Builder`. Downstream, Blueprint OS consumes the frozen
Blueprint input manifest as a source of authority.

## Layout

| Path | What |
|---|---|
| `pack/` | The zip verbatim: START_HERE, the one-file COMPLETE_MANUAL, VALIDATION_REPORT, and `market-research-os/` (the modular skill: SKILL.md, 13 references, 20 assets incl. templates + schemas, 3 scripts, agents/openai.yaml) |
| `MASTER.md` | The master agent (TUI Enter + Telegram bot brain) |
| `bin/omega-research` | The OmegaOS CLI - the pack's deterministic workspace engine (stdlib Python, no venv): init / validate / status / allocate / checkpoint / score / export / demo |
| `commands/codex-researcher-os.md` | The OpenAI/Codex command (installed to `~/.codex/prompts/researcher-os.md`) |

The Claude command is the `market-research-os` skill (the pack's modular
skill vendored verbatim at `skills/market-research-os/`), installed as
`/market-research-os`, `/omg-market-research-os` and the `/research` alias.
(The bare `/market-research` command stays the marketing suite's gooseworks
data-API skill — a source lane inside this OS, never overwritten.)

## Run it

```bash
omega-research demo /tmp/mr-demo          # create + validate a demo workspace
omega-research init ./research-state --project-id my-idea \
  --project-name "My Idea" --decision "Proceed to Blueprint?" \
  --mode FULL_VALIDATION --depth VALIDATION
omega-research validate ./research-state
omega-research status   ./research-state
omega-research score    ./research-state  # gate + hypothesis diagnostics
```

The reasoning half runs in an agent: `/market-research <idea>` (scan /
validate / diligence / deep / audit / delta / continue / status / score /
handoff) in Claude, the Codex prompt, or the OS master agent (TUI OS tab ->
Enter, Telegram bot via `T`).

## Hard rules

- Depths SIGNAL / VALIDATION / INVESTMENT_GRADE - desk research alone never
  claims full validation; GO and PIVOT are always bounded with kill criteria
  and expiry.
- Scraping boundary: mandatory source preflight; technical access never
  grants permission; no bypass of auth, paywalls, CAPTCHAs, rate limits.
- Distinct from the marketing suite's `/omg-market-research` (gooseworks):
  that skill is a usable SOURCE LANE inside this OS, not a replacement for
  its evidence contract.

## v1 scope vs pack spec (honest divergences)

Same posture as Blueprint v3 / Builder v1: the single-runtime profile. The
15-role orchestrated graph runs sequentially in one agent (or via the
OmegaOS Workflow primitive); `assets/market-research-tools.json` is honored
as the contract kept via the CLI, not a typed dispatch server; external
scraping adapters (Apify, Firecrawl, ...) stay opt-in behind the source
preflight and the operator's own keys.

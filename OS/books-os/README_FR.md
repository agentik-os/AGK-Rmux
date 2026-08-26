# Books OS

Books OS transforme une bibliothèque en système opératoire pour comprendre, retenir, relier, contester et appliquer les idées. Il expose le moteur canonique Agentik Book OS, ALEXANDRIA, sans créer une seconde personnalité concurrente.

## Sources canoniques

| Élément | Source |
| --- | --- |
| Persona complète | `agents/librarian.md`, installée dans `~/.omega/agents/librarian.md` |
| Skill Claude | `skills/alexandria/`, accessible avec `/alexandria` et `/books-os` |
| Agent Telegram | `telegram-bot/omega-tg-bot.ts`, type `persona` |
| Surface OS | `MASTER.md`, `bin/omega-books` et `commands/` |

## Modes principaux

`/book`, `/espresso`, `/chapter`, `/idea`, `/compare`, `/apply`, `/challenge`, `/decision`, `/council`, `/teach`, `/quiz`, `/cards`, `/map`, `/memory`, `/review`, `/capture`, `/readingpath`, `/audio`, `/focus`, `/masterclass`, `/best`, `/bestsellers` et `/gem`.

## Surfaces

1. Claude et Codex utilisent le skill canonique `alexandria`.
2. `omega-books` ouvre le master agent dans une session terminale.
3. Telegram peut lier un bot dédié qui utilise la même persona.
4. Le panneau OS ouvre la même source, sans duplication.

## Confidentialité

Le profil, les captures et le journal d'application restent dans `~/.omega/os/books-os/ledger/`. Les corpus privés ne sont jamais intégrés au dépôt public.

## Intégration

Books OS peut transmettre des notes confirmées à Context & Memory OS. Le contrat d'événements est décrit dans `OMEGA_INTEGRATION.md` et `MANIFEST.json`.

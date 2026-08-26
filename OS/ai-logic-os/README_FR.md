# AI Logic OS

Doctrine d'arbitrage transversale pour OmegaOS : une etape releve-t-elle du code deterministe ou d'un vrai jugement IA, environ 80 pour cent de code et 20 pour cent de jugement IA, biais par defaut NON. Elle termine chaque consultation par une decision argumentee plus une specification executable, jamais par une liste d'idees.

## Position dans la suite

Core Stack (transversal) : une couche de conseil, pas une etape figee d'un pipeline. N'importe quel OS peut l'invoquer ponctuellement pour remettre en cause sa propre automatisation, un agent, un skill ou un pipeline LLM. Operations & Automation OS est le passage de relais principal et permanent (l'arbitrage tourne avant qu'un candidat d'automatisation soit score et approuve) ; Review & Governance OS peut aussi l'invoquer lorsqu'il audite une automatisation ou un changement de systeme agentique.

## Ce que contient cet OS

C'est un pack mince : deux fichiers de reference de doctrine et rien d'autre. Il n'y a pas de repertoire `agents/` ni de repertoire `scripts/` ici ; le pack fonctionne uniquement via le contrat operatoire `SKILL.md` plus ces deux references, chargees dans l'ordre.

- `references/workflow-optimizer.md` : la doctrine d'arbitrage centrale (celle de l'operateur, verbatim). La regle 80/20 code contre jugement, le triage en quatre bacs (Codifier / Augmenter / Garder humain / Supprimer), le score de priorite, la boucle de travail, les questions a toujours poser, les coups interdits et le format de sortie. C'est la colonne vertebrale.
- `references/system-challenger.md` : l'extension qui retourne le meme arbitrage contre un systeme agentique (OmegaOS lui-meme, un agent, un skill, un pipeline LLM, un outil de code, un cas d'usage IA). Les cinq questions de challenge, le triage applique aux agents, la veille sur les outils et les cas d'usage, et comment auditer OmegaOS contre ses propres Lois et Regles.

## Commandes

- `/ai-logic` (racine)
- `/ailogic` (alias)

## Passages de relais principaux

- Produit `ailogic.arbitration.decided`, consomme par Operations & Automation OS comme entree d'arbitrage avant `automation.candidate.scored`.
- Consomme `automation.candidate.scored` depuis Operations & Automation OS lors de l'arbitrage d'un candidat d'automatisation existant.

## Declencheurs

- audit de workflow
- triage d'automatisation
- arbitrage code contre IA
- revue de systeme agentique
- "qu'est-ce qu'on devrait automatiser"
- "challenge cet agent / ce skill / ce pipeline"
- AI Logic OS, `/ai-logic`, `/ailogic`

### Declencheurs (FR)

- audit de workflow
- qu'est-ce qu'on devrait automatiser
- challenge ce pipeline / cet agent / ce skill
- arbitrage code vs IA
- triage d'automatisation
- decision de deterministe ou IA

## Pour aller plus loin

Voir `OMEGA_INTEGRATION.md` pour l'enregistrement complet, le schema d'evenements, l'ordre d'injection du contexte et la classification d'etat.

## Installation

Le pack est installé sous `~/.omega/os/ai-logic-os` et `~/.omega/skills/ai-logic-os`. Le manifeste et les checksums vérifient les fichiers distribués.

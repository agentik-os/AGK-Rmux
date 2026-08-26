# Builder OS

Builder OS réalise un produit à partir d'un Blueprint et d'un graphe Stepper validés. Il orchestre les rôles de développement, garde les changements bornés et exige des preuves d'exécution avant livraison.

## Position

```text
Blueprint -> Design -> Stepper -> Builder -> Quality -> Release
```

## Contenu

- `SKILL.md` définit le routeur, les gates et les contrats de sortie.
- `references/` documente le système, la sécurité, la vérification et les handoffs.
- `assets/` fournit les schémas d'état, outils et profils de rôles.
- `scripts/builder_os.py` est le runtime déterministe.
- `pack/` conserve le pack intégré d'origine.
- `bin/omega-builder` expose la CLI locale.

## Commandes

Les interfaces racines sont `/build` et `/builder-os`. Elles lisent l'état, sélectionnent la prochaine étape autorisée, exécutent un changement borné, capturent les preuves et s'arrêtent sur tout gate non satisfait.

## Contrat de qualité

Builder ne transforme jamais une compilation verte en preuve produit. Les critères d'acceptation, tests, journaux et artefacts doivent correspondre à la dernière mutation. Les exceptions sont transmises à Review & Governance, jamais masquées.

## Intégration

Builder consomme les handoffs de Blueprint, Design et Stepper, puis produit des artefacts vérifiables pour Quality. Les événements sont spécifiés dans `OMEGA_INTEGRATION.md` et `MANIFEST.json`.

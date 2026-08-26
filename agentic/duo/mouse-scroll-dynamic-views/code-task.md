# Tâche d'implémentation — accès dual molette (rmux ⇄ app)

Repo : OmegaOS. Fichiers autorisés, et EUX SEULEMENT :
- `config/rmux.conf.omega`
- `scripts/verify-install.sh`

Ne touche à aucun autre fichier. Ne modifie pas le repo rmux. Ne lance pas de build Rust.

## Contexte retenu (ta critique a été acceptée)

Ta conclusion est validée : l'alternate screen suspend l'historique du grid
(`crates/rmux-core/src/screen/writer.rs:292`), donc « molette dans la vue dynamique ET même
conversation dans le scrollback rmux » est structurellement impossible sans un nouveau
sous-système. Le mécanisme retenu est donc TON option 1 : l'accès dual par molette modifiée.

Il a été vérifié au runtime avant de te le confier (rmux 0.3.1, injection SGR sur un pane
`claude` en plein écran, alternate_on=1 mouse_any_flag=1) :

```
rmux bind-key -n S-WheelUpPane copy-mode -e
# injection de \033[<68;20;10M  (64 + 4 = Shift + molette haut)
# => pane_in_mode passe de 0 à 1, pane_mode = copy-mode
```

Donc : molette nue = à l'application (Claude plein écran, Codex transcript via le fallback
flèches), molette + modificateur = reprise de la main par rmux (copy-mode, recherche, copie).

## Ce qu'il faut écrire

### 1. `config/rmux.conf.omega`

Juste APRÈS le bloc existant « Alternate-scroll fallback » (les deux `bind-key -n WheelUpPane`
/ `WheelDownPane`), ajoute un bloc commenté dans le MÊME style que le reste du fichier
(commentaires denses qui expliquent le pourquoi et le piège, pas seulement le quoi) qui :

- explique que quand une app capture la souris (Claude en renderer plein écran demande
  1000+1006), rmux ne voit plus la molette : plus de copy-mode, plus de drag-select ;
- explique que l'alternate screen suspend l'historique du grid rmux, donc copy-mode sous
  altscreen ne montre que l'écran alternatif, PAS la conversation — c'est une reprise de la
  main sur la vue courante, pas un scrollback de la conversation ;
- pose Shift+molette ET Alt+molette (deux modificateurs, car beaucoup d'émulateurs
  interceptent Shift+molette pour leur propre scroll et ne l'émettent jamais) :

```
bind-key -n S-WheelUpPane copy-mode -e
bind-key -n M-WheelUpPane copy-mode -e
```

- note que la descente n'a pas besoin de binding modifié : une fois en copy-mode, la table
  `copy-mode` gère déjà `WheelUpPane` / `WheelDownPane` (`send -N5 -X scroll-up/-down`).

### 2. Clarifier le fallback de `WheelDownPane` (angle mort que TU as levé)

Vérifié au runtime : sur un pane shell normal (alternate_on=0, pas de mouse reporting), la
molette vers le bas ne fait rien — mais c'était DÉJÀ le comportement avant le changement, car
rmux ne définit aucun `WheelDownPane` dans la table root par défaut
(`crates/rmux-core/src/keys/defaults.rs:204` ne définit que `WheelUpPane`). Ce n'est donc pas
une régression, mais le commentaire du fichier doit le dire explicitement, sinon le prochain
lecteur croira à un oubli. Ajoute cette précision dans le bloc de commentaire existant.

### 3. `scripts/verify-install.sh`

Étends le contrôle existant « rmux alternate-scroll fallback bound » (ou ajoute-en un juste
après) pour vérifier aussi la présence des deux bindings modifiés `S-WheelUpPane` et
`M-WheelUpPane`. Même style que les autres contrôles (`ok` / `bad`).

## Critères de succès (je les rejouerai moi-même)

1. `bash scripts/verify-install.sh` : les contrôles rmux passent au vert.
2. `rmux source-file /home/vibe/.rmux.conf` ne produit AUCUNE erreur et
   `rmux list-keys -T root | grep -iE "WheelUpPane|WheelDownPane"` liste les 4 bindings
   (nu haut, nu bas, S-haut, M-haut). Rappel : une seule ligne rejetée par rmux 0.3.1 avorte
   TOUT le source-file, donc n'invente aucune option non validée.
3. Aucune ligne de commentaire n'utilise de tiret cadratin dans une phrase marketing ; garde le
   style technique existant du fichier.

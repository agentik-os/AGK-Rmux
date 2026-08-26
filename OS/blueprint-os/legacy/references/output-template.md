# Les livrables

*À lire en phase 4 (flux) et phase 14 (livrables).*

---

## Livrable 1 — `<nom>-blueprint.md`

Quatorze sections, dans cet ordre.

```
1.  Résumé — ligne interne, ligne externe, cible, prix, socle
2.  Le problème — cinq problèmes vécus par l'acheteur
3.  La primitive — le tableau du renversement
4.  Le marché — matrice concurrentielle + colonne « personne ne le fait »
5.  Utilisateurs — trois personas maximum
6.  Objectifs et non-objectifs
7.  Les flux — 3 à 6, déclencheur / étapes / objets / sortie / signaux
8.  Modèle de données — le schéma Convex complet
9.  Features — input / action / output, par module
10. Automatisations — les trois niveaux
11. Couche IA — features + system prompt rédigé de chaque agent
12. Layout Stax — panneaux, inspecteurs, registre d'actions, URL
13. Positionnement — quatre couches, contraste, vocabulaire, prix
14. Release, métriques, risques — v0 → v5, une étoile, hors-scope
```

---

## Livrable 2 — `<nom>-blueprint.html`

**Ce n'est pas un diagramme. C'est un artefact de compréhension navigable.**

Un seul fichier, aucune dépendance, aucun CDN, aucune police distante. Il s'ouvre hors ligne.

### Les trois lois

**1. Chaque affirmation porte sa source.** Format `index:ligne`. Une cellule sans source est une invention.

**2. `[unknown]` est un citoyen de première classe.** Quand une décision n'est pas prise, on écrit `[unknown]` **et ce qui la trancherait**. On n'invente jamais un placement, une valeur ou un comportement.

> Exemple : *`foot_primary` est `[unknown]`. Ce qui le trancherait : une ligne par panneau dans la table des pieds de `docs/05-adaptation.md` section 7.*

**3. Les contradictions s'enregistrent, elles ne se lissent pas.** Quand deux sources disent l'inverse, on cite les deux, on dit laquelle on applique, et on nomme ce qui réglerait le conflit. Un artefact qui masque une divergence la rend irréparable.

Ces trois lois sont ce qui sépare un document de compréhension d'une jolie maquette.

### Le modèle de données

```js
const NODES = {
  "panel:<space>:<name>": {
    k: "panel",              // panel | ds | facet | query | flow | agent
    t: "searcher",           // le titre
    sz: "m",                 // s | m | l | xl | xxl
    what: "M 480, ligne de registre …",   // une ligne, la raison d'être
    s: [                     // les sections
      ["Anatomie", [
        ["Largeur", "M", "4:214"],
        ["Pourquoi", "…", "5:903"]
      ]],
      ["Le corps", [ ["Le corps", "…", "5:927"] ]],
      ["Le pied", [ ["Pied déclaré", "…", "4:214"] ]],
      ["Le drill", [ ["Le drill", "…", "5:958"] ]],
      ["Les six états, plus refusé", [
        ["défaut", "…", "5:967"], ["survol", "…", "5:967"],
        ["focus", "…", "5:967"], ["vide", "…", "5:967"],
        ["chargement", "…", "5:967"], ["erreur", "…", "5:967"],
        ["refusé", "…", "5:967"]
      ]],
      ["Données et gate", [
        ["Liaison Convex", "api.panels.searcher.load", "4:214"],
        ["Capacité", "searcher.read", "4:214"]
      ]],
      ["Signaux émis", [ ["entries", "metric, todo", "—"] ]],
      ["Écrit à", [ ["Bloc layout", "docs/layout/…:901", "5:901"] ]]
    ],
    d: ["panel:<space>:<child>", "query:…"],   // ce qu'il ouvre
    warn: "<b>foot_primary</b> est [unknown]. Ce qui le trancherait : …"
  }
};

const SOURCES = { 4: "docs/nomenclature/…", 5: "docs/layout/…" };
```

**Les sept sections obligatoires** de chaque nœud : Anatomie · Le corps · Le pied · Le drill · Les six états plus refusé · Données et gate · **Signaux émis**.

La dernière est propre à AgentikOS : c'est le contrôle visuel de la doctrine 4.

### Les six états, plus refusé

Chaque nœud les déclare tous les sept. C'est ce qui empêche de livrer une maquette qui ne marche qu'au cas nominal.

| État | Ce qu'on écrit |
|---|---|
| **défaut** | Le rendu normal |
| **survol** | Ce qui change, et rien de plus |
| **focus** | Le rendu clavier **et** le rendu actif — deux rendus, pas un |
| **vide** | Une phrase avec une action suivante. Jamais un cadre vide |
| **chargement** | Squelette **à la géométrie finale**, aucun reflow |
| **erreur** | Une phrase. Ce qui a été écrit ou non |
| **refusé** | **Absent, jamais grisé.** Un élément grisé révèle que la donnée existe |

La dernière ligne est la plus importante et la plus violée. Sur un produit élite, un contrôle grisé divulgue une information.

### La navigation

| | |
|---|---|
| **Stage horizontal** | Les panneaux s'ajoutent à droite, le parent reste |
| **Drill** | Cliquer un lien de `d` ouvre le nœud enfant |
| **Le parent persiste** | On ne perd jamais le contexte |
| **Recherche** | Filtre sur titres et contenus |
| **Thème** | Clair / sombre, `data-theme` prioritaire sur `prefers-color-scheme` |
| **Légende** | Les kinds de nœuds, avec leur pastille |

### Le socle CSS

```css
:root{
  --font-serif:'Instrument Serif',Georgia,serif;
  --font-sans:'IBM Plex Sans',system-ui,sans-serif;
  --font-mono:'IBM Plex Mono',ui-monospace,Menlo,monospace;
  --fz-body:13.5px; --fz-mono:10px;
  --ease:cubic-bezier(.32,.72,0,1);

  --background:#f7f6f4; --card:#fff; --secondary:#efedea; --border:#e0dcd6;
  --foreground:#1a1917; --muted-foreground:#6f6a63;

  /* L'ACCENT EST UNE VARIABLE, jamais un hex de marque en dur.
     Régler la marque = une seule édition ici. */
  --accent:oklch(.46 .13 258);
  --accent-soft:color-mix(in oklch,var(--accent) 10%,transparent);
  --alert:oklch(.55 .20 27);

  --r-card:12px; --r-row:10px; --r-btn:8px;
}
@media (prefers-color-scheme:dark){ :root:not([data-theme]){ /* … */ } }
:root[data-theme="dark"]{ /* … */ }
```

### Les règles typographiques

| Règle | Pourquoi |
|---|---|
| **Serif pour les titres**, 27px, roots à 32 | La hiérarchie de rang se voit sur un stage où le root reste à l'écran |
| **Sans pour le corps**, 13.5px | Lecture |
| **Mono tabulaire pour tout chiffre** | Un nombre en serif ne s'aligne pas. Jamais de chiffre en serif |
| **Mono uppercase pour les labels**, tracking .14em | Le registre technique |
| **Mesure de lecture 54ch** | Élargir le panneau élargit la marge, jamais la ligne |
| **Aucun emoji** | Glyphes éditoriaux uniquement |

### Les règles de couleur

- **L'accent est une variable.** Aucun hex de marque en dur, nulle part
- **Les états sur la rampe d'accent**, jamais vert/orange/rouge
- **Le rouge est réservé** à deux cas nommés — une rupture de seuil et une violation de SLA. Il porte alors un token mono, ce qui le distingue d'une erreur applicative
- **Pas de pastille pleine** pour un état : texte mono nu

### Ce qu'on ne met pas

| | Pourquoi |
|---|---|
| Une bibliothèque de graphiques | Les barres sont des `div` à un pourcentage du maximum de leur propre série |
| Un CDN, une police distante | L'artefact doit s'ouvrir hors ligne |
| Un placement inventé | Si aucune source ne le place, on ne le dessine pas |
| Une valeur devinée | `[unknown]` plus ce qui la trancherait |

---

## La clôture

Après avoir présenté les deux fichiers, terminer par **exactement trois questions ouvertes** — celles que le blueprint n'a pas tranchées et qui changent la suite.

Pas plus de trois. Un blueprint qui se termine par dix questions n'a rien décidé.

---

*Un artefact qui invente une valeur manquante est pire qu'un artefact incomplet : il transforme une question ouverte en fausse certitude.*

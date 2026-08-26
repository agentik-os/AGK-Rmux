# Le stack et la grammaire Stax

*À lire en phase 9 (modèle de données) et phase 10 (layout).*

---

## 1. Le stack, non négociable

| Couche | Techno | Rôle dans l'OS |
|---|---|---|
| **Shell / UI** | **Stax** | La couche interface. Grammaire de panneaux, drill-down, workspace dans l'URL |
| Front | Next.js | App router |
| Données et temps réel | Convex | Le modèle unique. Réactif par défaut |
| Auth | Clerk | Identité, organisations, rôles |
| Paiement | Stripe | Abonnement, one-shot, par produit et par groupe |
| Intégrations | Composio | Calendrier, mail, tiers |
| IA | Claude, appelé depuis une **action Convex** | Jamais depuis le client |

**Stax n'est pas une bibliothèque de composants. C'est le shell de chaque OS.** C'est ce qui rend la famille d'OS cohérente : même grammaire de navigation partout.

Repo : `github.com/agentik-os/stax` · démarrage : `create-stax-app`

---

## 2. La grammaire des panneaux

Le modèle mental : **tout objet ayant de la profondeur s'ouvre en panneau à droite, et le parent reste visible.**

| | |
|---|---|
| **Drill-down** | Cliquer un objet ouvre son inspecteur à droite |
| **Persistance du parent** | On ne perd jamais le contexte d'où l'on vient |
| **Profondeur** | 3 à 5 panneaux côte à côte, sans se perdre |
| **URL = workspace** | L'arrangement complet est encodé. Donc partageable |
| **Registre d'actions** | La couche IA peut ouvrir des panneaux via le bridge |

### Pourquoi c'est le bon modèle pour un OS

Un OS demande de relier des domaines : une journée → une métrique → un projet → une tâche. En navigation classique, chaque clic fait perdre le contexte précédent. En panneaux, **la lecture en travers devient visuelle** au lieu d'être un paragraphe généré.

C'est aussi ce qui permet à un agent de **montrer** une corrélation plutôt que de la raconter : il ouvre les deux panneaux côte à côte, preuve à l'écran.

### Ce que la phase 10 doit produire

| Livrable | Contenu |
|---|---|
| **Carte des panneaux** | Quel type d'objet ouvre quel panneau |
| **Inspecteurs** | Ce qu'on voit en drillant sur chaque type |
| **Registre d'actions** | Ce que la couche IA peut ouvrir, et avec quels paramètres |
| **État d'URL** | Ce qui est encodé, ce qui est partageable, ce qui est filtré |
| **Boards canvas** | S'il y en a : types de nœuds, statuts, couleurs, actions clic droit |

---

## 3. Les boards canvas

Quand l'OS a une dimension spatiale — vision produit, architecture, relations, roadmap.

**Le principe :** un nœud n'est pas un dessin, c'est une entité. Il drille vers l'inspecteur de la vraie donnée.

| | Un canvas classique | Un board Stax |
|---|---|---|
| Le nœud | Un dessin | **Une entité** |
| Il ment | Dès que la réalité bouge | Jamais — il lit la donnée |
| Il se construit | À la main | **Il se génère** |

### Le cycle de vie d'un nœud

Statuts et couleurs, à adapter au domaine :

| Statut | Couleur | Actionnable |
|---|---|---|
| Idée | Gris | Non |
| **Incomplet** | **Rouge** | **Non — bloquant** |
| Prêt | Bleu | Oui |
| En cours | Ambre | — |
| En revue | Violet | — |
| Terminé | Vert | — |

**Le rouge est délibérément agressif.** Un nœud incomplet est un risque, pas une tâche. C'est ce qui empêche de lancer un agent sur du flou et de récupérer 900 lignes inutilisables.

### Les quatre blocs

Un nœud ne devient actionnable que si les quatre sont remplis :

1. **Objectif** — ce que ça doit permettre
2. **Contraintes** — stack, performance, dépendances
3. **Définition du fini** — **mécaniquement vérifiable** : tests, typecheck, lint, taille du diff
4. **Ne pas toucher** — les fichiers et comportements hors périmètre

Le quatrième est celui que tout le monde oublie, et c'est celui qui évite les diffs de 900 lignes.

**Le bloc 3 décide de la voie d'exécution :** vérifiable par une machine → lot autonome. Sinon → un agent à la fois, avec un humain devant.

---

## 4. Le patron de modèle de données

```ts
// convex/schema.ts
export default defineSchema({

  // 1 — LA PRIMITIVE. Toujours la première table.
  <primitive>: defineTable({
    tenantId: v.string(),          // dès le jour 1 si le produit sera vendu
    // … les champs qui définissent l'objet central
  }).index("by_tenant", ["tenantId"]),

  // 2 — LES RELATIONS entre primitives, si le domaine est relationnel
  edges: defineTable({
    tenantId: v.string(),
    from: v.id("<primitive>"),
    to: v.id("<primitive>"),
    kind: v.union(/* les types de lien du domaine */),
    weight: v.number(),
    createdAt: v.number(),
  }).index("by_tenant_from", ["tenantId", "from"]),

  // 3 — LES SIGNAUX. Obligatoire. Doctrine 4.
  entries: defineTable({
    tenantId: v.string(),
    actorId: v.string(),
    date: v.string(),              // YYYY-MM-DD
    kind: v.string(),              // le discriminant du domaine
    payload: v.any(),
    createdAt: v.number(),
  }).index("by_tenant_date", ["tenantId", "date"])
    .index("by_actor_kind", ["actorId", "kind"]),

  // 4 — LES SORTIES IA. Toujours séparées des signaux.
  syntheses: defineTable({
    tenantId: v.string(),
    scope: v.string(),
    observation: v.string(),
    correlation: v.optional(v.string()),   // optionnel : « rien de notable » est valide
    citations: v.array(v.string()),        // doctrine 5 : citer sa donnée
    proposals: v.array(v.string()),
    model: v.string(),
    createdAt: v.number(),
  }).index("by_tenant_scope", ["tenantId", "scope"]),

  // 5 — LES RITUELS. La couche que tout le monde oublie.
  rituals: defineTable({
    tenantId: v.string(),
    name: v.string(),
    cadence: v.string(),           // RRULE
    ownerRotation: v.array(v.string()),
    nextOwnerIndex: v.number(),
  }).index("by_tenant", ["tenantId"]),
});
```

### Les règles

- **La primitive est la première table.** Si on hésite, la phase 2 a échoué
- **`entries` est obligatoire.** C'est ce qui rend la lecture en travers possible
- **`syntheses` est séparée de `entries`.** Ne jamais mélanger l'observé et l'interprété
- **`tenantId` partout dès le premier jour** si le produit sera vendu. Le rétrofit est un enfer
- **Un index par requête réelle.** Pas d'index spéculatif
- **Aucune table qui n'apparaît dans aucun flux de la phase 4**

---

## 5. La couche IA — mise en œuvre

```
Client (Next/Stax)
      │  jamais d'appel modèle direct
      ▼
Action Convex  ──►  Claude  ──►  sortie structurée
      │
      ├──► écrit dans `syntheses`
      └──► optionnel : ouvre des panneaux via le registre d'actions
```

**Règles :**
- Le modèle est appelé depuis une action Convex, jamais depuis le client
- La sortie est structurée et validée avant écriture
- Chaque affirmation porte ses citations
- La sortie vide est un cas nominal, pas une erreur
- Les agents de niveau 3 écrivent une intention, pas un effet — l'exécution attend la confirmation

---

## 6. Checklist avant de livrer le blueprint

- [ ] La primitive est la première table du schéma
- [ ] `entries` existe et chaque module y écrit
- [ ] `syntheses` est distincte de `entries`
- [ ] `tenantId` est partout
- [ ] Chaque agent a un system prompt rédigé, avec sortie vide autorisée
- [ ] Chaque agent a un niveau (1, 2 ou 3) et ~70 % sont niveau 2
- [ ] La carte des panneaux couvre chaque type d'objet du schéma
- [ ] Le registre d'actions liste ce que l'IA peut ouvrir
- [ ] Les rituels sont modélisés, pas seulement décrits
- [ ] La couche commodité est en dernier dans la séquence de release

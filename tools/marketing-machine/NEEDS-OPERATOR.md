# NEEDS-OPERATOR : ce qu'il faut demander à l'opérateur

**À lire en ouvrant la marketing machine, avant de promettre quoi que ce soit.**

Cette liste existe parce qu'un rail bloqué se redécouvre à chaque session, toujours au
pire moment : au milieu d'un run, après avoir annoncé que ça marchait. Elle tient debout
entre les sessions. Elle n'est pas un journal : chaque ligne porte **la commande qui
re-vérifie**, jamais un état affirmé de mémoire.

## Règle d'usage

1. **Re-vérifie avant de demander.** L'opérateur a peut-être déjà payé. Une relance sur
   un rail déjà réglé fait perdre la confiance dans toute la liste.
2. **Un 402 / 403 est un ABORT, jamais un PASS** (L5). On ne contourne pas, on demande.
3. **Ne demande pas tout.** Demande le point 1, il est gratuit et débloque le plus.
4. Quand un point est réglé : coche-le ici avec la date et la preuve runtime.

## Le piège à connaître

`omega marketing doctor` vérifie la **présence des clés**, pas la **santé des rails**.
Il affiche `✓ ZERNIO_API_KEY` et `✓ higgsfield account` alors que Zernio répond 402 et
qu'il reste 0,03 crédit Higgsfield. Une clé valide ne prouve rien.

Pour l'état réel : **`reels doctor`**, qui sonde chaque rail en live et ne verdit jamais
à crédit. Même logique côté Apify : le token reste valide et `/users/me` répond 200 alors
que tous les runs 403, le vrai signal est `effectivePlatformFeatures.ACTORS.disabledReason`.

---

## La liste (dernier constat : 2026-07-26)

### 1. Token Instagram Graph : GRATUIT, priorité absolue

Le seul rail qui donne la **rétention**, la métrique qui décide de la distribution. Aucune
source payante ne la donne. Débloque `reels mine`, `reels ledger`, et le scan des
concurrents via `business_discovery`, donc les briques 1 **et** 4 d'un coup.

- **Vérifier :** `reels doctor` (ligne `instagram graph`)
- **Attendu :** `~/.omega/secrets/instagram.env` avec `IG_USER_ID` et `IG_ACCESS_TOKEN`
- **Procédure :** imprimée par `reels doctor` quand le credential manque
- **Prérequis opérateur :** compte IG en Business ou Creator, lié à une Page Facebook
- **Sans ça :** `reels mine` et `reels ledger` ne tournent pas, la boucle de mesure est morte

### 2. Quel compte Instagram on fait grossir : GRATUIT

Question, pas dépense. On ne devine jamais un compte (R-PROJ : un identifiant qui va à un
projet est faux par défaut pour tous les autres).

- **Vérifier :** `grep NOVA_INSTAGRAM_HANDLE ~/.omega/secrets/nova-accounts.env`, et
  `omega-zernio accounts <projet>` une fois Zernio revenu
- **Sans ça :** on ne sait pas sur quel compte pointer le token du point 1

### 3. Zernio : réactiver l'abonnement

Débloque la publication (brique 3) et sert de socle à ZernFlow pour les DM (brique 4).

- **Vérifier :** `omega-zernio status` → `API reachable: true`
- **Constat 2026-07-26 :** HTTP 402, `Account paused: payment required`
- **Prix affiché :** 2 premiers comptes gratuits, puis 6$/mois par compte (3-10),
  3$/mois (11-100). Le nombre de comptes connectés est **illisible tant que c'est en
  pause**, donc pas de montant exact à annoncer.
- **Billing :** zernio.com/dashboard/billing
- **À sonder au retour :** la page pricing annonce `analytics` dans les features incluses.
  Si l'API l'expose vraiment, ça doublerait la brique 1. Le CLI `omega-zernio` n'a aucune
  commande analytics aujourd'hui, donc **ne pas compter dessus** avant de l'avoir vu.

### 4. Higgsfield : recharger les crédits

La moitié visuelle de la brique 2 (plans générés, avatars, hooks visuels, R-VISUAL-ID).

- **Vérifier :** `higgsfield account status`
- **Constat 2026-07-26 :** `x@agentik-os.com, ultra plan, 0.03 credits`. Le plan est
  actif, ce sont les **crédits** qui sont à sec : un plan payé ne veut pas dire générable.
- **Sans ça :** tout `higgsfield generate` échoue, la face cam et le montage restent manuels

### 5. Apify : factures impayées (OPTIONNEL)

Débloque les vraies **vues de lecture** des reels concurrents, la seule chose que le rail
Graph gratuit ne donne pas.

- **Vérifier :** `reels doctor` (ligne `apify`)
- **Constat 2026-07-26 :** compte `Agentik_os`, plan STARTER 29$/mois,
  `Too many outstanding invoices`. Le montant dû n'est **pas** exposé par l'API.
- **Billing :** console.apify.com/billing
- **Optionnel assumé :** avec le point 1, le scoring concurrent bascule sur l'engagement
  au lieu des vues. On perd en finesse, on ne perd pas la boucle.

### 6. ScrapeCreators : NE PAS PAYER

- **Constat 2026-07-26 :** HTTP 402, crédits épuisés
- **Verdict :** redondant avec Apify. Si un jour on remet de l'argent sur un scraper, c'est
  Apify, pas les deux.

---

## Ce qui tourne sans rien payer

`/reel-script`, `/reel-lint`, `reels score`, `reels hooks`, `reels ledger`, `reels report`.
Autrement dit : écrire, noter avant de tourner, scorer des outliers déjà scannés, et lire
le pattern ledger. Les rails externes sont des **sources**, pas le moteur.

## Envoyer la liste à l'opérateur

Il lit ses livrables depuis son téléphone (R-TGDELIVER). Le topic marketing est `1081`,
le hub `-1003946690016`. Envoyé une première fois le 2026-07-26 (message 1911).
Ne la renvoie pas telle quelle : re-vérifie d'abord, et n'envoie que ce qui est encore vrai.

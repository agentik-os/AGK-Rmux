# IG Reel Loop : brancher Claude sur Instagram

Le système en 4 briques : donner à Claude **les yeux** (tes données) et **les mains**
(créer, publier, répondre), puis refermer la boucle sur la **mesure**. On passe de
« je poste et je croise les doigts » à une machine qui se corrige à chaque publication.

Ce n'est pas de l'automatisation d'engagement. Pas de bot follow, pas de like en masse,
pas de spam DM : c'est le chemin le plus court vers le bannissement, et OmegaOS le refuse
(cf. le refus du mass-automation dans `growth-engine`). Ici on automatise la **création
calibrée par la donnée** et la **réponse à des gens qui ont commenté ton post**.

Doctrine : **R-MARKETING** (ordre de dépendance), **R-ZERNIO** (publication),
**R-ZERNFLOW** (engagement entrant), **R-CLI** (REST direct, jamais un serveur MCP),
**R-NODASH**, **L1** (le runtime est la seule vérité).

---

## Vue d'ensemble

```
BRIQUE 1 : LES YEUX            bin/reels scan / score / hooks / mine
   niche + ton compte     →    bibliothèque de hooks classée P1-P11
            ↓
BRIQUE 2 : LES MAINS           /reel-script  →  /reel-lint  →  tournage  →  montage
   le pattern gagnant     →    script 🟢 ≥ 85/100 avant que tu allumes la caméra
            ↓
BRIQUE 3 : LA PUBLICATION      omega-zernio post (instagram, tiktok, facebook…)
            ↓
BRIQUE 4 : LA BOUCLE           ZernFlow (comment-to-DM)  +  bin/reels mine / ledger
   la mesure revient      →    le classement des patterns bouge, la brique 1 apprend
```

La brique 1 et la brique 4 sont **le même moteur** (`bin/reels`) : c'est ce qui ferme
la boucle. Ce que tu mesures en 4 re-classe ce que tu produis en 1.

---

## Brique 1 : les yeux

Deux regards, un seul binaire.

**Le regard extérieur** : les reels qui surperforment dans ta niche.

```bash
cd <projet>
../OmegaOS/tools/marketing-machine/bin/reels doctor            # quels rails sont vivants
reels scan  --accounts marketing/00-context/swipe/accounts.txt --limit 25
reels score --min 10
reels hooks --top 30
```

Le score est un **ratio**, jamais un compteur brut : `vues du reel / médiane du compte`.
900k vues sur un compte à 20k de médiane t'apprend quelque chose ; 2M sur un compte à 5M
de médiane ne t'apprend rien. `>= 10x` = signal. `>= 30x` = format à reskinner cette
semaine, la fenêtre dure environ deux semaines. On copie **la forme, pas le sujet**.

**Le regard intérieur** : tes propres reels, avec leurs vraies métriques.

```bash
reels mine --limit 30      # vues, reach, saves, shares, temps de visionnage moyen
```

`reels hooks` sort deux choses dans `marketing/00-context/swipe/hooks.json` :

1. **La bibliothèque de hooks** : les hooks qui ont surperformé, avec leur lift.
2. **Le classement des patterns P1-P11** dans TA niche, par lift moyen mesuré.

### La taxonomie partagée, le vrai point de la machine

Les hooks ne sont pas rangés dans une catégorie inventée. Ils sont classés dans les
**mêmes patterns P1-P11** que `/reel-script` utilise pour écrire et que `/reel-lint`
utilise pour noter. C'est ce qui fait que la mesure est actionnable : quand le ledger
dit « P2 contraste financier, 6.0x chez toi », tu passes ce pattern à `/reel-script` et
`/reel-lint` vérifie qu'il est bien tenu. Une bibliothèque scorée dans un vocabulaire et
écrite dans un autre ne sert à rien.

`reels doctor` compare la table embarquée dans `bin/reels` avec celle de
`lint_script.py` et gueule en cas de dérive.

### Les sources, par ordre de préférence

| source | coût | ce qu'elle donne | statut |
|---|---|---|---|
| `graph` | gratuit | ton compte **avec insights** (vues, reach, saves, watch time) + tout compte business public via `business_discovery` | demande un token IG Business, une fois |
| `apify` | payant | vraies vues de lecture sur n'importe quel compte | compte bloqué (factures impayées) |
| `scrapecreators` | payant | idem | crédits épuisés |

Le rail **gratuit** est le meilleur des trois pour ton propre compte : c'est le seul qui
donne la **rétention**, et la rétention est la métrique qui décide de la distribution.
Limite connue (L1) : `business_discovery` renvoie les likes et commentaires des
concurrents mais **pas** les vues, donc le scoring concurrent bascule sur l'engagement.
Le moteur l'indique dans la colonne `métrique`, il ne fait pas semblant.

Ouvrir le rail gratuit : `reels doctor` imprime la procédure exacte (compte Business lié
à une Page, app developers.facebook.com, scopes `instagram_basic` +
`instagram_manage_insights`, token longue durée, puis `~/.omega/secrets/instagram.env`).
Aucun serveur MCP là-dedans : REST direct depuis le CLI (R-CLI).

---

## Brique 2 : les mains

```bash
# 1. prends le pattern en tête du classement, pas celui qui te plaît
/reel-script un reel sur <ton sujet>       # pattern imposé + 3 variantes de hook
/reel-lint le-script.md                    # note /100 AVANT le tournage
```

Le linter note en 6 blocs : HOOK 35 · RÉTENTION 25 · FORMAT 20 · CTA 10 · VOIX 10 ·
PUISSANCE 5. Règle de production : **on ne tourne qu'à 🟢 (≥ 85)**. En dessous, on
corrige les ❌ listés et on relance. Le piège numéro un qu'il attrape en une seconde :
le script trop long, 285 mots = 1 min 38 au téléprompteur pour un format 60 s.

Un 100/100 veut dire « conforme aux patterns mesurés », pas « viral garanti ». C'est un
filtre d'erreurs, pas une boule de cristal.

**Le visuel** : la partie face cam reste à toi, c'est le point du format. Tout le reste
passe par les skills existantes plutôt que par un montage à la main :
`/omg-higgsfield-generate` pour les plans générés, les avatars et les hooks visuels
(R-VISUAL-ID), `/hyperframes` pour les schémas animés et les captions à l'écran,
`/art-director-content-engine` quand il faut choisir le format avant de produire.

---

## Brique 3 : la publication

**Tout passe par Zernio** (R-ZERNIO). On ne hand-roll jamais la Graph API d'Instagram ni
un uploader maison pour publier : c'est exactement le footgun que la règle existe pour
tuer.

```bash
omega-zernio post <projet> --text "…" --platforms instagram,tiktok,facebook \
    --media ./reel.mp4 --dry-run                    # toujours valider d'abord
omega-zernio post <projet> --text "…" --platforms instagram,tiktok,facebook \
    --media ./reel.mp4 --schedule 2026-08-01T09:00:00Z
```

Pièges vérifiés : YouTube et TikTok **exigent une vidéo** (une image renvoie un 400 qui
fait tomber tout le batch), Reddit exige un subreddit, et la validation est
tout-ou-rien à la création. Et surtout : `posted:true` veut dire **accepté**, pas
publié. Instagram finalise les reels en asynchrone (`awaiting-finalize`) : on vérifie
sur le vrai profil avant de dire que c'est en ligne (R-PROD / L1).

La file par projet vit dans `marketing/04-publishing/calendar.json`.

---

## Brique 4 : la boucle

Deux moitiés : répondre, et mesurer.

**Répondre, comment-to-DM via ZernFlow** (R-ZERNFLOW), l'alternative open source à
ManyChat, self-hostée, adossée au même compte Zernio. Chaque commentaire reçoit la
ressource promise en DM, et un commentateur devient un abonné.

```bash
bash tools/zernflow/install-zernflow.sh    # opt-in, clone le commit épinglé + npm install
```

Le flux se construit dans l'app (Next.js + Supabase dédié, ref `mbsncijxqvawawpgjbkp`).
Le CTA du script (« commente ce que tu veux, je t'envoie la ressource en DM ») est
délibérément **sans mot-clé imposé** : le linter le vérifie, et un mot-clé imposé baisse
le taux de commentaire.

**Mesurer, le pattern ledger.**

```bash
reels mine        # re-lit tes reels et leurs insights
reels ledger      # ton lift par pattern, en face du lift de la niche
```

Le ledger sort un verdict par pattern, et c'est la colonne qui compte :

- **double la dose** : ce pattern marche chez toi, ≥ 1.5x ta médiane.
- **marche dans la niche, pas chez toi** : le format est bon, ton exécution ne l'est pas.
  Problème de craft (hook mou, rythme, première seconde), pas de format. Ne jette pas le
  pattern, refais-le mieux.
- **arrête ce pattern** : < 0.7x, il te coûte de la distribution.

C'est cette distinction qui empêche l'erreur classique : abandonner un format qui marche
parce qu'on l'a mal exécuté deux fois.

Cadence : `reels mine && reels ledger` après chaque publication, `reels scan` une fois
par semaine. Le classement bouge, la brique 1 apprend, la brique 2 écrit autrement.

---

## Le run complet

```bash
reels doctor                       # 0. quels rails sont vivants aujourd'hui
reels scan --accounts …/accounts.txt --limit 25
reels score --min 10
reels hooks --top 30               # 1. la bibliothèque de hooks
/reel-script un reel sur <sujet>   # 2. le script, sur le pattern en tête
/reel-lint le-script.md            #    ne tourne qu'à 🟢
# tournage face cam + montage (higgsfield / hyperframes)
omega-zernio post <projet> --platforms instagram,tiktok --media ./reel.mp4 --dry-run
omega-zernio post <projet> --platforms instagram,tiktok --media ./reel.mp4 --schedule …
# ZernFlow répond aux commentaires en DM
reels mine && reels ledger         # 4. la mesure revient dans la brique 1
```

## Ce qui bloque aujourd'hui (état vérifié le 2026-07-26)

| rail | état | ce qu'il faut |
|---|---|---|
| Instagram Graph | pas de credential | un token IG Business, gratuit, procédure dans `reels doctor` |
| Apify | HTTP 403, factures impayées | régler le compte Apify, ou s'en passer (le rail Graph suffit) |
| ScrapeCreators | HTTP 402, crédits épuisés | recharger, ou s'en passer |
| Zernio (brique 3) | HTTP 402, abonnement inactif | réactiver l'abonnement Zernio |
| ZernFlow (brique 4) | credential présent, app non déployée | lancer `install-zernflow.sh` puis déployer |

Les briques 2 (script + lint) et la logique de scoring, de bibliothèque et de ledger
tournent **sans aucune de ces dépendances**. Les rails externes sont des sources, pas le
moteur : `reels doctor` dit toujours lequel est vivant, et rien ne prétend fonctionner
quand ce n'est pas le cas.

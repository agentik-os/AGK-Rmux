# L'interrogatoire — phase 0

*Le moment où l'on peut encore tuer une mauvaise idée sans que ça coûte.*

---

## Comment mener

**Par ronde, pas d'un bloc.** Poser les questions d'une ronde, attendre, lire les réponses, **relever ce qui est faible**, puis passer à la suivante.

**Ne pas flatter.** Une réponse floue relevée maintenant vaut trois mois de build économisés. Une réponse floue acceptée devient une hypothèse silencieuse que personne ne rouvre.

**Marquer chaque réponse.** Solide · flou · **absent**. Le compte final décide.

---

## Ronde 1 — La cible

Cinq questions. C'est la ronde qui tue le plus d'idées, et c'est très bien.

**1. Nomme UNE personne réelle qui achèterait ça.**
Pas un segment, pas un persona. Un nom, ou un rôle précis avec une organisation en tête.
> ❌ « Les communautés » · « Les entrepreneurs » · « Les gens qui veulent s'organiser »
> ✅ « L'opérateur du club X à Londres, 60 membres, fait tout seul »

**2. Elle utilise quoi aujourd'hui, et combien elle paie ?**
Si la réponse est « rien » ou « Excel », attention : soit c'est un marché neuf, soit le problème ne vaut pas d'être payé. Il faut trancher lequel.

**3. Qu'est-ce qui la fait changer d'outil ?**
Une **douleur**, pas une amélioration. Personne ne migre pour 20 % de mieux.
> ❌ « C'est plus beau » · « C'est plus rapide » · « C'est mieux pensé »
> ✅ « Elle perd des membres et ne sait pas pourquoi »

**4. Qui signe le chèque, et est-ce la même personne que celle qui utilise ?**
Sur un OS professionnel, c'est presque toujours non. Ça change tout le produit.

**5. Si elle dit non, c'est pour quelle raison ?**
La vraie, pas « pas le temps ». Prix, risque, inertie, politique interne, peur de la migration.
Une idée dont on ne sait pas pourquoi elle serait refusée n'a jamais été confrontée.

**Verdict de ronde :** deux réponses absentes ou plus sur cinq → **s'arrêter là.** Le produit n'a pas d'acheteur, il a une intuition.

---

## Ronde 2 — Le produit

**6. Quel est l'objet central ? Un mot.**
C'est le gate de la phase 2, posé tôt exprès. `post`, `content`, `message` → recommencer.

**7. Qu'est-ce qui casse si tu le retires ?**
Si la réponse est « pas grand-chose », ce n'est pas la primitive.

**8. Quelle est la seule chose que l'utilisateur doit pouvoir dire après 90 jours ?**
Une phrase, à la première personne. C'est la promesse du produit.
> ✅ « J'ai rencontré trois personnes qui comptent »
> ❌ « J'ai gagné du temps »

**9. Qu'est-ce que tu ne construis PAS, alors que tous les concurrents l'ont ?**
Si la réponse est « rien », le produit est un clone légèrement meilleur.
Si la réponse est longue, vérifier en phase 5 que ce n'est pas de la parité indispensable déguisée en choix.

**10. Quelle feature construirais-tu en dernier, et pourquoi ce n'est pas grave ?**
Teste la compréhension de la séquence. Une réponse honnête ici évite six mois d'ordre inversé.

**11. Ton utilisateur a déjà quatre outils. Lequel tu remplaces ?**
On n'ajoute pas un cinquième outil à quelqu'un qui en a quatre. On en supprime un, ou on n'existe pas.
Sur un OS personnel c'est la question la plus dure : **quelle habitude existante remplaces-tu ?**

---

## Ronde 3 — Le business

**12. Combien de clients faut-il pour que ça vaille ton temps ?**
Calcul explicite : prix × nombre − coûts = ce que ça doit rapporter. Pas d'ordre de grandeur vague.

**13. Combien de clients possibles existent au monde ?**
Recherche réelle, pas une estimation. Si la 13 est inférieure à la 12, **s'arrêter.**

**14. Qu'est-ce qui empêche un concurrent de copier en six mois ?**
Les features ne sont pas un moat. Les moats réels : un effet de réseau entre clients · une donnée qui s'accumule et ne se rattrape pas · une position de distribution · un coût de changement opérationnel.
> « On ira plus vite » n'est pas une réponse.

**15. Si ça marche, qui vient te tuer, et avec quoi ?**
L'incumbent ajoute-t-il la feature en un trimestre ? Alors ce n'était pas une catégorie, c'était une feature manquante chez lui.

**16. Qu'est-ce qui doit être vrai pour que ça échoue ?**
Le pré-mortem. Trois causes, écrites maintenant. Elles deviennent les risques de la phase 13.

---

## Ronde 4 — Toi

Les trois questions que personne ne se pose et qui décident de tout.

**17. Tu l'utiliserais toi-même, tous les jours ?**
Si non : **s'arrêter.** Doctrine 7 — on ne conçoit pas un OS pour un métier qu'on ne pratique pas. On construirait les features qu'on imagine, pas celles qui manquent.

**18. Tu as un premier client, nommé, pour la v1 ?**
Pas « j'en trouverai ». Un nom. Si c'est toi, c'est une réponse valide sur un OS personnel — pas sur un OS professionnel.

**19. Combien de temps avant que ça devienne un sixième client ?**
Estimation honnête des heures par semaine. Si le produit demande plus de temps que le bloc disponible, il ne se construira pas — quelle que soit sa qualité.

---

## Le verdict

| Signal | Conduite |
|---|---|
| ≥ 2 absentes en ronde 1 | **Stop.** Pas d'acheteur identifié |
| Q6 = `post` / `content` | **Stop.** Reprendre la primitive |
| Q13 < Q12 | **Stop.** Le marché est plus petit que le seuil de viabilité |
| Q14 = « on ira plus vite » | **Stop.** Aucun moat |
| Q17 = non | **Stop.** Pas de dogfood possible |
| Tout le reste | Continuer, en notant les flous comme risques |

**Un arrêt en phase 0 n'est pas un échec du skill. C'est son meilleur résultat possible.**

---

## Les questions supplémentaires par branche

### OS personnel
- Quelle est la **boucle quotidienne** ? Combien de secondes ?
- Qu'est-ce qui ramène l'utilisateur au jour 12, quand la nouveauté est passée ?
- Le produit **remplace** une habitude ou en **ajoute** une ? (ajouter = mort)
- Que voit l'utilisateur au premier lancement, avec zéro donnée ?
- Combien de temps avant la première valeur ressentie ? Si c'est plus d'une session, revoir.

### OS professionnel
- **Quelle ligne budgétaire ?** Formation, outillage, direction, marketing ?
- Combien d'interlocuteurs pour signer, et lequel peut dire non seul ?
- Que faut-il pour un achat en entreprise : SSO ? journal d'audit ? RGPD ? DPA ? SLA ?
- À quoi ressemble le déploiement pilote, et quand devient-il irréversible ?
- Que se passe-t-il au renouvellement si le champion interne part ?

---

*Une idée qui ne survit pas à dix-neuf questions ne survivra pas à dix-neuf mois.*

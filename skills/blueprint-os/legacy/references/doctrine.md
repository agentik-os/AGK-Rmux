# Les dix doctrines AgentikOS

*À lire avant toute conception. Ce sont elles qui distinguent un OS d'un SaaS de plus.*

---

## 1. Un OS n'est pas une automatisation

| Automatisation | Système d'exploitation |
|---|---|
| Relie des outils existants | **Supprime le besoin d'outils** |
| Optimise une tâche | Définit comment le travail se fait |
| Se vend au responsable technique | **Se vend au dirigeant** |
| Se facture en projet | Se facture en poste de direction |
| Remplacée en six mois | Devient l'infrastructure |

Un OS a cinq couches : **données · interface · agents · rituels · correction.** Si le blueprint n'en produit pas cinq, ce n'est pas un OS.

**La couche « rituels » est celle que tout le monde oublie.** Ce sont les boucles récurrentes qui font tourner l'organisation — check-in, revue, synthèse hebdomadaire. Un système sans rituel est un classeur.

**La couche « correction »** corrèle les résultats à l'intention d'origine et mesure la dérive. Un OS qui ne se corrige pas dérive en six mois.

---

## 2. La ligne 80/20

> **~80 % de code déterministe. ~20 % de jugement IA.**

| | Ce qui se passe |
|---|---|
| 100 % IA | Imprévisible, non auditable, coûteux, impossible à débugger |
| 0 % IA | Rigide, redevient un SaaS de plus |
| **80/20** | Prévisible là où ça compte, adaptatif là où c'est nécessaire |

Le code fait le travail : règles, calculs, flux, garde-fous. L'IA fait ce qu'aucune règle ne sait faire : classer l'ambigu, arbitrer, synthétiser, écrire.

**Savoir où tracer la ligne est le vrai métier.** Chaque blueprint doit dire explicitement, pour chaque module, de quel côté il tombe.

---

## 3. Les trois niveaux d'agent

| Niveau | Rôle | Approbation | Part de la valeur |
|---|---|---|---|
| **1 — Capteur** | Observe, enregistre | Aucune | Socle |
| **2 — Analyste** | Corrèle, alerte, propose | Aucune — il ne touche à rien | **~90 %** |
| **3 — Exécutant** | Agit | **Systématique** | Le plus fragile et coûteux |

L'erreur universelle est de courir au niveau 3. Un agent qui envoie des mails casse. Un agent qui dit « ce client représente 61 % de ton revenu depuis trois semaines » ne casse jamais et change une décision.

**Un blueprint majoritairement niveau 3 est mal conçu.**

---

## 4. Tout émet un signal

Une seule table d'événements, horodatée, indexée par acteur.

> **Un module qui n'émet aucun signal est du poids mort, aussi bon soit-il.**

C'est ce qui rend possible la lecture en travers — relier des domaines que l'utilisateur tient séparés. Sans elle, on a neuf modules qui ne se parlent pas : exactement le produit qu'on prétend remplacer.

**Test avant d'ajouter quoi que ce soit :** *quel signal ce module envoie-t-il dans la boucle ?* Pas de réponse = pas de module.

---

## 5. Les agents doivent pouvoir dire des choses inconfortables

Un agent qui flatte est pire qu'un agent absent : il donne une fausse assurance sur une décision.

- **Toute affirmation cite sa donnée.** Une corrélation sans les chiffres qui la fondent est refusée
- **« Rien de notable » est une sortie valide et attendue.** Un agent qui trouve toujours quelque chose invente
- **Le taux de faux positifs est une métrique du produit**, pas un détail
- Une recommandation forte déclenche sa propre réfutation
- Aucun contenu généré n'est publié au nom d'un humain sans relecture

---

## 6. L'anti-scale est le moat

Distinguer deux couches, et ne jamais les confondre :

| | Doit rester artisanal | Doit devenir scalable |
|---|---|---|
| **Quoi** | Le produit — rareté, sélection, jugement, goût | L'opération — admin, tri, rituels, acquisition |
| **Pourquoi** | C'est la valeur | C'est le coût |

L'erreur classique est de scaler le produit tout en gérant l'opération à la main. C'est l'inverse qu'il faut faire.

> **Ce qui ne scale pas dans le produit n'est pas un problème à résoudre. C'est le moat.**

---

## 7. La séquence dogfood

On construit d'abord pour soi. Au moins douze mois d'usage réel avant de vendre.

**Raison :** on ne peut pas concevoir un OS pour un métier qu'on ne pratique pas. On construirait les features qu'on imagine, pas celles qui manquent.

Conséquence sur la release :
- **v0** = le morceau autonome, utile seul, livrable en une semaine
- **v1-v3** = la primitive et le différenciant, utilisés en solo
- **v4+** = la commodité — conversation, cours, paiements
- **v5** = multi-tenant

**La commodité se construit en dernier et s'achète en attendant.** C'est la seule partie où l'on n'a rien à prouver.

---

## 8. Ce qui reste humain

Non par incapacité technique — par nature de la décision.

| Jamais automatique | Pourquoi |
|---|---|
| Message envoyé au nom de quelqu'un | La relation est l'actif |
| Mouvement d'argent | L'agent calcule et rappelle. Il ne vire pas |
| Acceptation ou refus d'un engagement | L'agent chiffre. L'arbitrage reste humain |
| Modification d'un seuil écrit à froid | Sinon le garde-fou ne sert à rien |
| Publication sans relecture | Le contenu est la marque |
| **Le jugement sur une personne** | L'IA propose une mise en relation, jamais un verdict |

---

## 9. Une seule métrique étoile

Un blueprint qui affiche six métriques au même niveau n'a pas choisi.

L'étoile doit être :
- **Relationnelle ou de résultat**, jamais d'engagement
- Calculable automatiquement
- Prédictive de la rétention
- Impossible à optimiser par la triche

| Bon | Mauvais |
|---|---|
| Liens réels par membre | Messages postés |
| Décisions améliorées | Sessions actives |
| Demandes résolues sous 48 h | Temps passé |
| Humains-colle supprimés | Utilisateurs actifs |

---

## 10. Le vocabulaire est un actif

> **Ceux qui nomment définissent.**

Chaque OS produit 5 à 8 termes propriétaires, employés sans relâche et sans approximation. Douze mois d'usage cohérent et ils deviennent identifiants — et un client qui reprend ton vocabulaire a déjà accepté ton cadre.

Le lexique commun d'AgentikOS, à ne jamais relâcher :

| On emploie | On n'emploie pas |
|---|---|
| Système d'exploitation, OS | Solution, plateforme |
| La ligne 80/20 | « De l'IA partout » |
| Humain-colle | Tâche répétitive |
| Rituel | Process |
| Dérive, correction | Optimisation |
| Agent capteur / analyste / exécutant | « Un agent IA » |
| Shell | Interface |
| Primitive | Fonctionnalité principale |

---

## 11. On ne devine jamais. On nomme le trou.

La doctrine la plus violée, et celle qui coûte le plus cher six mois plus tard.

Quand une décision n'est pas prise, un blueprint honnête écrit **`[unknown]`** — et surtout **ce qui la trancherait**.

> `foot_primary` est `[unknown]`. Ce qui le trancherait : une ligne par panneau dans la table des pieds, section 7.

Trois corollaires :

**Un placement non sourcé ne se dessine pas.** Si aucune source ne place un objet quelque part, on ne l'invente pas — on écrit que la place est ouverte.

**Les contradictions s'enregistrent, elles ne se lissent pas.** Deux sources qui disent l'inverse : on cite les deux, on dit laquelle on applique et pourquoi, on nomme ce qui réglerait le conflit. Une divergence masquée devient irréparable.

**Une valeur devinée est pire qu'une valeur absente.** L'absence est une question ouverte. La devinette est une fausse certitude que quelqu'un construira.

---

## 12. Le différenciant se construit, la parité s'achète

> **Un produit qui n'a que ses différenciateurs est une démo.**

| | Rôle |
|---|---|
| **Le différenciant** | Fait qu'on te choisit |
| **La parité** | Fait qu'on peut t'utiliser |

Il manque une seule capacité de parité et le produit est inutilisable, quelle que soit la qualité du reste.

**Elle n'est pas optionnelle. Elle est tardive.** On la construit en dernier, on l'achète en attendant, on ne l'ignore jamais.

Quatre verdicts, jamais un non-dit : **construire · acheter · différer · refuser**. Une capacité de socle sans verdict est une dette qu'on découvre à la livraison.

Ratio sain : 15 à 30 % de différenciant. Au-delà, on décrit une démo.

---

## 13. Personnel et professionnel ne posent pas les mêmes questions

| | OS personnel | OS professionnel |
|---|---|---|
| Utilisateur | = l'acheteur | ≠ l'acheteur, souvent |
| Ce qui décide | **La boucle quotidienne** | **La dépendance opérationnelle** |
| Le risque n°1 | Abandon au jour 12 | Le pilote qui ne devient jamais infrastructure |
| Question clé | Quelle habitude remplaces-tu ? | Quelle ligne budgétaire ? |
| Rétention | Le rituel | L'irréversibilité |
| Prix | 9-99 €/mois | 500-5 000 €/mois |
| Surface en plus | — | SSO, audit, RGPD, DPA, export, SLA |

**Sur un OS personnel :** on n'ajoute pas un cinquième outil à quelqu'un qui en a quatre. On en remplace un, ou on n'existe pas.

**Sur un OS professionnel :** un pilote qui ne devient pas infrastructure est une dépense. L'irréversibilité est la vraie métrique de rétention.

---

## Le résumé en une page

1. Un OS a cinq couches — données, interface, agents, rituels, correction
2. 80 % de code déterministe, 20 % de jugement
3. Trois niveaux d'agent — 90 % de la valeur au niveau 2
4. Tout module émet un signal, sinon il ne se construit pas
5. Les agents citent leurs données et peuvent ne rien dire
6. Ce qui ne scale pas dans le produit est le moat
7. Douze mois d'usage sur soi avant de vendre
8. Certaines décisions ne s'automatisent jamais
9. Une seule métrique étoile, relationnelle ou de résultat
10. Le vocabulaire est un actif — nommer, c'est définir
11. On ne devine jamais — `[unknown]` plus ce qui le trancherait
12. Le différenciant se construit, la parité s'achète — jamais 100 % différenciant
13. Personnel et professionnel ne posent pas les mêmes questions

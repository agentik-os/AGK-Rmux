# La matrice de parité — phase 5

*La phase que tout le monde saute, et l'erreur la plus coûteuse du processus.*

---

## Le principe

> **Un produit qui n'a que ses différenciateurs est une démo.**

Le différenciant fait qu'on te choisit. **La parité fait qu'on peut t'utiliser.** Il en manque une seule et le produit est inutilisable, quelle que soit la qualité de l'autre.

La parité n'est pas optionnelle. Elle est **tardive** : construite en dernier, achetée en attendant, mais jamais ignorée.

---

## Comment trancher

Pour chaque capacité de la liste ci-dessous, un verdict et une seule ligne de justification :

| Verdict | Sens | Ce qu'il faut préciser |
|---|---|---|
| **CONSTRUIRE** | On le fait nous-mêmes | À quelle version |
| **ACHETER** | Brique tierce | Laquelle, nommée, et son coût |
| **DIFFÉRER** | Plus tard | Jusqu'à quand, et comment on vit sans |
| **REFUSER** | Jamais | Pourquoi c'est cohérent avec le segment |

**Gate :** une capacité de socle ni construite, ni achetée, ni explicitement différée = un produit inutilisable découvert à la livraison.

---

## A. Socle — personne ne peut s'en passer

| Capacité | Note |
|---|---|
| Authentification | Clerk. ACHETER par défaut |
| SSO / SAML | Sur OS professionnel, souvent bloquant à la vente |
| Rôles et permissions | Granularité à décider tôt : elle contamine tout le schéma |
| Structure — espaces, canaux, sections | La navigation Stax en tient lieu, mais il faut le dire |
| Fils de discussion | La commodité par excellence. Chronologique, sans algorithme |
| Recherche transverse | Sous-estimée. Sans elle, un produit à 200 objets devient inutilisable |
| Notifications in-app | |
| **Email transactionnel** | **Le canal réel.** La délivrabilité est un projet à part entière |
| Notifications push | Mobile natif seulement |
| Mobile | PWA suffit souvent. À dire explicitement |
| Onboarding | Le premier lancement à zéro donnée est une feature, pas un détail |
| Profils utilisateur | |
| Upload de fichiers | Stockage, quotas, types autorisés |
| Mentions et réponses | |
| Modération de base | Signalement, masquage, suspension |

## B. Monétisation

| Capacité | Note |
|---|---|
| Abonnement récurrent | Stripe. ACHETER |
| Paiement unique | Souvent oublié. Une formation, une retraite, un produit |
| Tarification par groupe et par produit | La plus souple des quatre concurrents. Kommu l'a |
| Paliers multiples | |
| Paywall par espace ou par objet | |
| Essais, coupons, codes | |
| Facturation et relance d'impayés | **Le dunning.** Sans lui on perd 5-10 % de revenu par an, silencieusement |
| Prorata, changement de plan, remboursement | |
| Factures et TVA | Bloquant en B2B européen |

## C. Contenu et formation

| Capacité | Note |
|---|---|
| Bibliothèque de fichiers | |
| Cours / classroom | Circle et Skool l'ont. DIFFÉRER est légitime, pas ignorer |
| Diffusion échelonnée, cohortes | |
| Quiz, certificats | Aucun des quatre ne le fait bien. Opportunité ou hors scope |
| Versionnage de contenu | |

## D. Événements

| Capacité | Note |
|---|---|
| Calendrier, RSVP | |
| Rappels automatiques | Le taux de présence en dépend directement |
| Live / streaming | Coûteux. ACHETER ou REFUSER |
| Enregistrement et rediffusion | |
| Billetterie | |
| Fuseaux horaires | Trivial en apparence, cassant en pratique |

## E. Communication

| Capacité | Note |
|---|---|
| Messages directs | Sur un produit sélectif, à cadrer : le DM ouvert détruit une salle |
| Annonces | |
| Digests périodiques | Ce qui ramène les gens sans notification agressive |
| Traduction avant envoi | Kommu l'a. Unique sur le marché |
| Correction orthographe et grammaire | Abaisse la barrière pour les non-natifs |

## F. IA de production — la parité IA

Les quatre concurrents l'ont ou l'ajoutent. **La proposer n'est pas un différenciateur ; ne pas l'avoir est un manque visible.**

| Capacité | Note |
|---|---|
| Assistance à la rédaction | |
| Traduction | |
| Correction grammaticale | |
| Génération d'images | Souvent REFUSER sur un produit premium : ça produit du volume |
| Génération de contenu structuré | Cours, résumés, brouillons |
| Modération assistée | |
| Recherche sémantique | Devient un socle, plus un différenciant |

> **À distinguer de l'IA de jugement** — routage, mémoire, corrélation, arbitrage. Celle-là est le différenciant, et elle se spécifie en phase 8b.

## G. Administration

| Capacité | Note |
|---|---|
| Analytics et tableaux de bord | |
| Export de données | Bloquant en B2B, et exigence RGPD |
| Journal d'audit | Bloquant en entreprise |
| API publique | |
| Webhooks | |
| Zapier / Make / n8n | Souvent suffisant à la place d'une API publique |
| Domaine personnalisé | Circle l'a dès l'entrée de gamme. Bloquant en premium |
| White-label | |
| Import depuis un concurrent | **Le coût de migration décide de la vente.** Souvent oublié |

## H. Confiance et conformité

| Capacité | Note |
|---|---|
| RGPD — export, suppression, consentement | Non négociable en Europe |
| DPA et sous-traitants | Demandé dès le premier client sérieux |
| Chiffrement au repos et en transit | |
| Sauvegardes et restauration | |
| Page de statut et SLA | |
| Contrôle d'indexation | Sur un produit discret, c'est un socle |
| Rétention paramétrable | |

---

## Le tableau à produire

```markdown
| # | Capacité | Verdict | Version | Justification |
|---|---|---|---|---|
| A3 | SSO / SAML | DIFFÉRER | v5 | Aucun client pilote ne l'exige. Bloquant au-delà de 3 clients entreprise |
| A8 | Email transactionnel | ACHETER | v1 | Resend. La délivrabilité n'est pas notre métier |
| C2 | Cours / classroom | REFUSER | — | Commodité. Nos membres n'achètent pas de formation |
| F1 | Assistance rédaction | CONSTRUIRE | v4 | Parité attendue. Sur notre couche IA, coût marginal faible |
```

**Contrôle final :** compter les lignes du blueprint par couche.

| Couche | Attendu |
|---|---|
| Socle | Présent, majoritairement ACHETER ou DIFFÉRER |
| **Parité** | **Présent.** Un blueprint sans ligne de parité a raté cette phase |
| Différenciant | 15 à 30 % des features, pas 100 % |

Un blueprint 100 % différenciant décrit une démo, pas un produit.

---

## Les pièges

**Confondre « on ne le fait pas » et « on ne l'a pas encore ».** Le premier est un positionnement, le second est une dette. Les deux sont légitimes, mais il faut savoir lequel on écrit.

**Différer sans dire comment on vit sans.** « Pas de mobile en v1 » demande la suite : et alors, l'utilisateur fait quoi dans le train ?

**Refuser une capacité de socle pour faire élégant.** Refuser la recherche parce que « la navigation Stax suffit » est vrai à 50 objets et faux à 5 000.

**Oublier l'import depuis le concurrent.** Le coût de migration est souvent le vrai motif de refus, et il ne se voit dans aucune démo.

---

*Le différenciant fait qu'on te choisit. La parité fait qu'on peut t'utiliser.*

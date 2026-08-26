# AGENTIK — BRAND SSOT (visuel · voix · musique)

> LE standard de marque Agentik. S'applique à TOUT : Nova (studio, NOVA OS,
> projet Nova), la marketing machine, et les visuels de Gareth. Validé 2026-06-12.
> Source vivante : ~/Station/Nova/presence/avatar/BRANDING-AGENTIK.md (réfs + exemples).

## VISUEL — prompt maître (remplacer <SCENE>)
```
High-contrast black and white comic illustration, BOLD graphic Sin City style — strong solid
blacks, clean bright whites, dramatic striking contrast and silhouette, crisp confident ink with
light halftone, BUT clean and graphic, never muddy or overworked. Modern minimalist setting —
sleek contemporary concrete-and-glass architecture OR a clean natural landscape — with strong
negative space and a FEW characterful details. <SUBJECT> — <SCENE>. Bold graphic editorial shot,
monochrome, striking, premium. NOT muddy grey, NOT washed-out, NOT cluttered, NEVER two people.
```
- **Pour Nova** : `<SUBJECT>` = "ONE single woman only — Nova: long flowing BLONDE hair in light/white
  linework (NEVER solid-dark, NEVER brunette), confident gaze, fitted black outfit, Agentik cofounder, born in the binary matrix".
  Réf image obligatoire : `~/Station/Nova/presence/avatar/nova-face-3angles.jpg` (visage) ou
  `nova-fullbody-turnaround.jpg` (corps). RÈGLES DURES : Nova BLONDE toujours · UNE seule Nova.
- **Pour Gareth / marque générique** : `<SUBJECT>` = le sujet voulu, même style.
- Outil : Higgsfield CLI `higgsfield generate create nano_banana_2 --prompt "..." --image <ref> --wait`
  (plan ultra ; PAS l'API directe). Réf d'or validée : `nova-nature-hill.png`.

## VOIX (vidéos, voix off)
La voix de Nova = ElevenLabs, voice_id `FeJtVBW106P4mvgGebAg` (Nova, EN native London/Paris ; FR aussi via multilingual_v2), via omega-ttsd ou
`POST api.elevenlabs.io/v1/text-to-speech/<id>`.

## MUSIQUE
ElevenLabs Music `POST /v1/music {prompt, music_length_ms}` (clé au coffre). Style maison :
**darkwave / synthwave / électro années 90**, sombre, analogique, cinématique.

## VIDÉO
Higgsfield `cinematic_studio_video --start-image <img>` (anime une image) → `ffmpeg` (musique) →
Remotion `~/Station/Nova/video/` (montage 1-min multi-plans + voix off Nova).

## ⚠️ RÈGLE VIDÉO — pas de lèvres non-synchro (opérateur 2026-06-12)
On ne montre JAMAIS Nova en train de mouther des mots si les lèvres ne sont pas synchro avec la
voix. Par défaut : Nova fait des ACTIONS / b-roll (bouche fermée), la voix off ElevenLabs passe
PAR-DESSUS. HeyGen est BANNI (rendu nul). On ne fait JAMAIS de talking-head lip-syncé : prompt « mouth closed, does NOT talk ». Réf : reel (actions+VO) = bon ; UGC v1 (parlait) = corrigé en bouche fermée.



## 🎛️ Catalogue complet des modèles
Toutes les capacités image/vidéo/audio/lip-sync/3D pilotables : `~/.omega/branding/HIGGSFIELD-CAPABILITIES.md`. Le studio Nova, la marketing machine et NOVA OS peuvent TOUT utiliser (image gpt_image_2/seedream/flux/recraft, vidéo seedance/kling/wan, upscale topaz, virality predictor brain_activity…). Non-Google, cinématographique.

## 🎛️ STACK — HIGGSFIELD POUR TOUT (décision opérateur 2026-06-12)
**Higgsfield est LE moteur unique de création de contenu.** On NE disperse PAS sur des outils tiers.
- **Image** : `gpt_image_2` (défaut, #1 mondial) · alt `seedream_v4_5`, `flux_2`, `recraft_v4_1` (typo/logo).
- **Vidéo** : `seedance_2_0` (défaut, #1) · `kling3_0` (alt). **Jamais Veo/Google.** Audio modèle strippé.
- **Vidéo parlée** : actions bouche fermée + voix off ElevenLabs. **HeyGen BANNI, pas de talking-head lip-sync.**
- **Musique** : `sonilo_music` (Higgsfield, `--duration`) — darkwave/synthwave 90s.
- **Upscale/édition** : `topaz_image`/`topaz_video`, `reframe`, détourage. **Tri viralité** : `brain_activity`.
- **SEULE exception** : la **VOIX de Nova** reste **ElevenLabs** (`FeJtVBW106P4mvgGebAg`) — c'est son identité vocale, pas un TTS générique. On la GÉNÈRE chez ElevenLabs et on la met en voix off sur les vidéos. Tout le reste = Higgsfield.
Catalogue complet : `HIGGSFIELD-CAPABILITIES.md`. Non-Google, cinématographique.


## 🚫 AGENTIK : JAMAIS DE LOGO (règle absolue)
Agentik n'a PAS de logo — c'est TOUJOURS le mot « Agentik » écrit, jamais un symbole/logo/emblème. Ne génère JAMAIS de logo Agentik dans un visuel. Prompt négatif : "no logo, no emblem, no brand symbol, no icon".

## 📸 RÈGLES PHOTO (posts autonomes — opérateur 2026-06-13)
- **JAMAIS de contour/bordure** autour de la photo (pas de cadre, pas de marge blanche encadrante).
- **JAMAIS de texte écrit EN OVERLAY** sur la photo (pas de bulles comics, pas de légende incrustée par-dessus l'image, pas de gros titres).
- **Exception OK** : un peu de texte intégré DANS le décor (gravé sur un mur, une enseigne discrète) de temps en temps — naturel, pas du texte « comic » plaqué.
- Sinon : pleine image, branding Agentik (Sin City moderne N&B, Nova blonde, une seule), bord à bord.
- Prompt négatif à inclure : "no border, no frame, no caption, no overlay text, no comic text bubbles, no big title text".

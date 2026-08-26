# CAPACITÉS DE GÉNÉRATION — ce que je peux faire via l'API (Higgsfield + co)

> Catalogue complet des modèles image/vidéo/audio/3D pilotables par CLI
> (`higgsfield generate create <model> --prompt … --image/--audio/--video … --wait`).
> La marketing machine, le studio Nova et NOVA OS DOIVENT savoir que tout ceci est dispo.
> Règle marque : **éviter Google (Veo, Nano Banana)** ; viser le cinématographique.

## 🖼️ IMAGE — génération
| Modèle | Pour quoi | Note |
|---|---|---|
| `gpt_image_2` (GPT Image 2) | **DÉFAUT** — image cinéma, premium, tient le perso + le blond | OpenAI, non-Google ⭐ |
| `seedream_v4_5` / `seedream_v5_lite` | alternative forte, réaliste/stylé | ByteDance, non-Google |
| `flux_2` / `flux_kontext` | qualité + édition par référence (kontext = edit/variations) | Black Forest Labs |
| `recraft_v4_1` | design graphique, logos, vectoriel, typo propre | excellent pour visuels marque |
| `grok_image` / `openai_hazel` / `z_image` | autres moteurs | au cas par cas |
| `text2image_soul_v2` / `soul_cinematic` / `soul_location` | **Soul** = perso cohérent (ancrer un visage), décors | cohérence personnage |
| `marketing_studio_image` | visuels pub/ads brandés | marketing |
| `nano_banana*` | Google Gemini | **ÉVITER (Google)** |

## ✨ IMAGE — édition / amélioration
`topaz_image` / `bytedance_image_upscale` (upscale HD) · `image_background_remover` ·
`outpaint` (étendre le cadre) · `nano_banana_2_skin_enhancer` · `nano_banana_2_ai_stylist` ·
`color_grading_lut` (étalonnage) · `nano_banana_2_shots` (variations de plans).

## 🎬 VIDÉO — génération (image/texte → vidéo)
| Modèle | Pour quoi | Note |
|---|---|---|
| `seedance_2_0` (Seedance 2.0) | **DÉFAUT** — vrai mouvement cinéma | ByteDance, non-Google ⭐ |
| `kling3_0` / `kling2_6` | mouvement top, alternative | non-Google ⭐ |
| `wan2_7` / `wan2_6` | très bon mouvement, accepte `--audio` | non-Google |
| `minimax_hailuo` | dynamique, expressif | non-Google |
| `grok_video` / `grok_video_v15` | xAI | non-Google |
| `seedance1_5` | version précédente | — |
| `cinematic_studio_video_3_5` / `_v2` / `3_0` | rendu studio cinéma | — |
| `marketing_studio_video` | pubs vidéo brandées | marketing |
| `draw_to_video` | anime un croquis | créatif |
| `veo3_1` / `veo3` | Google | **ÉVITER (Google)** |

## 🗣️ VIDÉO PARLÉE (décision opérateur 2026-06-13 — DÉFINITIF)
**HeyGen = BANNI** (rendu nul sur le comic). Pas de lip-sync talking-head : Higgsfield ne fait pas d'audio-driven sur voix externe. Format unique pour TOUTE vidéo où Nova "parle" : actions/b-roll **bouche fermée** + sa **voix off ElevenLabs** par-dessus = rendu cinéma.

## 🎞️ VIDÉO — édition / amélioration
`topaz_video` / `video_upscale` (HD/4K) · `video_deflicker` · `reframe` (recadrage format réseaux) ·
`video_background_remover` · `bytedance_video_upscale`.

## 🔊 AUDIO
**Musique = `sonilo_music`** (Higgsfield, `--duration`, darkwave/synthwave 90s) — DÉFAUT (consolidation Higgsfield). `mirelo_text_to_audio`, `inworld_text_to_speech` aussi.
**VOIX = ElevenLabs** (voix Nova `FeJtVBW106P4mvgGebAg`) — la seule exception non-Higgsfield (identité vocale).

## 🧊 3D
`image_to_3d` / `multi_image_to_3d` · `sam_3_3d` · `3d_rigging` (rigger un mesh).

## 🧠 SPÉCIAL
- `brain_activity` (**Virality Predictor**) — prédit le potentiel viral d'un visuel. À utiliser AVANT de publier pour trier les meilleurs.
- `soul_cinema_studio` / `cinematic_studio_*` — chaînes studio cinéma.
- `marketing_studio_image` / `marketing_studio_video` — Marketing Studio (ads brandés).

## Comment je m'en sers (workflow type)
1. IMAGE perso cohérente → `gpt_image_2` (ou Soul) avec réf `nova-face-3angles.jpg`.
2. MOUVEMENT → `seedance_2_0` / `kling3_0` (strip l'audio modèle).
3. VIDÉO PARLÉE → actions bouche fermée + voix off ElevenLabs (pas de lip-sync, HeyGen banni).
4. MUSIQUE → `sonilo_music` (Higgsfield). VOIX → ElevenLabs Nova.
5. POST → upscale (`topaz_video`), recadrage (`reframe`), tri viralité (`brain_activity`).
Branding = `AGENTIK-BRAND.md`. Non-Google. Cinématographique.

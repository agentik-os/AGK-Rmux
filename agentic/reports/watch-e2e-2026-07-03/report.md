---
source: https://www.youtube.com/watch?v=dQw4w9WgXcQ
title: Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)
duration: 03:33
watched_at: 2026-07-03T03:52:09.180062+02:00
intent: analyze the hook and pacing, OmegaOS E2E verification of the captions path
hero_frames: [frame_0001.jpg, frame_0014.jpg, frame_0027.jpg, frame_0040.jpg, frame_0056.jpg]
transcript_source: captions
---

# Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)

## TL;DR

- E2E verification: the captions path worked end to end for the MAIN transcript. It came from native YouTube captions (43 segments) and the main Whisper fallback was correctly skipped.
- **BUG FOUND: `--no-whisper` does not cover the hook microscope.** watch.py passes `backend=None, api_key=None` when the flag is set, but `analyse_hook` in hook.py treats `None` as "not provided" and calls `load_api_key()` itself, so the first 10s of audio (`hook_audio.mp3`) was extracted and uploaded to the Whisper API anyway. The skill doc's "pass --no-whisper to guarantee zero egress" claim is currently false.
- The hook is a pure in-medias-res performance open: no lyric for the first ~18 seconds. The intro cuts on the synth riff between anticipation details (tapping shoe at the mic stand base, jacket flick, step up to the mic).
- Pacing is textbook MTV-era: 19.15 cuts/min, median shot 2.02s, cuts locked to the beat. Fast feel is achieved by rotating four fixed sets (window-scrim stage, night brick arches, daytime chain-link fence, dance-hall runway + bar) rather than new locations.
- One pacing valley: the longest gap between detected shot changes spans 02:25 to 02:59, across the instrumental bridge.
- The "[9.86s] Thanks for watching" in the hook word-level transcript is a Whisper hallucination on instrumental music from that unintended API call: neither video.en.vtt nor video.en-orig.vtt contains those words (grep verified). Also note the auto-captions have heavy line-overlap duplication, normal for YouTube.

## Key moments

- **[00:00] Cold open**: black frame, then a shoe tapping on the beat at the base of the mic stand (frame_0001, hook frames 1-2).
- **[00:02] Identity reveal**: first full shot of Rick Astley at the vintage mic under the gothic-window shadow scrim (frame_0004).
- **[00:07] Set-rotation teaser**: quick cuts introduce the night brick-arch set in trench coat, the blonde dancer, and the chain-link fence set before any vocal (frames 0005 to 0007).
- **[00:21] First verse lands**: "We're no strangers to love" over the fence-set close-ups (frame_0008 to 0010).
- **[00:43] First chorus**: "Never going to give you up", cutting accelerates and starts alternating all four sets per line (frames 0016, 0017).
- **[00:51] Bartender character enters** at the bar set, the video's secondary performer (frame_0019).
- **[01:18] Dance-break stretch**: bartender flair plus runway dancing, the densest cutting run of the video (hero frames frame_0027, frame_0040).
- **[02:10] Shadow choreography**: dance shots done entirely as shadows on pavement for visual variety (frames 0049, 0050).
- **[02:25] Pacing valley**: longest measured gap between shot changes, 02:25 to 02:59, over the instrumental bridge (frames 0056 to 0057).
- **[03:11] Final chorus loop**: all sets recycled to the fade-out (frames 0062 to 0068).

## Hook microscope (0-10s)

- Frames: 20 at 2 fps
- Word-level transcript (3 words):

```
  [  9.86s] Thanks
  [  9.90s] for
  [  9.92s] watching
```

Frame-by-frame (2 fps, 0.0s to 9.5s): 0.0 to 1.0s holds on a black shoe tapping the floor next to the mic-stand base, in white trousers, cut exactly on the drum-machine hits. Around 1.5s a close detail of the striped shirt and a hand flicking the jacket. From 2.0s to 4.5s the camera reveals Rick stepping up to the vintage mic in the wide window-shadow scrim set. From 5.0s to 9.5s the video alternates micro-poses at the mic with one-beat teasers of the other sets (night arches, blonde dancer), all still instrumental.

No words are actually sung in the first 10 seconds; the 3-word "Thanks for watching" at 9.86s is a Whisper hallucination on instrumental music, produced by a hook-microscope API call that should have been disabled by `--no-whisper` (see TL;DR bug note). Hook pattern: **in-medias-res performance / groove-first hook**. Attention is held by rhythm-synced body-language details (tap, flick, step) that promise a performance, identity is revealed at ~2s, and the first lyric is withheld until ~0:18, by which point the visual grammar of the whole video (four sets, on-beat cuts) has already been taught.

## Editorial profile

- Shots: 68
- Cuts/min: 19.15
- Mean shot length: 3.13s
- Median shot length: 2.02s
- Talking-head ratio: n/a (opencv not installed)

MTV-era performance clip: on-beat ~2s cuts rotating four fixed sets, close-up-heavy, dance-move and shadow inserts for variety, zero on-screen text.

## Quotable moments

- [00:43] "Never going to give you up. Never going to let you down."
- [00:22] "We're no strangers to love. You know the rules and so do I."
- [00:30] "You wouldn't get this from any other guy."
- [02:21] "Your heart's been aching, but you're too shy to say it."
- [02:32] "We know the game and we're going to play it."

## Entities mentioned

- People: [[rick-astley]]
- Companies: none mentioned in the video or transcript
- Tools / products: none mentioned

## Concepts surfaced

- groove-first hook: opening on rhythm-synced physical details (tapping shoe, jacket flick) instead of words, so the beat itself is the retention device.
- on-beat cutting: median 2.02s shot length locked to the song's tempo; the edit is the percussion.
- set rotation: four fixed sets recycled in quick alternation to simulate variety and speed at low production cost.
- rickroll: this video is the internet's canonical bait-and-switch meme, so any hook analysis of it is culturally overloaded; its modern view count measures the meme, not the hook.

## Transcript

_Source: captions._

```
[00:22] We're no strangers to love. You know the rules and so do
[00:26] love. You know the rules and so do I. I feel commitments from what I'm
[00:29] I. I feel commitments from what I'm thinking
[00:30] thinking of. You wouldn't get this from any other
[00:34] of. You wouldn't get this from any other guy. I just want to tell you how I'm
[00:40] guy. I just want to tell you how I'm feeling. Got to make you understand.
[00:43] feeling. Got to make you understand. Never going to give you up. I'm going to
[00:46] Never going to give you up. I'm going to let you down. I'm going to run around
[00:49] let you down. I'm going to run around and desert you. I'm going to make
[00:53] and desert you. I'm going to make you say goodbye.
[00:56] you say goodbye. Tell a lie and hurt
[01:00] Tell a lie and hurt you. We've known each other for so
[01:04] you. We've known each other for so long. Your heart's been aching, but
[01:07] long. Your heart's been aching, but you're too shy to say we don't know
[01:11] you're too shy to say we don't know what's been going
[01:13] what's been going on. We know the game and we're going to
[01:18] on. We know the game and we're going to play. If you ask me how I'm
[01:21] play. If you ask me how I'm feeling, don't tell me your truth. to
[01:25] feeling, don't tell me your truth. to see. I will give
[01:32] you around
[01:44] goodbye and hurt you. I will give [Music]
[01:47] [Music] you
[01:49] you around you. I will leave you. I will say
[02:10] Never going to give. Never going to give.
[02:13] give. Never going to give. Never going to
[02:14] Never going to give. Never going to give.
[02:17] give. We've known each other for so long.
[02:21] We've known each other for so long. Your heart's been aching, but you're too
[02:24] Your heart's been aching, but you're too shy to say it. We both know what's been
[02:28] shy to say it. We both know what's been going
[02:29] going on. We know the game and we're going to
[02:32] on. We know the game and we're going to play it. I just want to tell you how I'm
[02:38] play it. I just want to tell you how I'm feeling. Got to make you understand.
[02:41] feeling. Got to make you understand. Never going to give you up. Never going
[02:44] Never going to give you up. Never going to let you down. Never going to run
[02:47] to let you down. Never going to run around and desert you. Heat.
[02:59] [Music] Heat. Heat. Heat.
[03:11] [Music] Never going to tell a lie and hurt you.
[03:15] Never going to tell a lie and hurt you. Never going to give you up. Never going
[03:18] Never going to give you up. Never going to let you down. Never going to run
[03:21] to let you down. Never going to run around and desert you. Never going to
[03:24] around and desert you. Never going to make you cry. Never going to say
[03:27] make you cry. Never going to say goodbye. Never going to say goodbye.
```

## All frames

_Total: 68. Hero frames flagged with star._

* `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0001.jpg` (t=00:00)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0002.jpg` (t=00:00)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0003.jpg` (t=00:02)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0004.jpg` (t=00:02)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0005.jpg` (t=00:07)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0006.jpg` (t=00:09)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0007.jpg` (t=00:11)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0008.jpg` (t=00:18)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0009.jpg` (t=00:23)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0010.jpg` (t=00:29)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0011.jpg` (t=00:31)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0012.jpg` (t=00:33)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0013.jpg` (t=00:38)
* `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0014.jpg` (t=00:40)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0015.jpg` (t=00:42)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0016.jpg` (t=00:44)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0017.jpg` (t=00:44)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0018.jpg` (t=00:48)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0019.jpg` (t=00:51)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0020.jpg` (t=00:54)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0021.jpg` (t=00:56)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0022.jpg` (t=00:57)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0023.jpg` (t=01:01)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0024.jpg` (t=01:13)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0025.jpg` (t=01:16)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0026.jpg` (t=01:17)
* `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0027.jpg` (t=01:18)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0028.jpg` (t=01:20)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0029.jpg` (t=01:20)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0030.jpg` (t=01:22)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0031.jpg` (t=01:29)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0032.jpg` (t=01:30)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0033.jpg` (t=01:31)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0034.jpg` (t=01:33)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0035.jpg` (t=01:34)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0036.jpg` (t=01:36)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0037.jpg` (t=01:37)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0038.jpg` (t=01:38)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0039.jpg` (t=01:40)
* `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0040.jpg` (t=01:41)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0041.jpg` (t=01:43)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0042.jpg` (t=01:46)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0043.jpg` (t=01:48)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0044.jpg` (t=01:50)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0045.jpg` (t=01:51)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0046.jpg` (t=01:55)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0047.jpg` (t=02:00)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0048.jpg` (t=02:03)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0049.jpg` (t=02:10)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0050.jpg` (t=02:14)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0051.jpg` (t=02:15)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0052.jpg` (t=02:18)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0053.jpg` (t=02:20)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0054.jpg` (t=02:21)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0055.jpg` (t=02:24)
* `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0056.jpg` (t=02:25)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0057.jpg` (t=02:59)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0058.jpg` (t=03:01)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0059.jpg` (t=03:04)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0060.jpg` (t=03:07)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0061.jpg` (t=03:10)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0062.jpg` (t=03:11)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0063.jpg` (t=03:13)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0064.jpg` (t=03:15)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0065.jpg` (t=03:17)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0066.jpg` (t=03:19)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0067.jpg` (t=03:21)
  `/tmp/claude-1000/-home-vibe-Station-SideBusiness-OmegaOS/18bea271-e5e7-489f-bd4f-e6af8a2272cc/scratchpad/e2e/run1/frames/frame_0068.jpg` (t=03:23)

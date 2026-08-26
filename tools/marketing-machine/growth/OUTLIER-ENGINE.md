# Outlier Engine — predict the video before you build it

> Project-agnostic playbook baked into the marketing machine, distilled from the X article
> "How I Predict Viral Videos Before They Explode" (@0x_fokki, 2026-07-08) and **rebuilt on
> the VPS stack**: the article's four SaaS tools are replaced by what we already run.
> Fulfills capabilities.toml: `outlier-engine` (G11), `format-library` (G12),
> `publish-and-learn` (G13). Pairs with `/watch` (G10) and the `pattern-ledger` (B7).
> HARD RULE R-NODASH: never an em-dash in any copy. Human voice.

---

## 0. The model

Most channels die the same way: pick a topic you like, make a video you think is good, get
400 views, quit. The editing was never the problem. **The input was.**

A viral video is **a format that already worked, pointed at a fresh subject.**
The format is the asset. The subject is disposable.

The window on any format is about **2 weeks**. Then everyone floods it and it dies.
Speed is the edge, and speed comes from never guessing.

So the loop runs backwards from how most people work: **find the format that is already
winning, feed it a fresh subject, then build.**

---

## 1. The stack swap (why this file exists)

The article runs on Claude + CapCut + Make + Google Sheets, about $50/month. Every one of
those has a native equivalent here, and the swap is not a downgrade: it removes two paid
SaaS, the browser, and the manual copy-paste between them.

| Article | Ours | Why |
|---|---|---|
| Google Sheet swipe feed | `bin/outlier` + `00-context/swipe/swipe.json` | Scriptable, diffable, no browser, feeds the other skills directly |
| Manual view logging | `yt-dlp --flat-playlist` on `/shorts` | Free, keyless, ~0.7s per channel, verified live |
| "Read the first 2 seconds" by hand | `/watch <url>` (G10) | Already does a 0-10s hook microscope at 2fps plus word-level transcript |
| Claude Pro chat, copy-paste | The prompts in §4, run in-session | The swipe JSON is already on disk, no paste |
| CapCut Pro | `higgsfield generate` (VD1) + `hyperframes` (VD3) + Kokoro/ElevenLabs TTS (A1/A4) | Higgsfield makes the frames and clips, hyperframes does captions, titles, cuts, export |
| Recurring characters as CapCut assets | `higgsfield-soul-id` (U1) + `06-branding/avatar-persona.md` (B4) | A trained Soul is identity-faithful across every clip, not a per-project asset bin |
| Make scenario | `bin/outlier` in cron + `omega-zernio post` (P1) | Same trigger-to-publish loop, no 1000-op ceiling |
| Telegram ping from Make | `omega-alert-send.sh` (the alert funnel) | `outlier report --alert` lands in the Alerts topic, per the routing doctrine |
| "Write the score back into the sheet" | `06-branding/pattern-ledger.md` (B7) | The ledger already carries the promote/retire rule. The outlier score is the number it was missing |

**Cost: $0/month in new spend.** Higgsfield and ElevenLabs are already paid for. The research
half, which the article calls 80% of the result, is entirely free.

---

## 2. Step 1, the research engine (this is the whole game)

This step is 80% of the result. Skip it and the other steps produce polished videos nobody
watches. Do not ask "what should I post." Ask **"what is already outperforming, and why."**

### Build the swipe feed

Track 40 faceless channels in one lane, AI stories, history facts, motivation edits, pet
clips, whatever you ship. One `@handle` per line, `#` comments allowed:

```bash
cat > marketing/00-context/swipe/channels.txt <<'EOF'
@zackdfilms
@MrBeast
# add the channels that keep surfacing in `outlier discover`
EOF

bin/outlier scan --channels marketing/00-context/swipe/channels.txt --limit 40
```

### Score the outliers

An outlier is a video pulling far more views than **that channel's own average**:

```
outlier score = video views / channel median views
```

A channel that medians 20,000 views drops a video at 900,000. Score of 45. That video found
something.

```bash
bin/outlier score            # >=10 is a signal, >=30 is a format you copy this week
bin/outlier score --json     # pipe it into a skill
```

**Ignore the raw view count.** A 2M-view video on a 5M-median channel taught you nothing. A
900k-view video on a 20k-median channel is a map.

### Find lanes you do not track yet

```bash
bin/outlier discover "POV you are the last human on earth" --n 20
```

Discovery ranks by raw views and reports no score, because a search result has no channel
median attached. It exists to tell you **which channels to add to `channels.txt`**. Score
them properly on the next scan.

### Read the first 2 seconds

The hook does most of the work. For every COPY-NOW row, log two things: the exact first
frame, and the first line of on-screen text. That is what stopped the scroll. The rest of
the video only has to not disappoint.

```bash
/watch https://youtube.com/shorts/<id>     # 0-10s microscope at 2fps + opening transcript
```

### Honest limits of the data source (L1, verified live 2026-07-09)

- The `/shorts` tab reports **rounded** views (`45000000`, not `45123456`). The ratio survives
  the rounding. `discover` returns exact counts, because search does not round.
- The tab carries **no upload date**. Recency is approximated by position, since the tab is
  reverse-chronological. If you need real dates, you need a paid source.
- **YouTube Shorts only.** The channel-videos tab stopped returning view counts, and per-video
  extraction is blocked from this VPS by YouTube's bot check.
- **TikTok and Instagram need a paid source.** `SCRAPECREATORS_API_KEY` exists but the account
  is **out of credits** (HTTP 402, verified 2026-07-09). Apify is live (STARTER, ~$29/cycle,
  effectively unused) and has actors for both. Neither is wired into `bin/outlier` yet.
  Run `bin/outlier doctor` to see the live state.

---

## 3. Step 2, formats that keep hitting

The same shapes surface every month. Log them once, reuse forever. Separate **topic** from
**format**: topics die in a week, formats repeat for months.

Worked example. "POV: you're the last human on Earth" hits. The topic is loneliness. The
format is `POV: you're the last X in Y`. That reskins into 200 videos: last knight, last
barista, last dragon, last dial-up user. One outlier, a season of content.

| Format | The shape | Why it works |
|---|---|---|
| `POV: you're the [role] when [event]` | Viewer is inside the scene in one line | Identity plus stakes, zero setup |
| `What if [familiar thing] but [twist]` | One-line premise, no context needed | Pure curiosity gap |
| The invisible-force gag | Objects move on their own, chaotic reveal at the end | Confusion held, then paid off |
| Oddly satisfying process | Start-to-finish build or transformation, no talking | Completion compulsion, no language barrier |
| `1 vs escalation` | 1 vs 100, level 1 vs level 100, small stake to huge stake | Escalation promises a payoff you must wait for |
| The creature reveal | A normal scene, the last 2 seconds flip it | The whole video is a setup for the last beat |
| `Day in the life of [unexpected subject]` | A rock, a medieval peasant, a deep-sea fish | Familiar frame, absurd subject |

Each of these carried a different subject to millions. **The shape is the reusable part. Your
subject is the variable you swap in.**

When `outlier score` surfaces a shape that is not in this table, add a row. This table is the
format library, and it is the durable half of the machine.

---

## 4. Step 3, Claude writes to the format

Three prompts. The swipe JSON is already on disk, so paste the path, not the data.

### Pattern extraction

```
Here are 20 outlier videos, each with its outlier score, channel median, and title:
<paste `bin/outlier score --json`>

Extract the FORMAT, not the topic. For each cluster you find:
- the reusable shape, written as a template with [variables]
- the first-frame image and the first line of on-screen text that stopped the scroll
- why the shape works psychologically, in one line
- how many of the 20 share it
Ignore subjects entirely. I want skeletons I can reskin.
```

### Idea generation

```
Format: <the winning template>
My subject lane: <e.g. medieval history, AI tools, deep-sea creatures>
My brand voice: <read 06-branding/SOCIAL-BRAND-BOOK.md>

Generate 30 concepts on this ONE format, my subject.
For each: the title, the exact first line of on-screen text, and the last-2-seconds payoff.
Kill any concept whose hook needs context to land.
```

### Script for shots, not prose

```
Concept: <the picked concept>
Format: <the template>
Target: 9:16, <n> seconds, <platform>

Write a SHOT LIST, not a script. For each shot:
- duration in seconds
- what is on screen, as a Higgsfield image or video prompt
- on-screen text, verbatim
- VO line, verbatim, or "none"
Question me on any gap in the concept before you write. Do not pad to fill time.
```

The output is a shot list you can build directly, and the shot prompts drop straight into
Step 4. Read `06-branding/prompt-library/kill-list.md` before generating: no em-dash, ever.

---

## 5. Step 4, build it on our stack

CapCut becomes three tools we already run and already pay for.

1. **Frames and clips** — `higgsfield generate create seedance_2_0 …` for motion,
   `nano_banana_2` for stills. Pass `--soul-id` (U1) so a recurring character stays the same
   face across the whole series. Cast once, change the plot, ship forever.
2. **Voice** — `hyperframes-cli tts` for local Kokoro (free), ElevenLabs for the branded VO
   when the series has a named voice (A1).
3. **Assembly and captions** — `hyperframes-cli render` for the cut, title cards, burned
   captions, and the 9:16 export. `hyperframes-cli transcribe` (Whisper) makes the captions
   from the VO, so on-screen text and audio never drift.

Everything reads `06-branding/tokens.json` (B2) first, so a reskinned format still looks like
the brand and not like the channel you copied it from.

---

## 6. Step 5, publish and let the loop learn

The research loop only compounds if your own results feed back into it. This is the half that
makes it a machine instead of a swipe file.

The cron scenario, replacing Make:

```bash
# 1. publish the finished vertical
omega-zernio post <slug> --text "<title>" --platforms tiktok,youtube,instagram --media ./out.mp4

# 2. 48h later: re-scan your OWN channel, score your own videos against your own median
bin/outlier scan --channels <(echo "@your_handle") --out marketing/00-context/swipe/self.json
bin/outlier score --in marketing/00-context/swipe/self.json --min 3

# 3. weekly: what won, straight to the Alerts topic
bin/outlier report --alert
```

Then write the result into **`06-branding/pattern-ledger.md` (B7)**, tagged with the format
name from §3. The ledger already carries the governing rule, and the outlier score is the
number it was missing:

- a format that scores **>=10x your own median** is a **winner**: promote it into
  `templates/` and `prompt-library/`, and ship a follow-up within 48h
- a format that **fails twice** is **retired**, do not regenerate it

Now your own hits are inputs. The formats that work **for your account** rise to the top.
The factory learns. Setup is a few hours once, then it runs.

---

## 7. The economics, ours

The article reports $11,900/month against $50/month of tooling. Our tooling cost for this loop
is **$0 in new spend**, since Higgsfield, ElevenLabs and zernio are already running and the
research half is free.

The five revenue paths it names, ranked by how well they fit what we already have:

1. **The research service.** The most valuable output is the swipe sheet itself. `outlier report`
   already produces the digest. 40 subscribers at $29/month is $1,160/month for research the
   machine does for itself anyway. Closest fit, lowest marginal cost.
2. **Format and prompt packs.** The §3 library plus the §4 prompts, sold as a kit. Sells indefinitely.
3. **Done-for-you shorts.** $80 to $250 per short, one intake form. Pipeline cost per short is
   pennies once the shot list exists.
4. **Platform payouts.** 3 to 6M monthly views clears roughly $2,000 to $6,000. Requires a
   channel with real cadence, so it is the slowest to start.
5. **Faceless channel sales.** A proven format plus 100k followers sells for $3,000 to $10,000.
   Downstream of 4.

Do not present the article's month-by-month curve as our forecast. It is one operator's
reported result, not a projection we have evidence for.

---

## 8. The bottom line

Most people make the video first and hope it finds an audience. This runs the other way.

Find the format that is already winning. Feed it a fresh subject. Then build.

The research does the heavy lifting, and on this VPS the research is free.

# Growth Kit — the mechanics, made runnable

> Project-agnostic playbook baked into the marketing machine, distilled from the X teardown
> (@tibo_maker, @antinertia) and the multi-platform analysis (IG, YouTube, TikTok, LinkedIn).
> Fulfills capabilities.toml: hook-engine, carousel-system, comment-to-DM, series, build-in-public,
> platform-protocols. Every skill/engine reads this before producing content.
> HARD RULE R-NODASH: never an em-dash in any copy. Human voice.

Verba is used as the running example. Swap in any project's 06-branding voice.

---

## 1. Hook engine (per platform)

The hook is 80% of the result. Answer-first, no throat-clearing.

**YouTube (0-5-15-30):** 0-5s state the payoff, 5-15s raise the stakes, 15-30s show the plan.
- Templates: "How I <result> without <expected cost>." · "I tried <thing> for <time>. Here is what nobody tells you." · "The <n> <things> that <outcome>." · "Stop doing <common thing>. Do this instead." · "<Surprising number> reasons <belief> is wrong."
- Verba: "I have not typed a prompt to Claude Code in a week. Here is the setup."

**TikTok / Reels (0-2s, answer-first):** the value or the surprise in the first two seconds, no logo intro.
- Templates: "Your <tool> is <bad thing> right now." · "Here is how to <result> in <n> seconds." · "POV: you <situation>." · "Nobody talks about this <thing>." · "<Do X>. Watch what happens."
- Verba: "Your dictation app is uploading your voice right now."

**LinkedIn (sub-10-word contrarian or data line):** first line is the whole bet, it must earn the "see more".
- Templates: "<Common practice> is dead." · "<Metric> beats <metric>. Here is why." · "I was wrong about <thing>." · "Most <role> get <thing> backwards." · "<Number>. That is the only metric that matters."
- Verba: "Your voice should not just type. It should act."

**X thread openers:** promise a payoff + a numbered path.
- "You can <do X> fine. Doing <X at scale> is where it breaks. 🧵" · "I <did thing>. Here is the exact breakdown."

Rule: generate 20-50 hook variants per month, same body, kill the losers fast, scale winners into paid.

---

## 2. Carousel system (IG + LinkedIn, the format we were missing)

Carousels win saves + dwell on both platforms. One system, two outputs.
- **Canvas:** 4:5 (1080x1350), 8-10 slides. Design on the 06-branding tokens.
- **Grid + safe zone:** locked margins, one idea per slide, big type, arm's-length legible.
- **Cover:** an incomplete promise (open loop) that forces slide 2. Named framework beats generic ("The 3-file setup", not "tips").
- **Slide archetypes (6):** Cover/Hook · Big-Stat · Step · Quote · Chart · CTA.
- **Seamless bleed** between slides (a visual thread) so swiping feels continuous.
- **Numbered slides** (1/8) so people know the length and finish it (swipe-completion is the ranked signal).
- **Render:** HTML template -> Playwright screenshot at 1080x1350 (per 06-branding/templates/stills), or pdfgen for a LinkedIn PDF carousel. Same content, both channels.
- Verba example series-cover: "How I made my Mac do the task, not just type it. (swipe)"

---

## 3. Comment-to-DM funnel (the demo-and-capture loop, the Arcads move)

The single highest-leverage play from @antinertia. A working demo + a keyword CTA that captures warm leads AND manufactures first-30-minute velocity (comments + follows rank the post).
- **Format:** a native video that SHOWS the product doing the thing (working demo > opinion), ending on: "Comment <KEYWORD> and I will send you the exact setup."
- **Loop:** keyword in comments -> auto-DM the lead magnet (the recipe / workflow / template) -> capture into the owned list.
- **Tooling (R-CLI, not ManyChat):** a lightweight CLI/API watcher on the post comments that DMs the keyword responders. Plugs into the UGC avatar (the Dev cool delivers the CTA) and zernio.
- **Verba keyword:** VERBA -> DM a short "the dictation-to-agent setup" (on-device, confirm-gated) + a trial link.
- Velocity rule: reply to every comment in the first hour. The first hour decides the reach.

---

## 4. Build-in-public engine (the Tibo system)

Building in public is not the promo, it IS the growth engine. Tibo went 0 -> 115k by building Tweet Hunter in the open.
- **Post the real:** what shipped, what broke, what you learned, screenshots, receipts.
- **Revenue / metric transparency:** specific numbers = instant credibility ("week 14", MRR, users, a churn number). Vague = ignored.
- **Dogfooding as marketing:** the product is used to make the content about the product (Verba writes the Verba posts, on camera).
- **One owned list:** every launch reuses the same newsletter/list. Attention is rented, the list is owned.
- **Cadence:** high volume. 3x/day on X, threads as the format the algorithm pushes.
- Verba/OmegaOS/CAIO: ship notes, "here is what I automated this week", honest comparisons (we do not win every row).

---

## 5. Series formats (named, recurring, episodic)

Turn content pillars into named series with Part 1/2/3 so people come back and binge.
- Label episodes explicitly ("Voice-agent setup, Part 2/5") across Shorts, TikTok, YouTube.
- Verba example: "Talk to your Mac" series: 1 dictation basics, 2 it runs the task, 3 the privacy setup, 4 JARVIS confirm, 5 the full workflow.

---

## 6. Platform protocols (the daily checklist the engine + operator follow)

- **Instagram:** Stories 50/30/20 (value/behind-the-scenes/promo), a sticker every frame, 24h story reset, feed carousels + Reels. Optimize for sends-per-reach.
- **LinkedIn:** post from the FOUNDER personal profile (561% reach vs company page), golden-hour posting, comment-first loop (seed the discussion), document/carousel posts, sub-10-word hooks. Optimize for dwell time. Feed the newsletter.
- **TikTok:** answer-first hook, native + trend/sound aware, first-comment pin (extra context/CTA), reply in the first hour. Optimize for rewatch + completion.
- **YouTube:** packaging first (design title + thumbnail together, 3-5 variants, One Face / One Object / One Question), retention editing, Shorts as top-of-funnel. Optimize for CTR x avg-view-duration.
- **Everywhere:** 3-5 posts/week minimum is itself an algorithm input. Retire vanity metrics (likes); track the costly signals (saves/sends, dwell, rewatch, completion).

Pre-ideation ritual: run /watch on 3-5 competitor outliers to reverse-engineer their packaging + hooks before writing.

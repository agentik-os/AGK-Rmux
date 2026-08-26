# Stax mobile shell — hard requirements (enforce on every migration)

A Stax app MUST pass these on a REAL phone (headless Playwright has no notch, so it
CANNOT verify safe-area — check on device or trust the CSS floors):

1. **Top app bar clears the OS status bar** (clock / 5G / battery). Use a solid bar
   background + `padding-top: max(env(safe-area-inset-top, 0px), 14px)` — the 14px FLOOR is
   mandatory because Android/PWA-standalone often reports `safe-area-inset-top: 0`. Never a
   translucent top bar over the status-bar zone.
2. **Bottom tab bar never wraps.** Labels: `white-space: nowrap; overflow: hidden;
   text-overflow: ellipsis; font-size ~9px`. `padding-bottom: max(env(safe-area-inset-bottom,0px), 6px)`.
3. **No horizontal overflow.** `html, body, .stax-root { overflow-x: hidden }`; every wide
   block (tables, code, diagrams) scrolls inside its own container.
4. **Secondary / destructive actions live in the slide menu** (the ☰ bottom sheet), not the
   tab bar: e.g. export/download, reset, theme, settings → a "Réglages" section. The bottom
   tab bar holds only the primary destinations (the Spaces + the main view).
5. **The primary action buttons are preserved and clean** — the panel footer CTA (accent),
   the composer send (accent, flush-right). No stranded/mis-aligned buttons; no floating.
6. **Bottom sheet clears the home indicator**: `padding-bottom: max(calc(14px +
   env(safe-area-inset-bottom,0px)), 20px)`.
7. **PushHost** = one full-width panel back-stack; a top bar with back · title · ☰ menu.

`viewport-fit=cover` MUST be set (Next: `export const viewport = { viewportFit: "cover" }`)
or the insets are 0 everywhere. Reference implementation of these rules: the one-life-os
shell (`components/stax-shell.tsx` + the mobile blocks in `app/globals.css`).

# UI Patterns Reference — What Great Apps Do

## Reference Apps Deep Dive

### Linear — The Gold Standard for SaaS Dashboards
- **Spacing:** Religious 8px grid. Every gap is intentional.
- **Typography:** 3 sizes max per view. Heavy use of text-sm for density.
- **Color:** Dark-first design. Minimal color — gray + one accent.
- **Components:** Every list item is identical. Zero variation.
- **Density:** Dense but breathable. Keyboard-first navigation.
- **Motion:** Subtle, fast (150ms). No unnecessary animation.
- **Key lesson:** Restraint. Less is more. Every pixel earns its place.

### Discord — Dense UI That Feels Effortless
- **Spacing:** Compact but consistent. sidebar items are all h-8.
- **Typography:** Clear hierarchy despite density. Timestamps in text-xs.
- **Color:** Role-based colors (online green, idle yellow, DND red).
- **Components:** Message layout is pixel-perfect across millions of messages.
- **Key lesson:** Consistency at scale. Same component rendered 1000x looks intentional.

### Warp — Reimagined Terminal
- **Spacing:** Generous for a terminal. Command blocks have clear separation.
- **Typography:** Monospace + sans-serif mixed deliberately.
- **Color:** Dark theme with vibrant accents. High contrast.
- **Components:** Block-based UI, clear visual grouping.
- **Key lesson:** You can make ANY UI beautiful with systematic design thinking.

### Claude (Anthropic) — Warm, Clean, Professional
- **Spacing:** Generous. Conversation messages have significant breathing room.
- **Typography:** Clean sans-serif. Perfect line-height for readability.
- **Color:** Warm neutrals. Subtle accent colors. Never harsh.
- **Components:** Minimal UI chrome. Content is king.
- **Key lesson:** Warmth through whitespace and typography, not decoration.

### Vercel — Dark Mode Perfection
- **Spacing:** Precise grid. Dashboard cards are perfectly aligned.
- **Typography:** Geist font. Clean, modern, technical.
- **Color:** Pure black backgrounds with white text. Minimal accent.
- **Components:** Every table, every card follows the exact same spec.
- **Key lesson:** Monochrome can be stunning. Color should be information, not decoration.

## Common Dashboard Patterns

### Page Layout
```
[Sidebar 256px] [Main Content (flex-1)]
                [Header 56px - page title + actions]
                [Content Area (p-6 or p-8)]
                  [Stat Cards Grid (gap-4)]
                  [Data Table Section (mt-8)]
                  [Charts Section (mt-8)]
```

### Settings Page Layout
```
[Sidebar 256px] [Main Content]
                [Header - "Settings"]
                [Secondary Nav - tabs or sidebar]
                [Settings Content]
                  [Section Title + Description]
                  [Form Fields (space-y-4)]
                  [Save Button (sticky bottom or inline)]
```

### Detail/Edit Panel Pattern
```
[Main List/Table] [Side Panel 400px]
Click item ->     [Panel slides in from right]
                  [Panel Header: title + close X]
                  [Panel Content: scrollable]
                  [Panel Footer: actions (sticky)]
```

## Anti-Patterns to Avoid

| Anti-Pattern | Why It's Bad | Fix |
|-------------|-------------|-----|
| Rainbow dashboard | Too many colors = visual noise | Max 3 accent colors + grays |
| Giant buttons on data pages | Wastes space, looks amateur | Use sm/md buttons in data contexts |
| Card soup | Everything in a card = nothing stands out | Use cards for grouped content, not everything |
| Modal for everything | Context-switching fatigue | Use panels for detail views, modals only for confirmations |
| Center-aligned content | Hard to scan, looks like a landing page | Left-align data, labels, text |
| Inconsistent empty states | Some say "No data", others blank | One empty state component everywhere |
| Too many font weights | Bold everywhere = nothing is bold | Max 3 weights per page |
| Colored backgrounds for sections | Looks like a children's app | Use borders or subtle bg changes |

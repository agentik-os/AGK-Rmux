# Advanced chat and agent interaction system

## Contents

1. Product laws
2. Composer OS
3. Menus and command palette
4. Conversation tree and message lifecycle
5. Agent execution rendering
6. Artifacts and side surfaces
7. Navigation, projects, and suggestions
8. Mobile transformation
9. Technical interaction contracts
10. Build order and definition of done

Use this reference for chat-first products, copilots, agent workspaces, or any product where natural language coordinates actions. Adapt names to the product; preserve behavioral invariants.

## 1. Product laws

1. The composer is the command surface, not merely a text input. Anything discoverable in a menu should have a fast keyboard path when safe.
2. Context is visible and removable. Never send hidden files, project instructions, modes, sources, or prior selections.
3. Replace confirmation with undo for reversible local actions. Confirm consequential external writes, payments, publication, access changes, or irreversible destruction.
4. Branch instead of overwrite. Editing or regenerating preserves prior results.
5. Stream typed events and persist them. Never infer execution state from a spinner or a collection of booleans.
6. Let the user read. Suspend forced auto-scroll when the user leaves the live edge.
7. One interaction registry owns commands, menus, shortcuts, availability, analytics, and permission checks.

## 2. Composer OS

### 2.1 Anatomy

Use four optional regions in this order:

1. context chip rail;
2. editable input;
3. command/source/mode controls;
4. send/stop/voice actions.

Do not render empty chrome. The chip rail exists only when context exists. Grow the input to a bounded height, then scroll internally. Keep the primary send/stop action stable.

### 2.2 Context chips

Represent project, file, URL, quoted message, selected artifact range, pasted text/code, agent, mode, and connector as typed tokens. Each chip declares:

- stable ID and type;
- label and middle-truncation behavior;
- metadata such as pages, size, lines, source, freshness, or permission;
- preview behavior;
- removal and restoration behavior;
- serialization contract for the request.

Rules:

- Remove the last chip with Backspace only when the input is empty; announce removal accessibly.
- Preview on hover/focus without navigation.
- Warn before send when a chip is stale, unavailable, or permission-lost.
- Distinguish context sent verbatim, retrieved on demand, and referenced only by ID.
- Let the user inspect the final context manifest before sensitive requests.

### 2.3 Plus menu

Trigger with `+` and a registered upload shortcut. Group only real capabilities:

| Group | Typical entries |
| --- | --- |
| Local | File, screenshot/camera, clipboard |
| Connected | Connected Drive, Notion, Linear, GitHub, or product sources |
| Context | Add to project, prior conversation, saved collection |

Hide disconnected providers. Offer one “Manage connectors” action after a separator instead of listing disabled advertisements.

### 2.4 Slash commands

Open `/` only at the start of an otherwise empty composer, unless the product explicitly supports inline commands. Treat `/` inside text as text.

- Filter instantly; support arrows, Home/End, Tab/Enter, and Escape.
- Show name, single-line description, category, and shortcut.
- Insert argument slots as structured tokens and focus the first unresolved slot.
- Close after two unmatched characters and preserve typed text.
- Source skills, saved prompts, modes, and system commands from the command registry.
- Validate permissions and surface availability before executing.

### 2.5 Mentions

Open `@` at a token boundary, not in the middle of a word. Search project files, conversations, agents, connectors, people, and domain objects allowed by the product.

- Insert atomic tokens rather than editable character strings.
- Rank recent/frequent/relevant results before alphabetical fallback.
- Highlight fuzzy matches.
- Group results by type and permission.
- Keep a plain-text/accessible representation for copy, paste, and screen readers.

### 2.6 Paste and drop intelligence

| Input | Behavior |
| --- | --- |
| Long text beyond product threshold | Convert to a `Pasted text` chip with word count and preview |
| Image | Thumbnail chip; offer OCR only if available |
| URL alone | Fetch title/favicon asynchronously; expose “Read page” state |
| Code block | Detect language; show language and line count |
| File drop | Show drop target over the whole chat/work surface |
| Multiple files | Validate count/size/type individually; retain valid files when some fail |

Never allow a paste to explode input height. Preserve original bytes/text and show processing status per item.

### 2.7 Modes

Show at most three high-frequency modes inline; place the rest in an overflow menu. A mode changes both request semantics and the input placeholder.

Typical modes:

- `Think`: extended reasoning budget;
- `Search` / `Deep`: web or source research depth;
- `Image`: generation/edit request schema;
- product-specific planning, execute, review, or private modes.

Show cost, latency, data-access, or capability implications before use. Do not encode active state by color alone.

### 2.8 Model selection

Use a compact anchored menu rather than a modal. Each model entry includes a short purpose statement and material limits. Support `Auto` only when routing is real and observable.

- A manual model choice locks routing until the user changes it.
- A model change applies to the next turn and creates a subtle thread separator.
- Record the model actually used on each turn.
- Show fallback/escalation after the fact with reason when allowed.

### 2.9 Composer state machine

At minimum:

`empty | composing | attaching | validating | ready | sending | queued | running | stopping | error`

Define input editability, send/stop availability, shortcut behavior, and screen-reader announcements for each state.

## 3. Menus and command palette

### 3.1 Shared menu laws

- Maximum two navigation levels. Use a searchable dialog/page for deeper structures.
- Group more than seven items with section labels.
- Support arrows, Home/End, typeahead, Enter/Space, and Escape.
- Align shortcuts in a dedicated column.
- Put destructive actions last, separated, and visually distinct without relying on color alone.
- Use icons only when they improve scan speed.
- Target approximately 120 ms open and 80 ms close; respect reduced motion.
- Keep a menu anchored during scroll or close it deterministically.
- Use the same registry for `...`, context menu, command palette, and shortcuts.
- Never expose an enabled action that will certainly fail a permission check.

### 3.2 Menu inventory

Specify at least:

| Surface | Actions |
| --- | --- |
| Assistant message | Copy, markdown, regenerate intent, branch here, share turn, report |
| User message | Edit, copy, branch/delete from here as policy allows |
| Regenerate | Retry, another model, shorter, more detailed, alternative approach |
| Conversation | Rename, pin, duplicate, move, export, share, archive/delete |
| Account | Settings, personalization, connectors, shortcuts, billing, sign out |
| Text selection | Quote, ask about, explain, copy |
| Artifact | Versions, copy, download, publish, close |

Give rows and artifacts a custom context menu matching the visible `...` menu. Preserve native selection behavior where replacing it would reduce accessibility or platform affordances.

### 3.3 Command palette

Open instantly from a global shortcut and show local results before semantic/server results. Search conversations, projects, files, commands, settings, and domain objects.

- Prefix `>` searches commands first.
- Group results with counts.
- Show conversation excerpt and date.
- Let Tab reveal secondary actions without dismissing the palette.
- Empty query shows recent contexts and frequent commands.
- Keep list order stable while async results arrive; append/merge without focus jumps.
- Emit accessible result count updates without announcing every keystroke.

## 4. Conversation tree and message lifecycle

### 4.1 Tree model

Store messages with parent relationships and active-child selection. A linear transcript is one projection of a tree.

```text
Conversation
  rootMessageIds[]
  messagesById{}
Message
  id, parentId, childIds[], activeChildId
  role, contentParts[], status, model, createdAt
```

Editing a user message, regenerating, choosing another approach, or branching creates a child branch. Never overwrite the prior subtree.

### 4.2 Inline edit

- Replace the message body in place with an equal-width editor.
- Preserve attachments/context and make changes explicit.
- Submit with registered shortcut; cancel with Escape.
- Create a branch and show `previous / current / total` navigation.
- Restore scroll and selection when navigating branches.

### 4.3 Message actions

Reserve action-row height or overlay outside content bounds so actions do not cover text or cause layout shifts. Appear on hover, focus-within, or selection. On touch, expose an explicit action affordance.

### 4.4 Regeneration intents

Offer intent, not only “try again.” Examples: shorter, deeper, another approach, another model, correct with selected feedback. Each creates a branch and preserves provenance.

### 4.5 Truncation and continuation

Render `Continue` inline at the end of the same message. Append persisted content to the same logical message/branch. Explain whether continuation reuses the original model/context.

### 4.6 Scroll law

Maintain a live-edge state:

- follow streaming while the user remains near the bottom;
- stop following as soon as the user scrolls away;
- show a “return to latest” control with unread token/message signal;
- never snap the user to bottom on tool updates, source arrival, or branch navigation;
- preserve relative reading position when content above expands/collapses.

## 5. Agent execution rendering

### 5.1 Turn state machine

Use explicit server truth:

`idle -> queued -> thinking -> tooling -> streaming -> done | partial | error | stopped`

Allow `thinking <-> tooling` loops. Define terminal versus retryable errors. Never derive state from “has text” or `isLoading`.

### 5.2 Thinking block

- Expand while it is the active visible step when product policy permits.
- Collapse when answer streaming begins; remain inspectable afterward.
- Show a label and elapsed duration.
- Use readable secondary text, not long italics.
- Distinguish a user-facing progress summary from private chain-of-thought. Do not promise hidden reasoning disclosure.

### 5.3 Tool timeline

Group steps with count, total duration, and status:

```text
4 steps · 12 s
complete  Web search       query summary       1.2 s
complete  Read source      domain/title         0.8 s
running   Analyze file     report.pdf
pending   Draft response
```

Every active step has a specific label. An error row opens structured, actionable details. Collapse the group on completion while retaining the summary.

### 5.4 Deep research and sources

- Stream sources into a persistent source panel/tab.
- Show title, domain/favicon, access state, and reading status.
- Display a live count with careful semantics (“opened,” “read,” or “cited”).
- Link inline citations to exact sources/spans.
- Highlight the matching source on hover/focus and support keyboard navigation.
- Preserve sources with the final response and exports.

### 5.5 Clarification

Ask one compact set only when ambiguity materially changes the result. Use tappable/selectable choices with an editable “other” route where appropriate. Continue automatically after selection when the user expectation is clear.

### 5.6 Stop, retry, and reconnect

- Stop must cancel provider/tool execution when possible, not only hide output.
- Persist event sequence and partial text.
- Reconnect from last acknowledged event without duplication.
- Distinguish retry turn, retry failed tool, and resume interrupted stream.
- Show cost/consequence if retry repeats external work.

## 6. Artifacts and side surfaces

Create an artifact for autonomous, reusable, editable output—not for every long answer. Open its panel when created; never auto-close it.

Artifact contract:

- tabs appropriate to kind: preview, code, versions, sources;
- version selector with time, author/model, and summary;
- diff between versions;
- manual edits create marked user versions;
- selection actions add a reference chip to the composer;
- resizable width persisted as presentation preference;
- chat column recenters or changes host instead of becoming unusably narrow;
- publish/download actions follow external-write policy.

Use targeted patches for large artifacts. Full rewrite requires a restructuring reason and creates a new version.

## 7. Navigation, projects, and suggestions

### 7.1 Conversation navigation

Group by recent dates, pin separately, rename inline, search locally, and support move-to-project. Do not rely on dates alone for retrieval; command palette searches content.

### 7.2 Projects

- Project instructions and files are inspectable without leaving current work.
- Starting within a project preloads visible context chips.
- File metadata includes usage, freshness, and permission.
- Conversation-level overrides are explicit.

### 7.3 Long/shared conversations

- Offer a compact-and-continue/new-thread action before context degradation.
- Show what context transfers.
- Shared threads are read-only until forked.
- “Continue in my account/workspace” creates an attributed fork.

### 7.4 Suggestions

- Empty states show at most four context-derived suggestions.
- Post-answer follow-ups show at most three and disappear on typing.
- A clicked suggestion sends directly only when its consequences are read-only and obvious; otherwise populate for review.
- Never use “Do you want me to continue?” as ambient intelligence.

## 8. Mobile transformation

Do not compress the desktop shell.

- Use a single focused surface with semantic back/up behavior.
- Convert side panels to full-height pushed surfaces or modal sheets according to context and focus requirements.
- Keep composer controls within thumb reach and preserve safe-area insets.
- Use long-press/action buttons without erasing native copy/accessibility behavior.
- Make context chips horizontally scrollable with visible count/management entry.
- Keep a stop action available while the software keyboard is open.
- Preserve branch, source, tool, artifact, and reconnect states; change their host, not their meaning.
- Test orientation, keyboard resize, voice input, reduced motion, 200% text, and slow/offline transitions.

## 9. Technical interaction contracts

- Optimistically render user messages with temporary IDs and reconcile them.
- Persist streams on the server and expose resumable typed events.
- Centralize command and shortcut definitions.
- Centralize menu actions and availability predicates.
- Serialize navigation/workspace state where share/resume is promised.
- Keep device-local presentation preferences out of shareable navigation state.
- Use atomic content parts for text, images, files, tools, citations, artifacts, and notices.
- Apply idempotency keys to external writes and retryable submissions.
- Emit analytics for action, outcome, failure, undo, regenerate, copy, abandon, and resume without logging sensitive content by default.

## 10. Build order and definition of done

Design dependency order:

1. Conversation tree and turn state/event contracts.
2. Bare composer, context manifest, paste/drop.
3. Message projection, actions, branching, scroll law.
4. Shared menu and command registries.
5. Command palette.
6. Thinking/tool/source rendering.
7. Artifacts and side-surface host.
8. Slash/mentions/selection context.
9. Projects, suggestions, long/shared conversation behavior.
10. Full keyboard, mobile, accessibility, reconnect, and failure pass.

Definition of done:

- Complete critical session without mouse.
- Complete critical mobile session without desktop-only controls.
- Every menu and command is keyboard-operable and permission-aware.
- Every turn state has a distinct tested rendering.
- Long paste, multiple-file partial failure, drop, context menu, branch, and undo are tested.
- Closing during a stream and returning restores the correct turn without duplicate text.
- Auto-scroll never steals a reader's position.
- No enabled shortcut is documentation-only.


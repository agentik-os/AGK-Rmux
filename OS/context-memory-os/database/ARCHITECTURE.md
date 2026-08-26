# Context & Memory Storage Architecture

- **Object/source vault:** immutable originals addressed by hash.
- **Event log:** append-only capture, correction, supersession and deletion events.
- **Canonical relational/graph layer:** entities, atomic records, decisions, edges, contradictions and access policies.
- **Derived search index:** full text/vector retrieval; disposable and rebuildable.
- **Snapshot store:** versioned project/person/OS context packs.

The search index never becomes the source of truth. Every retrieved statement resolves to a canonical record and original source.

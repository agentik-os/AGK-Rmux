# Canonical Story Object and story-bank system

## Contents

1. Design principles
2. Canonical schema
3. Provenance and truth
4. Consent and privacy
5. Versions and derivatives
6. Lifecycle statuses
7. Storage and CLI

## 1. Design principles

Maintain one canonical object per distinct core change. Keep raw source immutable. Store interpretations, structures, drafts, and derivatives as linked layers.

Never overwrite:

- original transcript or note;
- provenance;
- consent restrictions;
- truth-class history;
- approved canonical version.

Create a new story object when the core change, truth boundary, central character, or meaning materially changes.

## 2. Canonical schema

```yaml
story_id: sto_YYYYMMDD_slug
title_working: ""
created_at: "ISO-8601"
updated_at: "ISO-8601"
status: seed

ownership:
  storyteller: ""
  recorder: ""
  primary_character: ""
  other_people: []

intent:
  job: connect
  audience: ""
  desired_update: ""
  channels: []
  agency_contract: coach

source:
  source_type: memory
  raw_text: ""
  source_refs: []
  artifacts: []
  captured_at: ""

truth:
  primary_class: remembered
  overall_confidence: medium
  chronology: approximate
  dialogue: paraphrased
  claims: []
  unresolved: []

privacy:
  level: private
  identifiable_people: []
  consent_records: []
  confidential_topics: []
  release_constraints: []

dna:
  core_change: ""
  pressure: ""
  hinge: ""
  proof_detail: ""
  meaning: ""
  truth_boundary: ""
  voice_marker: ""
  dignity_constraint: ""

craft:
  story_class: moment
  desire: ""
  old_belief: ""
  obstacle: ""
  stakes: ""
  choice: ""
  external_consequence: ""
  internal_update: ""
  residue: ""
  selected_structure: ""
  beats: []

voice:
  source_samples: []
  fingerprint: []
  preserve: []
  avoid: []

versions: []
derivatives: []

evaluation:
  truth_gate: pending
  consent_gate: pending
  scores: []
  release_verdict: needs_deepening

performance:
  publications: []
  audience_evidence: []
  learning: []

tags: []
connections: []
next_action: ""
```

Keep absent data empty; never infer it merely to complete the schema.

## 3. Provenance and truth

Represent each consequential claim:

```yaml
- claim_id: clm_001
  text: ""
  type: fact
  truth_class: documented
  confidence: high
  source_refs: []
  consequence_if_wrong: high
  verification_status: verified
  public_wording: ""
  notes: ""
```

Allowed `type` values:

- fact;
- number;
- quotation;
- chronology;
- attribution;
- motive;
- causal inference;
- emotional interpretation;
- future projection.

Allowed `truth_class` values:

- documented;
- corroborated;
- remembered;
- interpreted;
- reconstructed;
- composite;
- hypothetical;
- fictional.

Allowed `verification_status` values:

- unreviewed;
- needs_source;
- verified;
- qualified;
- disputed;
- removed.

## 4. Consent and privacy

Represent consent per person, version, channel, audience, and expiry:

```yaml
- consent_id: con_001
  person_ref: person_001
  status: granted
  scope:
    versions: [ver_003]
    channels: [keynote]
    audience: "private client conference"
  identity_mode: first_name_only
  evidence_ref: "email_2026-08-11"
  granted_at: "ISO-8601"
  expires_at: null
  restrictions: []
```

Consent statuses: unknown, requested, granted, limited, declined, withdrawn.

Identity modes: named, first-name-only, role-only, anonymized, composite-labeled, private-only.

If consent is withdrawn, do not erase the history. Mark affected versions blocked and stop future use.

## 5. Versions and derivatives

Canonical version record:

```yaml
- version_id: ver_001
  parent_version_id: null
  kind: canonical
  contract: write
  format: spoken_story
  language: fr
  duration_seconds: 240
  text_ref: ""
  created_at: ""
  approved_by: ""
  approval_status: draft
  truth_snapshot: ""
  change_summary: ""
```

Derivative record:

```yaml
- derivative_id: der_001
  canonical_version_id: ver_001
  channel: instagram_reel
  format: spoken_video
  target_length: 45s
  version_ref: ver_004
  dna_changes: []
  published_at: null
```

If `dna_changes` is non-empty, review whether this should be a new story object.

## 6. Lifecycle statuses

Use:

`seed → captured → mined → interviewing → verified → shaped → drafted → approved → published → learned → archived`

Side states:

- blocked_truth;
- blocked_consent;
- private_only;
- needs_deepening;
- retired;
- do_not_publish.

Do not advance from:

- mined to shaped without sufficient source material;
- interviewing to verified with unresolved high-consequence claims;
- drafted to approved with failed consent or truth gates;
- approved to published when the derivative changes the truth boundary.

## 7. Storage and CLI

Use `scripts/storyteller_os.py` for a deterministic local bank. It uses SQLite and standard-library Python only.

Examples:

```bash
python3 scripts/storyteller_os.py init --db stories.db
python3 scripts/storyteller_os.py capture --db stories.db --title "The broken demo" --raw-file note.txt --story-class decision
python3 scripts/storyteller_os.py list --db stories.db --status seed
python3 scripts/storyteller_os.py show --db stories.db sto_20260811_the_broken_demo
python3 scripts/storyteller_os.py score --db stories.db sto_20260811_the_broken_demo
python3 scripts/storyteller_os.py export --db stories.db --format jsonl --output stories.jsonl
python3 scripts/storyteller_os.py doctor --db stories.db
```

The CLI score checks structural completeness, not literary quality, audience response, or truth. A story cannot pass release solely because the CLI score is high.

Keep the database private by default. Do not put confidential raw stories into a public code repository.

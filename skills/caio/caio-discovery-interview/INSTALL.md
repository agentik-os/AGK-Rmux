# Install

1. Drop the `caio-discovery-interview/` folder into your skills directory
   (e.g. `/mnt/skills/user/` here, or your Claude Code skills path).
2. Trigger it: "employee discovery interview", "découverte de poste",
   "AI readiness intake", "interview this manager about how they work".
3. One run = one person = one standardized zip (18 files, identical for everyone).
   `metadata.json` carries consent, sharing level, reports-to and handoffs so the
   bundles stack into a company picture.
4. After several interviews, roll them up:
   `python scripts/consolidate.py --input <folder-of-zips> --out <dir>`

Structure:
- `SKILL.md` ............ workflow (boot, consent, company scan, 14-chapter walk, export, checks)
- `references/persona-language-packs.md` ... how to speak each job family's language
- `assets/templates/` ... the 18 fixed output files (identical for everyone)
- `scripts/build_bundle.py` ... validate one person's folder + zip it (name-aware / role-aware if anonymized)
- `scripts/consolidate.py` ... roll up many bundles → company-rollup.md + .json

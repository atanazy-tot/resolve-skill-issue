---
kind: skill
name: author-skill
description: >
  Creates new registry assets through an interactive intake: scaffolds the
  directory, drafts frontmatter and body, and registers tags and bundle
  membership. USE WHEN adding a skill or shared-reference to this registry,
  or when the user asks for a new asset.
tags: [meta, authoring]
requires: [audit-checklist, descriptions, restructuring]
---
# author-skill

Creates assets that pass the audit on the first try. The judgment playbooks
live in `references/` — consult them while drafting, not after.

## Authoring workflow

Copy this checklist and track progress:

```
Authoring progress:
- [ ] Step 1: Intake Q&A (purpose, kind, name, triggers)
- [ ] Step 2: Scaffold (dir/file + frontmatter)
- [ ] Step 3: Draft body
- [ ] Step 4: Quality gate (checklist + lint + metrics)
- [ ] Step 5: Register (taxonomy, bundles)
- [ ] Step 6: Report
```

**Step 1: Intake Q&A.** Ask, offering options per question (S.A. best /
S.B. alternative / S.C. wilder; batch questions when possible):

- What should the asset do, and when should it trigger? Collect trigger terms
  verbatim — they feed the description.
- Kind: `skill` or `shared-reference`? Decision rule:
  [references/restructuring.md](references/restructuring.md), "Skill-local vs shared-reference".
- Name: propose imperative verb-first candidates (`audit-skills`, `refactor-skill`).
  Persona names are reserved for future agents.
- Composition: fixed shared-references (`requires`) or per-project picks
  (`requires-one-of` / `requires-any`)?

**Step 2: Scaffold.** Create `assets/skills/<name>/SKILL.md` (or
`assets/shared-references/<name>.md`) with valid frontmatter: kind, name
(== path), description drafted per
[references/descriptions.md](references/descriptions.md), tags, requires/slots.

**Step 3: Draft the body.** Overview + navigation; deep material one level deep
in `references/`; multi-step procedures get a copyable checklist; concrete
examples; degrees of freedom matching fragility. Patterns:
[references/restructuring.md](references/restructuring.md), "How to split".

**Step 4: Quality gate.** Evaluate the draft against
[references/audit-checklist.md](references/audit-checklist.md), then run:

```bash
uv run tools/lint_v0.py
uv run tools/registry_metrics.py
```

Fix and re-run until green with no flags (feedback loop).

**Step 5: Register.** New tags or groups → propose additions to `taxonomy.toml`.
Bundle membership → propose adding the asset to the right `bundles/*.toml`,
or explain why it stays unbundled.

**Step 6: Report.** Summarize: files created, final description, composition
declared, registrations applied or proposed.

## Degrees of freedom

| Action | Freedom |
|---|---|
| Scaffold + frontmatter from intake answers | Apply directly |
| Body drafting | Apply directly; show the result |
| Taxonomy / bundle registration | Propose, apply after approval |

## Rules

- Never invent content the intake didn't cover — ask.
- One asset per session unless the user says otherwise.
- Follow every convention in
  [references/audit-checklist.md](references/audit-checklist.md) — this skill's
  output is graded by audit-skills.

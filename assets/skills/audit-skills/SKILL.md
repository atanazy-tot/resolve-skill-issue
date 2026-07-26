---
kind: skill
name: audit-skills
description: >
  Diagnoses registry assets against skill-authoring best practices and registry
  conventions, producing a severity-rated findings report without modifying any
  files. USE WHEN reviewing registry health, auditing skills or
  shared-references, or preparing input for refactor-skill.
tags: [meta, review]
requires: [audit-checklist, descriptions, restructuring]
---
# audit-skills

The registry's diagnostic instrument. Strictly read-only: findings go in the
report; treatment belongs to refactor-skill, authoring to author-skill.

## Audit workflow

Copy this checklist and track progress:

```
Audit progress:
- [ ] Step 1: Deterministic baseline (lint)
- [ ] Step 2: Metrics gathered
- [ ] Step 3: Judgment audit per asset
- [ ] Step 4: Report written (with treatment list)
```

**Step 1: Deterministic baseline.** From the registry root:

```bash
uv run tools/lint_v0.py
```

Record every error as a finding (severity: high). A red baseline does not block
the audit — it IS audit data — but flag it prominently.

**Step 2: Metrics.**

```bash
uv run tools/registry_metrics.py
```

Never estimate counts yourself; the table is the map.

**Step 3: Judgment audit.** For every asset — flagged ones first — evaluate
against [references/audit-checklist.md](references/audit-checklist.md). Judge
descriptions against [references/descriptions.md](references/descriptions.md)
and structure against [references/restructuring.md](references/restructuring.md).
Every finding cites file + section.

**Step 4: Report** using the template below. Do not modify any asset.

## Report template

Sensible default — adapt to the audit's size:

```markdown
# Registry audit — <date>

## Baseline
lint: <green | N errors> · assets: <N skills, M shared-references>

## Findings
| # | Asset | Issue | Severity | Evidence |
|---|---|---|---|---|
| 1 | programmer | body 612 lines | high | metrics: SPLIT flag |

## Treatment list (input for refactor-skill)
1. <issue, one line, self-contained> — suggested direction: <one line>
2. …
```

Severity: `high` = hurts discovery or breaks a rule soon; `medium` = degrades
quality; `low` = polish.

## Rules

- Read-only: never edit, scaffold, or delete anything.
- The treatment list must be copy-pasteable as refactor-skill input — one issue
  per line, self-contained.
- Deterministic violations come from the linter, not manual inspection;
  judgment findings come from the checklist.

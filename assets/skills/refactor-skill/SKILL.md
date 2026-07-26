---
kind: skill
name: refactor-skill
description: >
  Repairs registry assets given a list of issues: runs a structured Q&A to
  choose the fix per issue (options A/B/C), applies the approved changes, and
  verifies with lint and metrics. USE WHEN fixing known problems in a skill or
  shared-reference, applying an audit-skills report, or restructuring an asset.
tags: [meta, review, authoring]
requires: [audit-checklist, descriptions, restructuring]
---
# refactor-skill

The registry's treatment skill. Input: issues (from the user or an audit-skills
treatment list). Method: structured Q&A per issue. Output: applied, verified fixes.

## Refactor workflow

Copy this checklist and track progress:

```
Refactor progress:
- [ ] Step 1: Intake (collect issues; recommend audit-skills if none given)
- [ ] Step 2: Diagnose each issue (read the asset, cite evidence)
- [ ] Step 3: Q&A session (options per issue)
- [ ] Step 4: Apply approved fixes
- [ ] Step 5: Verify (lint + metrics, iterate)
- [ ] Step 6: Report (before → after)
```

**Step 1: Intake.** Issues come from the user or an audit report. If the user
describes vague symptoms ("the skill feels bloated") without specifics, say so
and recommend running audit-skills first — do not invent issues.

**Step 2: Diagnose.** For each issue: read the asset, confirm or refute the
claim, cite file + section as evidence. Drop refuted issues and say why.

**Step 3: Q&A session.** Present every confirmed issue in this format, batched
5–10 at a time:

```
Q<n>: <the problem in one line, with evidence>
S.A.: <best fix — clever, minimal>
S.B.: <equally valid alternative from a different angle>
S.C.: <wilder option — bolder restructuring>
```

The user picks per issue (A / B / C / custom). Never implement an unpicked option.

**Step 4: Apply.** Implement picks using the playbooks: descriptions per
[references/descriptions.md](references/descriptions.md); splits, merges, and
moves per [references/restructuring.md](references/restructuring.md). Minimal
diffs; behavior and meaning unchanged.

**Step 5: Verify.** Re-run until green with no new flags:

```bash
uv run tools/lint_v0.py
uv run tools/registry_metrics.py
```

**Step 6: Report.** Per issue: pick taken, before → after summary. Close with
the post-fix metrics table.

## Degrees of freedom

| Action | Freedom |
|---|---|
| Implementing a picked option | Apply directly |
| A custom user direction | Apply directly |
| Anything not covered by a pick | Do not improvise — ask |

## Rules

- Fixes follow [references/audit-checklist.md](references/audit-checklist.md);
  a refactor that fails the checklist is not done.
- Python runs via `uv run` only; dependencies via `uv add`.
- Never touch assets unrelated to the approved issues.

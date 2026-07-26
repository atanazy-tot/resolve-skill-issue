---
kind: shared-reference
name: restructuring
description: >
  Playbook for splitting, merging, and restructuring registry assets, including
  the skill-local vs shared-reference decision rules.
tags: [skill-authoring]
---
# Restructuring playbook

Splits, merges, and moves are **propose-only**: present the plan, get approval, then apply.

## Contents

- When to split a skill
- How to split
- Skill-local vs shared-reference
- Merge candidates
- Post-restructure verification

## When to split a skill

| Signal | Threshold |
|---|---|
| Body length | > 500 lines (split-watch at 400) |
| Multiple domains | Sections serve disjoint tasks — only one is ever needed at a time |
| Conditional detail | Advanced material only some invocations need |

## How to split

1. Choose the pattern:
   - **Overview + references**: move deep material to `references/<topic>.md`; leave quick start + navigation links in SKILL.md.
   - **Domain organization**: one `references/` file per domain; SKILL.md becomes the directory of domains.
   - **Conditional details**: basic content inline; each advanced topic gets its own file.
2. Keep every reference exactly one level deep — linked directly from SKILL.md.
3. Add a table of contents to any reference file that will exceed 100 lines.
4. Leave a one-line pointer in SKILL.md per moved section ("**Form filling**: see FORMS.md").

## Skill-local vs shared-reference

| Question | If yes |
|---|---|
| Would another skill plausibly need this content? | `shared-references/`, tagged into a taxonomy group |
| Is it picked per-project (one-of / any-of)? | `shared-references/` — skills compose it via slots |
| Only this skill ever needs it? | Skill-local `references/` |

Moving skill-local → shared-reference: move the file; add frontmatter
(`kind: shared-reference`, name = filename stem, group tag); declare the slot in
the skill's frontmatter (`requires-one-of` / `requires-any`); update body links
to slot-stable names (PRD §6.4).

## Merge candidates

Flag two skills for merge when BOTH hold:

- their descriptions share trigger terms (they would load in the same situations), and
- the merged body would stay under 300 lines.

Present: which skill survives, what moves, the merged description (per
[descriptions.md](descriptions.md)).

## Post-restructure verification

1. `uv run tools/lint_v0.py` — green.
2. `uv run tools/registry_metrics.py` — line counts back under thresholds, no broken links.
3. Bundles referencing moved assets still resolve — the linter checks ids; eyeball intent.

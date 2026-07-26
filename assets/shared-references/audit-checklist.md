---
kind: shared-reference
name: audit-checklist
description: >
  Judgment checklist for auditing and authoring registry assets, distilled from
  the Anthropic skill-authoring best-practices guide plus registry conventions.
tags: [skill-authoring]
---
# Audit checklist

Distilled from the [Anthropic skill-authoring best-practices guide](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices) plus registry rules (PRD). If a rule here ever conflicts with the live guide, the guide wins — update this file.

Deterministic rules (frontmatter schema, name == path, kind enum, TOML validity)
are enforced by `tools/lint_v0.py` / `quiver lint` — do not re-check them by hand.
This checklist covers what requires judgment.

## Contents

- Discovery (name + description)
- Body quality
- Structure and progressive disclosure
- Scripts (when present)
- Registry conventions
- Cross-asset hygiene

## Discovery (name + description)

- [ ] Description states WHAT the asset does with concrete verbs — not "helps with" or "processes stuff"
- [ ] Description states WHEN to use it, with trigger terms a user would actually type
- [ ] Third person only ("Audits…" — never "I can…", "You can…")
- [ ] Description ≤ 1024 characters, no XML tags
- [ ] Name ≤ 64 chars, lowercase letters/numbers/hyphens; not vague (`helper`, `utils`); no reserved words
- [ ] Skill names are imperative verb-first (`author-skill`, `refactor-skill`); persona names are reserved for agents
- [ ] Naming pattern consistent with the rest of the collection

## Body quality

- [ ] Body < 500 lines (400+ = split-watch → restructuring playbook)
- [ ] Token-cost test: every section answers "does the agent NOT already know this?" — cut what it knows
- [ ] One term per concept, used consistently (not endpoint / URL / route mixed)
- [ ] No time-sensitive info ("after release X, use Y") — legacy notes belong in an "old patterns" section
- [ ] Concrete examples over abstract prose; input/output pairs where output quality matters
- [ ] One default approach with an escape hatch — not a menu of equal options
- [ ] Workflows have numbered steps; complex ones have a copyable checklist
- [ ] Quality-critical operations have a feedback loop (validate → fix → repeat)
- [ ] Degrees of freedom match fragility: exact commands for fragile ops, heuristics for judgment tasks

## Structure and progressive disclosure

- [ ] SKILL.md is overview + navigation; deep material lives exactly one level deep in `references/`
- [ ] Every reference file is linked directly from SKILL.md (no reference → reference chains)
- [ ] Reference files > 100 lines start with a table of contents
- [ ] Filenames describe content (`form_validation_rules.md`, not `doc2.md`)
- [ ] Forward slashes only, in all paths

## Scripts (when present)

- [ ] Instructions state whether to EXECUTE the script or READ it as reference
- [ ] Scripts handle errors explicitly — they solve, not defer to the agent
- [ ] No voodoo constants; every fixed value justified by a comment
- [ ] Dependencies stated explicitly — never assume a package is installed

## Registry conventions

- [ ] All tags declared in `taxonomy.toml` (under `[tags]` or a `[groups]` name)
- [ ] Fixed shared-reference dependencies declared via `requires: [ids]`; bodies link them as `references/<id>.md`
- [ ] `requires-one-of` / `requires-any` slots reference declared groups only
- [ ] Body links to picked shared-references use slot-stable names (`references/<slot>.md`, `references/<slot>/`)
- [ ] No `quiver:managed` markers in canonical sources
- [ ] Shared-references carry a composition-group tag; skills never do
- [ ] No secrets, tokens, or internal URLs — the registry is public-ready

## Cross-asset hygiene

- [ ] No two skills with overlapping trigger terms (merge candidates → restructuring playbook)
- [ ] Content needed by 2+ skills lives in `shared-references/`, not duplicated in skill-local `references/`
- [ ] Every asset belongs to at least one bundle, or is intentionally unbundled (reason noted)

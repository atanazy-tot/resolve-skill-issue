# Renderer spec — target: `opencode`

v1 spec — verified against the opencode docs on 2026-07-26 (all `[VERIFY]` items resolved).
Implemented in `quiver/src/render.rs`.

## Install layout

| Source (registry) | Destination (target repo) |
|---|---|
| `assets/skills/<name>/SKILL.md` | `.opencode/skills/<name>/SKILL.md` (plural `skills/` — confirmed) |
| `assets/skills/<name>/references/*` (skill-local) | `.opencode/skills/<name>/references/` (verbatim copy) |
| `assets/skills/<name>/scripts/*` (utility scripts) | `.opencode/skills/<name>/scripts/` (verbatim copy — executed, not loaded) |
| picked shared-reference, one-of slot `S` | `.opencode/skills/<name>/references/<S>.md` |
| picked shared-references, any slot `S` | `.opencode/skills/<name>/references/<S>/<asset-id>.md` |
| required shared-references (`requires: [ids]`) | `.opencode/skills/<name>/references/<asset-id>.md` |

opencode also reads `.claude/skills/` and `.agents/skills/` (project and global) as
Claude/agent-compatible fallbacks. We still render to the native `.opencode/skills/` root —
native discovery, no ambiguity.

## Frontmatter dialect (verified)

Only these fields are recognized: `name` (required), `description` (required), `license`,
`compatibility`, `metadata` (string map). **Unknown fields are ignored.**

- `name`: `^[a-z0-9]+(-[a-z0-9]+)*$`, 1–64 chars, must match the directory name — enforced
  canonically by quiver's `AssetId` validation.
- `description`: 1–1024 chars — enforced canonically by lint rule QV-002's sibling checks and
  the metrics script (`DESC-TOO-LONG` flag).

The renderer emits only `name` + `description` and strips quiver-only fields
(`kind`, `tags`, `requires*`) — unnecessary since unknown fields are ignored, but cleaner
and uniform with the claude renderer.

## Marker placement (verified)

`<!-- quiver:managed <id>@sha256:<hash> -->` as the first line after the frontmatter block.
Safe: it is a body-level HTML comment, outside the parsed frontmatter.

## Discovery (informational)

opencode walks up from the cwd to the git worktree root, loading `skills/*/SKILL.md` under
`.opencode/` (plus the fallback roots). Agents load skills on demand via the native `skill`
tool; only `name` + `description` occupy context until then.

## Body

Verbatim copy of the canonical body. No link rewriting (slot-naming convention, PRD §6.4).

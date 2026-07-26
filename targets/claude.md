# Renderer spec — target: `claude` (Claude Code)

v0 spec. `[VERIFY]` items are M1 acceptance checks (PRD §10).

## Install layout

| Source (registry) | Destination (target repo) |
|---|---|
| `assets/skills/<name>/SKILL.md` | `.claude/skills/<name>/SKILL.md` |
| `assets/skills/<name>/references/*` (skill-local) | `.claude/skills/<name>/references/` (verbatim copy) |
| `assets/skills/<name>/scripts/*` (utility scripts) | `.claude/skills/<name>/scripts/` (verbatim copy — executed, not loaded into context) |
| picked shared-reference, one-of slot `S` | `.claude/skills/<name>/references/<S>.md` |
| picked shared-references, any slot `S` | `.claude/skills/<name>/references/<S>/<asset-id>.md` |
| required shared-references (`requires: [ids]`) | `.claude/skills/<name>/references/<asset-id>.md` |

## Frontmatter dialect

Emitted frontmatter contains **only** tool-native fields:

| Field | Required | Notes |
|---|---|---|
| `name` | yes | Copied from canonical. |
| `description` | yes | Copied verbatim — carries trigger conditions ("USE WHEN…"). |
| `allowed-tools` | no | Passed through if present in canonical frontmatter. |

Quiver-only fields (`kind`, `tags`, `requires-one-of`, `requires-any`) are **stripped** at
render time. [VERIFY] whether Claude Code tolerates unknown frontmatter fields — if confirmed
harmless, stripping may be relaxed to pass-through for debuggability.

## Marker placement

`<!-- quiver:managed <id>@sha256:<hash> -->` inserted as the first line **after** the closing
`---` of the frontmatter. [VERIFY] the marker does not affect skill loading.

## Body

Verbatim copy of the canonical body. No link rewriting: links are stable by the slot-naming
convention (PRD §6.4).

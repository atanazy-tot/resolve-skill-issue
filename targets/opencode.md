# Renderer spec — target: `opencode`

v0 spec. `[VERIFY]` items are M1 acceptance checks (PRD §10).

## Install layout

| Source (registry) | Destination (target repo) |
|---|---|
| `assets/skills/<name>/SKILL.md` | `.opencode/skill/<name>/SKILL.md` — [VERIFY] singular `skill/` vs plural `skills/` |
| `assets/skills/<name>/references/*` (skill-local) | same directory structure as the claude target |
| `assets/skills/<name>/scripts/*` (utility scripts) | same directory structure as the claude target (executed, not loaded) |
| picked shared-references (one-of / any) | same slot-stable naming as the claude target |
| required shared-references (`requires: [ids]`) | `.opencode/skill/<name>/references/<asset-id>.md` (same as claude) |

The install root is renderer **config**, not schema — both path candidates must be testable
without code changes.

## Frontmatter dialect

[VERIFY] exact frontmatter requirements for opencode skills (required fields; tolerance of
unknown fields). Until verified: emit the same minimal field set as the claude renderer
(`name`, `description`) and strip quiver-only fields (`kind`, `tags`, `requires-one-of`,
`requires-any`).

## Marker placement

Same rule as claude: `<!-- quiver:managed <id>@sha256:<hash> -->` as the first line after the
frontmatter block. [VERIFY] opencode ignores it when loading the skill.

## Body

Verbatim copy of the canonical body. No link rewriting (slot-naming convention, PRD §6.4).

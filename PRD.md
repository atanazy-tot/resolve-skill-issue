# PRD: quiver — a personal AI-asset registry and distribution tool

- **Status:** Draft v1.0 (output of a 20-round Q&A design session)
- **Repository:** `resolve-skill-issue` (display name; the CLI/TUI binary is **`quiver`**)
- **Author:** atanazy
- **Date:** 2026-07-26

---

## 1. Problem statement

AI-engineering assets (skills, prompts, agents) accumulate across many projects. Copy-pasting them
repo-to-repo causes drift, bloat, and **context pollution** — agents load instructions irrelevant to
the project at hand. There is no single source of truth, no way to install "just the Python setup"
into one repo and "just the Scala setup" into another, and no way to know which assets are actually
pulling their weight.

**quiver** solves this: one central registry repo holding all assets, grouped into bundles, with a
CLI/TUI that renders and installs *only the selected assets* into target repos, tracks them with a
lockfile, and reports on their real-world usage.

## 2. Goals / Non-goals

### Goals (MVP → v1.1)

1. Single canonical asset format; one registry repo as source of truth.
2. Declarative bundles; install assets into target repos as native tool files (copy + lockfile).
3. **Selective composition**: a skill can declare "needs exactly one of: language guides" — resolved
   interactively at install time, recorded in the lockfile.
4. Multi-target rendering (Claude Code + opencode) from canonical format.
5. Drift detection, explicit updates, clean uninstall, eject (fork) escape hatch.
6. Linting that codifies agent-skill best practices (incl. the 500-line progressive-disclosure rule)
   in CI, plus meta skills for authoring, auditing, and refactoring assets (§6.7).
7. Local-only usage telemetry with a prune report.
8. Ratatui-based TUI (Elm architecture) as a thin shell over a tested core.

### Non-goals (explicitly rejected during design)

| Rejected idea | Why | Revisit? |
|---|---|---|
| Two-way sync (push edits back to registry) | Mini-git-ops complexity for one user | Maybe v2 |
| Per-asset semver | Ceremony for an audience of one; repo tags + hashes suffice | No |
| Submodule/symlink distribution | Hostile UX, fragile tooling | No |
| Runtime loader (fetch assets at session start) | Network-dependent, poor auditability | Explore in v2 as opt-in |
| Multi-registry / private overlay | Premature before one registry works | v2 candidate |
| Versioned dependency resolver | Only unversioned one-of/any composition is needed | No |

## 3. Users

Primary: a single AI engineer (the author) fueling N personal/work repos from one registry.
Secondary (enabled by the public-ready posture, §10): publishing selected bundles for community use.

## 4. Decision register (Q&A traceability)

| # | Decision | Choice |
|---|---|---|
| Q1 | Distribution model | **Copy + lockfile** (package-manager model) |
| Q2 | Grouping | **Declarative bundle manifests** (TOML; bundles may include bundles; tags for search only) |
| Q3 | Asset format | **One canonical format**: Markdown + YAML frontmatter, all kinds |
| Q4 | Multi-target | **Canonical + per-target renderers** (pure functions) |
| Q5 | Versioning | **Registry git tags + per-asset content hashes** in lockfile |
| Q6 | Drift policy | **One-way**; detect via hash → `restore` or `eject` |
| Q7 | TUI tech | **Rust + Ratatui, Elm architecture** (Model–Update–View) |
| Q8/Q9 | Composition | **`requires-one-of` / `requires-any` in frontmatter, interactive resolution**; non-interactive escape hatches (§6.4). Plain `requires: [ids]` added for fixed shared-reference deps (2026-07-26) |
| Q10 | Quality gates | **Both**: full linter in CLI/CI **and** shipped meta skills (§6.7) |
| Q11 | Registry layout | **Kind-first**: `assets/<kind>/<name>/` |
| Q12 | Target layout | **Native tool dirs + marker headers + root lockfile** |
| Q13 | Transport | **Local git cache** (`~/.local/share/quiver/registry`), offline after first clone |
| Q14 | MVP strategy | **CLI-first vertical**; TUI after core loop proven |
| Q15 | Differentiators | **All four**: telemetry+pruning, meta skills (§6.7), export/publish, update digest |
| Q16 | Privacy posture | **Public-ready + secret scanning** in lint/CI |
| Q17 | MVP targets | **Both** Claude Code and opencode (cheap: only one kind exists) |
| Q18 | MVP kinds | **Skills-family only**: `{skill, shared-reference}` (see §5.1) |
| Q19 | Telemetry storage | **Local-only JSONL** (`~/.local/share/quiver/telemetry.jsonl`) |
| Q20 | Naming | **`quiver`** (binary), repo keeps `resolve-skill-issue` display name |
| — | Skill naming convention | **Imperative verb-first** (`author-skill`, `audit-skills`, `refactor-skill`); persona names reserved for agents — `asset-doctor` earmarked for a v1.1 agent (2026-07-26) |

## 5. Core concepts

### 5.1 Asset (canonical format)

Every skill asset is a directory under `assets/skills/<name>/`; shared-references are flat `.md`
files under `assets/shared-references/`. All assets are Markdown + YAML frontmatter.

MVP kinds (Q18):

- **`skill`** — discoverable, installed into a tool's skills dir. A skill directory may contain a
  **skill-local** `references/` subfolder and/or `scripts/` dir: private payload, copied verbatim
  into the rendered skill dir on every install (references are read on demand; scripts are
  executed, not loaded into context).
- **`shared-reference`** — a registry-wide library doc in `assets/shared-references/` (e.g.
  `lang-python.md`, `paradigm-fp.md`), tagged into composition groups (`lang-guide`,
  `paradigm-guide`). *Never* independently installable; lands inside a skill's `references/` only
  via composition picks (§6.4). This is what keeps main `SKILL.md` files under 500 lines
  (progressive disclosure).

**Naming rule (disambiguation):** `skills/<name>/references/` = skill-local, always ships with the
skill; `assets/shared-references/` = registry-wide, ships only when picked or required. The two are
never merged in the registry — only in the rendered output.

**Naming convention:** skill names are imperative verb-first (`author-skill`, `refactor-skill`);
persona names are reserved for agents (`asset-doctor` is earmarked for a v1.1 agent).
Shared-reference files use plain noun names (`descriptions.md`, `lang-python.md`).

`prompt` and `agent` kinds are reserved in the schema for v1.1 — `kind` is just a frontmatter field,
so this is a non-breaking extension.

```markdown
---
kind: skill
name: programmer
description: >
  General programming workflow. USE WHEN writing, reviewing, or refactoring
  code in any language.
tags: [programming, workflow]
requires-one-of:
  lang-guide: { tag: lang-guide, into: references/ }
requires-any:
  paradigm: { tag: paradigm-guide, into: references/ }
---
# Programmer
…(body < 500 lines; deep material lives in references/)…
```

### 5.2 Bundle manifest

Bundles are TOML files in `bundles/`. They are the **only** grouping mechanism with membership
semantics; tags are for search/discovery only (Q2).

```toml
# bundles/python-backend.toml
name = "python-backend"
description = "Python backend development setup"
includes = ["core-workflow"]              # bundle composition (cycle-checked)
assets   = ["programmer", "git-conventions"]

# OPTIONAL: pre-declared composition picks — makes "the Scala setup" a named,
# shareable thing and skips interactive prompts (closes Q9 option C's main weakness).
picks = { lang-guide = "python", paradigm = ["fp-lite"] }
```

### 5.3 Lockfile (target repo)

`ai-assets.lock.json` at the target repo root (Q12). Records everything needed for drift detection,
updates, uninstall, and non-interactive replay.

```json
{
  "version": 1,
  "registry": { "url": "git@github.com:atanazy/resolve-skill-issue", "tag": "v2026.07.1" },
  "installed": [
    {
      "id": "programmer",
      "kind": "skill",
      "hash": "sha256:…",
      "paths": [".claude/skills/programmer/", ".opencode/skill/programmer/"],
      "picks": { "lang-guide": "python", "paradigm": ["fp-lite"] },
      "bundle": "python-backend"
    }
  ]
}
```

Every rendered file carries a marker header (placed *after* the frontmatter block to avoid breaking
tool parsers — placement to be verified per target during M1):

```markdown
<!-- quiver:managed programmer@sha256:abc123 — do not edit; changes will be detected -->
```

### 5.4 Registry layout (kind-first, Q11)

```
resolve-skill-issue/
├── assets/
│   ├── skills/
│   │   ├── author-skill/          # SKILL.md (playbooks pulled in via requires)
│   │   ├── audit-skills/          # SKILL.md
│   │   └── refactor-skill/        # SKILL.md
│   └── shared-references/         # audit-checklist.md, descriptions.md, restructuring.md,
│                                  # lang-rust.md (+ more lang-*/paradigm-* as they arrive)
├── bundles/                       # *.toml manifests (core-workflow.toml)
├── targets/                       # renderer specs (claude.md, opencode.md)
├── quiver/                        # the Rust CLI (fmt/clippy clean, 21 tests): list, install, status
├── tools/                         # lint_v0.py + registry_metrics.py — superseded by `quiver lint` in M1
├── taxonomy.toml                  # controlled tag/group vocabulary (QV-005, QV-007)
├── pyproject.toml + uv.lock       # uv-managed Python env for utility scripts (uv add / uv run)
├── PRD.md
└── .github/workflows/lint.yml     # v0 parse checks → `quiver lint` + secret scan in M1
```

### 5.5 Local machine layout

```
~/.local/share/quiver/
├── registry/          # git clone of the registry (Q13); `quiver registry sync` = git pull
└── telemetry.jsonl    # local-only usage events (Q19)
```

## 6. Functional requirements

### 6.1 Install (`quiver install <asset|bundle>`)

1. Resolve bundle → asset set (recursive `includes`, cycle error).
2. For each asset with `requires-one-of` / `requires-any`: prompt interactively (default),
   or take picks from (a) manifest `picks`, (b) `--pick lang-guide=python` flags, (c) lockfile
   replay (`--replay`) for CI. Interactive is the default per Q9; these are the agreed escape hatches.
3. Render canonical → target-native format per selected target(s) (`--target claude,opencode`;
   both at MVP, Q17).
4. Copy into native tool dirs, insert marker headers, write/update lockfile with hashes + picks.

Renderer note (resolved 2026-07-26): opencode reads `.opencode/skills/<name>/SKILL.md` (plural),
recognizes only `name` + `description` frontmatter (unknown fields ignored), validates `name` as
`^[a-z0-9]+(-[a-z0-9]+)*$` matching the directory, and limits descriptions to 1–1024 chars — all
now enforced by the quiver renderer and `AssetId` validation.

### 6.2 Status, drift, restore/eject (Q6)

- `quiver status` — per installed asset: `ok | modified | outdated | missing`.
- Drift = rendered file hash ≠ lockfile hash → show diff → offer:
  - `quiver restore <id>` — overwrite local copy from registry.
  - `quiver eject <id>` — detach asset from registry management (remove marker, drop from lockfile,
    file stays). Backporting improvements upstream is a deliberate manual step.

### 6.3 Update with digest (Q15)

`quiver update [--dry-run]`: pulls registry cache, compares lockfile hashes vs registry at its tag,
and shows a **per-asset changelog digest** generated from `git log` between the locked tag and HEAD —
upgrading is a decision, not a leap of faith. `--all` applies; otherwise interactive selection.

### 6.4 Selective composition (Q8/Q9)

Assets declare `requires-one-of` / `requires-any` slots bound to **tag groups** (declared in
`taxonomy.toml`). Resolution installs the picked `shared-reference` assets into the requiring
skill's `references/` dir. No versioned dependency resolver exists (explicit non-goal). The linter
guarantees every tag group resolves to ≥1 asset (QV-007).

**Slot-stable install names** — skill bodies must link deterministically without knowing the pick:

- one-of slot `S` → rendered as `references/<S>.md`; the body links `references/<S>.md`, valid for
  any pick;
- any slot `S` (multi-pick) → rendered as `references/<S>/<asset-id>.md`; the body references the
  *directory*, not individual files;
- provenance lives in the marker header + lockfile `picks`, so **no link rewriting** happens at
  render time.

**Fixed dependencies** (`requires: [ids]`, added 2026-07-26): unversioned, always-installed
shared-references — the Q8 self-containment rule relaxed for reference docs. Installed flat as
`references/<asset-id>.md`; bodies link that path directly. Motivating case: the meta skills
(§6.7) share three playbooks without duplication.

### 6.5 Lint (`quiver lint`, pre-commit + CI) (Q10)

| Rule | Level | Description |
|---|---|---|
| QV-001 | error | Frontmatter schema valid (kind, name, description, tags) |
| QV-002 | warn >500 / error >800 | `SKILL.md` body line count (progressive disclosure) |
| QV-003 | error | Referenced files exist (`references/…` links resolve) |
| QV-004 | warning | Description quality: length, contains trigger conditions ("USE WHEN…") |
| QV-005 | warning | Tags exist in registry taxonomy |
| QV-006 | **error** | Secret-pattern scan (gitleaks-style) — public-ready posture (Q16) |
| QV-007 | error | `requires-one-of`/`requires-any` tag groups resolve to ≥1 asset |
| QV-008 | error | Bundle manifests reference existing assets/bundles; no include cycles |

### 6.6 Telemetry + pruning (Q15/Q19)

- A hook installed into target repos (opt-in per repo, `quiver hooks install`) appends events to
  `~/.local/share/quiver/telemetry.jsonl`:
  `{"ts":"…","repo":"sha256-of-remote-url","asset":"programmer","event":"skill_triggered"}`
- **Local-only.** Nothing leaves the machine; cross-machine aggregation via explicit
  `quiver telemetry export` + merge (post-MVP).
- `quiver report`: hot assets (frequently triggered → refine), dead assets (installed, zero triggers
  in N days → prune candidates). This is the data-driven anti-pollution feedback loop.

### 6.7 Meta skills: author-skill / audit-skills / refactor-skill (Q10/Q15)

Three shipped skills dogfood the registry (delivered 2026-07-26, ahead of the v1.1 slot):

- **author-skill** — interactive authoring: intake Q&A → scaffold → draft → quality gate → register.
- **audit-skills** — strictly diagnostic: lint baseline + `tools/registry_metrics.py` + judgment
  audit → severity-rated report ending in a copy-pasteable treatment list. Never modifies files.
- **refactor-skill** — treatment: takes issues (from the user or an audit report), runs a
  structured Q&A (S.A./S.B./S.C. per issue, batched 5–10), implements picks, verifies.

Shared playbooks (`audit-checklist`, `descriptions`, `restructuring`) live in
`assets/shared-references/` and are pulled in via plain `requires` (§6.4) — the mechanism added
for exactly this. `asset-doctor` is retired as a skill name and earmarked for a v1.1 agent
(skills are imperative; agents get persona names).

### 6.8 Export / publish (Q15, v1.1)

`quiver export <bundle> --format claude-plugin|tarball` produces a shareable artifact (manifest +
rendered assets + install script). Viable because of the Q16 public-ready posture.

### 6.9 TUI (`quiver tui`, M3)

Rust + **Ratatui**, **Elm architecture** (Model–Update–View) (Q7): pure `update(msg, model)`
transitions over an event loop; all logic lives in `quiver-core` so the TUI is a thin, testable
shell. Screens: registry browser (search/tags), bundle picker, pick-resolution wizard, status/drift
view, update digest, telemetry report.

## 7. Architecture

```
┌────────────────────────────────────────────┐
│ quiver (CLI)        quiver tui (Ratatui)   │
│      └──────────────┬──────────────────────┘
│              quiver-core (Rust lib)        │
│  schema · manifest · resolver · renderers  │
│  lockfile · lint · drift · digest · report │
└──────┬───────────────────┬─────────────────┘
       │ git pull          │ read/write
~/.local/share/quiver/registry   target repos (lockfile + native dirs)
```

- **`quiver-core`**: all parsing, resolution, rendering (pure functions canonical → target),
  lockfile I/O, lint rules, drift/digest logic. Fully unit-tested without UI.
- **`quiver` CLI**: `registry sync · list · search · bundle list/show · install · uninstall ·
  status · update · restore · eject · lint · report · telemetry export · export · hooks install · tui`
- **Transport (Q13)**: registry is a git clone; auth = existing git credentials; offline after
  first clone; versioning = repo tags (Q5).

## 8. Non-functional requirements

- **Offline-first**: all operations except `registry sync`/`update` work without network.
- **Deterministic rendering**: same asset + picks + target ⇒ byte-identical output.
- **Public-ready hygiene (Q16)**: zero secrets; QV-006 enforced pre-commit and in CI from day one
  (retro-scanning git history is painful).
- **Single static binary** distribution for `quiver` (Rust).
- **Idempotent installs**: re-running `install` converges to lockfile state.
- **Python via uv**: Python utility scripts run with `uv run` in the project environment;
  dependencies are managed exclusively with `uv add` (`pyproject.toml` + committed `uv.lock`).
  No pip, no manual venvs.

## 9. Roadmap

| Milestone | Contents | Exit criteria |
|---|---|---|
| **M1 — core vertical** | Registry layout + schema, linter (QV-001…008), CLI install/status/restore/eject, one-of resolver w/ prompts + `--pick`/`--replay`, lockfile + markers, claude + opencode renderers, git-cache transport, registry CI | Install `programmer` + one lang guide into two real repos; drift detected and restored; CI green |
| **M2 — lifecycle** | `uninstall`, update changelog digest, telemetry hooks + `report` | Digest shows real git-log changes; dead-asset report runs on author's repos |
| **M3 — TUI** | Ratatui/Elm shell over quiver-core, all core workflows reachable | Full install→update→eject loop doable without typing CLI flags |
| **v1.1** | `prompt`/`agent` kinds, `export` (claude-plugin/tarball) | A bundle exported and installed on a fresh machine |
| **v2 (candidates)** | Telemetry cross-machine merge, multi-registry/private overlay, runtime-loader experiment (Q1 S.C.) | — |

**M1 progress (2026-07-26):** the `quiver` crate landed (`quiver/`, Rust per the Canonical
best-practices guide — functional core, thiserror enums, zero panics, fmt/clippy clean, 21 unit
tests): `list`, `install`, `status` with lockfile + markers + drift exit codes; fixed `requires`
rendering; both targets; kind/location validation. The registry dogfoods itself — `quiver install
core-workflow` installed the three meta skills into this repo's own `.claude/skills/` and
`.opencode/skills/`, tracked by `ai-assets.lock.json`. Remaining for M1: interactive one-of
resolution, `uninstall`, `restore`/`eject`, git-cache transport, full lint port (QV-001…008),
crate CI.

## 10. Risks & open questions

| Risk / question | Mitigation |
|---|---|
| Both renderers at MVP widens test surface (Q17 vs Q14) | Single kind keeps renderer delta ≈ paths + minor frontmatter dialect; renderer contract tests in M1 |
| Interactive resolution breaks non-interactive contexts (Q9 weakness) | `--pick`, manifest `picks`, `--replay` from lockfile |
| Telemetry hooks are the most invasive feature | Opt-in per repo; local-only storage; deferred to M2 |
| Marker header could break tool parsing | Place after frontmatter; **verify per target in M1** (acceptance item) |
| opencode skills path uncertainty | Verify at M1 start; path is renderer config, not schema |
| Tag taxonomy growth chaos | Taxonomy file in registry, linted (QV-005); changes via PR to self |
| Confusion risk: skill-local `references/` vs registry-wide reference library | Resolved (2026-07-26): renamed to `shared-references/` + slot-stable install names (§5.1, §6.4) |

## 11. MVP acceptance criteria (M1)

1. `quiver install programmer --target claude,opencode` prompts for `lang-guide`, installs
   `SKILL.md` + picked reference into both tools' native dirs, writes lockfile + markers.
2. Editing an installed file → `quiver status` reports `modified`; `restore` reverts; `eject` detaches.
3. `quiver lint` fails CI on a 900-line skill, a missing reference, or an AWS-key-shaped string.
4. Entire flow works offline after initial `registry sync`.

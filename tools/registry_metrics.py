#!/usr/bin/env python3
"""registry_metrics.py — neutral measurements of registry assets for the meta skills.

Execute, don't read: only the output table consumes context.

Usage (from the registry root):
    uv run tools/registry_metrics.py          # markdown table
    uv run tools/registry_metrics.py --json   # machine-readable

Dependencies live in pyproject.toml (managed exclusively with `uv add`); `uv run`
resolves them automatically. Exit code is always 0: this tool measures;
pass/fail belongs to tools/lint_v0.py.
"""
from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

try:
    import yaml
except ImportError:
    sys.exit("PyYAML is missing from the project environment. Add it with: uv add pyyaml")

# This script lives in tools/ — one parent up is the registry root. Paths are
# derived from the script location so it works regardless of the caller's
# working directory.
REGISTRY_ROOT = Path(__file__).resolve().parents[1]

# Thresholds mirror lint rule QV-002 and the frontmatter spec, so metrics and
# lint never disagree about limits.
BODY_WARN_LINES = 400  # split-watch: approaching the 500-line best-practice limit
BODY_MAX_LINES = 500   # guide limit (QV-002 warning; >800 is a lint error)
DESC_MAX_CHARS = 1024  # frontmatter spec maximum for descriptions

FM_RE = re.compile(r"\A---\n(.*?)\n---\n?(.*)\Z", re.DOTALL)
MD_LINK_RE = re.compile(r"\[[^\]]*\]\(([^)]+\.md)\)")


def parse_asset(path: Path) -> dict:
    """Extract frontmatter and body metrics from one asset file.

    Parse failures are reported in the row, not raised: the doctor audits
    broken assets too, so a failure is data, not a crash.
    """
    row = {
        "id": path.parent.name if path.name == "SKILL.md" else path.stem,
        "kind": "skill" if path.name == "SKILL.md" else "shared-reference",
        "path": str(path.relative_to(REGISTRY_ROOT)),
        "body_lines": None,
        "desc_chars": None,
        "tags": [],
        "slots": [],
        "broken_links": [],
        "error": None,
    }
    text = path.read_text(encoding="utf-8")
    m = FM_RE.match(text)
    if not m:
        row["error"] = "missing or malformed frontmatter"
        return row
    try:
        fm = yaml.safe_load(m.group(1)) or {}
    except yaml.YAMLError as e:
        row["error"] = f"frontmatter YAML: {e}"
        return row

    body = m.group(2)
    row["body_lines"] = len(body.splitlines())
    row["desc_chars"] = len(str(fm.get("description", "")).strip())
    row["tags"] = list(fm.get("tags") or [])
    row["slots"] = sorted(
        set(fm.get("requires-one-of") or {}) | set(fm.get("requires-any") or {})
    )
    # Links to composition products resolve only after rendering: fixed requires
    # land at references/<id>.md, one-of picks at references/<slot>.md, and any
    # picks under references/<slot>/. Treat those as virtual targets, not broken.
    requires = set(fm.get("requires") or [])
    one_of = set(fm.get("requires-one-of") or {})
    any_of = set(fm.get("requires-any") or {})
    virtual_files = {f"references/{i}.md" for i in requires}
    virtual_files |= {f"references/{s}.md" for s in one_of}
    virtual_dirs = tuple(f"references/{s}/" for s in any_of)
    for link in MD_LINK_RE.findall(body):
        if (path.parent / link).resolve().exists():
            continue
        if link in virtual_files or link.startswith(virtual_dirs):
            continue
        row["broken_links"].append(link)
    return row


def load_known_tags() -> set[str]:
    taxonomy = REGISTRY_ROOT / "taxonomy.toml"
    if not taxonomy.exists():
        return set()
    data = tomllib.loads(taxonomy.read_text(encoding="utf-8"))
    return set(data.get("tags", {})) | set(data.get("groups", {}))


def flag(row: dict, known_tags: set[str]) -> str:
    flags = []
    if row["error"]:
        flags.append("PARSE-ERROR")
    if row["body_lines"] is not None:
        if row["body_lines"] > BODY_MAX_LINES:
            flags.append("SPLIT")
        elif row["body_lines"] > BODY_WARN_LINES:
            flags.append("SPLIT-WATCH")
    if row["desc_chars"] is not None and row["desc_chars"] > DESC_MAX_CHARS:
        flags.append("DESC-TOO-LONG")
    unknown = [t for t in row["tags"] if t not in known_tags]
    if unknown:
        flags.append(f"UNKNOWN-TAGS:{','.join(unknown)}")
    if row["broken_links"]:
        flags.append(f"BROKEN-LINKS:{','.join(row['broken_links'])}")
    return "; ".join(flags) if flags else "ok"


def print_table(rows: list[dict], known_tags: set[str]) -> None:
    print("| asset | kind | body lines | desc chars | tags | slots | flags |")
    print("|---|---|---|---|---|---|---|")
    for r in rows:
        print(
            f"| {r['id']} | {r['kind']} | {r['body_lines']} | {r['desc_chars']} "
            f"| {', '.join(r['tags']) or '-'} | {', '.join(r['slots']) or '-'} "
            f"| {flag(r, known_tags)} |"
        )
    skills = sum(1 for r in rows if r["kind"] == "skill")
    print(f"\n{skills} skill(s), {len(rows) - skills} shared-reference(s)")


def main() -> int:
    rows = [
        parse_asset(p)
        for p in sorted(REGISTRY_ROOT.glob("assets/skills/*/SKILL.md"))
        + sorted(REGISTRY_ROOT.glob("assets/shared-references/*.md"))
    ]
    known_tags = load_known_tags()
    if "--json" in sys.argv[1:]:
        for r in rows:
            r["flags"] = flag(r, known_tags)
        print(json.dumps(rows, indent=2))
    else:
        print_table(rows, known_tags)
    return 0


if __name__ == "__main__":
    sys.exit(main())

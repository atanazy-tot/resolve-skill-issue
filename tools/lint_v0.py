#!/usr/bin/env python3
"""lint v0 — parse-level registry checks.

Superseded by `quiver lint` (PRD §6.5, rules QV-001…QV-008) in M1.
Checks: frontmatter presence/validity, required keys, kind enum,
name==path conventions, bundle/TOML parseability, asset references,
requires existence.
"""
from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
errors: list[str] = []
warnings: list[str] = []

FM_RE = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)
VALID_KINDS = {"skill", "shared-reference"}


def frontmatter(path: Path) -> dict | None:
    m = FM_RE.match(path.read_text(encoding="utf-8"))
    if not m:
        errors.append(f"{path.relative_to(ROOT)}: missing YAML frontmatter block")
        return None
    try:
        fm = yaml.safe_load(m.group(1))
        return fm if isinstance(fm, dict) else {}
    except yaml.YAMLError as e:
        errors.append(f"{path.relative_to(ROOT)}: invalid frontmatter YAML: {e}")
        return None


def check_required(path: Path, fm: dict) -> None:
    rel = path.relative_to(ROOT)
    for key in ("kind", "name", "description"):
        if not fm.get(key):
            errors.append(f"{rel}: missing required frontmatter key '{key}'")
    kind = fm.get("kind")
    if kind and kind not in VALID_KINDS:
        errors.append(f"{rel}: kind '{kind}' not in {sorted(VALID_KINDS)}")


def check_skills() -> None:
    ref_ids = {f.stem for f in (ROOT / "assets/shared-references").glob("*.md")}
    for d in sorted((ROOT / "assets/skills").iterdir()):
        if not d.is_dir():
            continue
        f = d / "SKILL.md"
        if not f.exists():
            errors.append(f"{d.relative_to(ROOT)}: skill directory without SKILL.md")
            continue
        fm = frontmatter(f)
        if not fm:
            continue
        check_required(f, fm)
        if fm.get("kind") not in (None, "skill"):
            errors.append(f"{f.relative_to(ROOT)}: kind must be 'skill'")
        if fm.get("name") and fm["name"] != d.name:
            errors.append(
                f"{f.relative_to(ROOT)}: name '{fm['name']}' != directory '{d.name}'"
            )
        for req in fm.get("requires") or []:
            if req not in ref_ids:
                errors.append(
                    f"{f.relative_to(ROOT)}: requires unknown shared-reference '{req}'"
                )


def check_shared_references() -> None:
    for f in sorted((ROOT / "assets/shared-references").glob("*.md")):
        fm = frontmatter(f)
        if not fm:
            continue
        check_required(f, fm)
        if fm.get("kind") not in (None, "shared-reference"):
            errors.append(f"{f.relative_to(ROOT)}: kind must be 'shared-reference'")
        if fm.get("name") and fm["name"] != f.stem:
            errors.append(
                f"{f.relative_to(ROOT)}: name '{fm['name']}' != filename '{f.stem}'"
            )


def check_bundles() -> None:
    skill_ids = {d.name for d in (ROOT / "assets/skills").iterdir() if d.is_dir()}
    for f in sorted((ROOT / "bundles").glob("*.toml")):
        rel = f.relative_to(ROOT)
        try:
            data = tomllib.loads(f.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError as e:
            errors.append(f"{rel}: invalid TOML: {e}")
            continue
        for key in ("name", "description", "assets"):
            if key not in data:
                errors.append(f"{rel}: missing key '{key}'")
        for asset in data.get("assets", []):
            if asset not in skill_ids:
                errors.append(f"{rel}: unknown skill asset '{asset}'")


def check_taxonomy() -> None:
    f = ROOT / "taxonomy.toml"
    try:
        data = tomllib.loads(f.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as e:
        errors.append(f"taxonomy.toml: invalid TOML: {e}")
        return
    for section in ("tags", "groups"):
        if section not in data:
            errors.append(f"taxonomy.toml: missing section [{section}]")


def main() -> int:
    check_skills()
    check_shared_references()
    check_bundles()
    check_taxonomy()
    for w in warnings:
        print(f"WARN  {w}")
    for e in errors:
        print(f"ERROR {e}")
    print(f"\nlint v0: {len(errors)} error(s), {len(warnings)} warning(s)")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())

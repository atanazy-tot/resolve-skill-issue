---
kind: shared-reference
name: descriptions
description: >
  Playbook for writing and rewriting asset descriptions: the formula,
  anti-patterns, rewrite process, and before/after examples.
tags: [skill-authoring]
---
# Description rewrite playbook

## Contents

- The formula
- Anti-patterns
- Rewrite process
- Before / after

## The formula

```
<what it does — concrete verbs and objects>. USE WHEN <trigger contexts and terms a user types>.
```

- Third person, present tense.
- Key terms early — discovery matches on them.
- Both halves mandatory: what-without-when never triggers; when-without-what triggers wrongly.
- ≤ 1024 characters, no XML tags. Folded YAML (`description: >`) is fine.

## Anti-patterns

| Pattern | Example | Problem |
|---|---|---|
| Vague | "Helps with documents" | Matches everything, triggers for nothing |
| First/second person | "I can help you process PDFs" | Breaks discovery — descriptions are injected into the system prompt |
| Implementation dump | "Uses pdfplumber 3.1 with regex fallback" | Belongs in the body, not the description |
| Missing when | "Extracts text from PDFs" | Why pick this over any other skill? |
| Marketing | "A powerful, blazing-fast tool" | Zero information |

## Rewrite process

1. Read the asset's body first — the description must cover what IS there, not what should be.
2. List trigger situations: what is the user doing or saying when this asset should load?
3. Draft with the formula; name file types, tools, and tasks explicitly.
4. Check against the Discovery section of [audit-checklist.md](audit-checklist.md).
5. Show before → after in the report; apply directly (medium freedom).

## Before / after

Weak:
```yaml
description: Helps with code review
```

Strong:
```yaml
description: >
  Reviews staged changes for bugs, convention violations, and missing tests.
  USE WHEN the user asks for a code review, pre-commit check, or PR feedback.
```

Weak:
```yaml
description: I can help you write commit messages
```

Strong:
```yaml
description: >
  Generates conventional-commit messages from git diffs.
  USE WHEN the user asks for a commit message or is about to commit staged changes.
```

Weak:
```yaml
description: Processes data files
```

Strong:
```yaml
description: >
  Analyzes Excel spreadsheets: pivot tables, charts, column statistics.
  USE WHEN working with .xlsx files, spreadsheets, or tabular data.
```

Reference point: this skill's own description passes the checklist — use it as the house style.

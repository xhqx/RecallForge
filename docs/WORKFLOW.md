# RecallForge workflow

RecallForge is project-scoped evidence, not a global memory stream. SQLite is
authoritative; Markdown is export and skill guidance. The skills may help an
agent decide when to read or propose a write, but they do not observe all
work.

## Invocation limits

The recall skill is narrowly applicable to prior evidence for the current
project, particularly before debugging. Capture applies only to a non-obvious
lesson, decision, constraint, or pattern from current work. Promotion applies
only to an explicit request to draft a skill from verified evidence. None of
these skills should run for general research, unrelated tasks, or another
project.

There is no automatic background transcript watcher, shell hook, or daemon.
An agent must initiate capture while context is fresh. Automatic skill
selection, when provided by a host, is only a suggestion based on the skill
description and current context; it is not a guarantee that every lesson is
captured.

## Project identity and safety

Before any memory operation, determine the canonical project root and call
`project_list`. Proceed only with an exact registered-root match. Never guess
an id, use a directory name as identity, or query across projects. Recall is
read-only. Memory writes require explicit write opt-in for that project.

## Install and restart

Install the three skill directories where the agent host discovers project or
user skills; keep their names and `SKILL.md` files intact. Skill discovery is
host-specific. After installing or editing them, reload or restart the agent
host if it caches skills, and verify that the narrow descriptions are visible.
Installation does not import existing memory, enable writes, or install a
draft skill.

## Write opt-in

Treat capture and promotion as opt-in per project. Without explicit user or
project authorization, an agent may search and may display a proposed entry or
draft, but must not call `memory_save` or alter skill instructions. The
MCP write is `memory_save`; the CLI fallback is:

```text
recallforge save --project ID --file ENTRY_JSON
```

Use the exact project id found through `project_list` / `recallforge project list`.
There is no implicit global write or automatic promotion.

## Evidence lifecycle

New records default to `candidate`. Keep a candidate when a useful observation
is not fully verified. Set `verified` only with nonempty verification and
supporting sources. Updates require `expected_revision` and retain history;
resolve a conflict by reading the latest entry rather than overwriting it.

Use `stale` when its source no longer describes the current project, and
`superseded` when a newer record replaces it. Normal search excludes stale and
superseded records; include inactive records only deliberately and label them.
Before relying on any result, fetch it with `memory_get` (or `recallforge get`)
and compare its cited source with the current source version. Never silently
revive stale evidence.

Promotion reads verified records and produces a reviewable draft containing
applicability, counterexamples, and evidence citations. It never silently
installs a skill or changes hidden instructions. Installation requires user authorization, which may already be present in the session.
Existing project write authorization is sufficient for subsequent captures; do not ask again for each lesson.

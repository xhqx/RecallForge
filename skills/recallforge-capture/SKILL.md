---
name: recallforge-capture
description: Use only to record a non-obvious, project-specific lesson, decision, constraint, or pattern discovered during current work, including useful hypotheses labeled as candidates. Exclude routine progress and unrelated project information.
---

# Capture evidence while it is fresh

Capture a useful finding promptly; mark it verified only after checking the result. Capture is a
project-scoped write and requires explicit write opt-in for that project. An explicit standing project instruction or session authorization is sufficient; do not ask for each record. If
write is not opted in, show the proposed entry instead of saving it.

## Select the exact project

Determine the canonical project root and call `project_list`. Use only the
project whose registered root exactly matches it. Never guess an id or write
to another project. If there is no exact match, stop and report that the
project is not registered. If MCP is unavailable, use `recallforge project
list` to perform the same exact-root check.

## Check for duplicates

Before creating an entry, search the matching project with the MCP tool
`memory_search`, or the CLI fallback:

```text
recallforge search --project ID --query "distinctive lesson or symptom" --mode hybrid --limit 5
```

Inspect a plausible match with `memory_get` or:

```text
recallforge get --project ID --id ENTRY_UUID
```

Preserve a useful candidate; do not discard it merely because it is not yet
verified. Do not create a duplicate. If an existing entry needs a meaningful
correction, update that entry rather than creating a near-duplicate.

## Write a factual entry

Use `memory_save({"project":"ID","entry":{...}})`, or save equivalent JSON
with:

```text
recallforge save --project ID --file ENTRY_JSON
```

Include a concise `title`, one of `lesson`, `decision`, `constraint`, or
`pattern` as `kind`, and concrete `problem`, `cause`, `solution`,
`verification`, `sources`, `applies_to`, `tags`, and `failed_attempts` fields.
Each source needs a `reference` and an explanatory `note`. Prefer repository-relative file paths, commit IDs, and exact test commands.
Exclude secrets and private conversation content; synthetic examples belong only in public documentation.

Set `status` to `verified` only when `verification` is nonempty and the entry
has nonempty sources that support it. Otherwise use `candidate`, preserve the
uncertainty and failed attempts, and do not manufacture evidence. An exact
content duplicate is returned by the store; still report that no new lesson
was created.

For an update, include the existing `id` and its current
`expected_revision`. Re-read the entry after a revision conflict and reconcile
instead of overwriting someone else’s change. Revisions retain history.
Never silently change a verified lesson into an instruction or install
anything as a side effect of capture.

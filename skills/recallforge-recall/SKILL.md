---
name: recallforge-recall
description: Use only for project-specific prior lessons, decisions, constraints, or patterns that may inform current work, especially before debugging. Do not use for general research, unrelated tasks, or cross-project memory.
---

# Recall project evidence

Use this skill when the current work needs evidence from the current project’s
RecallForge memory. Recall is read-only.

## Identify the project first

1. Determine the canonical root of the current project.
2. Call the MCP tool `project_list`.
3. Continue only when one project has an exact canonical-root match. Use that
   project’s id. Never infer an id from a directory name, guess a project, or
   query across projects. If there is no exact match, skip memory retrieval and continue the
   user task from current evidence. Registration is a separate explicit action.

With MCP, use the matching id with:

```text
memory_search({"project":"ID","query":"focused symptom or topic","mode":"hybrid","limit":5})
memory_get({"project":"ID","id":"ENTRY_UUID"})
```

If MCP is unavailable, use the equivalent CLI sequence:

```text
recallforge project list
recallforge search --project ID --query "focused symptom or topic" --mode hybrid --limit 5
recallforge get --project ID --id ENTRY_UUID
```

## Recall before debugging

Search before changing code or repeating a failed debugging path. Query the
observable symptom, component, error, and relevant constraint rather than a
vague request. Search results are leads: candidates must be labeled as such,
and stale or superseded entries are not current evidence by default.

Fetch each relevant result with `memory_get` (or `recallforge get`) before
relying on it. Check its status, revision, verification, failed attempts, and
source references. Compare those references with the current source version,
tests, or commit in the checkout. If the source has changed or cannot be
checked, state the uncertainty and do not present the entry as current fact.
When reporting a recalled item, include its id, revision, and source reference.

Do not silently update memory, promote an entry, edit instructions, or widen a
query to another project. Existing project authorization can enable capture; do not request it again for each finding.

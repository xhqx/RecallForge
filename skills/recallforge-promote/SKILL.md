---
name: recallforge-promote
description: Use only when asked to turn verified, project-specific RecallForge evidence into a reviewable draft skill. Do not use for ordinary recall, capture, generic prompt writing, or unverified notes.
---

# Promote evidence to a draft skill

Promotion is a proposal, not installation. It requires an explicit request
and explicit project write opt-in if the proposal is to be stored. Never make
hidden instruction changes.

## Gather evidence from one project

Determine the canonical project root and call `project_list`. Require an exact
root match and use only that project id; never guess or combine projects. If
there is no exact match, stop.

Find candidate evidence with the MCP tool `memory_search`, then fetch every
source used with `memory_get`. The CLI fallback is:

```text
recallforge project list
recallforge search --project ID --query "recurring verified practice" --mode hybrid --limit 5
recallforge get --project ID --id ENTRY_UUID
```

Use only entries whose fetched status is `verified`. Exclude candidates,
stale entries, and superseded entries. Check each entry’s revision and source
against the current source version; if evidence is stale, conflicting, or
uncheckable, say so and do not promote it as current guidance. A single
verified entry may support a narrow draft; repeated or corroborating evidence
should be preferred.

## Produce a reviewable draft

Write a portable English draft with:

- a narrow name and trigger description, including explicit exclusions;
- applicability: project context, prerequisites, and when not to use it;
- the evidence-backed procedure, with uncertainty called out;
- counterexamples, failure modes, and cases where the procedure does not
  apply;
- evidence citations containing project id, entry id, revision, and source
  references; and
- a review checklist and a clear statement that the draft is not installed.

Present the draft for human review. There is no planned promotion or install
MCP tool: do not call `memory_save` as a substitute for creating a skill draft,
modify installed skills without authorization, or claim installation/restart happened. Install or update a skill only when the user has authorized that action;
existing explicit authorization remains valid. Otherwise leave the draft for review. If the
approved proposal is saved as memory, use `memory_save` (or
`recallforge save`) with an appropriate status and preserve its evidence;
never silently auto-promote.

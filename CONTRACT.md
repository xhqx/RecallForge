# v0.1 implementation contract

Binary: recallforge. Global `--data-dir DIR` (RECALLFORGE_DATA_DIR), then subcommand.
Default storage is OS application data outside repositories. Local multilingual E5 small 384 dimensions;
sqlite-vec vec0 exact cosine KNN, project/status prefilters, FTS5 and reciprocal rank fusion.

CLI planned: `init`; `project add --id ID --name NAME --root PATH`; `project list`;
`save --project ID --file JSON [--lexical-only]`; `search --project ID --query TEXT
--mode hybrid|lexical|semantic [--limit 5] [--include-inactive]`; `get --project ID --id UUID`;
`history --project ID --id UUID`; `link --project ID --from UUID --to UUID --relation relates_to|caused_by|solved_by|supersedes`;
`neighbors --project ID --id UUID`; `reindex --project ID`; `export --project ID --format json|markdown`;
`backup --output PATH`; `doctor`; `mcp`.

Save JSON: optional id, expected_revision for updates (required if id supplied),
title, kind (lesson|decision|constraint|pattern), problem, cause, solution,
verification, status (candidate|verified|stale|superseded), tags [], sources [],
applies_to, failed_attempts []. Source object: {"reference":"file/commit/test reference","note":"explanation"}.
Verified records require nonempty verification AND sources. Default candidate.
Exact-content duplicate create returns existing entry. Update increments revision with CAS and retains history.
Local vectors are automatic unless lexical-only; lexical-only clears previous vector on update.
Search excludes stale/superseded by default; candidate results explicitly labeled. Search shows brief result + source
reference and revision; get fetches full record. No implicit cross-project query or global auto-promotion.
MCP tools: project_list, memory_search, memory_get, memory_save, memory_link, memory_neighbors,
memory_history, memory_reindex, memory_export, memory_doctor. Inputs mirror CLI snake_case (save uses entry object).
There is no automatic background transcript watcher. Skills initiate capture while context is fresh.
Skill promotion drafts are generated/reviewed by agents from verified records, never silently installed.
Repository code is public; runtime database is private local data. No migration of existing user memory.

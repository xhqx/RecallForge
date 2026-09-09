# Architecture

One Rust process owns SQLite and local inference; no HTTP server or Python inference sidecar.
The CLI and MCP server share the same service layer and data validation.

| Module | Responsibility |
| --- | --- |
| `model` | Public entry types and evidence validation |
| `store` + `schema.sql` | Atomic persistence, revisions, FTS/vector consistency |
| `registry` | Project roots and scoped reads |
| `embedding` | Lazy E5 loading, query/passage prefixes, overlapping text chunks |
| `search` | Project/status filtering, lexical/vector retrieval, document rank fusion |
| `graph` | Typed project-local edges |
| `maintenance` | Online backup and database health |
| `service` | Shared application operations and exports |
| `mcp` | Bounded line-delimited JSON-RPC and MCP tool metadata |
| `main` | CLI argument parsing and JSON output |

## Write transaction

Validate the project and payload, generate embeddings outside the write transaction, then begin
an immediate SQLite transaction. Check expected revision, write the new content and history,
update FTS through triggers, replace vector chunks, and commit. Failure rolls back the whole
entry mutation. SQLite serializes writers with a bounded busy timeout; this is not a distributed
multi-writer protocol. Content fingerprinting deduplicates equivalent complete create payloads.

## Vector space

The configured embedding model and preprocessing version are stored in database metadata.
A binary declaring another model refuses the database rather than mixing incompatible vectors.
E5 uses `query:` and `passage:` prefixes, including for non-English input. 384-dimensional
vectors are validated for dimension, finite values and a nonzero norm.

Text is chunked into at most 800 Unicode characters with 200-character overlap. This is a
conservative character heuristic, not exact tokenizer-aware chunking. The vector table uses
cosine distance. Project and status predicates are applied inside KNN before selecting candidates.
The service groups matching chunks by entry and fuses lexical/vector ranks at document level.

## Extensions

To introduce another model, use an explicit migration and rebuild the vector table; the
model metadata check intentionally blocks accidental upgrades across vector spaces. Improve
retrieval against a labeled project-specific evaluation before adding rerankers or ANN.
For large datasets, measure candidate diversity, latency and recall before changing storage.
The typed graph is stored relationally; neighboring entries are fetched explicitly, not blindly
injected into every search response.

## MCP scope

The synchronous server exposes tools only, over stdio. Initialize, ping, tools/list, tools/call,
and notifications are supported; resources, prompts, subscriptions and HTTP are not advertised.
Frames are capped at 1 MiB. Entry payloads are capped at 64 KB. Inference and tools are serialized
within a server process. Graceful cancellation of in-flight inference is not implemented.
The host controls process lifetime, authentication and access to the local data directory.

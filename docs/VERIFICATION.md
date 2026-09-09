# v0.1 verification

Local verification on macOS Apple silicon with Rust 1.98:

- 17 deterministic Rust tests: project isolation, exact duplicate suppression, revision conflicts,
  concurrent creates, FTS updates, inactive vector prefilters, index repair, backup/restore,
  database ownership guards, malformed input and MCP negotiation.
- CLI/stdio MCP smoke: 23 checks passed against the built binary.
- Real local multilingual E5 evaluation: 6/6 correct first results and 6/6 top-three results,
  for both semantic-only and hybrid search. Six synthetic Russian queries / English lessons.
- `cargo fmt --check` and Clippy with warnings denied passed.
- All three agent skills passed the skill frontmatter validator; CLI/MCP examples were reviewed
  against the implemented interface.

The model snapshot is pinned to `614241f622f53c4eeff9890bdc4f31cfecc418b3`. Tests run local
ONNX inference, not mock embeddings. Deterministic storage tests use controlled vectors to
isolate ranking/filter/transaction behavior from model quality.

Independent read-only review identified a missing-vector repair bug in duplicate saves and
incomplete initialize validation; both were fixed and covered by regression tests. A suggested
rejection of unknown protocol versions was not adopted: MCP requires responding with a
supported version for negotiation. See the [MCP lifecycle specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle#version-negotiation).

## Practical limits of this evidence

This small evaluation establishes that multilingual retrieval is functioning. It does not
measure general precision/recall, adversarial robustness, or large-dataset performance.
No million-record benchmark, multi-machine sync or Windows installation was tested.
Skill description matching and automatic capture remain host/model behavior, not a hard guarantee.
GitHub Actions runs the deterministic checks on Linux and macOS; inspect the current workflow
run for the published commit rather than treating this document as its CI status.

---
id: SPEC-checked-semantics-v1
companions:
  - implementation-contract.md
  - ../../planning-artifacts/architecture/architecture-Morva-2026-08-13/ARCHITECTURE-SPINE.md
  - ../../planning-artifacts/architecture/architecture-Morva-2026-08-13/MORVA-CHECKED-SEMANTICS-PROTOCOL-v1.md
  - ../../planning-artifacts/architecture/architecture-Morva-2026-08-13/VALIDATION-REPORT.md
  - ../../../docs/project-context.md
  - ../../../docs/architecture.md
  - ../../../docs/testing-strategy.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability only.

# Checked Semantics v1 Production Slice

## Why

Morva needs a stable machine boundary that exposes checker-owned facts without letting CLI or AI consumers reconstruct semantics. The immediate mandate is to turn the validated protocol design into the smallest production-quality, deterministic single-file core slice before any AI orchestration is built.

## Capabilities

- **CAP-1**
  - **intent:** A core caller can submit one logical source name and exact UTF-8 Morva source and receive a version-1 checked-semantics document derived from the existing parser, checker, and analyzer.
  - **success:** Public-API tests prove valid, invalid, and warning-only inputs produce protocol documents with the required status, coverage, provenance, and checked-model gate.

- **CAP-2**
  - **intent:** A core caller can determine whether a checked-semantics document satisfies all single-file v1 invariants before it crosses the machine boundary.
  - **success:** Public-API negative tests prove wrong digests, invalid ranges, inconsistent status/coverage, duplicate identities, dangling semantic references, and invalid model/type relationships are rejected with typed errors.

- **CAP-3**
  - **intent:** A core caller can serialize a conforming document into canonical, reproducible JSON.
  - **success:** Identical logical names, source bytes, producer/language versions, and capabilities produce byte-identical RFC 8259 UTF-8 JSON with the specified ordering, two-space indentation, and final LF.

- **CAP-4**
  - **intent:** Maintainers can continuously verify the production slice against independent protocol examples and failure cases.
  - **success:** Repository-owned tests cover the published digest vectors, the three real AI sample classes, deterministic serialization, and deliberately malformed envelopes without internal mocks or one-off external validators.

## Constraints

- `morva-core` owns projection and conformance; CLI, filesystem discovery, and AI workflow behavior are outside this slice.
- Protocol types form an explicit closed v1 algebra and MUST NOT directly serialize AST layout, private checker indices, virtual spans, or unresolved facts.
- Invalid parse/check results carry `checked_model: null`; warning-only results remain valid while reporting incomplete coverage.
- Exact UTF-8 source bytes and half-open local byte ranges are authoritative; no absolute host path, timestamp, randomness, or process value enters output.
- Existing parse, check, analyze, simulate, diagnostics, examples, and CLI behavior remain compatible.
- TDD uses only the agreed public producer, validator, and canonical-serialization seams; no internal mocks or horizontal test batch.
- `[ASSUMPTION]` SHA-256 is the v1 revision algorithm only after dependency/change-control approval and actual known-answer tests.

## Non-goals

- Multi-file project production, CLI command/flag design, JSONL, streaming, or source redaction.
- AI provider calls, prompting, diagnosis, revision generation, persistence, MCP, authentication, or automatic write-back.
- Cross-revision semantic identity, incremental analysis, language expansion, or changes to simulator behavior.
- Claiming full protocol conformance before the production digest and invariant validator pass their complete test corpus.

## Success signal

One public `morva-core` call can produce and canonically serialize a valid single-file v1 document, while independent public tests prove invalid, warning-only, deterministic, wrong-digest, and dangling-reference behavior. The full workspace quality gate and existing examples remain green.

## Assumptions

- `[ASSUMPTION]` The approved implementation will use maintained Rust ecosystem implementations for SHA-256 and JSON rather than an ad-hoc cryptographic or JSON encoder.

## Open Questions

- Does dependency/change control approve adding `sha2`, `serde` with derive, and `serde_json` to `morva-core` for this production slice?

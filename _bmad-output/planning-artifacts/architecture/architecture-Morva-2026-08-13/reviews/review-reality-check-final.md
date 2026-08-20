# Final Reality Gate — Morva Checked Semantics Protocol

> Review date: 2026-08-13
> Scope: revised final architecture spine, protocol, schema, validation report, and current Morva repository contracts
> Mode: read-only gate review
> Verdict: **PASS — architecture/design gate; NOT a production-release approval**

## Verdict

The revised candidate passes the reality gate as an architecture and implementation-specification substrate. All previously blocking protocol contradictions and namespace/conformance gaps are closed. No new architecture-level integration blocker was found.

This verdict does **not** assert that `morva.checked-semantics` v1 is implemented or shippable. The documents now correctly make SHA-256 approval plus executable core-producer conformance evidence a hard pre-production gate, and the validation report clearly distinguishes prototype shape evidence from production proof.

## Closure of previous blockers

### F1 — Source-less project location: CLOSED

The protocol's top-level invariant now says every **source-owned** location uses a source-local half-open byte range and explicitly permits only a source-less core project finding to use subject location. This agrees with:

- AD-3's source-owned qualification;
- AD-14's `ProjectDiagnostic::Project` boundary;
- the schema's `sourceLocation | subjectLocation` finding union;
- the validator rule restricting subject locations to source-less project findings;
- current `Project::parse([]) -> MORVA2023` repository behavior.

Semantic nodes and coverage entries remain source-owned, so the exception does not weaken exact provenance.

### F2 — Schema identity: CLOSED

The unregistered `urn:morva:...` `$id` has been removed. The schema retains only the official Draft 2020-12 `$schema` URI and uses local fragment references, so it no longer claims an uncontrolled HTTPS authority or unregistered URN namespace.

The schema parses successfully as JSON with `jq empty`.

### F3 — Digest/order/cardinality conformance: CLOSED

AD-13 and the protocol validator now explicitly require:

- valid, unique logical names;
- canonical exact-UTF-8 name-byte order;
- consecutive `source:<index>` IDs;
- recomputation of every exact-content source digest;
- exactly one source and equal subject/source revision for file subjects;
- recomputation of project revision over the exact canonical framing;
- `sources: []` only for a source-less invalid project result;
- no zero-source file or valid checked model;
- source-reference/range, model completeness, semantic-key, expression/type, clause/phase, and shell-provenance invariants.

This now supports the claim that schema plus the core-owned validator—not schema alone—defines protocol conformance.

The published SHA-256 vectors independently reproduce:

- empty file bytes: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`;
- empty project framing: `9e6148ade2eb6fb7153b1c330e6f2db3f4d4e88a2c00b9f41a29d7ffb9befc9f`;
- published UTF-8-name/CRLF two-source preimage: `5d7da2df72ca38343090ae86ae63ab778d94b33c39be3fb34b8b81057c3469ee`.

### F4 — SHA-256 production boundary: CORRECTLY GATED

SHA-256 remains a current, appropriate content-identity primitive under NIST FIPS 180-4. The documents no longer overclaim origin, authorship, authenticity, or approval.

AD-11 and the protocol now explicitly say the architecture/prototype is not production-shippable until:

1. Morva dependency/change-control approval is recorded; and
2. the actual core producer passes known-answer and protocol conformance tests.

Because no shipping claim is made, this explicit pre-production gate is not an architecture failure. It becomes a blocking acceptance criterion for the future implementation.

## Closure of previous non-blocking items

- **Logical identity:** closed at design level. Logical names are separate from host paths, non-empty UTF-8 final components, non-normalized, separator-free, duplicate-rejected, and canonically sorted in core.
- **Integer range:** closed. Protocol integer values are canonical decimal strings limited to the current parser's signed `i64` range, and the core validator checks that boundary.
- **Diagnostic category:** closed at design level. The protocol assigns category from typed pipeline origin in core and explicitly forbids message-text inference by CLI/adapters.
- **Semantic keys:** strengthened. They are unique, fully resolved, opaque, and revision-local; consumers may compare but not parse them.
- **AI decision duplication:** closed. The human outcome list now has one `accept`, one `reject`, and one `block` entry.
- **BCP 14:** closed with the standard RFC 2119/RFC 8174 interpretation clause.

## Repository and standards reality confirmed

- Root manifest remains Rust edition 2024 and package version 0.1.0.
- Rust 2024, RFC 8259, JSON Schema Draft 2020-12, and SHA-256/FIPS 180-4 remain valid named baselines.
- `morva-core` owns language semantics; `morva-cli` owns filesystem discovery/read/render/exit behavior.
- CLI project discovery currently uses exact UTF-8 filename-byte ordering and operational failures use exit 2 without stdout, matching AD-9/AD-14.
- Core locations remain UTF-8 byte spans; multi-file merged spans are private and map back through `SourceMap`.
- Current warnings and coverage behavior (`MORVA5001`, `MORVA5002`) support the independent validity/coverage model.
- Current requirements preserve human-reviewed `.morva` as Source of Truth and prohibit silent AI authority.
- The checked algebra is now closed over current L0 declarations, expressions, assignments, clauses, scenarios, bindings, operators, and types, with `language_version = 0.1` and fail-closed unknown semantic discriminants.

## Non-blocking implementation notes

1. Define “path separator” as exact forbidden code points/bytes in the implementation specification so logical-name validation cannot vary by host platform. This is a wording precision issue, not a remaining ordering/identity divergence.
2. The existing public `Diagnostic` does not yet store lexical/syntax category. The implementation must retain typed pipeline origin inside the future core protocol path; it must not classify in the CLI or parse message text.
3. Integer range is enforced by the specified core validator rather than fully expressible in the current regex-only schema. That is consistent with the declared two-layer conformance model.
4. The architecture workspace does not contain a reusable production conformance runner. Add automated wrong-digest, reordered-source, invalid-cardinality, subject-location, semantic-reference, and i64-boundary negatives with the implementation.
5. Serializer failure, broken stdout, peak-memory limits, and streaming thresholds remain correctly deferred to the implementation specification, which must preserve atomic/no-partial output.

## Verification notes

- Schema JSON parse: passed (`jq empty`).
- Published digest vectors: independently reproduced.
- `git diff --check` for the architecture workspace: passed.
- `cargo test -p morva-core --lib`: passed (2 tests).
- `cargo test -p morva-cli`: passed (44 tests).
- Full `cargo test --workspace`: currently fails to compile the unrelated untracked `crates/morva-core/tests/scale.rs` because it references nonexistent `SimulationPhase::ALL`. This predates/is outside the reviewed architecture deliverables and does not invalidate the protocol decisions, but the repository owner should resolve it before using a green workspace test run as implementation evidence.
- `check-jsonschema` is not installed in the current shell; this review therefore confirms schema JSON integrity and internal rule alignment, while relying on the validation report for its prior Draft 2020-12 metaschema/sample runs. Production work must make those checks repository-owned and repeatable.

## Gate conditions carried forward

The architecture reality gate is **PASS**. Production v1 remains gated on all of the following:

1. approved SHA-256 implementation/dependency choice;
2. implemented core projection and semantic-envelope validator;
3. actual producer execution of the published hash vectors and full positive/negative conformance suite;
4. atomic CLI machine-output contract and regression tests;
5. human review of the implementation specification and resulting code.

Until those conditions pass, the correct claim remains “validated design/prototype,” not “implemented protocol” or “production-ready v1.”

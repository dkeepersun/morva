# Implementation Contract

## Approved first slice

The first slice is single-file only. Its public seam accepts a logical final filename and an exact UTF-8 source string, runs existing parse/check/analyze behavior, projects the v1 protocol data model, validates the resulting envelope, and returns a canonical JSON representation on request.

Suggested public surface; exact Rust spelling may change during the first red/green cycle only if the observable seam remains equivalent:

```rust
pub fn checked_semantics_single_file(
    logical_name: &str,
    source: &str,
) -> Result<CheckedSemanticsDocument, ProtocolBuildError>;

impl CheckedSemanticsDocument {
    pub fn validate(&self) -> Result<(), ProtocolInvariantError>;
    pub fn to_canonical_json(&self) -> Result<String, ProtocolSerializationError>;
}
```

`ProtocolBuildError` is reserved for invalid logical input or an internal projection invariant failure. Morva lexical, syntax, and semantic diagnostics are successful protocol production with `result.status = invalid`, not build errors.

## Module ownership

| Area | Owner | Rule |
|---|---|---|
| Public protocol algebra | `morva-core::protocol` | Explicit structs/enums matching v1; independent of AST serialization |
| Single-file projection | `morva-core::protocol` | Calls existing parser/checker/analyzer; does not duplicate language rules |
| Revision digest | `morva-core::protocol` | Exact source bytes; lowercase SHA-256 hex; known-answer tested |
| Invariant validation | `morva-core::protocol` | Runs before any serialization; returns stable typed error variants |
| Canonical JSON | `morva-core::protocol` | Struct field order, fixed capability order, two spaces, final LF |
| CLI exposure | Deferred | No command, flag, stdout, or exit-code change in this slice |

## TDD seams and vertical cycles

Tests live in `crates/morva-core/tests/checked_semantics.rs` and observe public APIs only.

1. **Valid tracer bullet:** minimal checked source produces `valid`, complete coverage, exact inline source/revision, and a non-null system model.
2. **Invalid tracer bullet:** an existing semantic error produces an error finding and `checked_model: null` without turning protocol production into an API error.
3. **Coverage tracer bullet:** compatibility or soft behavior produces `valid`, a coverage warning, and `fully_modeled: false`.
4. **Canonical JSON tracer bullet:** fixed input equals an independently checked literal and repeated serialization is byte-identical.
5. **Validator tracer bullets:** mutate one public document at a time to prove wrong digest, out-of-range location, status mismatch, duplicate IDs/keys, dangling keys, and incompatible expression/clause facts are rejected.

Each cycle is one failing public behavior test followed by the minimum implementation needed to pass. Internal helpers are not tested directly.

## Single-file projection rules

- Reject an empty logical name or a name containing `/` or `\\`; do not normalize Unicode.
- Use `source:0`; file subject revision equals the exact-source revision.
- Map lexer/parser/checker diagnostics from typed pipeline origin, never message text. Preserve code, message, order, and local span.
- Map analyzer notices one-to-one into typed coverage warnings and `coverage.unmodeled` records.
- Parse failure sets coverage to unavailable and omits the checked model.
- Semantic failure may retain complete coverage information but always omits the checked model.
- A zero-error document emits the complete closed L0 checked-model algebra. Every collection is present even when empty.
- `node_id`, `finding_id`, and `semantic_key` are unique, opaque, deterministic, and revision-local. Consumers cannot depend on their textual construction.

## Conformance rules for this slice

Before serialization, the validator proves at minimum:

- exact source digest recomputation, file/source revision equality, one source, and `source:0`;
- required protocol identity, language version, and canonical capability list;
- every source-owned range is ordered, in bounds, and on UTF-8 boundaries;
- status/error/checked-model and coverage/warning relationships;
- unique finding IDs, node IDs, and semantic keys;
- all semantic keys and node-owned locations resolve inside the same revision;
- checked-model completeness relative to the source projection;
- normalized type, expression, assignment, path-root, clause/state-phase, and scenario relationships.

The validator returns a typed error with a stable machine-oriented variant and contextual identity. Human-facing wording is not a compatibility boundary in this slice.

## Canonical JSON rules

- Serialize only a document that passes `validate()`.
- Emit UTF-8 RFC 8259 JSON, two-space indentation, and one final LF.
- Struct declaration order defines object member order; JSON object order remains non-semantic.
- Mandatory capabilities use protocol order. No optional capability is introduced in this slice.
- Do not emit absolute paths, timestamps, random values, process data, or Rust debug names.

## Dependency decision [ASSUMPTION]

Recommended production choice:

- `sha2` for reviewed SHA-256 behavior and known-answer interoperability;
- `serde` with derive for the explicit protocol algebra;
- `serde_json` for correct escaping and deterministic struct serialization.

This is preferable to maintaining custom cryptography or a hand-written JSON escaper. It increases the dependency surface of the currently dependency-free core, so no Cargo manifest or lockfile change is authorized until the open question in `SPEC.md` is approved.

## Completion gate

Run and pass:

```text
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p morva-cli -- check examples/order.morva
cargo run -p morva-cli -- parse examples/order.morva
cargo run -p morva-cli -- inspect examples/order.morva
cargo run -p morva-cli -- simulate examples/order.morva NormalConfirmation
```

Also demonstrate that two repeated canonical serializations are byte-identical and that the published empty-source SHA-256 vector is exact. Multi-file examples remain regression checks, not protocol-production coverage for this slice.

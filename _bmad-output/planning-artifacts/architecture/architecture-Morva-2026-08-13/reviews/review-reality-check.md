# Reality Check Review — Morva Checked Semantics Protocol

> Review date: 2026-08-13
> Reviewed artifact: `ARCHITECTURE-SPINE.md` and named companions
> Review mode: repository evidence plus current primary standards
> Verdict: **CHANGES REQUIRED before implementation specification**

## Executive verdict

The architecture spine is directionally consistent with Morva's current ownership, safety, source-of-truth, diagnostic, ordering, and coverage contracts. Its named baseline technologies are current: the workspace really uses Rust edition 2024 and version 0.1.0; RFC 8259 remains JSON's Internet Standard; JSON Schema Draft 2020-12 is still the current JSON Schema version; and SHA-256 remains specified by NIST's current FIPS 180-4 publication, although NIST has announced a revision of that publication.

The draft is **not yet fully reality-checked as an implementable v1 contract**. Two protocol boundaries conflict with, or are underspecified relative to, current repository behavior: project-level diagnostics without a source cannot be represented, and reproducible logical identity for single-file input is undefined while the CLI currently retains the caller-supplied path. SHA-256 is correctly marked `[ASSUMPTION]`, but neither its framing implementation nor its claimed provenance semantics has been validated. The schema identifier also claims an unverified `morva.dev` namespace.

## Findings

### RC-1 — BLOCKER: the envelope cannot represent Morva's existing source-less project diagnostic

**Claim under review**

- AD-2 says any lexical, syntax, **project**, or semantic error produces an invalid protocol result.
- The protocol says parse, project-assembly, and semantic errors share one result shape.
- Every finding and every protocol location requires a `source_id` and source-local byte range.
- The schema requires at least one source.

**Repository reality**

`Project::parse` has a public, existing project error for an empty source set: `MORVA2023`, represented as `ProjectDiagnostic::Project` with no `SourceId` (`crates/morva-core/src/project.rs:153-175`). `ProjectDiagnostic::source_id()` explicitly returns `None` for this variant. The current CLI prevents this particular core result by rejecting an empty discovered directory as an input/IO-style exit-2 condition, but the proposed producer is core-owned and the public core API remains able to produce it.

The v1 schema cannot encode that state because:

- `sources` has `minItems: 1`;
- `finding.primary_location` is mandatory;
- `location.source_id` is mandatory.

Therefore the universal claim “any project error” is false for the current core contract. This is not a cosmetic schema gap: inventing a source or mapping `Span::default()` to the first file would violate AD-3's exact provenance rule.

**Required resolution**

Choose and specify one of these boundaries before implementation:

1. define empty input/discovery failures as pre-protocol producer failures and narrow AD-2 plus the protocol wording accordingly; or
2. support source-less findings (and possibly a zero-source envelope) with an explicit project-level location variant.

Do not synthesize a source-local location for `ProjectDiagnostic::Project`.

### RC-2 — HIGH: logical source/subject naming is not deterministic against the current single-file CLI contract

**Claim under review**

- AD-3 forbids absolute host paths.
- AD-9 promises reproducible emission and no absolute paths.
- The protocol says `source_id` derives from a “logical input name,” while project names retain CLI discovery order.
- AD-11 hashes source names into the project revision.

**Repository reality**

Project input is well defined: the CLI accepts direct-child lowercase `.morva` regular files, requires UTF-8 names, and sorts them by filename bytes. This is documented and implemented (`docs/project-context.md`; `crates/morva-cli/src/main.rs`, `discover_project_sources`).

Single-file identity is different. `read_source` currently assigns `CliSource.name = path.to_string_lossy()`, preserving whatever relative or absolute spelling the caller supplied. The CLI accepts any file path; `.morva` is a convention, not a required suffix. No current contract defines whether protocol `subject.name`, `sources[].name`, or `source_id` uses basename, normalized relative path, the raw CLI argument, or another caller-provided logical name.

Consequences:

- an absolute single-file invocation cannot be projected directly without violating AD-3/AD-9;
- basename-only identity can collide in external multi-candidate workflows;
- raw relative path identity changes with working directory or spelling;
- project digest reproducibility depends on a precise byte-level definition of source names, while file subject identity remains underspecified.

**Required resolution**

Define a producer input contract that passes an explicit logical name independently of the host path. Specify normalization, allowed characters/bytes, collision handling, and whether `subject.name` participates in conformance or only presentation. Keep filesystem discovery in the CLI and identity validation/projection in core, preserving AD-1.

### RC-3 — HIGH: SHA-256 framing is plausible but unvalidated, and “provenance” overstates what a bare digest proves

**Claim under review**

AD-11 selects SHA-256 over exact file bytes and a length-framed multi-file stream; the protocol says a revision “binds provenance to exact bytes.”

**Standards and repository reality**

SHA-256 is a valid current standard choice. NIST FIPS 180-4 specifies SHA-256, and the published document remains current; NIST's 2023 planning note says FIPS 180-4 will be revised, principally to remove SHA-1 and update guidance, not to withdraw SHA-256. The proposed decimal-length framing is parseable in principle because each length terminates at `:` and then determines the following byte count.

However:

- the repository is currently dependency-free and contains no SHA-256 implementation;
- dependency/change-control approval is explicitly required by current project rules;
- the validation report states that multi-file digests and framing were not validated;
- the specification does not publish test vectors for empty content, non-ASCII UTF-8 names/content, CRLF preservation, or multiple files;
- a bare SHA-256 digest is a content identifier/integrity checksum, not evidence of origin, author, approval, or authenticity. “Binds provenance” is too strong unless provenance is narrowly defined as byte identity and the digest is anchored in a trusted evidence bundle.

**Required resolution**

Keep AD-11 marked `[ASSUMPTION]`. Before adoption, approve an implementation choice and add normative byte-level test vectors. Define the hash preimage with pseudocode or ABNF-equivalent precision, including filename encoding and order. Replace the broad provenance claim with “binds this record to content bytes” and explicitly state that the digest is not authentication or human approval.

### RC-4 — MEDIUM: the schema `$id` asserts an unverified public namespace

The schema declares:

```json
"$id": "https://morva.dev/schemas/checked-semantics/v1.json"
```

No repository document establishes ownership or publication of `morva.dev`; the repository's declared canonical URL is `https://github.com/dkeepersun/morva`. A direct HTTPS check did not yield a schema endpoint during this review. JSON Schema `$id` is an identifier and need not always be fetched, but using an HTTPS URI still creates a public identity and resolution expectation. Shipping it without namespace control risks future collision or misleading consumers.

**Required resolution**

Use a URI under a controlled namespace, publish the resource at the declared URI, or use an explicitly documented non-resolving URN/tag-style identifier until a canonical domain is approved.

### RC-5 — MEDIUM: normative-keyword semantics are asserted without adopting BCP 14

The protocol says “MUST, MUST NOT, SHOULD, and MAY are normative” but does not define their interpretation or cite RFC 2119 plus RFC 8174 (BCP 14). RFC 8259's own use of these words is scoped through BCP 14; merely listing the words does not establish the same semantics for this protocol.

For a stable machine protocol, add the standard BCP 14 boilerplate or define equivalent semantics locally. This is especially important for AD-9's `SHOULD` byte reproducibility and consumer behavior around unknown fields/discriminants.

## Verified decisions and baselines

### Repository-aligned

- **Rust edition and producer seed:** root `Cargo.toml` sets workspace edition `2024` and version `0.1.0`. Rust's official Edition Guide records Rust 2024 as released with Rust 1.85.0. The local review environment is rustc/cargo 1.95.0, but that is not an MSRV claim.
- **Ownership:** `morva-core` is dependency-free and owns parser/checker/analyzer semantics; `morva-cli` depends only on core and owns file discovery, safe reads, rendering, and exit codes. AD-1 and AD-10 match `docs/architecture.md` and `docs/project-context.md`.
- **Source locations:** the core uses UTF-8 byte spans, project virtual spans, and explicit virtual-to-source-local mapping. Excluding virtual spans and host paths from the protocol is consistent with current architecture.
- **Project order:** direct-child valid-UTF-8 lowercase `.morva` regular files are sorted by filename bytes before assembly. The proposed source ordering can reuse this established order.
- **Error gate:** current checked/simulation flows do not accept semantic diagnostics, and warning-only analysis remains successful. The proposed distinction between validity and semantic coverage matches `AnalysisReport` and `ProjectAnalysisReport` behavior.
- **Coverage warnings:** `MORVA5001` and `MORVA5002` are typed in `NoticeKind`, source-spanned, non-fatal, and deterministically merged with errors. AD-5 is grounded in current APIs.
- **Human authority:** `docs/requirements.md` explicitly says human-reviewed `.morva` source is the Source of Truth and AI must not silently modify it. AD-6 and AD-12 are grounded in product requirements.
- **Read-only/local scope:** the repository has no network, persistence, auth, MCP, or AI-provider runtime. Deferring them is accurate.

### Standards-aligned

- **JSON:** RFC 8259 is STD 90 / Internet Standard. UTF-8 JSON, arrays as ordered sequences, objects as unordered collections, and strict JSON generation are consistent with the protocol. The draft correctly states that object order is not semantic; its field order is an emission convention only.
- **JSON Schema:** the official JSON Schema site identifies Draft 2020-12 as the current version and publishes `https://json-schema.org/draft/2020-12/schema` as its meta-schema. The schema's `$schema` value is correct.
- **SHA-256:** SHA-256 remains specified by NIST FIPS 180-4. Its use is technically sound for deterministic content identity, subject to the qualification in RC-3.

## Claims that remain design-only, not repository facts

The spine generally labels these as future or deferred, but implementation planning must not cite them as existing capabilities:

- no checked-semantics protocol types or invariant validator currently exist in `morva-core`;
- no CLI machine serializer or command/flag contract exists;
- current `Diagnostic` does not expose a typed lexical/syntax/project/semantic category, so core-owned category production still needs design and tests;
- the checker returns diagnostics over the AST rather than a retained normalized “checked model”; resolved references/types must be projected without exposing or duplicating checker internals;
- the schema only partially constrains declaration children (`fields`, `parameters`, `clauses`, and `items` are largely open objects), so it is an envelope/seed schema, not proof that every current AST/expression/scenario variant has a stable machine shape;
- deterministic byte emission has not been validated with a selected JSON serializer;
- full multi-file virtual-to-local projection and the core semantic-envelope validator are explicitly unimplemented and untested.

These are acceptable in a draft architecture spine only if the implementation specification treats them as work to design and verify, not as already validated behavior.

## Primary evidence

Repository:

- `Cargo.toml`
- `crates/morva-core/src/lib.rs`
- `crates/morva-core/src/analysis.rs`
- `crates/morva-core/src/project.rs`
- `crates/morva-cli/src/main.rs`
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/project-context.md`
- `docs/language-evolution-policy.md`
- `MORVA-CHECKED-SEMANTICS-PROTOCOL-v1.md`
- `schema/morva-checked-semantics-v1.schema.json`
- `VALIDATION-REPORT.md`

External primary standards:

- Rust Edition Guide, Rust 2024: https://doc.rust-lang.org/stable/edition-guide/rust-2024/index.html
- RFC Editor, RFC 8259 / STD 90: https://www.rfc-editor.org/info/rfc8259/
- JSON Schema Draft 2020-12: https://json-schema.org/draft/2020-12
- JSON Schema current specification index: https://json-schema.org/specification
- NIST FIPS 180-4: https://csrc.nist.gov/pubs/fips/180-4/upd1/final
- NIST decision to revise FIPS 180-4: https://csrc.nist.gov/News/2023/decision-to-revise-fips-180-4

## Exit gate

The reality-check gate can pass after RC-1 and RC-2 are resolved in the protocol/spine, AD-11 remains explicitly conditional until approved test vectors and implementation evidence exist, and the namespace plus normative-language issues are made explicit. No production implementation should claim v1 conformance before the core-owned invariant validator and the complete checked-model shape have executable conformance tests.

# Reality Check Follow-up — Morva Checked Semantics Protocol

> Review date: 2026-08-13
> Scope: current final candidate of the architecture spine, protocol, schema, validation report, and present repository contracts
> Mode: read-only review; no deliverable modified
> Verdict: **FAIL**

## Verdict

The candidate has materially improved and closes most earlier divergence points, but it is not yet a self-consistent, reality-backed v1 substrate. Two previously reported findings remain open in altered form, and one new conformance hole affects the protocol's central traceability claim.

Blocking issues:

1. the normative protocol still says every location is source-local while later introducing subject-scoped locations;
2. the replacement schema `$id`, `urn:morva:...`, uses an unregistered URN namespace identifier;
3. the mandatory revision digest rules are not included in the core-owned invariant validator, so a shape-valid document with incorrect content/project hashes could still satisfy the stated validation gate;
4. SHA-256 remains explicitly conditional on dependency/change-control approval, so production v1 conformance cannot ship until that approval and implementation evidence exist.

## Follow-up on the five prior findings

| Prior finding | Status | Reality-check result |
|---|---|---|
| Source-less project diagnostic | **PARTIALLY CLOSED / still blocking** | The schema now supports `sources: []` plus `{ "kind": "subject" }`, AD-3/AD-14 define its ownership, and the validation report exercises `MORVA2023`. However protocol normative invariant 2 still requires **every** location to be a source-local byte range, directly contradicting the subject-location rule. |
| Logical name/order | **CLOSED for architecture** | AD-9/AD-16 and the protocol now separate host paths from logical final components, prohibit normalization, reject duplicate project names, sort exact UTF-8 bytes in core before assembly, and use opaque ordinal source IDs. This converges with the current CLI's project filename-byte ordering. Minor input-validation details remain for the implementation specification. |
| SHA boundary | **DESIGN CLOSED; production gate OPEN** | Content identity is no longer called origin/authenticity, framing is byte-defined, and all three published known-answer vectors independently reproduce. The repository still has no hash dependency/implementation and project policy requires approval before adding one; AD-11 correctly remains `[ASSUMPTION]`. |
| Schema ID | **NOT CLOSED** | The unverified `https://morva.dev/...` identifier was replaced by `urn:morva:schema:checked-semantics:v1`, but `morva` is not present in the current IANA URN Namespace registry. RFC 8141 requires managed/registered NIDs; syntactic resemblance alone does not make an unregistered string a valid globally unique URN. |
| BCP 14 | **CLOSED** | The protocol now uses the standard RFC 2119/RFC 8174 boilerplate and scopes the uppercase normative terms correctly. |

## Blocking findings

### F1 — Normative source-location rule contradicts subject-scoped project findings

The protocol's normative invariant 2 states:

> `sources[].content` is the exact UTF-8 input. **Every location MUST use a half-open, source-local byte range into that content.**

The same protocol later normatively defines `{ "kind": "subject" }` for a source-less core project diagnostic, with no source or byte range, and its invariant validator permits subject locations on source-less project findings. The spine's AD-3 is already correctly qualified as “for source-owned facts,” but the protocol's highest-level invariant was not updated.

This means a `MORVA2023` document cannot obey all normative rules simultaneously even though it passes the companion schema. The conflict also makes “schema + core validator = protocol conformance” impossible to state unambiguously.

Required closure: change invariant 2 to apply to every **source-owned** location and explicitly recognize subject locations for source-less core project findings. Keep semantic nodes and coverage entries source-only.

### F2 — `urn:morva` is not a controlled/registered schema identity

The schema currently declares:

```json
"$id": "urn:morva:schema:checked-semantics:v1"
```

The current IANA URN Namespace registry contains no `morva` NID. RFC 8141 defines formal NIDs through IANA registration and assigns informal NIDs in the form `urn-<number>`; it also states that unmanaged experimental strings conforming to URN syntax are not valid URNs. Consequently this change does not close the earlier namespace-ownership problem—it changes an unverified HTTPS authority into an unregistered URN authority.

Required closure: use an identifier under an authority the project controls (for example an approved repository/domain URI), register an appropriate URN namespace, or omit `$id` until a stable base identity is approved. Do not use `urn:morva` as though it were globally allocated.

Primary references:

- RFC 8141: https://www.rfc-editor.org/rfc/rfc8141
- IANA URN Namespace registry: https://www.iana.org/assignments/urn-namespaces/urn-namespaces.xhtml

### F3 — Revision integrity is mandatory but absent from the conformance validator

AD-11 and the protocol make source/project SHA-256 values a mandatory v1 identity. The protocol defines:

- each source digest over exact content bytes;
- file `subject.revision` equal to the one source revision;
- project `subject.revision` over the canonical length-framed, name-bearing ordered stream.

AD-13 then says only documents passing schema and the core-owned invariant validator are protocol-conforming. Yet the validator's enumerated checks cover IDs, locations, validity, coverage, model completeness, semantic references, and shell provenance—not digest recomputation, subject/source equality, canonical source ordering, or the subject-kind/source-cardinality relationship.

As written, a producer bug could emit a schema-valid document whose source content is `A` but revision is SHA-256(`B`), or a project digest built in caller order rather than canonical UTF-8-name order, and still pass every listed conformance check. That invalidates the main traceability/cache/review lineage premise.

Required closure: add core validator invariants that:

1. recompute every source digest over `content.as_bytes()`;
2. enforce file subjects have exactly one source and equal its revision;
3. enforce project subject revision over the exact canonical framing and canonical source order;
4. enforce canonical logical-name validity/uniqueness and consecutive `source:<index>` assignment;
5. enforce `sources: []` only for a source-less project result, never for a file or valid checked model.

The known-answer vectors are correct but do not replace these per-document checks.

### F4 — SHA-256 adoption remains a declared production blocker

The three published vectors independently reproduce:

- empty bytes: `e3b0c442...b855`;
- `morva-project-v1\0`: `9e6148ad...fc9f`;
- the published two-source hexadecimal preimage: `5d7da2df...69ee`.

NIST FIPS 180-4 remains a valid source for SHA-256; NIST's announced revision does not withdraw SHA-256. The remaining issue is repository integration, not algorithm currency: Morva is currently dependency-free, its architecture requires explicit dependency/change-control approval, and the validation report acknowledges that no production digest implementation was tested.

Required closure before production v1: obtain the approval, select the implementation approach, and run the known-answer vectors plus project-order/rename/CRLF tests through the actual core producer. It is acceptable for the architecture draft to retain `[ASSUMPTION]`, but implementation cannot claim v1 conformance while this gate is open.

## Non-blocking findings

### N1 — Logical-name validation needs executable edge rules

The architecture decision is now convergent, but the future core entry point should explicitly reject an empty logical final component and strings containing path separators rather than trusting non-CLI callers to have extracted a final component. It should also test canonically equivalent but byte-distinct Unicode names, control characters legal on Unix, and project-root paths whose filesystem representation has no final component.

This belongs in the implementation specification and tests; it does not reopen the chosen exact-byte/no-normalization policy.

### N2 — Integer protocol values should state the current i64 range

The current parser materializes integer literals as Rust `i64` and emits `MORVA1012` outside the supported 64-bit range. The schema represents integer values as unrestricted canonical decimal strings. A conforming core producer cannot currently emit an out-of-range value, but a consumer reading schema shape alone cannot discover the L0 range.

State the signed 64-bit range normatively or make it an explicit core-validator rule. Keeping the JSON representation as a string is reasonable and avoids JSON-number interoperability limits.

### N3 — Diagnostic category is a new typed projection, not an existing core fact

The current `Diagnostic` carries only code, message, and span. Lexical versus syntax category is not retained after `parse`; project/semantic ownership is known from pipeline context. A future protocol producer can remain core-owned by adding typed category at the protocol entry point, but the implementation specification must prohibit the CLI from classifying by message text and should test lexer codes separately from parser codes.

### N4 — Minor editorial duplication in the AI review outcomes

The protocol repeats the `accept` bullet twice in the human-review decision list. It does not change the state machine, but should be removed before publication.

### N5 — Current validation evidence remains prototype-level

The schema parses as JSON (`jq empty` passed), the vectors reproduce, and the validation report describes positive/negative schema exercises. However `check-jsonschema` is not available in the current shell and no reusable conformance runner is present in the architecture workspace. The report itself correctly says that full projection, core invariant validation, production digesting, and multi-file span projection are unimplemented.

Treat the validation report as design/prototype evidence, not executable production proof. The implementation specification should create repository-owned automated conformance tests rather than depend on a one-off tool environment.

## Reality-backed items confirmed

- Workspace edition `2024`, package version `0.1.0`, and two-crate ownership match current manifests.
- Rust 2024, RFC 8259, and JSON Schema Draft 2020-12 remain current named baselines.
- `morva-core` owns parser/checker/analyzer semantics; CLI owns discovery, safe reads, rendering, and operational exit behavior.
- Current project discovery sorts valid UTF-8 direct-child `.morva` filenames by exact bytes.
- `Project::parse([])` really yields source-less `ProjectDiagnostic::Project` / `MORVA2023`.
- Source/local versus merged virtual span separation matches current `Project`/`SourceMap` behavior.
- Multi-file checked-system `shell_locations` is a valid correction to the current merged AST's incidental first-shell span.
- `MORVA5001`/`MORVA5002`, independent validity/coverage, and the human-reviewed `.morva` Source-of-Truth boundary match current code and requirements.
- The schema now closes current declaration/expression/scenario discriminants and fixes `language_version` to `0.1`; unknown semantic values fail closed while unknown object metadata remains additive.
- BCP 14 adoption is correct.

## Gate to PASS

The reality-check gate passes when F1–F3 are corrected in the protocol/spine/schema boundary and F4 is either approved with executable producer evidence or remains explicitly marked as a pre-production implementation gate with no claim that v1 is shippable. Then rerun JSON Schema metaschema validation, the documented positive/negative samples, digest vectors through the actual implementation, source-less `MORVA2023`, multi-file shell provenance, and wrong-digest negative cases.

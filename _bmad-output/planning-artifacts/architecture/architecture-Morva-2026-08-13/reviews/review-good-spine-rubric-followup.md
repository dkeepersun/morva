# Good-Spine Rubric Follow-up Review

## Gate verdict

**PASS WITH MINOR CHANGES.** No critical or high good-spine finding remains. The revision now fixes the producer/consumer divergence points that previously prevented `morva.checked-model.v1` from being a stable build substrate: it closes the L0 algebra, represents source-less project findings, makes unknown semantics fail closed, defines complete system-shell provenance, canonicalizes logical inputs, anchors review lineage, and explicitly separates pre-core/serializer failures from protocol results. Three P2 clarifications remain; they tighten reproducibility and conformance validation but do not require reopening the architecture.

The deterministic spine lint passes with zero findings.

## Prior finding closure

| Prior finding | Result | Follow-up evidence |
|---|---|---|
| Checked-model payload deferred/unconstrained | **Closed** | AD-7 now binds a closed L0 algebra. The schema uses required discriminated variants for every declaration, child node, clause, assignment, expression, path root, scenario item, operator, and type. Required collections cannot be omitted. The revised validation includes a non-empty enum/entity/action projection and negative cases for unknown declarations and missing action clauses. |
| Source-less project errors unrepresentable | **Closed** | AD-3/AD-14 define `{ "kind": "subject" }`, allow `sources: []`, and keep pre-core failures out of the protocol. Schema and validation include an empty-project `MORVA2023` envelope. |
| Unknown semantic behavior contradictory | **Closed** | AD-8 and the protocol now make all v1 semantic discriminants closed: unsupported language versions and unknown kinds/operators/types reject checked-model consumption; only unknown object members are ignored. Schema negative cases exercise this. |
| Merged system provenance inherited first-file span | **Closed** | AD-15 and schema require ordered, non-empty `shell_locations`; the protocol forbids a singular location from substituting for the contributing shell set. Multi-shell shape validation is recorded. |
| Logical names/path privacy non-canonical | **Closed** | AD-9/AD-16 assign exact UTF-8 final-component names, no Unicode normalization, duplicate rejection, byte-order sorting, ordinal opaque source IDs, and path separation. Digest vectors cover empty input, multibyte names, and CRLF; the published hashes independently reproduce. |
| Language version ambiguous | **Closed** | AD-8 and schema bind `language_version` exactly to `0.1`; any different semantic surface requires a compatible protocol version. |
| Operational envelope incomplete | **Closed at spine altitude** | AD-14 fixes no-document/no-partial-stdout behavior and exit 2 for pre-core failures. Serializer failure, broken stdout, peak memory, and JSONL thresholds are explicitly deferred to the implementation specification with an atomic-emission requirement and measurement trigger. |

## Full checklist assessment

| Good-spine criterion | Result | Assessment |
|---|---|---|
| Fixes the real divergence points for the level below | **Pass** | Authority, validity, coverage, provenance, identity, compatibility, semantic algebra, review lineage, and operational boundaries now have single choices. |
| Every AD Rule is enforceable and prevents its stated divergence | **Pass with P2 clarification** | Rules are testable through schema negatives, known-answer vectors, the future core invariant validator, CLI integration tests, and review-state tests. The validator checklist should name several semantic equalities explicitly (finding 3). |
| Nothing under Deferred can let two units diverge | **Pass** | Deferred items either sit outside local v1 or name their future owner/revisit condition. Full L0 projection is no longer deferred. Atomic output is already fixed; only thresholds/mechanics remain open. |
| Named technology is verified-current | **Pass** | Rust 2024 matches the workspace and remains the latest stable edition; JSON Schema 2020-12 remains the current published dialect; RFC 8259 remains STD 90. NIST still lists FIPS 180-4 Update 1 as final while noting a planned revision; that does not alter SHA-256. |
| Ratifies the brownfield codebase | **Pass** | Core semantic ownership, CLI safe discovery/read, UTF-8 local spans, exact byte filename ordering, additive analysis warnings, global short-name semantics, same-name system shell assembly, and project/source diagnostic split are preserved. The protocol adds a separate canonical core entry point rather than changing arbitrary `Project::parse` caller order. |
| Covers the driving capability/spec | **Pass** | The representation and AI loop now carry checked facts, diagnostics, coverage gaps, exact source evidence, candidate/base lineage, rationale, CAS acceptance, and explicit incomplete-coverage acknowledgements. The three real AI samples retain accept/reject/block outcomes. |
| Does not weaken an inherited parent spine | **N/A** | No inherited parent spine is declared. |
| Every owned structural/operational dimension is decided, deferred, or open | **Pass** | Local deployment/provider scope, privacy/remote boundaries, I/O failure ownership, deterministic emission, compatibility, conformance, resource escalation, persistence/network/auth, and human workflow ownership are all covered at feature altitude. |

## Remaining medium findings

### 1. [P2] Companion determinism wording lags the new logical-input invariant

- **Evidence:** AD-9 correctly promises identical bytes for identical *logical inputs*. The protocol’s Determinism section still says identical “input bytes, producer version, language version, and options,” even though changing `subject.name` or a source logical name changes the envelope and project digest without changing content bytes. Its order list also says “established project-discovery order,” while AD-9/AD-16 now normatively sort exact logical-name UTF-8 bytes inside the protocol core boundary. No v1 `options` field exists.
- **Impact:** A test author could assert byte equality for two differently named files with identical content, or preserve arbitrary caller order based on the older sentence.
- **Disposition:** **Autofix.** Say “identical canonical logical names and exact content bytes, producer version, and language version”; remove `options` until options are represented, and replace discovery order with canonical logical-name byte order. No AD change is needed.

### 2. [P2] `semantic_key` reference scope and parsing contract remain implicit

- **Evidence:** AD-4 explicitly makes `source_id`, `node_id`, and `finding_id` document-local but omits `semantic_key`. The schema and examples use semantic keys as resolved references (`entity:Order`, `action:Fulfill.parameter:order`, enum/member keys), yet neither the spine nor protocol says whether consumers may parse their text, whether they are revision-local, or whether their exact constructors are a v1 compatibility promise.
- **Impact:** An editor may persist a semantic key across revisions or parse its delimiters while another consumer treats it as an opaque within-document reference. Both can currently claim conformance. Field/member/parameter key construction is especially vulnerable because those names require owner qualification.
- **Disposition:** **Discuss, then small fix.** Prefer declaring all `semantic_key` values opaque and document-local for equality/reference resolution, while retaining their human-readable form as non-normative; alternatively specify exact escaped constructors for every node kind and make them a v1 contract. Extend AD-4 and the core validator accordingly.

### 3. [P2] The semantic-envelope validator should explicitly cover cross-field type and digest invariants

- **Evidence:** The normative prose fixes clause-kind/phase pairs, Boolean result types for binary expressions, enum ownership, entity-only scenario run arguments, target/path type agreement, source digests, and the framed subject digest. The JSON Schema intentionally validates shape and therefore accepts, for example, `requires + state_phase: effect`, a Boolean literal whose `resolved_type` is `String`, a run argument whose semantic type is an enum, or a digest unrelated to inline content. AD-13’s validator list names completeness, key resolution, ownership, and shell provenance but not these exact type/phase/digest equalities.
- **Impact:** A future validator could satisfy the literal checklist yet allow a schema-valid projection whose typed facts disagree with the checked core, weakening AD-7/AD-11’s enforceability.
- **Disposition:** **Autofix.** Add “digest recomputation and all closed-algebra cross-field/type invariants” to AD-13 and the protocol validator list; exercise representative negative/property tests in the implementation specification. This is a validator-scope clarification, not a new architecture decision.

## Assumption and release gate

AD-11 remains correctly marked `[ASSUMPTION]`. The algorithm, framing, security meaning, and known-answer vectors now converge; only Morva dependency/change-control approval remains. Treat that approval as a production-v1 release gate. If approval is denied, revise the protocol major/design rather than substituting another digest under v1.

## Final gate recommendation

Apply findings 1 and 3 as clear wording fixes and resolve the small `semantic_key` contract choice before marking the spine final. No further architectural review is required after those edits; implementation should proceed through a specification that includes core-validator property tests, complete L0 projection fixtures, multi-file provenance tests, digest vectors, and atomic CLI emission tests.

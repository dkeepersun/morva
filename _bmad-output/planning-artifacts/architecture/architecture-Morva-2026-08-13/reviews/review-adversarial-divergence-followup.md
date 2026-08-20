# Adversarial Divergence Follow-up

## Verdict

**FAIL.** The revised candidate closes the original producer/consumer shape, unknown-semantics, project-ordering, coverage-policy, and pre-core-failure holes. Two review/identity decisions remain open, however. Independent downstream units can obey every current AD literally and still assign different Source-of-Truth state or incompatible cross-revision identity to the same evidence.

## Recheck of prior divergence points

| Boundary | Result | Evidence |
|---|---|---|
| Closed L0 algebra | **Closed** | AD-7 binds the schema as the complete v1 algebra; the schema now has required discriminated unions for declarations, children, clauses, assignments, expressions, path roots, scenario items, operators, and types. Empty collections are required rather than implied by absence. AD-13 additionally requires exact source-to-projection completeness and semantic-key resolution. |
| Unknown semantics / language | **Closed** | AD-8 fixes `language_version = 0.1`, rejects unsupported language versions and unknown semantic kinds/operators/types, and permits ignoring only unknown object members. The compatibility section makes new semantic discriminants a v2 change. |
| Revision lineage and CAS | **Partially closed; blocker remains** | Candidate parent is now the current accepted revision, cumulative history/diff is required, and acceptance is CAS. The compare set is still only the source revision; it does not include the versioned intent that the candidate claims to satisfy. Nor is `result.status = valid` an explicit acceptance precondition. See blocker 1. |
| Coverage acknowledgement | **Policy closed; wire identity non-blocking** | Incomplete coverage requires one human disposition and rationale/evidence per item, otherwise `block`; acknowledgement cannot promote an item to verified semantics. The external bundle has no schema or stable acknowledgement reference, but an immutable bundle can still bind acknowledgements structurally in the first local workflow. |
| Multi-file ordering / names / IDs | **Closed** | AD-9/AD-16 move exact UTF-8 logical-name ordering into core, prohibit Unicode normalization, reject duplicates, and assign ordinal source IDs after canonical sorting. The protocol supplies exact framing and checked known-answer vectors. AD-15 closes merged-system shell ownership. |
| Pre-core failure | **Closed at architecture altitude** | AD-14 and the protocol distinguish source-less core project findings from usage/discovery/read/identity/UTF-8 failures; the latter emit no protocol or partial stdout and retain operational exit 2. Serializer/pipe atomicity is explicitly assigned to the implementation specification. |

## Blocking findings

### 1. [P1] Acceptance can be both invalid-source accepting and stale-intent accepting

Two downstream review units remain simultaneously conforming:

- **Atlas** permits `accept` only when the candidate check is `status = valid`, and performs CAS over `(accepted_source_revision, intent_identity, intent_version)`.
- **Boreal** permits a human to accept an invalid candidate after reading its findings, and performs CAS only over `accepted_source_revision` as AD-12 literally requires.

Both use a complete freshly checked candidate, never auto-accept, name versioned intent in an immutable bundle, anchor `candidate_parent_revision` to the current accepted source revision, include cumulative evidence, and require an explicit human action. No current AD or normative AI-loop rule says that `accept` requires `result.status = valid`. The coverage rule does not close this: an invalid early-parse result has `assessment = unavailable` and `fully_modeled = null`, so the incomplete-coverage acknowledgement condition does not apply.

A second divergence occurs without changing source bytes. Build a bundle against intent `I@1` and accepted source `R0`; then replace the current requirement with `I@2` while source remains `R0`. Boreal's required source-revision CAS still succeeds and can accept a candidate generated for stale `I@1`. Atlas's source-plus-intent CAS fails. Both presented the bundle's original intent identity/version; the draft never requires that version to remain current at acceptance.

**Impact:** Two conforming workflows can assign opposite authoritative state to identical evidence. One can also make syntactically or semantically invalid AI output the human-reviewed Source of Truth, or commit a valid candidate against superseded intent.

**Required closure:** Add normative acceptance preconditions: candidate `result.status` MUST be `valid`; its checked subject revision MUST equal `candidate_revision`; the current `(accepted_revision, intent_identity, intent_version)` MUST equal the immutable bundle's comparison tuple at commit time. CAS failure leaves the decision uncommitted and requires a new bundle/check. Define `reject`/`block` as the only decisions available for invalid or unavailable-coverage candidates.

### 2. [P1] `semantic_key` is neither canonical nor declared revision-local

AD-4 scopes `source_id`, `node_id`, and `finding_id` to a document but omits `semantic_key`. The checked algebra uses semantic keys as the targets of resolved entity, enum, action, parameter, and enum-member references, while the prose only calls them human-readable and gives examples. It specifies neither exact constructors/escaping nor opacity, uniqueness, or cross-revision lifetime.

Two conforming core producers can therefore diverge:

- **Atlas** emits canonical-looking keys such as `entity:Order` and `action:Confirm.parameter:order`, and its review consumer treats equal strings as stable cross-revision semantic identity.
- **Boreal** emits different human-readable, internally unique keys such as `type/entity/Order` and `parameter/Confirm/order`, and treats them as opaque document-local references.

Both schemas validate. Both core validators can prove that every reference resolves inside its own document and every node belongs to the checked revision. Both satisfy AD-4 literally because it says nothing about semantic keys. Yet their consumers cannot exchange references, and Atlas can mis-associate rename/delete-and-recreate operations across revisions while Boreal cannot perform the same semantic diff.

**Impact:** The protocol closes data shape but leaves its reference identity contract open, directly affecting resolved-reference consumers and accepted-to-candidate comparison.

**Required closure:** Prefer the smaller contract: add `semantic_key` to AD-4's document-local identities; require uniqueness within the appropriate closed-model namespace; state that consumers use it only as an opaque equality/reference token inside one document and MUST NOT parse it or match it across revisions. Make the core validator enforce uniqueness and unambiguous resolution. If cross-revision matching is intended instead, specify exact escaped constructors and rename/reparent semantics for every key kind as v1 compatibility commitments.

## Non-blocking findings

### 1. [P2] Source locations are exact in units but not exact in extent

For a field, parameter, declaration, clause, scenario item, path segment, and system shell, one producer can use the name/keyword token range while another uses the complete grammar construct range. Both are source-local, half-open, byte-boundary-valid, source-owned ranges and therefore satisfy AD-3/AD-13. Consumers will highlight and extract different text.

This does not change semantic interpretation because names and structure are separately represented, so it need not reopen the architecture. The implementation specification should define the authoritative extent for each location-bearing shape and add multibyte/comment/newline fixtures.

### 2. [P2] Coverage acknowledgements lack an interoperable item reference

The policy is unambiguous, but the external bundle can identify an acknowledged item by array index, matching `finding_id`, or the typed identity-plus-location tuple. These are equivalent inside one immutable local bundle but not portable between independently implemented review stores. Evidence references may likewise be a mutable URL or immutable digest.

Before review state becomes a machine protocol or persisted/MCP capability, give each acknowledgement a bundle-local target defined as the exact coverage identity/location tuple (or a dedicated ordinal), bind it to candidate revision, and require immutable evidence content or a digest when disposition is `externally_evidenced`.

### 3. [P2] Core conformance validation should explicitly recompute digests and cross-field type invariants

AD-11 normatively fixes digest computation and the algebra prose fixes relationships such as clause-kind/state-phase and expression-kind/resolved-type, so a conforming producer has only one correct answer. AD-13's enumerated validator responsibilities do not explicitly say it recomputes source/subject digests or checks every algebraic cross-field relation. This is a verification gap rather than a viable alternate interpretation.

Add digest recomputation, canonical source ID/order checks, semantic-key uniqueness, and all closed-algebra cross-field/type invariants to the implementation-spec validator/property-test matrix.

## Gate condition

Close blockers 1 and 2 in AD-4/AD-12 and the normative AI-loop contract before marking the architecture final. The P2 items can be assigned to the implementation specification, provided the review workflow remains local and non-persisted in this increment.

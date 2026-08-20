# Final Adversarial Divergence Gate

## Verdict

**PASS.** No blocking divergence remains at the architecture-spine altitude. The final candidate now forces independent producers, consumers, and human-review workflows to agree on the checked L0 shape, unsupported-content behavior, revision/reference scope, project identity, pre-core failure boundary, and legal acceptance transition.

## Blocking findings

**None.**

## Closure verification

| Previously attacked boundary | Final result | Why the incompatible pair no longer survives |
|---|---|---|
| Closed L0 algebra | **Closed** | AD-7 makes the companion schema the complete v1 algebra; every current declaration, child, clause, assignment, scenario item, expression, root binding, operator, and type is a required discriminated shape. Empty collections are explicit. AD-13 requires exact projection completeness and checker-consistent cross-field/type invariants. Alternate field/clause/expression encodings cannot claim v1 conformance. |
| Unknown semantics and language | **Closed** | AD-8 fixes `language_version = 0.1`, closes all semantic discriminants, and requires fail-closed behavior for unknown kinds/operators/types while ignoring only additive object members. A producer cannot introduce semantic variants through optional capabilities, and a consumer cannot silently skip them. |
| Revision lineage and acceptance CAS | **Closed** | AD-12 requires `candidate_parent_revision` to equal the current accepted revision, a fresh `valid` candidate result, cumulative accepted-to-candidate evidence, and CAS over the accepted revision plus intent identity and intent version. The previous invalid-candidate and stale-intent acceptance implementations are now expressly non-conforming. |
| Revision/reference identity | **Closed** | AD-17 makes every `semantic_key` unique, referentially complete, opaque, and revision-local; consumers may compare it only inside the checked model and may neither parse it nor persist it as cross-revision identity. `node_id` has the same durability boundary. Canonical-key and arbitrary-opaque-key producers remain interoperable because consumers are forbidden to depend on spelling. |
| Coverage acknowledgement | **Closed** | Incomplete coverage requires one human acknowledgement per unmodeled item, otherwise `block`. The interoperable key is the pair `(candidate_revision, exact typed coverage identity + location)`, so array-index and warning-ID implementations can no longer claim equivalent portable identity. Acknowledgement remains explicitly non-verifying. |
| Multi-file ordering, provenance, and digest | **Closed** | AD-9/AD-16 put exact UTF-8 logical-name validation, no-normalization, duplicate rejection, canonical sorting, ordinal source-ID assignment, hashing, and emission under the core boundary. AD-15 requires all contributing system shell locations. AD-11 supplies exact framing and known-answer vectors; AD-13 requires digest recomputation, canonical order/IDs/cardinality, and shell/source correspondence. |
| Pre-core and source-less project failure | **Closed** | AD-14 distinguishes source-less core project findings, which use subject location and may have `sources: []`, from usage/discovery/read/identity/UTF-8 failures, which emit no checked-semantics document or partial stdout and exit operationally with code 2. Adapters cannot invent source provenance or reinterpret missing output as an invalid model. |

## Fresh adversarial sweep

The following alternate implementations were considered and are no longer viable incompatible pairs:

- accepting a human-approved but invalid candidate versus requiring a green check;
- source-only CAS versus source-plus-current-intent CAS;
- treating equal semantic-key text as durable across revisions versus treating it as local;
- acknowledging coverage by array position versus typed item identity;
- hashing caller order versus canonical core order;
- normalizing Unicode logical names versus preserving exact UTF-8;
- emitting a synthetic protocol error for unreadable input versus emitting no document;
- skipping an unknown semantic node versus rejecting checked-model consumption.

Each permissive side now violates an explicit MUST/MUST NOT or adopted AD.

## Non-blocking implementation notes

1. **Location extents:** byte units, ownership, boundaries, and references are fixed, but an implementation specification should still state whether each node location covers its name/keyword or complete grammar construct. This affects highlighting, not protocol semantic interpretation.
2. **`node_id` uniqueness:** AD-17 fixes opacity and lifetime, while semantic-key uniqueness resolves all semantic references. For defensive tooling, the implementation validator should also state and test `node_id` uniqueness explicitly rather than relying on the ordinary meaning of “ID.”
3. **Capability ordering:** AD-9 requires reproducibility for an enabled capability set, but an implementation specification should prescribe mandatory-first plus lexical extension order (or another exact order) for the JSON array.
4. **Evidence durability:** before remote or persisted review state is introduced, `externally_evidenced` references should use immutable content or a digest. The current local, non-persisted workflow can safely defer that transport/storage policy under AD-10.
5. **Production gate remains real:** AD-11 correctly prevents shipment until SHA-256 dependency/change-control approval and actual core producer conformance tests exist. The current artifacts validate the design/prototype, not a production implementation.

## Gate conclusion

The architecture may proceed to implementation specification. Carry the five notes above into validator, serializer, and workflow acceptance tests; none requires reopening the architecture spine.

# Input Reconciliation Review

## Verdict

**NEEDS CHANGES.** The four artifacts agree on the main boundary—core-owned, read-only checked semantics with coverage separated from validity and acceptance gated by a human—but the spine drops one validation-proven invariant, while the protocol/schema pair leaves two load-bearing contracts non-operational. The validation report's positive schema checks do not close those gaps.

## Major findings

### 1. [P1] The human-review evidence package was validated but did not enter the spine

- **Evidence:** `VALIDATION-REPORT.md` lines 31–34 and 176–184 state that review used, and must retain, the original requirement, both source revisions, all findings/coverage, and the AI rationale. Sample B and C establish why a green recheck alone is unsafe.
- **Mismatch:** `ARCHITECTURE-SPINE.md` AD-6 requires a fresh check and an explicit human decision, but does not bind what evidence the reviewer must receive. The protocol's AI loop mentions a rationale and says the human compares accepted and revised candidates, but likewise does not require the complete validated review bundle.
- **Impact:** Two workflow/adaptor implementations can both satisfy AD-6 while one presents only the revised green result. That implementation would recreate the exact semantic-drift failure the validation was designed to prevent.
- **Required reconciliation:** Add a spine invariant binding the external review workflow to the original intent, accepted/base source revision, complete revised source revision, findings and coverage for relevant checks, and AI diagnosis/rationale. Keep the human decision outside the read-only protocol, but make this evidence package non-optional.

### 2. [P1] Capability negotiation is promised but no required-capability contract exists

- **Evidence:** `ARCHITECTURE-SPINE.md` AD-8 says consumers inspect required capability strings; `MORVA-CHECKED-SEMANTICS-PROTOCOL-v1.md` line 294 says consumers MUST check required capabilities.
- **Mismatch:** Neither artifact defines which capabilities are required for v1 or how required capabilities differ from optional advertised capabilities. The schema accepts an empty `capabilities` array and any arbitrary strings, even when `checked_model`, inline sources, locations, findings, and coverage are present.
- **Impact:** Consumers cannot implement the stated negotiation rule consistently. A schema-valid producer can omit every example capability, and different consumers can choose incompatible private baselines.
- **Required reconciliation:** Either define the v1 mandatory capability set and encode it with schema `contains` constraints, or remove negotiation from v1 and make the envelope shape itself the complete major-version contract. If future documents may declare producer-required extensions, give those a distinct field and rejection rule.

### 3. [P1] Schema conformance does not enforce the result/coverage invariants on which consumers rely

- **Evidence:** The normative protocol requires every error to imply `status = invalid` and `checked_model = null`, requires semantic-coverage warnings in both `findings` and `coverage`, and requires locations to reference exact source-local ranges. AD-2, AD-3, and AD-5 elevate these to architectural invariants.
- **Mismatch:** The schema only gates `checked_model` from `status`. It accepts, among other invalid combinations: `status: valid` with error findings; `status: invalid` without an error; `status: valid` with `coverage.assessment: unavailable`; duplicate/unresolved source IDs; `start > end` or ranges outside source content; and coverage entries with no corresponding warning. `VALIDATION-REPORT.md` only reports six positive snapshots passing the schema and exact-content equality; it does not test these negative or cross-record cases.
- **Impact:** "Schema-valid" can be mistaken for "protocol-valid" even though safety-critical invariants are violated. This weakens the claim that the current protocol shape is sufficient for independently built producers and consumers.
- **Required reconciliation:** Define a separate semantic-envelope validator owned by `morva-core` for constraints JSON Schema cannot express, encode the expressible status/coverage relations in the schema, and add negative/property tests. Amend the validation report to distinguish schema-shape conformance from full protocol-invariant validation.

### 4. [P2] Typed coverage payloads disagree across prose, schema, and validated snapshots

- **Evidence:** `MORVA-CHECKED-SEMANTICS-PROTOCOL-v1.md` lines 176–191 defines compatibility details with `container_kind` and soft-behavior details with `action` plus `behavior`; its coverage example also retains `container_kind`.
- **Mismatch:** The schema reduces every `unmodeled` entry to `kind`, generic `name`, and `location`, and treats finding `details` as an arbitrary object requiring only `kind`. The validated prototype snapshots omit `container_kind`; soft-behavior snapshots use `name` instead of the protocol's `action` and `behavior`. They pass because the schema is looser than the documented typed contract.
- **Impact:** A consumer cannot reliably distinguish a module from a policy or associate a soft behavior with its owning action without parsing warning text, which the protocol explicitly forbids.
- **Required reconciliation:** Use discriminated schema variants for both finding details and coverage entries, require `container_kind` for compatibility containers and `action`/`behavior` for soft behavior, and regenerate/revalidate the six snapshots. AD-5 should state that unmodeled records preserve the typed identity needed to act without parsing messages.

## Supported commitments and scope check

- AD-1 through AD-7, AD-9, and AD-10 are supported by the protocol and validation evidence at the stated information-model scope.
- AD-11 remains correctly marked `[ASSUMPTION]`; the validation report explicitly says multi-file framing and the SHA-256 dependency decision were not validated.
- The spine correctly defers full AST/expression/scenario projection, production CLI naming, remote transport, MCP, mutation, and orchestration. No reviewed artifact supports promoting those to current implementation commitments.

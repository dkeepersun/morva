# Good-Spine Rubric Review

## Gate verdict

**NEEDS CHANGES.** The spine has a strong safety boundary and passes the deterministic lint, but it is not yet a convergent build substrate for a stable v1 protocol: the central checked-model payload is explicitly deferred, two existing brownfield result/provenance cases cannot be represented unambiguously, and the compatibility and pre-protocol failure behavior still let independently built units make incompatible choices.

## Checklist assessment

| Good-spine criterion | Result | Assessment |
|---|---|---|
| Fixes every real divergence point for the level below | **Fail** | The envelope and safety gates converge, but checked declaration payloads, unknown-semantic handling, logical naming, and failures before a source bundle exists remain open. |
| Every AD Rule is enforceable and prevents its stated divergence | **Partial** | AD-1/2/3/5/6/10/12/13 are directionally enforceable. AD-7 is not enforceable while kind-specific checked payloads are unconstrained; AD-8 gives two different unknown-semantic actions; AD-9 lacks canonical logical-name inputs. |
| Nothing under Deferred can let two units diverge | **Fail** | “Full protocol projection for every AST/expression/scenario shape” is the principal producer/consumer contract, not an implementation detail. Deferring it leaves conforming implementations mutually unintelligible. |
| Named technology is verified-current | **Pass** | The codebase is already Rust edition 2024/package 0.1.0. Rust 2024 is the latest stable edition, JSON Schema’s official specification page identifies 2020-12 as current, and RFC 8259 remains Internet Standard STD 90. References: <https://doc.rust-lang.org/edition-guide/editions/creating-a-new-project.html>, <https://json-schema.org/specification>, <https://www.rfc-editor.org/info/rfc8259/>. SHA-256 is correctly kept as an explicit `[ASSUMPTION]` subject to Morva change control. |
| Ratifies rather than contradicts the brownfield codebase | **Fail** | The ownership and source-local-span rules ratify current code, but the protocol cannot represent `ProjectDiagnostic::Project` and does not settle provenance for the merged multi-file system shell. |
| Covers driving capabilities/spec | **Partial / mostly pass** | Read-only checked semantics, independent coverage, exact local spans, deterministic output, fresh checks, and human gating all land. A production-complete semantic representation does not: the validation deliberately exercised empty checked models, not real declaration/expression serialization. |
| Does not weaken an inherited parent spine | **N/A** | No parent spine is declared or inherited. |
| Every owned dimension is decided, deferred, or open | **Partial** | Deployment/provider scope and source privacy are explicitly bounded to local execution; persistence/network/auth are out. Machine-output failure semantics, exit/stdout/stderr behavior, resource/size envelope, and compatibility failure behavior are not fully decided or explicitly opened. |

## Critical and high findings

### 1. [P1] The load-bearing checked-model contract is deferred and the schema accepts mutually incompatible “v1” models

- **Evidence:** `ARCHITECTURE-SPINE.md` AD-7 promises named concepts, normalized builtins, and resolved references, while Deferred says the full projection for AST/expression/scenario shapes will be decided later. In the companion schema, `declaration.fields`, `members`, `parameters`, `clauses`, and `items` are optional arrays of arbitrary objects; no declaration kind requires its corresponding payload, and the defined `expression` schema is not connected to clauses or scenario items. The executable validation snapshots use `declarations: []`, so they do not test this contract.
- **Divergence:** One core implementer can emit an action’s parameter type as `parameters[].type`, another as `parameters[].resolved_type`, and a third can omit parameters and clauses entirely. All are schema-valid `morva.checked-model.v1`; consumers cannot rely on any of them.
- **Why the AD does not prevent it:** “Represent named language concepts” is an intention, not an enforceable projection rule. AD-13’s validator cannot establish completeness or reference ownership without exact kind-specific shapes and invariants.
- **Disposition:** **Discuss / then fix before calling the protocol v1.** Either (a) fully specify and schema-wire every current L0 entity/enum/action/scenario, field/member/parameter/clause/expression/assignment/scenario-item shape, including required-by-kind fields and resolved-reference rules, or (b) narrow the current artifact to an envelope/diagnostics protocol and remove the mandatory `morva.checked-model.v1` capability until the semantic payload is specified. This item cannot remain under Deferred because it is the primary divergence point between producers and consumers.

### 2. [P1] Existing project-level failures have no conforming protocol representation

- **Evidence:** AD-2 says *any* project error produces an invalid protocol result. Brownfield `Project::parse([])` returns `MORVA2023` as `ProjectDiagnostic::Project`, whose `source_id()` is `None`; the overflow path is also project-level. The schema simultaneously requires at least one `sources[]` item and requires every finding to have a source-local `primary_location` referencing a source.
- **Divergence:** A core projection must either refuse to emit a protocol document, invent a source/range, drop the finding, or weaken the schema. Each choice violates a current rule. The existing CLI filters an empty directory before calling core, but AD-1 assigns projection ownership to core and the protocol never declares CLI discovery/read failures or source-less core failures as out-of-band preconditions.
- **Disposition:** **Fix.** Decide the production boundary explicitly: either define which discovery/read/project-construction failures produce no protocol document and specify their machine error/exit channel, or add a typed document/project-level location alternative and allow a source-less invalid envelope where necessary. Add brownfield conformance cases for empty projects and a synthetic project-level diagnostic. This also closes part of the currently incomplete operational envelope.

### 3. [P1] Unknown semantic evolution has contradictory and non-deterministic consumer behavior

- **Evidence:** AD-8 says consumers ignore unknown object members and never map unknown discriminants; the convention says “preserve/stop on unknown semantics”; the protocol compatibility section permits new discriminant values within v1 where consumers preserve/ignore them. The schema, however, closes current declaration kinds, type-reference kinds, expression kinds, operators, and several coverage discriminants with enums/consts, so a schema-validating consumer rejects those additions before it can preserve or ignore them.
- **Divergence:** “Preserve,” “ignore,” “reject the document,” and “stop processing only the affected subtree” have materially different safety and availability outcomes. Two adapters following the prose can choose incompatibly, and a new producer cannot know whether a new semantic kind is additive in v1 or requires v2.
- **Disposition:** **Discuss / fix.** Define an explicit compatibility matrix per extension point. For checked semantics, the safe default should state exactly whether an unknown discriminant invalidates the entire checked model/document or only disables the advertised optional capability. Align schema openness, capability negotiation, and consumer action with that choice. Replace “preserve/stop” with one normative action per context.

### 4. [P1] Multi-file system provenance silently inherits an incidental first-file span

- **Evidence:** Morva’s brownfield `Project::parse` merges same-named system shells, appends declarations from every source, and stores the merged `System.name` and `System.span` from the first source only. The protocol schema requires one `checked_model.system.location`, while AD-3 says nodes carry exact local provenance and virtual spans cannot cross the boundary. Neither the spine nor protocol says whether this location identifies the first shell, all contributing shells, or merely the system name token.
- **Divergence:** A projection that maps the existing merged span to the first source and a projection that emits every shell origin express different provenance yet both fit the architectural prose. First-shell-only provenance also makes the source-order choice look like semantic ownership of a project-wide construct.
- **Disposition:** **Fix.** Ratify a deliberate brownfield rule (for example, distinguish a canonical `name_location` from ordered `shell_locations`) or state explicitly that the first discovered shell is the canonical origin and test it. Do not leave a project-wide semantic node’s ownership implicit in an implementation accident.

## Medium findings

### 5. [P2] Logical subject/source naming is not canonical enough to enforce privacy or reproducibility

- **Evidence:** AD-3/AD-9 prohibit absolute host paths and promise reproducible bytes. The protocol only says `source_id` derives from a “logical input name”; `subject.name` has no normalization rule. The existing CLI accepts both relative and absolute paths, so an implementation must choose basename, CLI argument, project-relative path, or another value.
- **Impact:** Equivalent invocations can emit different bytes, absolute paths can leak through `subject.name`, and project digest framing can differ if logical-name normalization differs.
- **Disposition:** **Autofix/design clarification.** Define canonical `source.name` and `subject.name` algorithms for single files and directories, including separator normalization and allowed relative components. State that IDs are opaque to consumers. Add known-answer digest vectors covering multi-file order, non-ASCII UTF-8 names, empty content, and renames.

### 6. [P2] `language_version` has no single normative value or evolution owner

- **Evidence:** The checked-model schema accepts any non-empty string. Examples use `"0.1"`, the workspace producer version is `0.1.0`, and project docs variously say `v0.1` and package `0.1.0`. AD-8 explains protocol-major evolution but does not define who increments the language version or which syntax is canonical.
- **Impact:** Independent producers can emit `0.1`, `v0.1`, or `0.1.0`; consumers and cache keys cannot compare them reliably.
- **Disposition:** **Autofix/design clarification.** Bind v1 to one canonical language-version value and identify its owner/change rule, or omit it until Morva has a separate language-version contract and rely on producer/protocol versions.

### 7. [P2] The local operational envelope omits machine-output failure and resource behavior

- **Evidence:** AD-10 closes network/provider/auth/persistence concerns and Deferred mentions later streaming/redaction, but no invariant or open item covers unreadable/non-UTF-8 sources, discovery races, serializer/validator failure, broken stdout, exit-code correlation, partial JSON prevention, or output/input size behavior. Inline source plus a duplicated semantic tree can materially increase peak memory.
- **Impact:** CLI and adapters can disagree on whether failures produce JSON, stderr text, partial output, or which exit status is authoritative. A consumer cannot know whether a missing document means invalid Morva, failed I/O, or failed conformance validation.
- **Disposition:** **Defer explicitly with a revisit condition, then decide in the implementation spec.** Require atomic/no-partial JSON emission, distinguish pre-document operational failures from protocol-invalid results, preserve current CLI safety checks and exit contracts, and establish measured thresholds that trigger JSONL/redaction work.

## Supported decisions

- The named paradigm is appropriate and carries a useful model: deterministic core facts, pure read-only projection, non-authoritative AI candidates, and explicit human acceptance.
- AD-1 correctly ratifies the existing two-crate ownership boundary: language semantics stay in `morva-core`, while CLI owns safe discovery/read and rendering.
- AD-2, AD-5, AD-6, AD-12, and AD-13 jointly prevent the three validated unsafe shortcuts: partial checked models after errors, conflating validity with coverage, treating a green revision as acceptance, hiding original intent from review, and equating schema-valid with protocol-conforming.
- AD-3 correctly ratifies source-local UTF-8 byte spans and forbids the codebase’s private merged virtual spans from escaping.
- AD-10 supplies the essential deployment/provider/privacy boundary for this local feature; the missing operational details are narrower than a missing deployment strategy.
- The SHA-256 decision is honestly tagged `[ASSUMPTION]`, forbids incompatible fallback hashes, and is listed under Deferred. It should remain a release blocker for v1 rather than being silently resolved during implementation.

## Recommended gate action

Resolve findings 1–4 before finalizing the spine. Findings 5–6 are small protocol clarifications that should land with those fixes. Finding 7 may move to Deferred/open items only if it names the pre-document failure boundary and a concrete implementation-spec/revisit trigger. After revision, rerun lint plus conformance cases containing a non-empty checked model, a multi-file project, project-level failure, unknown extension data, and digest known-answer vectors.

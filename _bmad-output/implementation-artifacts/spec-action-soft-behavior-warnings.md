---
status: approved
story: 2.2
date: 2026-08-11
---

# Action Soft Behavior Warnings

## Frozen Intent

### Always

- Every parsed `atomic`, `idempotent`, `timeout`, `retry`, or `implementation_hint` action item produces exactly one structured, non-fatal `MORVA5002` notice during explicit analysis.
- The notice records its owning action, a structured behavior kind, the keyword's original UTF-8 byte span, and the fixed message `action '{action}' soft behavior '{behavior}' is parsed but not semantically validated or executed by simulation`.
- Line items remain opaque through the logical newline or action closing brace. A whitelisted word inside an item's payload does not create another notice. Each matched `implementation_hint` block is one item, including when its body contains nested braces.
- Document notices remain source-span ordered. Merged findings retain error-first ordering for identical spans. Project notices are mapped exactly once to the responsible `SourceId` and local span.
- Existing `check(&Document)` and `Project::check()` remain error-only. CLI `check` reuses the shared finding renderer: warning-only input prints `ok:` and exits 0; any error exits 1 without success output.
- Parser failures preserve their existing codes, messages, and spans and produce no partial notices. Simulation remains behaviorally identical with or without soft behavior items.

### Approved Public Rust Source Compatibility Exceptions

- `Action` gains `pub soft_behaviors: Vec<SoftBehavior>`. External struct literals and complete struct patterns must add or ignore this field.
- `NoticeKind` gains `ActionSoftBehavior { action, behavior }`. Exhaustive matches must handle the new variant.
- These are the only approved source compatibility exceptions. Existing function signatures, `Diagnostic`, error-only APIs, CLI text contracts, exit codes, and simulation behavior remain compatible. Derived `Debug` text is not a compatibility contract.

### Never

- Do not parse or retain soft payloads, paths, parameters, or implementation-hint bodies as semantic AST.
- Do not validate or execute atomicity, idempotency, timeout, retry, implementation hints, user code, IO, clocks, networks, or external state.
- Do not scan raw source after lexer/parser failure, introduce partial AST recovery, widen the whitelist, or turn soft behavior into clauses.
- Do not add warning output to `parse`, `inspect`, or `simulate`; do not add capability inventory, JSON, MCP, new Boolean syntax, dependencies, or a second CLI renderer.

### Ask First

- A different warning code, category, message template, keyword-span policy, item boundary, ordering rule, or exit-code behavior.
- Any additional public API break, parser grammar expansion, semantic rule, simulator phase, source-map rule, or directory-discovery change.

## Verification

- Core tests cover all five kinds, exact structured notices, keyword spans, opaque payload boundaries, repeated items, newline variants, parser-error parity, mixed findings, and legacy `check()` parity.
- Project and real CLI tests cover deterministic local mapping, warning-only and mixed outcomes, safe rendering, and syntax-failure behavior.
- Simulation tests compare models with and without soft behavior and require identical phases, changes, state, and result.
- Full formatting, locked strict Clippy, workspace tests, both eight-command example loops, and `git diff --check` must pass before review.

## Change Log

- 2026-08-11: Formally approved by alex through Story 2.2 authorization before production-code changes.

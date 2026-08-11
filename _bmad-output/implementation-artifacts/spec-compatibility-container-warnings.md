---
status: done
story: 2.1
date: 2026-08-10
---

# Compatibility Container Warnings

## Frozen Intent

### Always

- `module`, `service`, `event`, `flow`, `lifecycle`, and `policy` each produce one structured, non-fatal `MORVA5001` notice when a parsed document is explicitly analyzed.
- The notice records a structured compatibility-container category, kind, name, fixed English message, and the container name span.
- Existing `check(&Document)` and `Project::check()` remain error-only and preserve their public shapes and error ordering.
- CLI `check` renders warnings with the existing safe source/path view; warning-only input prints `ok:` and exits 0, while any error exits 1 without `ok:`.
- Project warnings map once from virtual spans to the responsible source and local span.
- Compatibility-only text remains opaque and non-executable.

### Never

- Do not add severity to `Diagnostic` or warnings to existing error-only check results.
- Do not scan raw text for warnings after lexer/parser failure or introduce partial AST recovery.
- Do not warn for action soft behavior, change inspect, add capabilities, or modify simulation in this increment.
- Do not add dependencies or duplicate parser/checker/source rendering logic in the CLI.

### Ask First

- A different warning code/message/category or span policy.
- Warning output from parse, inspect, or simulate.
- Any parser compatibility whitelist, AST, scope, exit-code, or simulation change.

## Verification

- Core public analysis tests cover six kinds, structured category, exact code/message/name span, error separation, and old `check()` parity.
- Project tests cover later-source local mapping.
- Real CLI tests cover warning-only success and mixed warning/error failure ordering.
- Full fmt, strict Clippy, workspace tests, single/multi-file command loops, and diff check must pass before review.

## Change Log

- 2026-08-10: Approved through Epic 2 Story 2.1 planning; implementation prepared for review.
- 2026-08-11: Formal review completed; merged views, shared rendering, and regression evidence verified.

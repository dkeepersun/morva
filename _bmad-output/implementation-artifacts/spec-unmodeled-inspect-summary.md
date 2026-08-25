---
status: approved
story: 2.3
date: 2026-08-26
---

# Unmodeled Inspect Summary

## Frozen Intent

### Always

- `morva inspect` derives its unmodeled summary from the same core analysis notices that `check` renders (`analyze()` on the loaded document); it never re-scans source text or duplicates the notice logic.
- When the model contains at least one compatibility container or action soft behavior, inspect appends after the existing semantic summary, on stdout:
  - `unmodeled: {N} item(s)` where `N` is the total notice count;
  - `  compatibility containers: {C}` followed by one `    {kind} {name}` line per container in notice (source-span) order;
  - `  action soft behaviors: {B}` followed by one `    {action}: {behavior}` line per soft behavior in notice order.
- Both section headers appear whenever the summary appears, even when one count is 0, so the summary shape is deterministic.
- Names and kinds come only from structured `NoticeKind` fields (parsed ASCII identifiers and fixed kind strings); skipped container or hint body text is never echoed.
- For projects, notice order follows the assembled document's virtual span order, which is discovery filename order then source order; repeat runs on identical input are byte-identical.
- Warning-only models keep inspect exit 0. Models without unmodeled content print no summary line at all; the previously committed inspect text is unchanged for them.

### Never

- Do not change `parse` or `simulate` output, the checker, the simulator, notice codes/messages, or exit codes.
- Do not claim any listed item is validated, executed, or covered; the section name stays `unmodeled`.
- Do not add spans, file paths, JSON, machine-readable formats, or a capability inventory (Story 2.4+).

### Ask First

- Any different summary wording, ordering rule, or extension of the summary to other commands.

## Verification

- CLI exact-output test on `examples/order.morva` locks the full inspect text including the summary (1 container + 2 soft behaviors).
- CLI project test locks cross-file ordering (two files, mixed containers and soft behaviors) and byte-identical repeat runs.
- Existing clean-model exact-output inspect test proves no summary noise is added.

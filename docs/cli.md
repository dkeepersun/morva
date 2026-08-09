# CLI design

The executable is `morva`. Commands should be scriptable, deterministic, and quiet on success unless useful output was requested.

## Implemented

```text
morva check <file>  Parse and run current semantic checks
morva parse <file>  Print declarations, enum members, fields, parameters, and clauses
morva inspect <file>  Print a stable text summary of the semantic model
morva simulate <file> <scenario>  Run one checked scenario in memory
morva help          Show concise usage
```

`check`, `parse`, and `inspect` all parse and run the current semantic checker before
producing success output. `parse` prints the typed AST surface that Morva currently
models; it does not reproduce ignored compatibility-only text.

Exit codes are `0` for success, `1` for invalid input, and `2` for command or file usage errors.
Invalid input diagnostics include a stable code, line and column, and a source marker. Tabs are expanded and control bytes are escaped when a source line is displayed.

## Planned, not implemented

```text
morva grill <file>
morva review <file>
morva map <file>
```

Machine-readable diagnostics and configuration should be introduced only when editor or automation integration begins.

`simulate` prints its selected action, fixed execution phases, state changes, final in-memory state, and PASS/FAIL result. A successful simulation exits `0`; a model or runtime failure exits `1`. It never executes implementation hints or accesses external state.

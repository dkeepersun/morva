# CLI design

The executable is `morva`. Commands should be scriptable, deterministic, and quiet on success unless useful output was requested.

## Implemented

```text
morva check <file>  Parse and run current semantic checks
morva parse <file>  Print the declaration tree
morva help          Show concise usage
```

Exit codes are `0` for success, `1` for invalid input, and `2` for command or file usage errors.

## Planned, not implemented

```text
morva simulate <scenario-or-flow>
morva grill <file>
morva review <file>
morva map <file>
```

Machine-readable diagnostics and configuration should be introduced only when editor or automation integration begins.


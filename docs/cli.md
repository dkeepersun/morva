# CLI design

The executable is `morva`. Commands should be scriptable, deterministic, and quiet on success unless useful output was requested.

## Implemented

```text
morva check <file-or-directory>  Parse and run current semantic checks
morva parse <file-or-directory>  Print declarations, enum members, fields, parameters, and clauses
morva inspect <file-or-directory>  Print a stable text summary of the semantic model
morva simulate <file-or-directory> <scenario>  Run one checked scenario in memory
morva help          Show concise usage
```

`check`, `parse`, and `inspect` all parse and run the current semantic checker before
producing success output. `parse` prints the typed AST surface that Morva currently
models; it does not reproduce ignored compatibility-only text.

`check` emits `warning[MORVA5001]` for each compatibility container. Warnings use
the same safe path/excerpt renderer, remain on stderr, and do not change success
exit `0`; only errors suppress the `ok:` line and exit `1`. Other commands do not
gain warning output in this increment.

A directory is a flat Morva project. Discovery includes only direct-child regular
files whose extension is exactly lowercase `.morva`, ignores other files,
subdirectories, and symlinks, and sorts candidates by filename bytes. Every file
must contain exactly one top-level `system` with the same name. Morva merges only
their child declarations and retains the existing global short-name rules. It reads
all candidates before model output; an empty directory, invalid UTF-8, or discovery/
read failure exits `2` without partial stdout. Model errors exit `1` and identify the
responsible file with its local line, column, excerpt, and marker.

Candidate filenames must be valid UTF-8; an otherwise matching non-UTF-8 filename
is an input error (`2`) rather than a lossy source identity. Project reads reject
symlinks and paths resolving outside the selected directory, then bind validation
and UTF-8 decoding to the opened file handle. Portable standard-library APIs cannot
make `open` atomically `nofollow` on every platform; metadata identity is rechecked
before reading, with the residual concurrent same-file mutation limit documented in
the architecture.

Exit codes are `0` for success, `1` for invalid input, and `2` for command or file usage errors.
Invalid input diagnostics include a stable code, line and column, and a source marker. LF, CRLF, and CR each advance one logical line, including in mixed files; newline terminators are omitted from excerpts. Each displayed source excerpt and marker is limited to 160 rendered characters, keeps the error start visible, and uses `...` for cropped context. Tabs expand to four spaces while the reported logical column still counts each tab as one character; control and non-ASCII bytes are escaped without splitting an escape fragment. This is not a total stderr limit: diagnostic messages and safely escaped UTF-8 paths are not truncated. Every CLI result safely escapes control characters in displayed paths.

## Planned, not implemented

```text
morva grill <file>
morva review <file>
morva map <file>
```

Machine-readable diagnostics and configuration should be introduced only when editor or automation integration begins.

`simulate` prints its selected action, fixed execution phases, state changes, final in-memory state, and PASS/FAIL result. A successful simulation exits `0`; a model or runtime failure exits `1`. It never executes implementation hints or accesses external state.

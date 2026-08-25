# CLI design

The executable is `morva`. Commands should be scriptable, deterministic, and quiet on success unless useful output was requested.

## Implemented

```text
morva check <file-or-directory>  Parse and run current semantic checks
morva parse <file-or-directory>  Print declarations, enum members, fields, parameters, and clauses
morva inspect <file-or-directory>  Print a stable text summary of the semantic model
morva simulate <file-or-directory> <scenario>  Run one checked scenario in memory
morva capabilities  Print the authoritative capability inventory
morva help          Show concise usage
```

`capabilities` takes no model input, reads no files, and prints the core
`capabilities()` inventory as stable, deterministic text: version, semantic
declarations, clauses, expression forms, operators, literals, builtin types and
aliases, simulation value types and phases, compatibility containers, soft
behaviors, and explicitly unsupported categories. Repeat runs are
byte-identical and exit `0`. The listed container and soft behavior categories
are the same constants the parser and `check`/`inspect` warnings use, so the
inventory cannot drift from executable behavior without failing tests.

`check`, `parse`, and `inspect` all parse and run the current semantic checker before
producing success output. `parse` prints the typed AST surface that Morva currently
models; it does not reproduce ignored compatibility-only text.

`check` emits `warning[MORVA5001]` for each compatibility container and
`warning[MORVA5002]` for each parsed action soft behavior. Both use the same safe
path/excerpt renderer, remain on stderr, and do not change success exit `0`; only
errors suppress the `ok:` line and exit `1`. `parse`, `inspect`, and `simulate` do
not render these warnings.

`inspect` appends an `unmodeled:` summary derived from the same core analysis
notices whenever the model contains compatibility containers or action soft
behaviors: total item count, then container kind/name pairs and action/soft
behavior pairs in source order (project file order first for directories). It
shows only structured kinds and names — never skipped body text — and claims no
validation or execution of the listed items. Models without unmodeled content
print no summary, keeping the existing inspect text unchanged. The summary is on
stdout and repeat runs are byte-identical.

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

## Machine output

`check` and `parse` accept `--format json`. Machine output writes exactly one
versioned JSON envelope to stdout and nothing to stderr:

```json
{ "protocol": "morva.cli", "schema_version": 1, "command": "check", "success": true, "diagnostics": [] }
```

Each diagnostic carries `severity` (`error`/`warning`), the stable `code`, the
complete `message`, and a `location` with the real source path, 1-based
line/column, and the file-local byte span — project diagnostics never expose
virtual offsets, and rare source-less project diagnostics use `location: null`.
Model errors keep exit `1` with `success: false`; unreadable input, discovery
failures, and usage errors (once `--format json` is recognized) return a
machine error envelope `{ "error": { "kind": "input" | "usage", "message": … } }`
with exit `2`. Output is deterministic — no timestamps, absolute machine paths,
or environment noise — and JSON escaping keeps control characters out of the
terminal. `schema_version` only changes with a reviewed incompatible protocol
change. Without `--format json` every command keeps its existing human output
and exit codes.

`parse --format json` embeds the structured AST as an `ast` member: every node
uses an explicit stable `kind` with semantic fields (declarations, members,
fields with written type names, parameters, soft behavior kinds, clauses, and
recursive `binary`/`not`/`or` expression nodes), and every location carries the
real source path with its file-local span. The merged multi-file system shell
is synthetic and reports `location: null` instead of impersonating one file.
Skipped compatibility and implementation-hint bodies are never echoed. The AST
JSON is a read-only structured view; it is not a lossless serialization of the
original `.morva` source.

## Planned, not implemented

```text
morva grill <file>
morva review <file>
morva map <file>
```

Machine-readable diagnostics and configuration should be introduced only when editor or automation integration begins.

`simulate` prints its selected action, fixed execution phases, state changes, final in-memory state, and PASS/FAIL result. A successful simulation exits `0`; a model or runtime failure exits `1`. It never executes implementation hints or accesses external state.

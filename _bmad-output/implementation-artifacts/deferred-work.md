# Deferred Work

## Engineering quality

- Add a minimal CI workflow that runs `cargo fmt --check`, strict workspace Clippy, workspace tests, and the executable example commands. This is a pre-existing repository gap surfaced by the documentation-baseline review; the current increment records and verifies local gates but does not add deployment or automation infrastructure.

## Source text portability

- Define universal newline behavior as a separate language-contract increment: LF, CRLF, and CR each represent one newline; CRLF is one token; `//` stops at any supported newline; lexer spans and CLI line/column/excerpts agree for mixed endings. This is intentionally split from diagnostic-window resource hardening because it changes parser-visible semantics.

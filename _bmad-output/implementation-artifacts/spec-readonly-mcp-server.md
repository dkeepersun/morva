---
status: approved
story: 5.1-5.4
date: 2026-08-26
---

# Read-Only MCP Server

## Frozen Intent

### Always

- `morva-mcp` is an isolated workspace crate over `morva-core` + `morva-machine`; core keeps no MCP, CLI, JSON-RPC, or async-runtime dependency. The server is std-only: a repository-owned bounded JSON parser (depth 128, 32 MiB line cap) for incoming JSON-RPC and the core canonical writer's compact form for outgoing frames. stdout carries protocol messages only.
- `initialize` supports protocol versions `2024-11-05`, `2025-03-26`, `2025-06-18` and echoes the requested one; an unsupported version returns a structured `invalid params` error listing supported versions. Malformed JSON → `-32700`; unknown method → `-32601`; unknown resource → `-32002`; parameter/limit violations → `-32602`. Notifications get no response; no protocol edge panics or pollutes stdout; EOF exits cleanly.
- The `instructions` text and every tool description state the read-only boundary: no file reads/writes, no external actions, no shell, no network; model changes are a separate human-reviewed step. No write/patch/commit/approval tool exists.
- One resource, `morva://capabilities`, serves byte-identical content to `morva capabilities --format json` — the same core inventory and capability model version, never a third classification table.
- Tools `morva_check`, `morva_parse`, `morva_inspect`, `morva_simulate` accept only `sources: [{name, text}]` (plus `scenario` for simulate). Names are diagnostic identities only — never paths, URLs, or commands — non-empty, ≤1 KiB, unique per request; ≤256 sources; combined text ≤8 MiB; violations are `invalid params` and nothing is parsed. Bundles sort by exact UTF-8 name bytes, then go straight to core `Project::parse`/`check`/analysis/simulate — the MCP layer copies no language rule.
- Tool results embed the identical `morva.cli` machine envelope produced by `morva-machine`. Language-level failures (lexical/syntax/project/semantic errors, simulation failures, unknown scenarios) are successful tool results with `success: false`; only parameter, protocol, and limit violations are MCP-level errors. Locations carry the caller's logical source names with 1-based line/column and file-local byte spans; virtual offsets never appear.
- Sources live only in request-scoped memory: no caching, no temp files, no state across requests; a failing request cannot pollute a later one. Identical requests produce byte-identical results with no time, process, or environment noise.

### Never

- No filesystem, network, shell, or model-action access from any tool; no workspace or path parameters; no write-back or auto-approval surface.
- No duplicated diagnostic/AST/summary/report schema — `morva-machine` remains the single implementation shared with the CLI.

### Ask First

- New tools or resources, protocol version changes, limit changes, or any dependency addition to the integration crate.

## Verification

- `crates/morva-mcp/src/json_parse.rs` unit tests: escapes, surrogates, malformed inputs, depth bound.
- `crates/morva-mcp/tests/server.rs` real-process seam tests: initialize + boundary text, unsupported version error, protocol edges without stdout pollution, deterministic capability resource, tools/list, check/parse/inspect/simulate payloads with logical-name mapping, bundle-limit and duplicate-name rejections, request isolation, and no-filesystem-trace proof.

# Morva Checked Semantics Protocol v1

> Status: design draft; not an implemented CLI contract
> Audience: Morva maintainers, AI-tool authors, future MCP/editor integrations
> Companion schema: [morva-checked-semantics-v1.schema.json](schema/morva-checked-semantics-v1.schema.json)

## Purpose

This protocol is the read-only machine boundary between Morva's deterministic language core and non-authoritative consumers such as AI review tools. It carries exact source provenance, check findings, semantic-coverage gaps, and—only for an error-free input—a normalized checked model.

It is designed to support:

```text
generate candidate sources
        ↓
Morva parse + check + analyze
        ↓
diagnose from structured findings
        ↓
generate a new complete candidate source set
        ↓
check again
        ↓
human compares and accepts, rejects, or blocks
```

The protocol never edits `.morva` files and never makes an AI response authoritative.

## Normative invariants

The key words MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, and MAY in this document are to be interpreted as described in BCP 14 ([RFC 2119](https://www.rfc-editor.org/rfc/rfc2119), [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174)) when, and only when, they appear in all capitals.

1. The producer MUST derive all semantic facts from `morva-core`; a CLI or adapter MUST NOT reconstruct language semantics.
2. `sources[].content` is the exact UTF-8 input. Every source-owned location MUST use a half-open, source-local byte range into that content. Only a source-less core project finding MAY instead use an explicit subject location; semantic nodes and coverage entries are always source-owned.
3. If any error exists, `result.status` MUST be `invalid` and `result.checked_model` MUST be `null`.
4. `result.checked_model` MUST be present only after successful parse, project assembly, and semantic checking.
5. Warnings do not make a result invalid, but semantic-coverage warnings MUST appear in both `findings` and `coverage`.
6. Output ordering MUST be deterministic. Wall-clock time, random IDs, absolute host paths, and process-specific values MUST NOT appear.
7. Consumers MUST reject unknown protocol major versions and any checked model whose `language_version` is not `0.1`. They MUST ignore unknown object members. All v1 semantic discriminants are closed; an unknown semantic `kind`, operator, or type makes the checked model unsupported and MUST fail closed.
8. A protocol document is evidence about one exact source revision. Node IDs and finding IDs MUST NOT be reused as cross-revision identity.
9. AI revisions MUST be submitted as new complete source sets. The protocol MUST NOT define patch application, mutation, approval, or write-back operations.
10. A human-reviewed `.morva` source set remains the only Source of Truth.

## Envelope

```json
{
  "protocol": "morva.checked-semantics",
  "version": 1,
  "capabilities": [
    "morva.sources.inline",
    "morva.locations.byte-range",
    "morva.findings.v1",
    "morva.coverage.v1",
    "morva.checked-model.v1"
  ],
  "producer": {
    "name": "morva",
    "version": "0.1.0"
  },
  "subject": {
    "kind": "project",
    "name": "order-project",
    "revision": {
      "algorithm": "sha256",
      "value": "..."
    }
  },
  "sources": [],
  "result": {}
}
```

`version` is the protocol major version, not the Morva language or producer version. `[ASSUMPTION]` v1 uses SHA-256 for interoperable revision identities; this architecture and prototype are not production-shippable until the project's dependency/change-control approval is recorded and an actual core producer passes the known-answer and conformance tests.

The five capabilities shown above are mandatory in every v1 document. Additional capability strings MAY advertise additive non-semantic data, but they cannot add semantic variants, weaken the mandatory v1 contract, or change checked-model interpretation.

## Source identity and provenance

Each source has:

```json
{
  "source_id": "source:0",
  "name": "20-actions.morva",
  "content_encoding": "utf-8",
  "content": "system Shop { ... }\n",
  "revision": {
    "algorithm": "sha256",
    "value": "..."
  }
}
```

- `source_id` is the opaque ordinal `source:<index>` after canonical ordering. It is unique only inside this document and carries no path semantics.
- `revision` binds this record to exact content bytes. It is not proof of origin, authorship, authenticity, or human approval.
- A protocol-producing core entry point accepts logical names separately from host paths. A logical source name is a non-empty exact UTF-8 final path component, contains no path separator, and undergoes no Unicode normalization or separator rewriting.
- For a project, core rejects duplicate logical names and sorts sources by exact UTF-8 name bytes before assembly, checking, ID assignment, digesting, and emission. This ratifies the CLI's current direct-child filename-byte order while making the core protocol boundary deterministic for other callers.
- For a single file, `subject.name` and `source.name` are the supplied logical final component. For a project, `subject.name` is the project root's supplied logical final component; each source name is its direct-child filename.
- For a file, `subject.revision` equals its source revision. For a project, hash the byte stream `morva-project-v1\0`, followed for each ordered source by `decimal-name-byte-length:name-bytesdecimal-content-byte-length:content-bytes`. Decimal lengths contain ASCII digits and `:` is the delimiter; lengths make the framing unambiguous.
- Inline content is mandatory in v1 so a finding can be independently verified and an AI can propose a complete replacement without hidden filesystem access.

Known-answer vectors:

| Case | SHA-256 |
|---|---|
| Empty file bytes | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| Empty project framing (`morva-project-v1\0`) | `9e6148ade2eb6fb7153b1c330e6f2db3f4d4e88a2c00b9f41a29d7ffb9befc9f` |
| Ordered sources `a.morva → ""`, `é.morva → "x\r\n"` | `5d7da2df72ca38343090ae86ae63ab778d94b33c39be3fb34b8b81057c3469ee` |

The two-file project preimage in hexadecimal is `6d6f7276612d70726f6a6563742d763100373a612e6d6f727661303a383ac3a92e6d6f727661333a780d0a`.

A source-owned location has one authoritative representation:

```json
{
  "kind": "source",
  "source_id": "source:0",
  "byte_range": { "start": 91, "end": 100 }
}
```

The range is half-open. `start == end` is allowed for an insertion point. Line/column and UTF-16 positions are intentionally absent from v1 because they are derived views with competing counting conventions.

A source-less core project diagnostic uses `{ "kind": "subject" }`. It MUST NOT invent a source or byte range. An empty `Project::parse` result may therefore have `sources: []`, a project subject revision, and a subject-scoped `MORVA2023` finding.

Failures before authoritative bytes reach core—usage, discovery, unreadable input, unstable file identity, or invalid UTF-8 decoding—produce no checked-semantics document, no partial stdout, and retain the CLI operational exit code `2` with stderr reporting.

## Result states

### Invalid

```json
{
  "status": "invalid",
  "findings": [
    {
      "finding_id": "finding:0",
      "severity": "error",
      "category": "semantic",
      "code": "MORVA2008",
      "message": "...",
      "primary_location": {
        "kind": "source",
        "source_id": "source:0",
        "byte_range": { "start": 120, "end": 129 }
      }
    }
  ],
  "coverage": {
    "assessment": "unavailable",
    "fully_modeled": null,
    "unmodeled": []
  },
  "checked_model": null
}
```

Parse, project-assembly, and semantic errors share this result shape. `category` identifies `lexical`, `syntax`, `project`, or `semantic`; the core-owned projection assigns it from typed pipeline origin (lexer, parser, project assembly, or checker), never by inspecting human message text. Consumers MUST use `severity`, not code ranges or message text, to decide validity.

### Valid

```json
{
  "status": "valid",
  "findings": [],
  "coverage": {
    "assessment": "complete",
    "fully_modeled": true,
    "unmodeled": []
  },
  "checked_model": {
    "language": "morva",
    "language_version": "0.1",
    "system": {}
  }
}
```

A warning-only result is `valid` but can have `coverage.fully_modeled == false`. Consumers MUST NOT treat `valid` as “all source intent is semantically modeled.” If parsing or project assembly fails before coverage can be collected, `assessment` is `unavailable`, `fully_modeled` is `null`, and `unmodeled` is empty; consumers MUST NOT interpret that as full coverage.

## Findings

Every finding contains:

| Field | Contract |
|---|---|
| `finding_id` | Deterministic ordinal within this document; not durable across revisions |
| `severity` | `error` or `warning` |
| `category` | `lexical`, `syntax`, `project`, `semantic`, or `coverage` |
| `code` | Stable Morva diagnostic/notice code |
| `message` | Complete human-readable message; consumers MUST NOT parse it |
| `primary_location` | Exact source-local location, or subject location for a source-less project diagnostic |
| `details` | Optional typed payload; v1 defines only the two coverage variants below |

Current typed coverage details are:

```json
{
  "kind": "compatibility_container",
  "container_kind": "policy",
  "name": "CancellationRules"
}
```

```json
{
  "kind": "action_soft_behavior",
  "action": "Confirm",
  "behavior": "idempotent"
}
```

Consumers MUST use these typed fields and MUST NOT recover `container_kind`, `action`, or `behavior` by parsing `message`.

## Coverage

Coverage prevents “parsed successfully” from becoming “semantically verified.”

```json
{
  "assessment": "complete",
  "fully_modeled": false,
  "unmodeled": [
    {
      "kind": "compatibility_container",
      "name": "CancellationRules",
      "container_kind": "policy",
      "location": {
        "kind": "source",
        "source_id": "source:0",
        "byte_range": { "start": 210, "end": 227 }
      }
    }
  ]
}
```

Each unmodeled entry MUST correspond to one warning finding. The reverse is not required for future non-coverage warnings.

`assessment: complete` means the producer completed the additive coverage scan, not that the model is valid. Semantic errors can therefore coexist with a complete coverage assessment. `assessment: unavailable` is required when an earlier failure prevented that scan.

## Checked-model algebra

The checked model is a projection, not a serialization of Rust structs. Its complete v1 algebra is normative in the companion JSON Schema. Producers MUST emit every required member, including empty arrays; absence never means an empty semantic collection.

### Common forms

All source-owned semantic objects carry a source location. Named nodes additionally carry revision-local `node_id`, `semantic_key`, and `name`. A `semantic_key` MUST be unique within one checked model and every reference to it MUST resolve there. It is an opaque exact UTF-8 string: consumers MAY compare it for equality but MUST NOT parse its spelling or treat it as durable identity across revisions. `node_id` has the same revision-local durability boundary.

Resolved types are closed to:

```json
{ "kind": "builtin", "name": "Boolean" }
{ "kind": "builtin", "name": "Integer" }
{ "kind": "builtin", "name": "Decimal" }
{ "kind": "builtin", "name": "String" }
{ "kind": "builtin", "name": "Id" }
{ "kind": "enum", "semantic_key": "enum:OrderStatus" }
{ "kind": "entity", "semantic_key": "entity:Order" }
```

Builtin aliases are always normalized to these spellings. Compatibility containers appear only as ordered `container_path` entries with `container_kind`, `name`, and location, plus their coverage warning. They are not semantic declarations. Soft behaviors never enter the checked model.

### System and declarations

The system has `kind`, `node_id`, `semantic_key`, `name`, ordered `shell_locations`, and source-ordered `declarations`. A multi-file project emits every same-name system shell location; no first source silently owns the merged system.

Declarations are exactly:

| Kind | Required semantic payload |
|---|---|
| `enum` | ordered `members` |
| `entity` | ordered `fields`, ordered `invariants` |
| `action` | ordered `parameters`, ordered `clauses` |
| `scenario` | ordered `items` |

Every declaration includes `container_path`, even when empty. Fields contain a resolved type. Enum members contain their owning enum semantic key. Parameters contain their owning action semantic key and resolved type.

### Clauses and assignments

An action clause has `clause_kind`, `state_phase`, ordered `expressions`, and location. Clause kinds are `requires`, `effects`, `ensures`, and `invariant`; phases are respectively `pre`, `effect`, `post`, and `both`. A clause expression is exactly `{ kind: "predicate", expression }` or `{ kind: "assignment", assignment }`.

Assignments contain a target path, `operator = set | add | subtract`, value expression, resolved `target_type`, and location.

### Expressions

The closed expression union is:

| Kind | Payload |
|---|---|
| `integer` | canonical decimal-string `value` in signed 64-bit range `[-9223372036854775808, 9223372036854775807]`, normalized `resolved_type` |
| `boolean` | Boolean `value`, normalized `resolved_type` |
| `enum_member` | enum semantic key, member semantic key/name, normalized enum type |
| `path` | ordered segments, root binding, normalized terminal type |
| `binary` | comparison operator, left/right expressions, Boolean result type |
| `not` | one Boolean operand expression, Boolean result type |
| `or` | left/right Boolean expressions (left-associative source shape), Boolean result type |

Comparison operators are `equal`, `not_equal`, `greater`, `greater_equal`, `less`, and `less_equal`.

> Pre-release design revision (2026-08-26): the language gained predicate
> negation and short-circuit disjunction before any v1 producer shipped, so the
> closed v1 expression union includes `not` and `or` from its first published
> revision. This is not a post-release discriminant addition. A path root is exactly one of `action_parameter`, `entity_self`, or `scenario_instance`; each names its resolved binding. Every path segment carries its name, resolved type, and exact location.

### Scenario items

Scenario items are exactly:

- `given`: one assignment;
- `run`: resolved action semantic key and ordered arguments, each mapped to its parameter semantic key and entity type;
- `expect`: one expression.

The representation does not introduce multi-action semantics, scalar scenario arguments, or values beyond the current checked language.

### Completeness and evolution

The core invariant validator MUST prove that every checked declaration and child in the source appears exactly once in this algebra, all semantic keys resolve inside the same checked model, and all source locations belong to the current revision. New declaration, expression, assignment, clause, scenario-item, root-binding, type, or operator discriminants require protocol v2. Unknown extra object members remain ignorable.

## Determinism

For identical canonical logical names, exact input bytes, producer version, language version, and enabled capability set, output bytes SHOULD be identical.

Canonical v1 emission order:

1. fixed envelope field order shown by the schema;
2. capabilities in the mandatory order listed in the envelope section, followed by any additive capabilities in exact UTF-8 byte order;
3. sources in canonical exact-UTF-8 logical-name byte order;
4. findings in existing `AnalysisReport::findings` / `ProjectAnalysisReport::findings` order;
5. declarations and expressions in source order;
6. object members in schema order;
7. UTF-8 JSON, two-space indentation, final LF.

JSON object order is not semantic; canonical ordering exists for snapshots, diffs, and reproducible AI context.

## Validation layers

JSON Schema validates the document shape, required v1 capabilities, known typed payloads, and expressible status/coverage relationships. It cannot prove that a byte range is within the referenced source, that source IDs are unique, or that a coverage entry corresponds one-to-one with a warning.

A conforming producer MUST therefore run a `morva-core`-owned semantic-envelope validator before serialization. It verifies:

1. every logical name is valid and unique, sources are in canonical exact-UTF-8 name-byte order, IDs are consecutive `source:<index>`, and every source revision digest recomputes over exact content bytes;
2. a file subject has exactly one source and a subject revision equal to that source revision; a project subject revision recomputes over the exact canonical framing; `sources: []` occurs only for a source-less invalid project result, never a file or valid checked model;
3. every source location references an existing source and satisfies `start <= end <= source.content.len()` at UTF-8 byte boundaries; subject locations occur only on source-less project findings;
4. `valid` contains no error finding and uses a complete coverage assessment;
5. `invalid` contains at least one error and never contains a checked model;
6. every coverage entry has exactly one matching coverage warning with the same typed identity and location;
7. `fully_modeled` is true exactly when a complete coverage assessment has no unmodeled entries;
8. every checked-model node and resolved reference belongs to the checked source revision;
9. the checked model contains every source declaration and child exactly once, with `node_id` and semantic keys unique and every semantic key fully resolved within that model;
10. ordered system shell locations correspond exactly to the canonically ordered project sources;
11. expression result types, signed-64-bit integer values, assignment target/value compatibility, root bindings, clause/state-phase pairs, and clause payload kinds agree with the checker-owned semantic facts.

Schema conformance alone MUST NOT be called protocol conformance.

## Compatibility

- Adding optional object members or new capability strings is backward-compatible within major version 1.
- Removing/renaming a member, changing its meaning, changing authoritative position units, or allowing invalid results to carry `checked_model` requires version 2.
- Checked-model and typed-coverage discriminants are closed in v1. New values require v2. Unknown extra object members are ignored; unknown semantic discriminants reject checked-model consumption.
- `language_version` is exactly `0.1` in v1. A different language semantic surface requires a protocol version whose checked algebra can represent it.
- A consumer MUST check `protocol`, `version`, and required capabilities before reading semantic content.

## AI loop contract

An AI tool may:

1. read sources, findings, coverage, and a valid checked model;
2. explain findings using their exact locations;
3. return a complete proposed source set plus a human-readable rationale;
4. request a fresh Morva check of that source set.

It must not:

- claim `checked_model` exists for an invalid result;
- convert a warning-only coverage gap into a verified fact;
- modify accepted files without a separate human action;
- suppress or rewrite Morva diagnostic codes;
- treat a second `valid` result as human acceptance.

The external human-review workflow MUST present one immutable evidence bundle containing the original requirement/intent identity and version, current accepted revision, complete revised candidate and its revision, all intermediate candidate/check/diagnosis events or an explicit cumulative diff from the accepted revision, relevant findings and coverage, and the AI diagnosis/rationale. Presenting only the latest green result is non-conforming.

Every candidate names `candidate_parent_revision`, which MUST equal the current accepted revision—not a preceding unaccepted AI candidate. Acceptance is permitted only when the candidate's fresh protocol result is `valid`. It uses compare-and-swap over all three frozen bases—accepted revision, intent identity, and intent version—and fails if any changed after the evidence bundle was created.

When `coverage.fully_modeled` is false, acceptance additionally requires one human acknowledgement per unmodeled item. Its `coverage_item_ref` is an exact copy of that item's typed identity fields plus location; it also carries disposition `out_of_scope` or `externally_evidenced`, a rationale, and any evidence reference. The pair `(candidate_revision, coverage_item_ref)` is the interoperable acknowledgement key. Without complete acknowledgements the only conforming decision is `block`. An acknowledgement does not convert the construct into Morva-verified semantics.

Human review compares the accepted revision with the revised candidate and chooses:

- `accept`: semantics match the stated intent;
- `reject`: candidate is valid Morva but changes/omits intent;
- `block`: the current language cannot express the intent without an approved language change or explicit opaque boundary.

These decisions are workflow state outside this read-only protocol.

## Deferred

- CLI command/flag naming and whether machine output shares existing commands.
- Streaming/JSONL for very large projects.
- Source redaction or external source bundles.
- UTF-16 editor coordinates.
- Cross-revision semantic identity and incremental invalidation graphs.
- A production AI orchestration API, authentication, persistence, or MCP surface.

None of these are required to validate the v1 information model.

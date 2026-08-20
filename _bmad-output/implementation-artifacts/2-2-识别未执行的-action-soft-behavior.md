---
baseline_commit: bc81facbf3afe9a76993444fbbcb07106a1f797c
---

# Story 2.2: 识别未执行的 action soft behavior

Status: review

## Story

As a Morva 模型作者,
I want `check` 明确提示 action 中只被接受但未被验证或执行的行为项,
so that 我不会把 `atomic`、重试设置或实现提示误认为模拟器已经兑现的保证。

## Acceptance Criteria

1. 给定 action 包含 `atomic`、`idempotent`、`timeout`、`retry` 或 `implementation_hint`，当 core 分析模型时，每个源码项产生一个稳定、非致命 `MORVA5002` warning；warning 结构化保留所属 action、behavior kind 和 keyword 的原始 UTF-8 byte span，并明确说明该项未被语义验证或模拟执行。
2. 带参数/路径的 line soft behavior 和带嵌套 block 的 `implementation_hint` 继续按现有 grammar 接受；parser 不新增任何 soft payload/body 字段，新增 provenance 仅包含 kind 与 keyword span，既有 `Action.span` 保持原定义。line item 的边界沿用现有 parser：action-block iteration 只识别行首白名单 keyword，余下 token 是该 item 的 opaque payload，直到逻辑换行或 action `}`；payload 中再次出现的白名单词不产生额外 warning。每个已匹配的 `implementation_hint` block 是一个 item。
3. 同一 action 或不同 action 中的多个/重复 soft behavior 按原始源码 span 稳定排序，每个源码项恰好报告一次；与 `MORVA5001` container warning 或 semantic error 混合时继续使用 Story 2.1 的 merged finding view 和同位置 error-first tie-break。
4. soft behavior 位于多文件项目时，Project analysis 将 virtual span 仅回映一次为责任文件的 `SourceId + LocalSourceSpan`；CLI 显示真实路径、本地行列和安全 marker，warning-only 时 stdout 保留 `ok:` 且退出 0。
5. 白名单之外的 action item、拼写错误、line soft behavior 中的非法 block 以及损坏的 `implementation_hint` 继续返回现有 parser error；lexer/parser 失败没有 partial AST，因此不得扫描原文伪造 warning 或把 error 降级。
6. 对含 soft behavior 的模型执行 `simulate`，七阶段、状态变化、最终状态和成败与删除 soft behavior 后相同；模拟输出不得暗示已执行原子性、幂等、超时、重试、实现提示、IO 或外部状态。
7. 不含 soft behavior 的模型不新增 warning；`check(&Document)`、`Project::check()`、`Diagnostic`、语义 error 顺序、`parse`/`inspect`/`simulate` 输出和退出码保持兼容。本 Story 明确批准两项公开 Rust 源码兼容例外：`Action` 新增 `soft_behaviors` 字段，`NoticeKind` 新增 `ActionSoftBehavior` variant；外部 struct literal、完整 struct pattern 和 exhaustive enum match 可能需要迁移。除此之外，既有函数签名、error-only API、CLI 文本与退出码保持兼容；派生 `Debug` 结构文本不是兼容承诺。

## Tasks / Subtasks

- [x] Task 0: 在修改生产代码前冻结 Story 2.2 契约（AC: 1-7）
  - [x] 创建并批准独立冻结规格，明确 `MORVA5002`、message/category/keyword span、line/block item 边界、排序、退出码、两项公开 Rust API 源码兼容例外、迁移说明与非目标；未获批准不得开始 Task 1。

- [x] Task 1: 以最小 AST provenance 保留 soft behavior（AC: 1, 2, 5, 7）
  - [x] 在 `morva-core` 定义结构化 `SoftBehaviorKind` 与 `SoftBehavior { kind, span }`，kind 仅包含现有五项白名单，span 固定为 keyword token span。
  - [x] 为 `Action` 增加按源码顺序的 `soft_behaviors`，并在 `RebaseSpans` 中 checked rebase 每个 soft span；不把 soft behavior 伪装成 `Clause` 或可执行表达式。
  - [x] 重构 `action()` 与现有 skip helpers，在保留 kind/keyword span 后继续丢弃其余 token；保持未知项 `MORVA1007`、line item 非法 block `MORVA1014`、hint 缺 opening block `MORVA1016`、已打开但未闭合 hint 的 `MORVA1003: unclosed compatibility block` 与 opening-brace span，以及 `MORVA1024/1025` 等现有错误路径；失败时不产生 partial notice。

- [x] Task 2: 扩展现有 additive notice 模型（AC: 1, 3, 7）
  - [x] 在 `NoticeKind` 新增结构化 `ActionSoftBehavior { action, behavior }`，复用 `SoftBehaviorKind`，供 Story 2.3 消费，不解析 message。
  - [x] 每个 soft behavior 生成 `MORVA5002`，固定消息模板为 `action '{action}' soft behavior '{behavior}' is parsed but not semantically validated or executed by simulation`，责任 span 为 keyword span。
  - [x] 扩展 `collect_notices` 而不复制 semantic check；保持 notices 的 source-span 排序、`AnalysisReport::findings()` 的 error-first tie-break 及旧 `check()` error-only 路径。

- [x] Task 3: 复用 Project/CLI 通用 warning 管线（AC: 3, 4, 5, 7）
  - [x] 证明既有 `Project::analyze()` 可将 soft notice 恢复为责任源的 local span；不修改或复制 source-map 规则。
  - [x] 证明 CLI `check` 通过既有 merged finding view 和共享 `render_source_finding` 自动呈现 `MORVA5002`；CLI 不能识别 soft behavior kind、重新排序或创建第二套 renderer。
  - [x] 保持 warning-only exit 0 + `ok:`、warning+error exit 1 + empty success stdout；语法失败只呈现原 error。

- [x] Task 4: 建立 core、project、CLI 和 simulation 回归矩阵（AC: 1-7）
  - [x] Core：五种 kind、line 参数/路径、嵌套 hint block、exact code/message/category/action/keyword span、多项/重复项 once-only 与源码顺序；`atomic retry 2` 只产生一个 `Atomic` notice，两行 `atomic` / `retry 2` 才产生两个 notice。
  - [x] Core：soft + container + semantic error 混合 finding 顺序、clean analysis、旧 `check()` parity，以及未知/损坏 action item 仅返回原 parser error。
  - [x] Newline/comment：LF/CRLF/CR 的原始 keyword byte span；soft/hint 跳过区的 token-split 仍以 `MORVA1025` 失败且不产生 warning。
  - [x] Project：后排序文件本地映射、两文件相同 local offset 不串源、`Project::check()` parity 与 merged finding 顺序。
  - [x] CLI：单文件/目录的 warning-only、warning+error、语法失败、路径控制字符、本地行列/marker、stdout/stderr/退出码和确定顺序。
  - [x] Simulation：带/不带 soft behavior 的模型产生相同七阶段、changes、final state 与 result，输出不暗示 soft behavior 被执行。

- [x] Task 5: 同步当前事实与交付证据（AC: 1-7）
  - [x] 同步 requirements、language reference、implementation status、testing strategy、architecture、project context、README/CLI 中与 soft warning、AST 形状和可见输出直接相关的表述。
  - [x] 同步 `docs/language-design.md`：soft items 仍不进入语义求值或模拟，但显式 analysis 和 CLI `check` 会对每个已解析 item 产生一个结构化非致命 warning。
  - [x] 运行 fmt、locked strict Clippy、workspace tests、单/多文件八条示例闭环和 `git diff --check`；`examples/order.morva` 必须恰好产生 3 个 warning（`MORVA5001 × 1` + `MORVA5002 × 2`），`examples/order-project` 必须恰好产生 2 个指向 `20-actions.morva` 的 `MORVA5002`；其余三命令语义不变。只记录本地门禁证据，不得声称 hosted workflow 或 required quality gate 已通过；现有 CI shards 已覆盖这些测试，除非测试位置超出现有 shards，不修改 workflow。

### Review Findings

- [ ] [Review][Patch] 拒绝在 `implementation_hint` block 同一逻辑行继续识别第二个 soft item，避免 `implementation_hint {} atomic` 成功并产生两个 `MORVA5002` [crates/morva-core/src/parser.rs:270]

## Dev Notes

### Current Implementation Facts

- `parser::action()` 现在对 `implementation_hint` 调用 `skip_implementation_hint()`，对其余四项调用 `skip_soft_behavior_line()`；两者均返回 `()`，所以当前 AST 完全无留痕。
- `Action` 当前只有 `name/parameters/clauses/span`。给该公开 struct 增字段会改变外部 struct literal 的源码兼容性，但这是不重扫源码、不伪造语义节点前提下保留 provenance 的最小明确改动；必须在冻结规格和文档中直说。
- `semantic::check_action` 和 simulator 只消费已建模 clauses。本 Story 不得让它们解释 `soft_behaviors`。
- Story 2.1 已建立 `AnalysisReport`/`ProjectAnalysisReport`、结构化 `NoticeKind`、稳定 merged `findings()`、单次 Project span 回映和 CLI 共享 source renderer。本 Story 只扩展这条管线。
- Lexer/parser 失败不产生 partial AST。任何基于原文搜索 soft keyword 的 warning 都会违反现有边界。

### Required Data and Analysis Shape

- 推荐精确形状：`SoftBehaviorKind::{Atomic, Idempotent, Timeout, Retry, ImplementationHint}`，提供稳定 `as_str()`；`SoftBehavior { kind, span }`；`Action.soft_behaviors: Vec<SoftBehavior>`。
- `SoftBehavior.span` 是 keyword token span，不是整行或 hint block span。这保持 marker 短且确定，也防止多行未建模正文被误表示为受语义覆盖。
- `NoticeKind::ActionSoftBehavior { action: String, behavior: SoftBehaviorKind }`。所属 action 必须保留为结构化字段，供 Story 2.3 建立未建模清单，不能从 message 反解。
- `MORVA5001` 已冻结为 compatibility-container warning；soft behavior 使用 `MORVA5002`。两者共享 notice severity 和 merged sorting，不给 `Diagnostic` 加 severity。

### Parser and Span Guardrails

- 改造 skip helper 时先 clone/bump keyword token 并保存其 span，再执行现有正文跳过。不要用消费 newline 后的 `previous()` 计算责任 span。
- `implementation_hint` 仍要求 block，仍允许嵌套 brace；缺 opening block 保持 `MORVA1016`，已打开但未闭合保持 `MORVA1003` exact message 与 opening-brace span；line soft behavior 遇 block 仍报 `MORVA1014`；未知项仍报 `MORVA1007`。
- `Action::rebase_spans` 必须 rebase 每个 soft span，否则第二个项目文件的 local offset 会被当作 virtual offset，造成误映射或 panic。
- 无换行块注释不得切分 soft/hint 跳过区中的 identifier、Integer 或复合 operator；现有 `MORVA1025` 必须保持优先失败。

### Ordering and Rendering Contract

- Document notices 依据 `(span.start, span.end)` 排序；merged findings 依据 `(start, end, severity)` 排序，同 span 时 error 先于 notice。
- Project merged findings 先按 project-level/source bucket，再按 `SourceId`、local start/end 和 severity 排序。既有 `Project::analyze()` 已对任意 `Notice` 做通用回映，通常无需修改生产 `project.rs`。
- CLI 已只区分 `AnalysisFinding::Error/Notice` 并通用渲染 notice code/message/span，通常无需修改生产 `main.rs`。若为 `MORVA5002` 新增 CLI 语义分支，即违反本 Story 的复用要求。

### Example and Regression Impact

- `examples/order.morva` 已含 `module Orders`、`idempotent` 和 `implementation_hint`；Story 2.2 后 `check` 应有 1 个 `MORVA5001` + 2 个 `MORVA5002`。
- `examples/order-project/20-actions.morva` 含同样两个 soft behavior；项目 `check` 应有 2 个 `MORVA5002` 且都映射到 `20-actions.morva`。
- `parse`/`inspect`/`simulate` 在 Story 2.2 不新增 warning 呈现；两套示例的七阶段、最终状态和退出码不变。

### Scope Boundaries

- 不验证或执行 atomic/idempotent/timeout/retry/implementation_hint；不建模其参数/路径/body；不读文件、环境、时钟、网络或用户代码。
- 不扩大 soft behavior 白名单；不修改 lexer、`Diagnostic`、semantic/simulator 规则、七阶段、值域、目录发现、模块作用域或全局短名解析。
- 不给 `parse` 文本输出回显 soft behavior，不扩展 `inspect`（Story 2.3），不实现 capability inventory（Story 2.4）、JSON、MCP 或 Boolean 表达式。
- 不实现 parser recovery/partial AST，不搜索原文伪造 warning，不因 warning 改变成功语义。

### Project Structure Notes

- 预计生产更新：`crates/morva-core/src/ast.rs`、`crates/morva-core/src/parser.rs`、`crates/morva-core/src/analysis.rs`。
- 通常不应修改生产 `semantic.rs`、`simulate.rs`、`project.rs`、`lib.rs` 或 CLI `main.rs`；若必须修改，先证明通用管线为何不足。
- 预计测试更新：`crates/morva-core/tests/language.rs`、`crates/morva-core/tests/project.rs`、`crates/morva-core/tests/simulation.rs`、`crates/morva-cli/tests/cli.rs`。
- 预计文档更新：`README.md`、`docs/requirements.md`、`docs/language-reference.md`、`docs/language-design.md`、`docs/implementation-status.md`、`docs/testing-strategy.md`、`docs/architecture.md`、`docs/cli.md`、`docs/project-context.md`，以及新的 Story 2.2 冻结规格。
- 当前 worktree 包含 Story 2.1 和用户的其他未提交改动。只做增量编辑，不重置、不覆盖、不把无关文件纳入 Story 2.2 File List。

### Testing Requirements

- 使用公开 `parse/analyze/check`、Project seam、真实 CLI process 和 simulator；不只测私有 helper。
- 新断言尽量锁定 exact code/message/category/action/kind/keyword span、warning/error 顺序、source/local span、stdout/stderr 和退出码。
- 对现有注释/换行/超长行/路径安全契约可复用已有矩阵，但至少要有一条真实 soft warning 路径直接证明共享 renderer 仍安全。
- 示例闭环串行运行，避免 CLI 临时夹具竞争。

### Previous Story Intelligence

- Story 2.1 的正式 review 发现并修正了三类高风险遗漏：Report 缺统一 merged view、CLI 复制 renderer、测试未覆盖嵌套/同 local offset/路径安全。Story 2.2 必须直接复用修正后的 API，不得退化。
- `MORVA5001` 的教训是：code/message/span 必须在实现前冻结，并由 exact 测试锁定；Story 文档不能出现与实现不同的 code。
- 出现 semantic error 时 analysis 仍必须完整收集 notice；只有 lexer/parser 失败时因无 AST 而不产生 notice。

### Git Intelligence

- 最近交付保持“独立冻结规格 + core 公开 seam + 真实 CLI 测试 + 文档同步”的单一可验证意图。
- 注释与多文件提交证明：byte span、newline、token-split 与 Project rebase 都是语言改动必须主动锁定的边界。
- 本 Story 无第三方库、外部 API 或网络协议；不需要最新技术调研。

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story-2.2-识别未执行的-action-soft-behavior]
- [Source: _bmad-output/implementation-artifacts/2-1-识别已解析但未验证的兼容容器.md#Review-Findings]
- [Source: docs/requirements.md#FR-03-解析兼容边界]
- [Source: docs/requirements.md#FR-05-诊断]
- [Source: docs/requirements.md#FR-07-最小模型级模拟]
- [Source: docs/requirements.md#FR-08-实现提示]
- [Source: docs/architecture.md#2-Workspace-组成]
- [Source: docs/architecture.md#3-核心数据模型]
- [Source: docs/architecture.md#6-错误边界]
- [Source: docs/testing-strategy.md#3-新语义的最低测试矩阵]
- [Source: docs/project-context.md#Critical-Implementation-Rules]
- [Source: crates/morva-core/src/ast.rs]
- [Source: crates/morva-core/src/parser.rs]
- [Source: crates/morva-core/src/analysis.rs]
- [Source: crates/morva-core/src/project.rs]
- [Source: crates/morva-cli/src/main.rs]

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- RED: public language test failed because `SoftBehaviorKind` and `Action.soft_behaviors` did not exist.
- RED: structured notice test failed because `NoticeKind::ActionSoftBehavior` did not exist.
- GREEN/REFACTOR: focused core, project, simulation, and real CLI tests passed after the minimal AST/parser/analysis changes; no Project, CLI, semantic, or simulator production branch was needed.
- Final local gates: `cargo fmt --all --check`, locked strict Clippy, `cargo test --workspace --locked` (151 tests), `git diff --check`, and eight single/project example commands all passed.

### Implementation Plan

- Freeze `MORVA5002` and the two approved public Rust source-compatibility exceptions before production changes.
- Retain only soft behavior kind and keyword span in the AST, then emit notices through the existing additive analysis path.
- Prove Project mapping, CLI rendering, parser-error parity, and simulation neutrality through public integration seams.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Added five-kind `SoftBehaviorKind` provenance and checked project-span rebasing without retaining payload or implementation-hint bodies.
- Added one structured, non-fatal `MORVA5002` per parsed source item with stable action/kind/message/keyword span and deterministic merged ordering.
- Reused the existing Project/CLI notice pipeline unchanged; warning-only stays exit 0 with `ok:`, mixed errors stay exit 1, and parser failures produce no partial warning.
- Verified soft behaviors do not change seven-phase simulation reports or parse/inspect/simulate CLI output.
- Synchronized public documentation and explicit migration guidance; recorded only local verification evidence.

### File List

- README.md
- _bmad-output/implementation-artifacts/2-2-识别未执行的-action-soft-behavior.md
- _bmad-output/implementation-artifacts/spec-action-soft-behavior-warnings.md
- _bmad-output/implementation-artifacts/sprint-status.yaml
- crates/morva-core/src/analysis.rs
- crates/morva-core/src/ast.rs
- crates/morva-core/src/parser.rs
- crates/morva-core/tests/language.rs
- crates/morva-core/tests/project.rs
- crates/morva-core/tests/simulation.rs
- crates/morva-cli/tests/cli.rs
- docs/architecture.md
- docs/cli.md
- docs/implementation-status.md
- docs/language-design.md
- docs/language-reference.md
- docs/project-context.md
- docs/requirements.md
- docs/testing-strategy.md

### Change Log

- 2026-08-11: Created comprehensive Story 2.2 context and marked it ready for development.
- 2026-08-11: Implemented structured action soft-behavior provenance and `MORVA5002` warnings; added cross-layer regressions, synchronized documentation, and marked the story ready for review.

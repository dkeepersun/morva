---
title: 'Readable Morva Source Comments'
type: 'feature'
created: '2026-08-10'
status: 'done'
review_loop_iteration: 0
baseline_commit: '6f6e862f6f70f0111ed13c27995a5f6fe8e45cb4'
context:
  - '{project-root}/docs/project-context.md'
  - '{project-root}/docs/requirements.md'
  - '{project-root}/docs/language-reference.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Morva 已支持 `//` 行注释，但较长说明、临时分段和跨行设计理由仍难以清晰表达，降低人工编写和评审 `.morva` 模型的可读性。

**Approach:** 保留现有行注释，并新增 Rust 风格、可嵌套的 `/* ... */` 块注释；lexer 忽略注释正文，但保留其中逻辑换行的分隔作用与原始 byte span。

## Boundaries & Constraints

**Always:** `//` 在 LF、CRLF、CR 或 EOF 前结束且行为不变；`/* ... */` 可出现在 token 之间、跨行并嵌套；块注释中的 LF、CRLF、CR 仍各产生一个逻辑换行，CRLF 不双计；注释不进入 AST、checker、inspect 或 simulator；未闭合块注释以稳定诊断指向最外层 opening delimiter；单文件和多文件项目行为一致。

**Ask First:** 改变注释定界符、允许注释切开一个 token、保留注释到 AST、生成文档注释、改变 parser 的换行规则或诊断 code/message 契约。

**Never:** 不增加字符串、文档生成、formatter、宏/条件注释、注释语义、第三方 lexer 依赖；不把注释文本执行或传给模拟器；不规范化源码后再计算 span。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| 行注释 | `// text` 后接任一换行或 EOF | 忽略正文，换行仍分隔语法 | 无 |
| 块注释 | token 间、独占行或跨多行的 `/* text */` | 忽略正文；外部模型与无注释版本等价 | 无 |
| 嵌套块 | `/* outer /* inner */ outer */` | 深度归零后继续 lex | 无 |
| 定界符文本 | 块内出现 `//`，行注释内出现 `/*` | 由当前注释模式解释，不错误切换 | 无 |
| 未闭合块 | EOF 前没有匹配 `*/` | parse/check 失败，无 panic | `MORVA1024`，span 为最外层 `/*` 两个 byte |

</frozen-after-approval>

## Code Map

- `crates/morva-core/src/lexer.rs` -- 注释识别、嵌套深度、换行 token 与词法诊断。
- `crates/morva-core/tests/language.rs` -- 公开 parse/check seam 的注释、span 和错误回归。
- `crates/morva-cli/tests/cli.rs` -- 真实 CLI 的块注释成功与未闭合诊断渲染。
- `docs/requirements.md`, `docs/language-reference.md` -- 长期词法契约和用户语法。
- `docs/implementation-status.md`, `docs/testing-strategy.md` -- 当前能力、限制与最低证据。

## Tasks & Acceptance

**Execution:**
- [x] `crates/morva-core/tests/language.rs` -- 先添加行/块/嵌套/混合换行/模式隔离/未闭合的失败测试。
- [x] `crates/morva-core/src/lexer.rs` -- 最小实现嵌套块注释，保留块内逻辑 Newline token，并产生 `MORVA1024`。
- [x] `crates/morva-cli/tests/cli.rs` -- 通过进程 seam 锁定成功、退出码 1、code/message/行列/excerpt/marker。
- [x] `docs/requirements.md`, `docs/language-reference.md`, `docs/implementation-status.md`, `docs/testing-strategy.md`, `README.md` -- 同步可依赖能力和示例。
- [x] workspace -- 运行格式、严格 Clippy、全测、单/多文件四命令和 diff check。

**Acceptance Criteria:**
- Given 含行注释、跨行块注释和嵌套块注释的有效单/多文件模型，when parse/check/inspect/simulate，then 注释不改变 AST 语义、引用解析或七阶段结果。
- Given 块注释跨越 LF、CRLF、CR 或混合换行，when 后续语法失败，then 诊断行列和本地 byte span 与原始源码一致。
- Given 未闭合的嵌套块注释，when parse/check，then 只返回稳定 `MORVA1024`，精确标记最外层 opening delimiter 且不 panic。

## Spec Change Log

- 2026-08-10: 按 public parse/check 与真实 CLI seam 完成 red→green；新增 6 项 core 注释测试与 2 项 CLI 进程测试，实现可嵌套块注释、块内通用 Newline 及 `MORVA1024`，同步单/多文件示例和长期文档，状态推进至 `in-review`。
- 2026-08-10 review patch: 全局增加 `MORVA1025` token-split guard，覆盖 typed、compatibility、soft/hint、连续 comment run 与含换行/不合并反例；显式锁定 `//` EOF、outer+inner 同时未闭合、后排序项目本地 `MORVA1024`，并把 CLI 四命令验证加强为 parse/inspect 等价输出及七阶段最终状态断言。
- 2026-08-10 review patch 2: 将 `MORVA1025` 扩展到现有 `== != >= <= += -=` 复合 operator；typed、parser-skipped、连续无换行 comment run 均锁定精确 code/message/opener span，含内部换行时保留为语法分隔而不报 token split。

## Design Notes

块注释扫描仍在 lexer 单次前向遍历中完成。扫描期间只为真实换行写入 `Newline` token；普通注释 byte 不分配 token，因此复杂度保持 O(source bytes)，AST 和公开 API 无变化。

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- workspace 格式正确。
- `cargo clippy --workspace --all-targets --locked -- -D warnings` -- 无 warning。
- `cargo test --workspace --locked` -- 全部既有和新增测试通过。
- `cargo run --locked -p morva-cli -- check|parse|inspect|simulate <commented-file-or-directory> ...` -- 单/多文件注释模型闭环通过。
- `git diff --check` -- 无空白错误。

**Result (2026-08-10 review patch 2):** 两轮 review patch 的定向 red→green 测试均通过；fmt、locked strict Clippy、全部 workspace test suites、带注释单/多文件 check/parse/inspect/simulate 及 `git diff --check` 以本次实际命令输出为准并全部通过。

## Suggested Review Order

**Lexing and token boundaries**

- 入口统一识别块注释、换行及 token-split 诊断。
  [`lexer.rs:44`](../../crates/morva-core/src/lexer.rs#L44)

- 单次前向扫描处理嵌套、连续注释和原始换行 span。
  [`lexer.rs:166`](../../crates/morva-core/src/lexer.rs#L166)

**Language contract**

- 长期需求明确空白等价、稳定诊断和 parser-skipped 边界。
  [`requirements.md:62`](../../docs/requirements.md#L62)

- 用户参考展示行注释、嵌套块注释与禁止切分规则。
  [`language-reference.md:10`](../../docs/language-reference.md#L10)

**Behavioral evidence**

- Core seam 锁定普通、嵌套、换行、EOF 与原始 byte span。
  [`language.rs:135`](../../crates/morva-core/tests/language.rs#L135)

- 对 parser-skipped 内容和连续注释执行全局 token 边界验证。
  [`language.rs:227`](../../crates/morva-core/tests/language.rs#L227)

- 六种复合操作符不能被无换行块注释拆开。
  [`language.rs:250`](../../crates/morva-core/tests/language.rs#L250)

- CLI 四命令比较等价输出并验证完整七阶段状态。
  [`cli.rs:883`](../../crates/morva-cli/tests/cli.rs#L883)

- 多文件词法错误回映到责任文件的本地 marker。
  [`cli.rs:973`](../../crates/morva-cli/tests/cli.rs#L973)

- 测试策略保存后续语法演进必须维持的注释矩阵。
  [`testing-strategy.md:50`](../../docs/testing-strategy.md#L50)

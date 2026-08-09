---
title: '统一 LF、CRLF 与 CR 换行契约'
type: 'feature'
created: '2026-08-10'
status: 'done'
baseline_commit: 'e8a95a999acf771666050d3f83d0bb0d8fe21c45'
context:
  - '{project-root}/docs/project-context.md'
  - '{project-root}/docs/requirements.md'
  - '{project-root}/docs/language-reference.md'
  - '{project-root}/docs/cli.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 词法器只把 LF 视为换行；CRLF 虽可解析，CR 被当作普通空白，导致 CR-only 文件的行分隔、`//` 注释边界和 CLI 位置错误。

**Approach:** 定义 LF、CRLF 和 CR 均为一个逻辑换行，并让 lexer、注释、parser 分隔与 CLI 行列/摘要使用同一契约。

## Boundaries & Constraints

**Always:** LF 和 CR 各生成一个 Newline token，CRLF 消费两个 byte 但只生成一个 token；token/span 仍是原源 UTF-8 byte range；`//` 在任一支持的换行前停止；混合换行的逻辑行列、excerpt 与 parser 一致；既有纯 LF 行为不变。

**Ask First:** 改变诊断 code/message、tab/Unicode column 语义、支持非 UTF-8 source/path，或引入源文本规范化层。

**Never:** 不改 AST、语义类型、simulator、CLI 退出码、诊断号段、语法表达式能力或机器可读协议；不新增第三方依赖。

## I/O & Edge-Case Matrix

| Scenario | Expected behavior |
|---|---|
| 等价 LF/CRLF/CR 模型 | parse/check 结果等价，各自 span 精确覆盖原源 byte |
| `// comment` + 任一换行 | 注释在换行前结束，下一行声明可见 |
| 混合 LF/CRLF/CR | 每个序列计一行，CRLF 不重复计数 |
| 换行处或其后 EOF 错误 | 物理行列、excerpt 和至少一个 caret 稳定，不 panic |

</frozen-after-approval>

## Code Map

- `crates/morva-core/src/lexer.rs` -- 唯一换行 token 与注释终止边界。
- `crates/morva-core/tests/language.rs` -- 通过公开 parse/check seam 验证语言契约。
- `crates/morva-cli/src/main.rs` -- 使 CLI 逻辑行列和 excerpt 与 lexer 换行序列一致。
- `crates/morva-cli/tests/cli.rs` -- 通过真实进程锁定混合换行的位置与渲染。
- `docs/language-reference.md`, `docs/cli.md`, `docs/requirements.md`, `docs/testing-strategy.md`, `docs/implementation-status.md` -- 同步已承诺契约与验收策略。

## Tasks & Acceptance

**Execution:**
- [x] `lexer.rs` -- 先以公开测试锁定 LF/CRLF/CR token 与注释边界，再实现单次消费。
- [x] `main.rs` -- 统一识别三种换行，保持有界窗口、路径安全和逻辑 column 契约。
- [x] `language.rs`, `cli.rs` -- 覆盖等价模型、注释、混合换行、CRLF 单计数和 EOF 诊断。
- [x] `docs/*.md`, `deferred-work.md` -- 将 CR-only 从延期项转为已实现契约。
- [x] workspace -- 通过格式、严格 Clippy、全部测试、四命令示例与差异检查。

**Acceptance Criteria:**
- Given 只替换换行 byte 的等价模型，when parse/check，then AST 结构与诊断意义等价。
- Given 任一换行或混合序列，when CLI 渲染错误，then line/column/excerpt 与逻辑物理行一致。
- Given 已有 LF 示例和回归，when 执行全闭环，then AST、诊断、模拟七阶段和退出码不变。

## Spec Change Log

- 2026-08-10：按公开 core parse/check 与真实 CLI process seams 完成 red→green。Lexer 为 CR、LF 各生成一个 Newline，CRLF 消费两个原源 byte 但只生成一个 token；`//` 在 CR 或 LF 前停止。CLI 对 LF/CRLF/CR 和混合序列统一计算逻辑行列并省略 terminator，同时保留 160 宽窗口。新增 4 个 core language 与 2 个 CLI 回归，既有 EOF 孤立 CR 呈现测试按批准的新契约更新；byte span、纯 LF 行为、诊断 code/message、退出码和模拟器均保持不变。

## Design Notes

实现应保留原源 byte span，而不是在入口把 CRLF/CR 重写成 LF；否则诊断 span 将与用户文件偏移失配。

## Verification

**Commands:**
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p morva-cli -- check examples/order.morva`
- `cargo run -p morva-cli -- parse examples/order.morva`
- `cargo run -p morva-cli -- inspect examples/order.morva`
- `cargo run -p morva-cli -- simulate examples/order.morva NormalConfirmation`
- `git diff --check`

2026-08-10 最终验证：formatting、严格 Clippy、91 个自动化测试、四命令示例闭环与 `git diff --check` 全部通过；盲审、边界审查和验收审计均 PASS，规格收口为 `done`。

## Suggested Review Order

**语言换行契约**

- 从词法入口理解三种换行的单 token 消费。
  [`lexer.rs:19`](../../crates/morva-core/src/lexer.rs#L19)

- 统一 CLI 的逻辑行列计算，保留原源 byte offset。
  [`main.rs:167`](../../crates/morva-cli/src/main.rs#L167)

- 让有界 excerpt 在 CR、LF 或 CRLF 前稳定结束。
  [`main.rs:195`](../../crates/morva-cli/src/main.rs#L195)

**公开行为回归**

- 锁定 CR-only 分隔与原源 span。
  [`language.rs:85`](../../crates/morva-core/tests/language.rs#L85)

- 验证注释在每种换行前停止。
  [`language.rs:112`](../../crates/morva-core/tests/language.rs#L112)

- 比较等价模型结构与不同原源 span。
  [`language.rs:135`](../../crates/morva-core/tests/language.rs#L135)

- 锁定混合换行下的 CLI 行列和 excerpt。
  [`cli.rs:330`](../../crates/morva-cli/tests/cli.rs#L330)

- 确认三种换行后 EOF 诊断完全等价。
  [`cli.rs:348`](../../crates/morva-cli/tests/cli.rs#L348)

**长期契约**

- 将通用换行提升为产品需求。
  [`requirements.md:59`](../../docs/requirements.md#L59)

- 记录混合换行的必测公开 seams。
  [`testing-strategy.md:43`](../../docs/testing-strategy.md#L43)

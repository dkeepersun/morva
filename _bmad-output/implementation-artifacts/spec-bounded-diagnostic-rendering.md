---
title: '有界诊断窗口与安全路径输出'
type: 'feature'
created: '2026-08-10'
status: 'done'
baseline_commit: '9ba684c6b8f8303e1ddf8213cc6cfcf814dd090a'
context:
  - '{project-root}/docs/project-context.md'
  - '{project-root}/docs/requirements.md'
  - '{project-root}/docs/cli.md'
  - '{project-root}/_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** CLI 当前为每条诊断渲染完整源码行和完整 marker；100KB 单行会产生约 200KB stderr，tab/escaped byte 还能放大。读取失败与成功输出也直接插入路径控制字符，CRLF 的 `\r` 会出现在 excerpt 中。

**Approach:** 让普通诊断与模拟失败共享一个有界的单行视图，以渲染后宽度限制源码窗口和 marker；所有用户可见路径统一安全转义，同时保持诊断 code/message、真实 line/column、span、退出码和短行输出契约。

## Boundaries & Constraints

**Always:** 每条 excerpt 和 marker 的内容宽度分别不超过 160；窗口包含 span.start、至少一个 caret，并用 `...` 表示左右裁剪；不得切断 tab 或 `\xNN` fragment；tab 仍显示四空格但逻辑 column 计一，控制/non-ASCII byte 继续转义；跨行 span 只标 start 所在行可见交集；CRLF 终止符不进入 excerpt；成功、读取错误、语言诊断和模拟失败都使用同一 UTF-8 path 安全格式；短 LF fixture 的必要文本保持不变。

**Ask First:** 截断 Diagnostic.message、改变 line/column/tab 语义、改变退出码、支持非 UTF-8 argv/source，或改变 CR-only lexer 行为。

**Never:** 不修改 core Diagnostic/Span、lexer/parser/AST/语义/simulator；不增加颜色、TTY 探测、配置、第三方依赖、机器协议、全文件大小限制或总 stderr 硬上限；本规格不定义 CR-only 语言契约。

## I/O & Edge-Case Matrix

| Scenario | Expected behavior |
|---|---|
| 100KB line; error at start/middle/end/EOF | excerpt/marker ≤160，真实位置不变，按需显示左右省略号 |
| Long or multiline span | 只标 start 行可见部分，至少一个 caret，无巨量空格/caret |
| Tab/control/non-ASCII near window | escape fragment 完整，caret 对齐，无原始控制 byte |
| CRLF source | 行列不变，excerpt 不显示 `\x0D` |
| Long-line simulation failure | 与普通 diagnostic 使用同一窗口规则 |
| UTF-8 path with newline/tab/ESC | success/read error/model/runtime 输出均不含原始控制字符 |

</frozen-after-approval>

## Code Map

- `crates/morva-cli/src/main.rs` -- 共用 line view、窗口、marker 和 safe path 输出。
- `crates/morva-cli/tests/cli.rs` -- 唯一公开 seam 的进程级资源与转义回归。
- `docs/cli.md`, `docs/requirements.md`, `docs/implementation-status.md`, `docs/testing-strategy.md` -- 输出边界与剩余非目标。

## Tasks & Acceptance

**Execution:**
- [x] `main.rs` -- 合并普通/模拟诊断视图，按 escaped fragment 计算 160 宽度窗口，安全处理 EOF/空行/超长或跨行 span。
- [x] `main.rs` -- 所有路径输出复用控制字符转义，不改变 UTF-8 正常路径文本。
- [x] `cli.rs` -- 先以公开 CLI process seam 覆盖长行四位置、长 span、tab/control/non-ASCII、CRLF、模拟失败和路径注入。
- [x] `docs/*.md` -- 记录 excerpt/marker 上限、逻辑 column 与显示宽度分离，以及 CR-only 仍延期。
- [x] workspace -- 通过格式、严格 Clippy、全部测试与四命令示例闭环。

**Acceptance Criteria:**
- Given 任意长度的单行及合法 core span，when 渲染诊断，then excerpt/marker 各不超过 160 且 marker 指向可见错误起点。
- Given 短 LF fixture，when check/simulate，then既有 code/message/路径/line/column/必要 excerpt 不变。
- Given CRLF、escaped bytes 或 UTF-8 路径控制字符，when 任一 CLI 结果输出，then 无原始控制字符污染终端。
- Given 超长行模拟失败，when render，then与静态诊断遵循同一窗口和 caret 契约。

## Spec Change Log

- 2026-08-10：按真实 CLI 进程 seam 完成逐行为 red→green；普通诊断与模拟失败共享 `SourceView`，源码窗口和 marker 各自限制为 160 个渲染字符，CRLF terminator 不回显，四类 CLI 结果统一安全转义 UTF-8 路径控制字符。新增 10 个 CLI 回归测试；保持 core、lexer、parser、simulator、诊断 code/message、line/column、span 与退出码不变。
- 2026-08-10 review：移除整行宽度 `Vec` 和 marker 后全行扫描；窗口仅即时计算左 72 与右侧有界 fragment，辅助内存为 O(window)，并用 checked/saturating 算术保护宽度不变量。CR 只在已确认的 CRLF terminator 中剔除；测试 helper 绑定同一诊断块，锁定 marker 对齐、159/160/161 与 72/73 阈值、EOF 孤立 CR、CR/DEL 路径和 `simulate` 读取失败。CLI 进程测试增至 21 个。KEEP：规格保持 `in-review`，CR-only lexer 语义仍延期。
- 2026-08-10 review follow-up：`location` 不再从 marker 搜索物理行尾；`SourceView` 把当前行起始后的借用 slice 交给 renderer，由 renderer 在右侧窗口预算内识别 LF/CRLF 或判定省略号。超长行起点 CLI 回归加入下一行 sentinel，证明窗口不跨行；复杂度由结构审查确认，不使用脆弱计时断言。KEEP：line/column、CRLF 与 EOF 孤立 CR 行为及公开 API 不变。

## Design Notes

窗口单位是 renderer 产生的 ASCII fragment 宽度，不引入 Unicode terminal-width 模型。160 上限只约束源码 excerpt 与 marker；message 和路径采用安全转义但不截断，因此不宣称每条 stderr 总字节硬上限。

## Verification

**Commands:**
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p morva-cli -- check examples/order.morva`
- `cargo run -p morva-cli -- parse examples/order.morva`
- `cargo run -p morva-cli -- inspect examples/order.morva`
- `cargo run -p morva-cli -- simulate examples/order.morva NormalConfirmation`

2026-08-10 初始验证：formatting、严格 Clippy、82 个自动化测试及四命令示例闭环全部通过；`git diff --check` 无空白错误。

2026-08-10 review 最终验证：formatting、严格 Clippy、85 个自动化测试及四命令示例闭环全部通过；`git diff --check` 无空白错误。验收与边界定向复审均 PASS，规格收口为 `done`。

## Suggested Review Order

**共用诊断视图**

- 从入口理解普通诊断与模拟失败如何共用视图。
  [`main.rs:139`](../../crates/morva-cli/src/main.rs#L139)

- 以有界扫描构建不切断转义片段的 160 宽窗口。
  [`main.rs:187`](../../crates/morva-cli/src/main.rs#L187)

- 统一转义所有 CLI 结果中的 UTF-8 路径控制字符。
  [`main.rs:155`](../../crates/morva-cli/src/main.rs#L155)

**公开行为回归**

- 锁定超长行起点的有界输出与不跨行行为。
  [`cli.rs:127`](../../crates/morva-cli/tests/cli.rs#L127)

- 锁定 159/160/161 宽度阈值与裁剪边界。
  [`cli.rs:147`](../../crates/morva-cli/tests/cli.rs#L147)

- 确认模拟失败遵循同一窗口和 caret 契约。
  [`cli.rs:346`](../../crates/morva-cli/tests/cli.rs#L346)

- 覆盖成功、读取、模型与运行时路径输出。
  [`cli.rs:373`](../../crates/morva-cli/tests/cli.rs#L373)

**契约与后续边界**

- 将窗口上限提升为可验收的 CLI 需求。
  [`requirements.md:89`](../../docs/requirements.md#L89)

- 保留 CR-only 为独立语言契约增量。
  [`deferred-work.md:7`](deferred-work.md#L7)

2026-08-10 review follow-up 验证：移除 line-tail 搜索后的当前工作树通过 formatting、严格 Clippy、85 个自动化测试与 `git diff --check`；规格已收口为 `done`。

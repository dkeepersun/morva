---
title: '多文件 Morva 项目装配'
type: 'feature'
created: '2026-08-10'
status: 'done'
baseline_commit: '2e6f0432431839888dffa652df6ee99e313f0af4'
context:
  - '{project-root}/docs/project-context.md'
  - '{project-root}/docs/requirements.md'
  - '{project-root}/docs/architecture.md'
  - '{project-root}/docs/cli.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Morva 模型当前只能放在单文件中，实体、枚举、动作和场景无法按关注点独立维护。

**Approach:** 四个 CLI 命令接受单文件或目录；目录内文件保留同名 `system` 外壳，由 core 装配后统一检查、查看和模拟，位置映回原文件。

## Boundaries & Constraints

**Always:** 单文件完全兼容；目录只读直接子级的小写 `.morva` 普通文件，忽略其他文件、子目录和 symlink，按 UTF-8 文件名字节序排序；每个文件可独立解析、恰有一个同名顶层 `system`；只合并根 system 子声明并沿用现有全局语义；诊断和模拟失败显示真实文件、本地 span/行列和安全路径；全部读取成功后才输出模型结果。

**Ask First:** 递归目录、文件列表/manifest、省略 system 外壳、改变选择/排序、模块作用域/限定名、改变 `Span`/`Diagnostic` 公共形状或访问目录外内容。

**Never:** 不增加 import/include、包/循环依赖、container reopening、跨项目依赖、增量/watch/LSP、并行编译或第三方依赖；不拼接原始文本；CLI 不复制语义。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| 单文件 | 现有文件/无后缀文件 | 四命令与输出不变 | 现有 0/1/2 |
| 项目 | 多个同名 system，声明互引 | 稳定顺序合并，四命令使用完整项目 | 成功 0 |
| 模型错误 | 异名/缺失/多个 system、重复名、未知引用 | 指向责任文件与本地行列 | 1 |
| 输入错误 | 不存在、无候选、非 UTF-8、发现/读取失败 | 无部分 stdout，路径安全 | 2 |
| 过滤 | 非 `.morva`、子目录、symlink | 忽略并保持重复运行一致 | 最终为空则 2 |

</frozen-after-approval>

## Code Map

- `crates/morva-core/src/project.rs`, `ast.rs`, `lib.rs` -- 装配、span 回映射与 additive API。
- `crates/morva-core/tests/project.rs` -- 跨文件结构、引用、模拟、映射和 parity。
- `crates/morva-cli/src/main.rs`, `tests/cli.rs` -- 目录 preflight、执行、渲染及真实进程回归。
- `README.md`, `docs/*.md` -- 项目约定、CLI、架构、需求、状态和非目标。

## Tasks & Acceptance

**Execution:**
- [x] `project.rs`, `ast.rs`, `lib.rs` -- TDD 实现同名 system 装配、checked rebasing 和来源回映射。
- [x] `tests/project.rs` -- 覆盖结构/引用/重复名、换行 span、AST roundtrip、模拟失败和旧 API parity。
- [x] `main.rs`, `tests/cli.rs` -- 覆盖排序/过滤/UTF-8/IO、安全路径、四命令和退出码。
- [x] `README.md`, `docs/*.md` -- 同步长期契约、使用方式、状态与非目标。
- [x] workspace -- 通过格式、严格 Clippy、全测、单/多文件闭环和 diff check。

**Acceptance Criteria:**
- Given enum/entity、action、scenario 分布在同名 system 文件，when 执行四命令，then 跨文件引用通过、输出确定、七阶段成功。
- Given 后排序文件出现语法/语义/运行期失败，when 渲染，then code/message 指向正确文件、本地行列和 marker。
- Given 旧示例与 core API，when 回归，then AST、诊断、模拟和 CLI 行为不变。
- Given 项目发现/读取失败，when 执行，then 返回 2、无部分 stdout 且路径安全。

## Spec Change Log

- 2026-08-10: 实现 core `Project`/`SourceMap` additive API、完整 AST checked span rebase、CLI file-or-directory preflight、单/多文件示例及 CI 门禁；状态推进至 `in-review`。
- 2026-08-10 review fixes: `ProjectDiagnostic` 明确区分 project-level 与 source-local；`locate_virtual_span` 拒绝反向/跨 base span；精确 system shell 责任 span；完整嵌套 AST roundtrip 与 CRLF runtime span；UTF-8 文件名、句柄身份/canonical-root 校验、RAII 测试夹具及精确 CLI marker。macOS workspace 118 项测试通过；Linux 额外执行非 UTF-8 候选文件名回归，共 119 项。

## Design Notes

Core 保留本地 AST，并构造私有 merged Document：完整 AST span 加互不重叠且 checked 的 virtual base，复用既有 checker/simulator，再由 `SourceMap::locate_virtual_span` 恢复 `SourceId + LocalSourceSpan`。`ProjectDiagnostic::Source.local_diagnostic` 已是本地 span，不再回映射。这保持公开 Span/Diagnostic 兼容并避免跨文件 offset 碰撞；根 system 外的 container 不合并。

CLI 在纯标准库范围内以 symlink metadata、canonical root、打开句柄及前后身份复核绑定读取；Unix 使用 device/inode。跨平台原子 `nofollow` 和并发原地写入的快照隔离无法由当前 std-only 边界保证，作为明确残余而非隐藏承诺。

## Verification

**Commands:**
- `cargo fmt --all -- --check` -- workspace 格式正确。
- `cargo clippy --workspace --all-targets --locked -- -D warnings` -- 无 warning。
- `cargo test --workspace --locked` -- 全部现有与新增测试通过。
- `cargo run --locked -p morva-cli -- check|parse|inspect|simulate <file-or-directory> ...` -- 单文件与多文件闭环均满足契约。
- `git diff --check` -- 无空白错误。

**Result (2026-08-10):** 全部命令通过；本机 `cargo test --workspace --locked` 为 118 passed / 0 failed（Linux CI 额外包含 1 项非 UTF-8 文件名测试）。`examples/order.morva` 与 `examples/order-project` 均完成 check/parse/inspect/simulate 闭环；`git diff --check` 通过。

## Suggested Review Order

**Project assembly and provenance**

1. [`Project::parse`](../../crates/morva-core/src/project.rs#L90) — 了解多源解析、同名 system 校验与合并入口。
2. [`ProjectDiagnostic`](../../crates/morva-core/src/project.rs#L14) — 核对项目级与来源级错误的显式边界。
3. [`SourceMap::locate_virtual_span`](../../crates/morva-core/src/project.rs#L63) — 检查 virtual span 的安全回映射。
4. [`RebaseSpans`](../../crates/morva-core/src/ast.rs#L23) — 审阅完整 AST span rebasing 覆盖。

**CLI discovery and safe loading**

5. [`load_checked_model`](../../crates/morva-cli/src/main.rs#L168) — 查看单文件与目录输入的统一执行管线。
6. [`discover_project_sources`](../../crates/morva-cli/src/main.rs#L225) — 核对筛选、排序和 symlink 边界。
7. [`read_project_source`](../../crates/morva-cli/src/main.rs#L296) — 检查 canonical-root 与打开句柄身份复核。
8. [`render_project_diagnostics`](../../crates/morva-cli/src/main.rs#L390) — 确认诊断选择正确来源文件。

**Behavioral evidence**

9. [`project.rs` integration tests](../../crates/morva-core/tests/project.rs#L33) — 跨文件引用、七阶段和本地 span 证据。
10. [`cli.rs` project tests](../../crates/morva-cli/tests/cli.rs#L559) — 真实进程 seam 的四命令、I/O 与渲染证据。
11. [`requirements.md`](../../docs/requirements.md#L58) — 对照已批准的多文件语言项目契约。
12. [`deferred-work.md`](./deferred-work.md#L3) — 查看并发文件系统威胁模型的明确延期项。

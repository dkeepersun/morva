---
title: '收口当前状态与历史验证证据'
type: 'chore'
created: '2026-08-10'
status: 'done'
baseline_commit: '0023bc8fd9ae74b2a5b8312b2bccc278a0ca424c'
context:
  - '{project-root}/docs/project-context.md'
  - '{project-root}/docs/requirements.md'
  - '{project-root}/docs/ai-handoff.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 连续实施后，当前状态文档仍指向 `ae90f8d`、50 个测试和“诊断边界待做”，文档索引也未列入最近完成规格；历史扫描报告容易被误读为当前事实。

**Approach:** 明确区分“当前状态”与“指定提交的历史验证快照”，只更新可变的导航、交接顺序和候选路线，不改写既有冻结意图或历史验证记录。

## Boundaries & Constraints

**Always:** 当前状态以最新已提交实现、公开测试和 CI 为证据；扫描报告必须显式标注 source commit 和历史快照属性；AI 先读强制边界，再用任务相关源码/测试建立当前心智模型；链接使用仓库相对路径。

**Ask First:** 改变产品路线优先级、删除历史产物、改写冻结块，或把候选能力提升为承诺。

**Never:** 不修改 Rust 行为或测试；不把历史规格中的测试计数追溯改成当前数量；不声称远程 CI 已运行或 branch protection 已配置。

## I/O & Edge-Case Matrix

| Scenario | Expected behavior |
|---|---|
| 阅读当前实现状态 | 看到最近类型、诊断、换行和 CI 能力，不再建议已完成工作 |
| 阅读扫描报告 | 立即知道数量与命令结果只对 `source_commit` 有效 |
| AI 接手新任务 | 在说明性文档之前用相关源码、测试和示例核验行为 |
| 查看已完成规格 | 索引与实现状态列出所有现行已完成增量 |

</frozen-after-approval>

## Code Map

- `docs/implementation-status.md` -- 当前能力、已完成规格和下一候选。
- `docs/roadmap.md` -- 已完成项与未承诺候选。
- `docs/index.md` -- 人类和 AI 的导航入口。
- `docs/ai-handoff.md` -- 以当前代码证据优先的读取顺序。
- `docs/ci.md`, `docs/project-context.md` -- 区分本地 CI 配置与尚未发生的远程托管门禁。
- `docs/project-scan-report.json` -- 明确限定在 source commit 的历史扫描证据。
- `spec-bounded-diagnostic-rendering.md` -- 冻结块外的最终状态证据。

## Tasks & Acceptance

**Execution:**
- [x] `implementation-status.md`, `roadmap.md` -- 移除已完成候选，补齐已完成规格与 CI，避免用无时间边界的测试计数代替行为证据。
- [x] `index.md`, `ai-handoff.md` -- 补齐导航，将任务相关源码/测试核验提前。
- [x] `project-scan-report.json` -- 增加历史快照说明，保留当时 source commit、命令和计数作为审计记录。
- [x] docs -- 验证本地 Markdown 链接目标、JSON 语法、格式、严格 Clippy、workspace tests 和四命令闭环。

**Acceptance Criteria:**
- Given 面向当前的文档，when 搜索旧基线、旧测试数或已完成候选，then 只在显式历史快照/历史规格中出现。
- Given 一个新 AI 任务，when 按交接顺序读取，then 先获得强制边界和任务相关实现证据，再使用较宽的说明文档。
- Given 当前文档集，when 执行链接、JSON 和工程门禁，then 全部成功且不修改产品行为。

## Spec Change Log

- 2026-08-10：当前状态锚定已提交的 `0023bc8`，补齐静态类型、有界诊断、通用换行与五份已完成行为规格；移除“诊断资源边界待做”并取消当前测试总数硬编码。
- 2026-08-10：索引与交接协议区分获批意图、当前代码事实和提交锚定的历史证据；扫描报告仅新增历史快照范围说明，保留 `ae90f8d`、原命令与 50 个测试记录。
- 2026-08-10：CI 统一标为本地已配置但尚未推送、尚无 GitHub 托管运行、尚未设置 branch protection；bounded diagnostic 规格冻结块外尾注与 `done` 状态一致。冻结块均未修改。
- 2026-08-10 review：边界审查发现“当前 main”会在文档提交后立即失真，且开发指南仍可能把本地 workflow 误读为已自动执行；已改为提交锚定的实现证据基线，并为 CI 外部状态增加核对日期、推送后生效条件和同步更新要求。KEEP：五份完成规格、历史扫描证据、冻结内容与未远程生效结论保持不变。

## Design Notes

历史规格中的精确测试数是阶段完成证据，不应随当前套件增长而改写。面向当前的状态文档则优先引用门禁和行为分类，降低每增一测就同步多处数字的漂移面。

## Verification

**Commands:**
- JSON parse and repository-relative Markdown link check
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- four `cargo run --locked -p morva-cli -- ...` example commands
- `git diff --check`

2026-08-10 最终验证：JSON 可解析；README、`docs/*.md` 与 implementation artifacts 的仓库相对 Markdown 链接目标全部存在；bounded diagnostic 冻结块与 `HEAD` 完全一致；formatting、严格 Clippy、workspace tests、四命令示例闭环与 `git diff --check` 全部通过。三层审查 PASS，规格收口为 `done`。

## Suggested Review Order

**当前状态与历史证据**

- 先确认当前实现基线、行为覆盖和完整规格清单。
  [`implementation-status.md:3`](../../docs/implementation-status.md#L3)

- 扫描报告只新增提交锚定的历史证据范围。
  [`project-scan-report.json:10`](../../docs/project-scan-report.json#L10)

- Roadmap 区分原始基线、完成增量和未承诺候选。
  [`roadmap.md:13`](../../docs/roadmap.md#L13)

**AI 读取与导航**

- 交接顺序先锁定意图，再核验当前源码与测试。
  [`ai-handoff.md:5`](../../docs/ai-handoff.md#L5)

- 索引汇总五份完成行为规格和历史快照入口。
  [`index.md:11`](../../docs/index.md#L11)

**CI 外部状态边界**

- 权威 CI 页记录核对日期和三项未生效状态。
  [`ci.md:3`](../../docs/ci.md#L3)

- 开发指南明确自动执行只在 workflow 推送后成立。
  [`development-guide.md:61`](../../docs/development-guide.md#L61)

- 强制上下文避免把本地配置误称为远程门禁。
  [`project-context.md:24`](../../docs/project-context.md#L24)

**历史规格一致性**

- 冻结块外尾注与既有 `done` 元数据对齐。
  [`spec-bounded-diagnostic-rendering.md:124`](spec-bounded-diagnostic-rendering.md#L124)

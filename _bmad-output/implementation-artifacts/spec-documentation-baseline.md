---
title: '项目文档与 AI 交接基线收口'
type: 'chore'
created: '2026-08-10'
status: 'in-progress'
baseline_commit: 'ae90f8d6126eb99cec6f4388ece790b04cf53e8d'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md'
  - '{project-root}/_bmad-output/implementation-artifacts/spec-v0-1-minimal-simulate.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Morva 的语义核心和最小模拟已经实现并通过测试，但新生成的项目文档尚未形成一致、可提交的基线：扫描报告仍显示流程未完成，`project-context.md` 留有 pending 占位，README 与新增文档也尚未经过统一的事实核验。该状态会让后续人类或 AI 误判项目进度、能力边界和修改规则。

**Approach:** 以当前源码、自动化测试、可执行示例和两份冻结规格为证据，审核并修正文档集合；补全短小且强制的 AI 项目上下文，关闭扫描流程状态，验证内部链接和质量门禁，最终形成一个独立、可审查的文档基线提交。

## Boundaries & Constraints

**Always:** 保留 `Morva` / `morva` / `.morva` 命名；准确区分强类型语义、兼容解析和未来方向；源码与测试优先于说明性文档；保留两份已批准规格中的冻结意图；文档必须明确当前无 CI、无第三方依赖、无完整静态类型系统；所有链接使用仓库相对路径并可解析。

**Ask First:** 删除现有文档、改变产品边界、重写已冻结规格、把候选功能标记为承诺、提交或推送文档基线以外的用户改动。

**Never:** 不修改 Rust 行为或测试来迁就文档；不把 simulator 的运行时守卫描述为完整静态保证；不宣称形式化验证、代码生成、模块语义、LSP 或生产就绪；不丢弃用户现有工作树内容。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|---------------------------|----------------|
| 新贡献者进入项目 | 从 README 或 `docs/index.md` 开始 | 能找到需求、实现状态、架构、语言参考、开发验证和 AI 交接规则 | 断链或入口冲突视为验收失败 |
| AI 接手实现任务 | 读取 `docs/project-context.md` 和交接协议 | 获得明确的语义层边界、测试门禁、人工确认条件与禁止漂移规则 | 不允许保留 pending/TBD 占位 |
| 核对项目进度 | 阅读实现状态、roadmap 和扫描报告 | 三者与提交 `ae90f8d` 后的实际代码、50 个测试和已知限制一致 | 未实现能力不得标为完成 |

</frozen-after-approval>

## Code Map

- `README.md` -- 仓库顶层入口和当前能力摘要。
- `docs/index.md` -- 项目文档导航与可信度规则。
- `docs/project-context.md` -- 后续 AI 必须优先读取的短规则集。
- `docs/implementation-status.md` -- 当前实现、兼容边界、技术债务和下一候选。
- `docs/project-scan-report.json` -- 文档扫描工作流的机器可读完成状态。
- `docs/requirements.md`, `docs/architecture.md`, `docs/language-reference.md` -- 产品、结构和语言事实基线。
- `docs/ai-handoff.md`, `docs/development-guide.md`, `docs/testing-strategy.md` -- 交接、修改顺序与质量门禁。
- `_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md` -- 冻结意图不可改，但冻结块外的评审行号需要与当前源码同步。

## Tasks & Acceptance

**Execution:**
- [x] `README.md`, `docs/*.md` -- 以源码、测试、示例和冻结规格交叉核对能力、限制、命令、路径与文档导航；统一 v0.1 完成口径、`.morva` 扩展名约定、`parse` 语义门禁、Integer 范围、runtime value 职责和工具链快照等已发现差异。
- [x] `docs/project-context.md` -- 用可直接执行的关键规则替换 pending 占位，保持其短小并避免复制完整需求。
- [x] `docs/project-scan-report.json` -- 将扫描状态、完成步骤、生成输出和恢复说明更新为真实的已完成状态。
- [x] `_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md` -- 只修正冻结块外已漂移的源码评审锚点，不改变人工批准意图。
- [x] `README.md`, `docs/index.md` -- 验证全部仓库内 Markdown 链接目标存在，确保两个入口指向同一权威文档集合。
- [x] 全工作区 -- 执行格式、lint、测试和示例闭环并检查文档差异。
- [x] 版本控制 -- 将已验证的文档基线形成独立阶段提交。

**Acceptance Criteria:**
- Given 当前工作树中的文档集合，when 搜索 pending/TBD/未完成工作流标记，then 不存在误导性的生成占位或恢复指令。
- Given README 和文档中的所有仓库相对链接，when 解析每个目标，then 目标文件全部存在。
- Given 文档对已实现能力和限制的陈述，when 与公开 API、50 个测试、`examples/order.morva` 及冻结规格核对，then 没有把兼容解析或运行时守卫夸大为静态语义。
- Given 文档收口后的工作树，when 执行 fmt、严格 Clippy、workspace tests 和 check/parse/inspect/simulate 示例，then 全部成功且无 warning。

## Spec Change Log

- 2026-08-10：文档事实、入口、AI 上下文、扫描状态和评审锚点已收口；链接、格式、严格 Clippy、50 个测试及四命令示例闭环均通过。独立阶段提交留待获授权的提交步骤。

## Design Notes

`project-context.md` 只保留高频且容易被 AI 忽略的规则；详细产品需求、架构和测试矩阵通过链接引用，避免多份规则在后续版本中漂移。扫描报告必须记录文档实际生成完成，而不是简单删除中断证据。

## Verification

**Commands:**
- `git diff --check` -- expected: 无空白错误。
- 文档相对链接检查 -- expected: README 与 `docs/*.md` 的本地目标全部存在。
- `cargo fmt --check` -- expected: workspace 格式正确。
- `cargo clippy --workspace --all-targets -- -D warnings` -- expected: 所有 target 无 warning。
- `cargo test --workspace` -- expected: 50 个测试全部通过。
- 四个 `cargo run -p morva-cli -- ...` 示例命令 -- expected: check/parse/inspect 成功，simulate 七阶段 PASS。

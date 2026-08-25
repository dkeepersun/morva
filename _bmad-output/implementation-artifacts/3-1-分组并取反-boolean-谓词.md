---
baseline_commit: f6dde0a
---

# Story 3.1: 分组并取反 Boolean 谓词

Status: done

## Story

As a 业务规则作者,
I want 使用括号和 `!` 对 Boolean 条件进行分组和取反,
so that 我可以准确表达"不是某状态"或"整个条件不成立"。

## Acceptance Criteria

见 `_bmad-output/planning-artifacts/epics.md#Story-3.1` 与冻结规格 `spec-boolean-negation-grouping.md`。

## Implementation Notes

- AST 新增 `ExprKind::Not`；括号不建独立节点，返回内部表达式并把 span 扩展到括号两端。
- Parser `expression()` 变为谓词层（`!` 前缀与括号分组），原比较逻辑下沉为 `comparison()`；比较操作数保持字面量/路径不递归。
- Checker：`Not` 递归复用 `check_predicate`，非 Boolean 操作数由既有 `MORVA2013` 报告；赋值值与 `Binary` 同规则；`literal_fact`/`plain_literal` 对 `Not` 返回 None（矛盾推导留给 Story 3.3）。
- Simulator：`Not` 严格求值操作数后取反，非 Boolean 运行时守卫；未初始化读取失败 span 保持指向责任路径。
- CLI `parse` 渲染 `!path` 与 `!(comparison)`；能力清单同步声明 negation/grouped predicate 并把 unsupported 缩窄为 `logical 'or'`。

## Verification

- 语言、模拟、CLI 三层新增专项测试（优先级等价、span、双重取反、1026/1013/2013、全谓词位置、七阶段求值、uninitialized span、渲染与能力行）。
- 本地门禁：fmt、locked strict Clippy、workspace tests、示例闭环全绿。

## Change Log

- 2026-08-26: Implemented predicate-level negation and grouping across parser, checker, simulator, CLI, capability inventory, docs, and regression tests; marked the story done.

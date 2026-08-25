---
baseline_commit: 92996fa
---

# Story 2.4: 查询 Morva 的权威能力清单

Status: done

## Story

As a 模型作者或工具集成者,
I want 从 Morva 本身查询当前支持和不支持的语言能力,
so that 我不需要猜测语法边界，也不会依赖可能漂移的手写列表。

## Acceptance Criteria

见 `_bmad-output/planning-artifacts/epics.md#Story-2.4` 与冻结规格 `spec-capability-inventory.md`。

## Implementation Notes

- 新增 `morva-core::capabilities` 模块：版本化 `CapabilityInventory` 结构与 `capabilities()` 构造函数，无 CLI/文件系统/网络依赖。
- 为消除第二张判断表：parser 的 `DECLARATION_KINDS` 拆分为 `SEMANTIC_DECLARATION_KINDS` + `COMPATIBILITY_CONTAINER_KINDS`（行为不变）；`SoftBehaviorKind`/`ClauseKind`/`BinaryOperator`/`AssignmentOperator` 新增 `ALL` 常量，操作符新增 `as_str()`；`SimulationPhase` 新增 `ALL`；semantic 模块提炼 builtin 类型名与别名常量。CLI `parse` 输出改用同一 `as_str()`，删除自带操作符表。
- 新增 `morva capabilities` 命令：无参数、不读文件、稳定文本、退出 0；help 文本同步。
- 未支持类别为 core 内声明式清单，与 docs/known limits 对齐；语言能力变化时必须同批更新（由漂移测试强制）。

## Verification

- 核心漂移测试逐项证明清单中的容器、soft behavior、操作符、clause、builtin 类型与别名被真实 parser/checker 接受并产生对应结构化 notice；模拟阶段与真实 simulation report 逐项相等。
- CLI 测试锁定稳定文本、byte-identical 重复运行、退出 0 与多余参数退出 2。
- 本地门禁：fmt、locked strict Clippy、workspace tests、示例闭环全绿。

## Change Log

- 2026-08-26: Implemented the versioned core capability inventory, the `morva capabilities` command, single-source constants shared with parser/AST/simulator, drift tests, and documentation sync; marked the story done.

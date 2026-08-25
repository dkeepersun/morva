---
baseline_commit: 912c2ef
---

# Story 4.1: 机器读取 check 结果与诊断

Status: done

## Story

As a 自动化工具开发者,
I want 以稳定的 JSON 格式读取 Morva 检查结果与诊断,
so that 我可以可靠地把模型验证接入脚本、编辑器和持续集成。

## Acceptance Criteria

见 `_bmad-output/planning-artifacts/epics.md#Story-4.1`。

## Implementation Notes

- CLI 装载层重构为输出无关的 `LoadFailure`（Input / SingleModel / ProjectModel）：人类路径经 `exit_human()` 逐字节保持既有 stderr 输出与退出码；机器路径序列化同一事实。
- 机器 envelope：`{protocol: "morva.cli", schema_version: 1, command, success, ...}`；诊断项含 severity/code/message/location（真实源路径、1-based line/column、文件本地 byte span；无源的 project 诊断 location 为 null）。IO/发现/用法错误在识别 `--format json` 后输出 `error: {kind: input|usage, message}` envelope 并退出 2。
- JSON 序列化复用 core 的 canonical `morva_core::json` writer（与 checked-semantics 协议同一实现，无第二份 escaping 表）；core 不依赖 CLI 参数或 JSON 实现。
- `--format` 值非 `json` 时保持人类 usage 行为。

## Verification

- 新增 4 个 CLI 进程级测试：clean 模型精确 envelope、warning/error 结构化诊断与位置、项目本地文件映射、input/usage machine envelope 与退出码边界、byte-identical 重复运行、stderr 为空。
- 全部 191 workspace tests、fmt、locked strict Clippy、示例闭环全绿。

## Change Log

- 2026-08-26: Implemented the versioned machine envelope and JSON check output with an output-agnostic CLI load layer; marked the story done.

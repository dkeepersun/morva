---
baseline_commit: c655c91
---

# Story 4.2: 机器读取结构化 AST

Status: done

## Story

As a 自动化工具开发者,
I want 以稳定 JSON 读取 Morva 的结构化 AST,
so that 我可以构建分析、转换和编辑器工具，而不依赖 Rust Debug 文本。

## Acceptance Criteria

见 `_bmad-output/planning-artifacts/epics.md#Story-4.2`。

## Implementation Notes

- `morva parse --format json` 复用 Story 4.1 的 morva.cli/v1 envelope，payload 为 `ast`：每个节点显式 `kind` + 语义字段（container 保留嵌套与 kind/name、enum members、entity fields（书写类型名）、action parameters/soft behavior kinds/clauses、scenario items、递归 `binary`/`not`/`or` 表达式），完整保留操作符与表达式 span。
- 位置通过 `Locator` 统一映射：单文件直接本地 span；项目经 `Project::locate_virtual_span` 回映真实文件与本地 span——JSON 中不出现 virtual offset。合并后的项目 system 外壳是合成节点，`location: null` 明确标识。
- 被跳过的兼容/hint 正文不回显；soft behavior 只输出 kind 与 keyword span。模型错误复用 machine diagnostics（退出 1），IO/usage 复用 error envelope（退出 2）。文档明确 AST JSON 是只读结构化视图，不承诺无损还原源码。

## Verification

- 新增 2 个进程级测试：CRLF+注释+容器+hint+递归 Boolean AST 的单文件（含正文不回显与 byte-identical 重复运行）；多文件合成外壳 `location: null`、真实文件映射与模型错误退出 1。独立 JSON parser 手工核对项目输出全部 source 均为真实文件。
- 193 workspace tests、fmt、locked strict Clippy 全绿。

## Change Log

- 2026-08-26: Implemented the structured JSON AST view with real-file location mapping and synthetic-shell marking; marked the story done.

---
baseline_commit: c0ad303
---

# Story 2.3: 在 inspect 中查看未建模内容

Status: done

## Story

As a 模型评审者,
I want `inspect` 汇总模型中尚未获得语义覆盖的声明和行为项,
so that 我可以判断当前模型有多少内容不能被 checker 或 simulator 证明。

## Acceptance Criteria

见 `_bmad-output/planning-artifacts/epics.md#Story-2.3` 与冻结规格 `spec-unmodeled-inspect-summary.md`。

## Implementation Notes

- 纯 CLI 增量：`inspect_document` 新增 `notices` 参数，`run()` 在 check 通过后调用既有 `analyze(document)` 取 notices；core 无任何改动。
- 摘要只消费结构化 `NoticeKind`（container kind/name、action/soft behavior kind），不回显任何被跳过正文。
- 项目输入通过装配后 virtual document 的 notice span 顺序天然获得"文件顺序 + 源码顺序"，无需额外排序。
- 无未建模内容时不输出摘要；既有 clean-model exact-output 测试原样通过，证明零噪声。

## Verification

- `crates/morva-cli/tests/cli.rs::check_parse_and_inspect_the_example` 锁定含摘要的完整 inspect 文本。
- `crates/morva-cli/tests/cli.rs::inspect_lists_unmodeled_project_content_in_stable_file_and_source_order` 锁定跨文件顺序与 byte-identical 重复运行。
- 本地门禁：`cargo fmt --all --check`、locked strict Clippy、`cargo test --workspace --locked`、示例四命令闭环全绿。

## Change Log

- 2026-08-26: Implemented the unmodeled inspect summary from existing analysis notices, locked exact CLI output and project ordering tests, synchronized docs/cli.md and docs/implementation-status.md, and marked the story done.

---
baseline_commit: 19119e9
---

# Story 3.3: 保守分析嵌套 Boolean 矛盾

Status: done

## Story

As a 模型评审者,
I want checker 识别由取反、析取和精确事实直接证明的明显矛盾,
so that 新表达力不会削弱现有的早期错误发现能力或产生误报。

## Acceptance Criteria

见 `_bmad-output/planning-artifacts/epics.md#Story-3.3` 与冻结规格 `spec-nested-boolean-contradictions.md`。

## Implementation Notes

- `check_predicate_group` 与 `check_final_effect_contradictions` 重写为统一的三态 `evaluate_formula`：literal/精确比较/`!`/`||`/括号递归求值，`used_paths` 记录求值中消费的事实路径，决定 "always false" 与 constraint-conflict 消息的选择——旧形状的消息、span、顺序逐字节兼容（全部既有回归原样通过）。
- 事实吸收保持旧规则：仅顶层平凡 `==`/`!=` 精确比较入组，first-wins equal、去重 not-equal；`!`/`||` 内部不贡献事实（无泄漏）。
- 后态公式对最终字面量 effects 构成的事实集求值，False 报 `MORVA2019`；compound/非字面量写入使路径 Unknown；`MORVA2018` 同 span 抑制规则保留。
- 无新诊断码、无 SAT/区间求解、无第三方依赖。

## Verification

- 新增 4 个专项测试（常量恒假嵌套、事实证伪嵌套、真分支免疫、Unknown 保守、双向无泄漏、嵌套 2019、compound 降级、主诊断抑制）；全部 176 workspace tests 全绿。
- 本地门禁：fmt、locked strict Clippy、示例闭环全绿。

## Change Log

- 2026-08-26: Rewrote contradiction analysis as conservative three-valued formula evaluation with byte-compatible legacy behavior; marked the story done.

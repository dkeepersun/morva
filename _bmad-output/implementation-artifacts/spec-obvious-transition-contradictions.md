---
title: '检测明显状态转换矛盾'
type: 'feature'
created: '2026-08-10'
status: 'done'
baseline_commit: 'd5f69caa04d43dd48e054622131dd26713838526'
context:
  - '{project-root}/CONTEXT.md'
  - '{project-root}/docs/project-context.md'
  - '{project-root}/docs/requirements.md'
  - '{project-root}/docs/language-reference.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 当前 checker 可接受必然为假的谓词、同一 action 阶段中互斥的精确值约束，以及已知最终 `=` effect 与后置约束的直接冲突；这些模型只会在模拟阶段必然失败。

**Approach:** 在现有引用和类型检查之后，对有限字面量事实做保守的同阶段一致性分析，并用顺序 effect 的最终已知赋值检查 action 后置约束。

## Boundaries & Constraints

**Always:** 多个谓词按合取解释；`requires + action invariant` 属前态，`action invariant + ensures` 属后态，不跨阶段合并；effects 按源码顺序，最后一个可解析的直接字面量 `=` 决定已知最终值；诊断指向较后的冲突谓词并保持源码顺序；引用/类型失败抑制派生矛盾诊断。

**Ask First:** 扩大到有序区间求解、entity invariant 参数实例化、scenario/action 内联、未写路径跨前后态推理，或改变诊断严重度模型。

**Never:** 不改 parser/AST/simulator/CLI 退出码；不引入 SAT/SMT、符号执行、通用数据流、隐式转换或 Decimal runtime；不把多次 effect 当作无序声明；不从软行为项推导真假。

## I/O & Edge-Case Matrix

| Scenario | Expected behavior |
|---|---|
| `requires false` 或常量比较为假 | `MORVA2018`，span=该谓词 |
| 同一前/后态中 `x == A` 与 `x != A` | `MORVA2018` 指向后者 |
| 同一前/后态中 `x == A` 与 `x == B` | A≠B 时 `MORVA2018`，相同时合法 |
| `requires x == A; effects x = B; ensures x == B` | 合法，不跨前后态误报 |
| 最终 `effects x = A`，后置要求 `x != A` 或 `x == B` | `MORVA2019` 指向后置谓词 |
| 多次 `=` 或非字面量/compound effect | 最后 literal `=` 可恢复 Known；其他最终状态 Unknown 且不推断 |
| 未知路径、enum member 或类型不匹配 | 只保留现有主诊断，不追加 2018/2019 |

</frozen-after-approval>

## Code Map

- `crates/morva-core/src/semantic.rs` -- 在既有类型解析后归一化字面量事实、分组检查并摘要顺序 effects。
- `crates/morva-core/tests/language.rs` -- 公开 parse/check seam 的正反与抑制级联矩阵。
- `crates/morva-core/tests/simulation.rs` -- 确认公开 simulate 在必败转换上前移为静态诊断。
- `CONTEXT.md`, `docs/requirements.md`, `docs/language-reference.md`, `docs/implementation-status.md`, `docs/architecture.md`, `docs/roadmap.md` -- 术语、承诺和保守边界。

## Tasks & Acceptance

**Execution:**
- [x] `semantic.rs` -- 先以公开测试锁定 MORVA2018/2019 message+span+顺序，再实现有限事实和 effect summary。
- [x] `language.rs`, `simulation.rs` -- 覆盖 Boolean/Integer/enum/Decimal-context、左右归一化、前后态隔离、多次写、Unknown 降级/恢复和主诊断抑制。
- [x] `CONTEXT.md`, `docs/*.md` -- 记录状态转换、前/后置与“明显矛盾”的保守定义，勾选 Roadmap 对应项。
- [x] workspace -- 格式、严格 Clippy、全部测试、四命令示例、差异检查与三层审查已通过。

**Acceptance Criteria:**
- Given 同一阶段的已解析字面量事实，when 它们可直接证明无法同时成立，then 只对较后谓词产生一个 MORVA2018。
- Given 直接字面量最终 effect，when 它使 action 后置谓词必然为假，then 产生 MORVA2019；最终值未知时不推测。
- Given 已有引用/类型错误、合法转换或本增量外推理，when check/simulate，then 不新增级联噪声或改变 runtime/AST。

## Spec Change Log

- 2026-08-10：按冻结边界实现 `MORVA2018/2019`；同一谓词 span 同时满足阶段事实冲突和最终 effect 冲突时保留 `MORVA2018`，抑制重复 `MORVA2019`。
- 2026-08-10：同步公开 API 测试、simulation 静态前移测试及需求、语言参考、实现状态、架构和 roadmap；状态进入 `in-review`。
- 2026-08-10 review：修正 action 参数与 enum member 同名时的字面量误判；仅未被参数绑定的裸 enum member 形成事实，并增加公开 `check` 回归。三层审查无 intent/spec 缺口，其余发现经源码核对拒绝。
- 2026-08-10 final review：将当前实现状态改为由已完成规格、源码/公开测试和门禁共同证明，避免上一提交锚点在新增量后立即过期；文档索引补入本规格。

## Design Notes

事实值只需表示 Boolean、Integer 和带 enum 声明身份的 member；Decimal 字面量沿用已有“明确 Decimal 上下文中的非负 Integer 常量”规则，不增加 runtime 值。首版不做有序区间推理；对 effect 路径遇到非字面量 `=` 或 `+=/-=` 就降为 Unknown，后续 literal `=` 可恢复 Known。

## Verification

**TDD evidence:**
- RED：`rejects_an_always_false_action_predicate` 首次运行得到 0 diagnostics；GREEN：最小 `MORVA2018` 实现后通过。
- RED：`rejects_constant_and_same_phase_literal_contradictions` 首次语义运行得到空 diagnostics；GREEN：常量折叠、左右归一化和阶段事实表实现后通过。
- RED：`final_literal_effects_reject_conflicting_postconditions_conservatively` 首次运行得到空 diagnostics；GREEN：顺序 effect `Known/Unknown` 摘要与 `MORVA2019` 后通过。
- RED：`a_postcondition_gets_one_primary_contradiction_diagnostic_per_span` 首次得到同 span 的 `MORVA2018` 与 `MORVA2019`；GREEN：重复派生诊断抑制后只保留 `MORVA2018`。

**Results:**
- Focused public `parse/check` and `simulate` tests: pass.
- `cargo fmt --all -- --check`: pass.
- Strict workspace Clippy: pass.
- Locked workspace tests: pass (100 tests: CLI 23, core unit 2, language 53, simulation 22).
- Locked `check` / `parse` / `inspect` / `simulate` commands for `examples/order.morva`: pass.
- `git diff --check`: pass.
- 2026-08-10 最终复验：三层审查已收口，enum member/同名 action 参数边界已修复；100 个测试、严格 Clippy、四命令和差异检查全部通过，规格状态收口为 `done`。

**Commands:**
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `cargo run --locked -p morva-cli -- check examples/order.morva`
- `cargo run --locked -p morva-cli -- parse examples/order.morva`
- `cargo run --locked -p morva-cli -- inspect examples/order.morva`
- `cargo run --locked -p morva-cli -- simulate examples/order.morva NormalConfirmation`
- `git diff --check`

## Suggested Review Order

**语义入口与阶段边界**

- 类型成功后才分析，并只排序当前 action 新诊断。
  [`semantic.rs:527`](../../crates/morva-core/src/semantic.rs#L527)

- 前态与后态分组共享 invariant，但不跨阶段传播事实。
  [`semantic.rs:586`](../../crates/morva-core/src/semantic.rs#L586)

**顺序 effect 与精确事实**

- 源码顺序维护 Known/Unknown，后置冲突产生 MORVA2019。
  [`semantic.rs:612`](../../crates/morva-core/src/semantic.rs#L612)

- 后出现的同阶段冲突产生单个 MORVA2018。
  [`semantic.rs:679`](../../crates/morva-core/src/semantic.rs#L679)

- enum 字面量尊重同名 action 参数的既有绑定。
  [`semantic.rs:810`](../../crates/morva-core/src/semantic.rs#L810)

**公开契约与边界回归**

- 核心矩阵覆盖阶段隔离、重复写和 Decimal-context。
  [`language.rs:728`](../../crates/morva-core/tests/language.rs#L728)

- shadowing 回归保护参数路径不被误判为 enum member。
  [`language.rs:873`](../../crates/morva-core/tests/language.rs#L873)

- simulate 公开 seam 证明必败转换前移为静态诊断。
  [`simulation.rs:455`](../../crates/morva-core/tests/simulation.rs#L455)

**承诺与限制**

- 语言参考定义保守检查范围与 Unknown 降级。
  [`language-reference.md:114`](../../docs/language-reference.md#L114)

- 架构明确排除区间、跨场景与 compound folding。
  [`architecture.md:114`](../../docs/architecture.md#L114)

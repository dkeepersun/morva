---
stepsCompleted: ['step-01-load-context', 'step-02-discover-tests', 'step-03-map-criteria', 'step-04-analyze-gaps', 'step-05-gate-decision']
lastStep: 'step-05-gate-decision'
lastSaved: '2026-08-10'
tempCoverageMatrixPath: '/tmp/tea-trace-coverage-matrix-2026-08-10T00-00-00-08-00.json'
workflowType: 'testarch-trace'
coverageBasis: 'acceptance_criteria'
oracleConfidence: 'high'
oracleResolutionMode: 'formal_requirements'
oracleSources:
  - 'docs/requirements.md'
  - 'docs/roadmap.md'
  - '_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md'
  - '_bmad-output/implementation-artifacts/spec-v0-1-minimal-simulate.md'
  - '_bmad-output/implementation-artifacts/spec-minimal-static-expression-types.md'
  - '_bmad-output/implementation-artifacts/spec-bounded-diagnostic-rendering.md'
  - '_bmad-output/implementation-artifacts/spec-universal-newline-contract.md'
  - '_bmad-output/implementation-artifacts/spec-obvious-transition-contradictions.md'
externalPointerStatus: 'not_used'
---

# Morva v0.1 需求追踪矩阵与质量门

**目标：** Morva v0.1 semantic core 的本地实现完成度
**日期：** 2026-08-10
**覆盖判据：** 正式需求与已批准规格的验收标准
**判据置信度：** high

本工作流只审计现有覆盖，不用测试数量代替行为证据，也不生成测试。

## Step 1：判据与上下文

- 正式判据是 `docs/requirements.md` 中的 FR-01..FR-08、NFR-01..NFR-07 和 v0.1 完成定义。
- 已批准规格用于细化输入、边界、诊断、模拟和回归验收；其范围不得扩大正式需求。
- `docs/roadmap.md` 用于区分 v0.1 与未来方向；Tree-sitter、LSP、AI review、graph、flow/lifecycle 与代码生成不属于本次完成门。
- 仓库没有需要解析的外部需求指针；源码和测试是实现证据，不是替代需求的 synthetic oracle。
- 质量判定采用风险优先：核心语言与模拟为 P0/P1；文档与运维激活项按其是否属于完成定义单独评估。

## Step 2：测试发现与分类

执行态清单来自 `cargo test --workspace -- --list` 与源码位置交叉核对；未发现 `ignore`、pending、fixme 或 `should_panic` 测试。Rust 公共库集成测试归为 API，真实 CLI 子进程测试归为 E2E，私有 runtime guard 归为 Unit。

| ID | 标题 | 文件:行 | 层级 | 状态 |
|---|---|---|---|---|
| CLI-001 | `check_parse_and_inspect_the_example` | `crates/morva-cli/tests/cli.rs:52` | E2E | active |
| CLI-002 | `syntax_and_semantic_errors_exit_one_with_stable_locations` | `crates/morva-cli/tests/cli.rs:77` | E2E | active |
| CLI-003 | `tabs_are_expanded_without_changing_the_reported_column` | `crates/morva-cli/tests/cli.rs:100` | E2E | active |
| CLI-004 | `control_characters_are_escaped_in_diagnostics` | `crates/morva-cli/tests/cli.rs:115` | E2E | active |
| CLI-005 | `long_line_diagnostic_at_start_has_a_bounded_right_window` | `crates/morva-cli/tests/cli.rs:127` | E2E | active |
| CLI-006 | `diagnostic_window_crops_only_above_the_160_width_threshold` | `crates/morva-cli/tests/cli.rs:147` | E2E | active |
| CLI-007 | `long_line_diagnostic_in_middle_keeps_the_marker_visible` | `crates/morva-cli/tests/cli.rs:165` | E2E | active |
| CLI-008 | `long_line_diagnostic_at_end_has_a_bounded_left_window` | `crates/morva-cli/tests/cli.rs:187` | E2E | active |
| CLI-009 | `diagnostic_window_adds_a_left_ellipsis_only_beyond_72_width` | `crates/morva-cli/tests/cli.rs:207` | E2E | active |
| CLI-010 | `long_line_eof_diagnostic_keeps_a_visible_bounded_caret` | `crates/morva-cli/tests/cli.rs:225` | E2E | active |
| CLI-011 | `multiline_span_marks_only_its_bounded_start_line_window` | `crates/morva-cli/tests/cli.rs:247` | E2E | active |
| CLI-012 | `diagnostic_window_preserves_escaped_fragment_boundaries` | `crates/morva-cli/tests/cli.rs:269` | E2E | active |
| CLI-013 | `long_line_non_ascii_diagnostic_keeps_the_complete_codepoint_escape` | `crates/morva-cli/tests/cli.rs:292` | E2E | active |
| CLI-014 | `crlf_diagnostic_excludes_the_carriage_return_from_its_excerpt` | `crates/morva-cli/tests/cli.rs:313` | E2E | active |
| CLI-015 | `mixed_newlines_share_one_logical_line_and_excerpt_contract` | `crates/morva-cli/tests/cli.rs:330` | E2E | active |
| CLI-016 | `eof_after_each_newline_sequence_has_the_same_location_and_caret` | `crates/morva-cli/tests/cli.rs:348` | E2E | active |
| CLI-017 | `carriage_return_at_eof_is_a_logical_line_terminator` | `crates/morva-cli/tests/cli.rs:366` | E2E | active |
| CLI-018 | `long_line_simulation_failure_uses_the_bounded_diagnostic_window` | `crates/morva-cli/tests/cli.rs:384` | E2E | active |
| CLI-019 | `control_characters_in_utf8_paths_are_escaped_for_every_cli_result` | `crates/morva-cli/tests/cli.rs:411` | E2E | active |
| CLI-020 | `usage_and_file_errors_exit_two` | `crates/morva-cli/tests/cli.rs:467` | E2E | active |
| CLI-021 | `simulate_reports_the_example_transition_and_passes` | `crates/morva-cli/tests/cli.rs:478` | E2E | active |
| CLI-022 | `simulation_model_failure_exits_one_and_renders_its_span` | `crates/morva-cli/tests/cli.rs:495` | E2E | active |
| CLI-023 | `unknown_simulation_selection_exits_one_but_usage_stays_two` | `crates/morva-cli/tests/cli.rs:522` | E2E | active |
| LANG-001 | `parses_the_complete_strongly_typed_core` | `crates/morva-core/tests/language.rs:34` | API | active |
| LANG-002 | `existing_example_remains_valid` | `crates/morva-core/tests/language.rs:79` | API | active |
| LANG-003 | `cr_only_newlines_separate_language_items_and_preserve_byte_spans` | `crates/morva-core/tests/language.rs:85` | API | active |
| LANG-004 | `line_comments_stop_before_every_supported_newline_sequence` | `crates/morva-core/tests/language.rs:112` | API | active |
| LANG-005 | `equivalent_newline_sequences_keep_model_shape_and_original_byte_spans` | `crates/morva-core/tests/language.rs:135` | API | active |
| LANG-006 | `mixed_newline_sequences_are_single_logical_separators` | `crates/morva-core/tests/language.rs:178` | API | active |
| LANG-007 | `enum_members_require_the_expected_enum_context` | `crates/morva-core/tests/language.rs:197` | API | active |
| LANG-008 | `unknown_bare_and_dotted_references_are_rejected` | `crates/morva-core/tests/language.rs:226` | API | active |
| LANG-009 | `duplicate_names_and_unknown_types_are_reported` | `crates/morva-core/tests/language.rs:244` | API | active |
| LANG-010 | `effects_must_write_a_parameter_field` | `crates/morva-core/tests/language.rs:274` | API | active |
| LANG-011 | `predicates_must_be_boolean` | `crates/morva-core/tests/language.rs:293` | API | active |
| LANG-012 | `boolean_literals_and_paths_are_valid_predicates` | `crates/morva-core/tests/language.rs:311` | API | active |
| LANG-013 | `equality_requires_matching_canonical_types` | `crates/morva-core/tests/language.rs:328` | API | active |
| LANG-014 | `inequality_accepts_matching_canonical_types` | `crates/morva-core/tests/language.rs:355` | API | active |
| LANG-015 | `ordered_comparisons_require_integer_or_decimal_operands` | `crates/morva-core/tests/language.rs:369` | API | active |
| LANG-016 | `set_effects_require_a_compatible_value_type` | `crates/morva-core/tests/language.rs:395` | API | active |
| LANG-017 | `compound_effects_require_integer_target_and_value` | `crates/morva-core/tests/language.rs:419` | API | active |
| LANG-018 | `integer_add_and_subtract_effects_are_valid` | `crates/morva-core/tests/language.rs:446` | API | active |
| LANG-019 | `compound_effects_reject_boolean_binary_values_once` | `crates/morva-core/tests/language.rs:460` | API | active |
| LANG-020 | `compound_effects_report_enum_values_as_type_errors` | `crates/morva-core/tests/language.rs:480` | API | active |
| LANG-021 | `ordered_enum_comparisons_are_rejected_with_contextual_members` | `crates/morva-core/tests/language.rs:496` | API | active |
| LANG-022 | `builtin_aliases_share_canonical_types` | `crates/morva-core/tests/language.rs:512` | API | active |
| LANG-023 | `distinct_builtin_families_remain_incompatible` | `crates/morva-core/tests/language.rs:537` | API | active |
| LANG-024 | `decimal_context_accepts_integer_constants_but_not_integer_paths` | `crates/morva-core/tests/language.rs:557` | API | active |
| LANG-025 | `decimal_targets_reject_integer_paths` | `crates/morva-core/tests/language.rs:586` | API | active |
| LANG-026 | `entity_values_cannot_be_compared_as_whole_objects` | `crates/morva-core/tests/language.rs:604` | API | active |
| LANG-027 | `resolution_failures_suppress_derived_type_diagnostics` | `crates/morva-core/tests/language.rs:624` | API | active |
| LANG-028 | `a_resolution_failure_keeps_its_primary_message_and_span` | `crates/morva-core/tests/language.rs:645` | API | active |
| LANG-029 | `scenario_expects_must_be_boolean` | `crates/morva-core/tests/language.rs:667` | API | active |
| LANG-030 | `scenario_diagnostics_remain_in_source_order` | `crates/morva-core/tests/language.rs:688` | API | active |
| LANG-031 | `rejects_an_always_false_action_predicate` | `crates/morva-core/tests/language.rs:710` | API | active |
| LANG-032 | `rejects_constant_and_same_phase_literal_contradictions` | `crates/morva-core/tests/language.rs:728` | API | active |
| LANG-033 | `final_literal_effects_reject_conflicting_postconditions_conservatively` | `crates/morva-core/tests/language.rs:778` | API | active |
| LANG-034 | `literal_fact_analysis_preserves_legal_and_unknown_transitions` | `crates/morva-core/tests/language.rs:836` | API | active |
| LANG-035 | `enum_member_facts_do_not_shadow_action_parameters` | `crates/morva-core/tests/language.rs:873` | API | active |
| LANG-036 | `primary_resolution_and_type_errors_suppress_literal_fact_diagnostics` | `crates/morva-core/tests/language.rs:895` | API | active |
| LANG-037 | `a_postcondition_gets_one_primary_contradiction_diagnostic_per_span` | `crates/morva-core/tests/language.rs:924` | API | active |
| LANG-038 | `literal_fact_diagnostics_remain_in_source_order_across_codes` | `crates/morva-core/tests/language.rs:944` | API | active |
| LANG-039 | `unknown_action_items_fail_but_documented_soft_items_remain_compatible` | `crates/morva-core/tests/language.rs:969` | API | active |
| LANG-040 | `a_container_missing_its_block_cannot_consume_the_next_declaration` | `crates/morva-core/tests/language.rs:993` | API | active |
| LANG-041 | `declaration_blocks_may_start_on_the_next_line` | `crates/morva-core/tests/language.rs:1004` | API | active |
| LANG-042 | `clause_blocks_may_start_on_the_next_line` | `crates/morva-core/tests/language.rs:1026` | API | active |
| LANG-043 | `a_same_line_declaration_cannot_become_a_missing_container_block` | `crates/morva-core/tests/language.rs:1041` | API | active |
| LANG-044 | `an_action_without_parentheses_is_compatible` | `crates/morva-core/tests/language.rs:1050` | API | active |
| LANG-045 | `booleans_and_other_keywords_cannot_be_names` | `crates/morva-core/tests/language.rs:1063` | API | active |
| LANG-046 | `scenario_item_keywords_are_contextual_names` | `crates/morva-core/tests/language.rs:1077` | API | active |
| LANG-047 | `out_of_range_integer_is_a_diagnostic_not_a_panic` | `crates/morva-core/tests/language.rs:1094` | API | active |
| LANG-048 | `nested_systems_are_rejected` | `crates/morva-core/tests/language.rs:1103` | API | active |
| LANG-049 | `exactly_one_system_must_be_at_the_document_root` | `crates/morva-core/tests/language.rs:1109` | API | active |
| LANG-050 | `globally_ambiguous_short_type_names_are_rejected` | `crates/morva-core/tests/language.rs:1115` | API | active |
| LANG-051 | `user_types_cannot_shadow_builtin_types` | `crates/morva-core/tests/language.rs:1131` | API | active |
| LANG-052 | `non_ascii_diagnostics_cover_the_complete_codepoint` | `crates/morva-core/tests/language.rs:1136` | API | active |
| LANG-053 | `incomplete_syntax_has_a_span` | `crates/morva-core/tests/language.rs:1142` | API | active |
| SIM-001 | `simulates_the_repository_example` | `crates/morva-core/tests/simulation.rs:19` | API | active |
| SIM-002 | `initial_invariant_failure_stops_before_effects` | `crates/morva-core/tests/simulation.rs:36` | API | active |
| SIM-003 | `duplicate_given_fails_in_the_givens_phase` | `crates/morva-core/tests/simulation.rs:60` | API | active |
| SIM-004 | `requires_failure_stops_before_effects` | `crates/morva-core/tests/simulation.rs:79` | API | active |
| SIM-005 | `final_invariant_failure_preserves_effect_changes` | `crates/morva-core/tests/simulation.rs:102` | API | active |
| SIM-006 | `ensures_and_expect_fail_in_their_own_phases` | `crates/morva-core/tests/simulation.rs:126` | API | active |
| SIM-007 | `uninitialized_read_is_a_stable_runtime_failure` | `crates/morva-core/tests/simulation.rs:165` | API | active |
| SIM-008 | `compound_integer_overflow_fails_without_panicking` | `crates/morva-core/tests/simulation.rs:182` | API | active |
| SIM-009 | `effects_execute_in_source_order` | `crates/morva-core/tests/simulation.rs:200` | API | active |
| SIM-010 | `run_arguments_bind_positionally_to_distinct_entity_instances` | `crates/morva-core/tests/simulation.rs:223` | API | active |
| SIM-011 | `boolean_state_is_supported` | `crates/morva-core/tests/simulation.rs:249` | API | active |
| SIM-012 | `unsupported_effect_value_types_fail_static_check_before_simulation` | `crates/morva-core/tests/simulation.rs:266` | API | active |
| SIM-013 | `set_effect_type_mismatches_fail_static_check_before_simulation` | `crates/morva-core/tests/simulation.rs:286` | API | active |
| SIM-014 | `entity_invariants_resolve_contextual_enum_members` | `crates/morva-core/tests/simulation.rs:305` | API | active |
| SIM-015 | `equality_type_mismatches_fail_static_check_before_simulation` | `crates/morva-core/tests/simulation.rs:324` | API | active |
| SIM-016 | `public_simulate_rejects_an_unchecked_invalid_scenario` | `crates/morva-core/tests/simulation.rs:345` | API | active |
| SIM-017 | `scenario_structure_is_checked` | `crates/morva-core/tests/simulation.rs:364` | API | active |
| SIM-018 | `action_selection_and_binding_are_checked` | `crates/morva-core/tests/simulation.rs:382` | API | active |
| SIM-019 | `action_and_scenario_names_must_be_globally_unique` | `crates/morva-core/tests/simulation.rs:397` | API | active |
| SIM-020 | `invalid_given_targets_operators_and_values_are_checked` | `crates/morva-core/tests/simulation.rs:421` | API | active |
| SIM-021 | `unknown_scenario_selection_is_reported` | `crates/morva-core/tests/simulation.rs:447` | API | active |
| SIM-022 | `obvious_transition_contradictions_fail_static_check_before_simulation` | `crates/morva-core/tests/simulation.rs:455` | API | active |
| UNIT-001 | `runtime_equality_evaluator_rejects_different_value_types` | `crates/morva-core/src/simulate.rs:833` | Unit | active |
| UNIT-002 | `runtime_effect_guard_preserves_expected_field_type` | `crates/morva-core/src/simulate.rs:871` | Unit | active |

### 覆盖启发式清单

- API/命令面：`parse`、`check`、`inspect`、`simulate` 均有真实进程覆盖；core 的 parse/check/simulate 公共 seam 有集成覆盖。
- 错误路径：语法、引用、类型、场景结构、模拟各阶段、溢出、未初始化、文件/用法错误及诊断渲染边界均存在失败路径测试。
- UI、HTTP endpoint、auth/authz：项目不包含这些能力，判定为不适用而非覆盖缺口。
- 选择策略：当前套件约半秒完成，完整回归成本很低；本次完成门直接执行全部测试，不采用选择性跳过。

## Step 3：需求到测试映射

`FULL` 表示需求要求的主要正常、失败和边界行为均有公开 seam 证据；`PARTIAL` 表示测试不能单独证明架构或渐近复杂度，需要源码审查补充。表中列出的是主要证据，完整测试身份见 Step 2。

| 判据 | 优先级 | 覆盖 | 主要测试证据 | 错误/边界信号 |
|---|---|---|---|---|
| FR-01 源码、命名、单 system、通用换行 | P0 | FULL | LANG-001, LANG-003..006, LANG-043..052；CLI-014..017 | 嵌套/缺失 system、保留字、非 ASCII、CR/LF/CRLF/混合序列 |
| FR-02 强类型 system/entity/enum/action/scenario 与 span | P0 | FULL | LANG-001, LANG-029, LANG-053；SIM-001, SIM-016 | 不完整语法、scenario 顺序与失败 span |
| FR-03 兼容容器、软 action 项、未知项拒绝 | P0 | FULL | LANG-039..045 | 缺块容器、同/跨行 block、未知 action 项与白名单软项 |
| FR-04 引用、类型、effect、scenario 与有限矛盾检查 | P0 | FULL | LANG-007..038；SIM-012..020, SIM-022 | alias/Decimal/Entity/enum、级联抑制、顺序、Unknown 降级与恢复 |
| FR-05 稳定诊断与有界安全渲染 | P0 | FULL | CLI-002..018, CLI-020；LANG-028, LANG-030, LANG-034..038, LANG-051..053 | code/message/span、159/160/161、72/73、tab/control/non-ASCII/path |
| FR-06 四命令与退出码 0/1/2 | P0 | FULL | CLI-001, CLI-020..023 | 模型失败、模拟失败、未知选择、用法/文件错误 |
| FR-07 单 action 七阶段内存模拟 | P0 | FULL | SIM-001..022；CLI-018, CLI-021..023；UNIT-001..002 | 每阶段、未初始化、顺序、溢出、失败后状态、runtime defense-in-depth |
| FR-08 implementation_hint 非语义、非执行 | P1 | FULL | LANG-039；SIM-001；CLI-001, CLI-021 | 示例携带 hint 仍只按已建模语义检查/模拟；未知 action 项仍失败 |
| NFR-01 新语义具备正常、失败、边界测试 | P0 | FULL | 六份行为规格的公开 API/CLI 回归；LANG-001..053, SIM-001..022, CLI-001..023 | 新增语义均有反例和边界，不以单一 happy path 判定 |
| NFR-02 诊断、退出码、已承诺输出稳定 | P1 | FULL | CLI-001..023；LANG-007..038 | 精确 code/message/span/phase/必要 stdout-stderr 断言 |
| NFR-03 不可信源码不 panic、不执行代码或 IO | P0 | FULL | LANG-051..053；CLI-003..018；SIM-007, SIM-008, SIM-013 | 控制/非 ASCII/长行/整数边界/未初始化/溢出 |
| NFR-04 单文件、线性或接近线性目标 | P1 | PARTIAL | CLI-005..013, CLI-018 | 100KB 长行和有界窗口有回归；没有独立 parser 性能/复杂度基准，需源码审查补充 |
| NFR-05 core 标准库、CLI 单二进制 | P1 | FULL | workspace 全 target 构建与测试；Cargo manifests | core 无 dependencies，CLI 仅依赖 workspace core |
| NFR-06 语义只在 core，CLI 不复制规则 | P1 | FULL | CLI-001, CLI-020..023；`cargo tree`; CLI 调用路径源码审查 | CLI 直接调用 core 的 parse/check/simulate；AST pattern match 仅用于呈现，未复制检查规则 |
| NFR-07 格式、lint、测试、示例可复现 | P0 | FULL | 本地质量命令与 `.github/workflows/test.yml` | `--locked` 全回归和四命令闭环；远端托管运行另行判定 |

### 覆盖汇总

| 优先级 | 判据数 | FULL | PARTIAL | FULL 比例 |
|---|---:|---:|---:|---:|
| P0 | 10 | 10 | 0 | 100% |
| P1 | 5 | 4 | 1 | 80% |
| 合计 | 15 | 14 | 1 | 93.3% |

没有 P0/P1 `NONE` 项。唯一 PARTIAL 是测试和当前实现都不能证明的渐近性能目标；Step 4 结合源码和运行证据定级。

## Step 4：缺口与剩余风险分析

- 执行模式：`agent-team`；三个只读 worker 分别复核缺口分类、heuristics 和统计，主流程做确定性合并。
- uncovered：0；P0 gap：0；P1 gap：0；PARTIAL：NFR-04 一项。
- NFR-04 的长行测试只证明诊断 excerpt/marker 有界。semantic 去重/字段查找和每条 CLI 诊断位置扫描仍存在构造出近似二次工作的可能，因此不能宣称完整的规模增长证据。
- NFR-06 经 `cargo tree` 和调用路径审查补足：依赖方向仅 CLI → core，CLI 直接调用 `parse/check/simulate`，呈现层没有名称解析、类型检查、矛盾或模拟规则，故升为 FULL。
- endpoint/auth/UI heuristics 全部 N/A；错误路径不是 happy-path-only。
- 低风险 CLI 盲点：`simulate` 未单独进程测试 syntax/static-invalid，`parse`/`inspect` 未分别测试 invalid/read-error，帮助表面未测试。前两者复用同一 `run()`，第三项不属于 FR-06；记录为残余风险，不影响正式判据覆盖。
- NFR-03 由代表性恶意/边界输入证明，不是 fuzz/property 完备性承诺；保留 FULL，同时记录未知输入空间的残余风险。

### Phase 1 统计

- 15 项判据：14 FULL、1 PARTIAL、0 NONE，FULL 93.3%。
- P0：10/10 FULL；P1：4/5 FULL，1 PARTIAL。
- 唯一测试 inventory：100 active，23 E2E + 75 API + 2 Unit；0 skipped/pending/fixme。
- Phase 1 机器可读矩阵：`/tmp/tea-trace-coverage-matrix-2026-08-10T00-00-00-08-00.json`。

### 测试质量复核

- 100 个测试均为确定性同步 Rust 测试，无 sleep、网络或外部服务；断言位于测试体，CLI 临时文件在用例内清理。
- 无 ignore、pending、fixme、`should_panic`、`todo!` 或 `unimplemented!`；完整套件远低于 90 秒。
- defense-in-depth overlap 可接受：静态 checker 阻止无效模型，同时两个私有 unit test 保留 runtime type guard 分支证据。
- INFO：三个集成测试文件分别约 528/1147/475 行，超过通用“单文件 300 行”偏好，但每个独立测试远低于 300 行且按 CLI/language/simulation concern 分离；当前不构成行为或 flaky 风险。后续增长时可按主题拆分文件。

建议：把 NFR-04 保持为非阻断、可监测风险；若未来要承诺大文件性能，再先定义输入规模与增长阈值并增加 scaling benchmark。远端 CI 首次运行属于发布后的运维证据，不伪装为本地测试证据。

## Gate Decision：CONCERNS

**理由：** P0 覆盖 100%，总体 FULL 覆盖 93%（最低 80%），但 P1 FULL 覆盖 80%，达到最低线而低于 90% PASS 目标。

- P0：100%（MET）
- P1：80%（PARTIAL）
- Overall：93%（MET）
- Critical/high uncovered：0
- Gate 含义：v0.1 本地功能交付不被阻断；在对大文件性能作硬承诺前，应定义规模/增长阈值并补 scaling 证据。
- 机器可读结果：`e2e-trace-summary.json` 与 `gate-decision.json`。

### 当前工作树验证证据

- `cargo fmt --all -- --check`：PASS。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`：PASS。
- `cargo test --workspace --locked`：100/100 PASS；随后三轮完整 burn-in 均 PASS，0 flaky/ignored。
- `examples/order.morva` 的 check/parse/inspect/simulate：四命令均退出 0，模拟七阶段 PASS，`Pending -> Confirmed`。
- 两个新增 JSON、历史扫描 JSON、仓库内 Markdown 链接和 `git diff --check`：PASS。

### 剩余风险

| 风险 | 概率 | 影响 | 分数 | 处置 |
|---|---:|---:|---:|---|
| R-01 构造型大文件可能触发近似二次的 semantic/diagnostic 工作 | 2 | 2 | 4 | MONITOR；v0.1 不作硬规模/SLA 承诺。若进入大文件用例，先定义阈值并做 scaling benchmark/针对性优化。 |
| R-02 本地 CI workflow 尚未推送，缺少 GitHub hosted run 和 required-check 证据 | 3 | 2 | 6 | MITIGATE；不影响本地 v0.1 行为完成，但远端发布前由仓库维护者 push、观察首次运行，再启用 branch protection。 |

R-01 没有当前用户路径失败证据，且回滚容易；R-02 是授权边界外的远端运维动作，不能用本地自报伪装为已完成。

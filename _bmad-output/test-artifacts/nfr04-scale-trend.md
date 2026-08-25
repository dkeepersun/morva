# NFR-04 规模与趋势基线

- **日期：** 2026-08-26
- **主机：** macOS (Darwin 25.5.0), Apple Silicon, rustc 1.95.0
- **方法：** `crates/morva-core/tests/language.rs::nfr04_scale_trend_evidence` — 确定性合成模型（每 unit 含 enum、entity、action、scenario、兼容 module 各一），对 `parse + check` 取 best-of-5。该测试只断言正确性，不做墙钟比率门禁（NFR4 明确排除严格墙钟比率作为首个阻断门禁）。
- **可重复：** `cargo test -p morva-core --test language --release -- nfr04_scale_trend --ignored --nocapture`
- **常驻回归：** 非 ignored 测试 `nfr04_scaled_models_stay_clean_and_deterministic_across_scales` 在每次 CI 分片运行中验证 25/100 unit 模型解析、检查、分析的正确性与 AST 确定性。

## 实测（release）

| units | bytes | parse+check best-of-5 | 相邻比率 |
|---|---|---|---|
| 8 | 3241 | 44.6 µs | — |
| 16 | 6591 | 85.2 µs | 1.91 |
| 32 | 13375 | 159.3 µs | 1.87 |
| 64 | 26943 | 317.0 µs | 1.99 |
| 128 | 54667 | 623.1 µs | 1.97 |

## 实测（dev，同套件）

| units | parse+check best-of-5 | 相邻比率 |
|---|---|---|
| 8 | 229.5 µs | — |
| 16 | 451.3 µs | 1.97 |
| 32 | 892.9 µs | 1.98 |
| 64 | 1.807 ms | 2.02 |
| 128 | 3.570 ms | 1.98 |

## 结论

在 8–128 unit（3 KB–55 KB）区间内，输入规模翻倍时 parse+check 耗时比率稳定在 1.87–2.02，两种编译 profile 一致——与 NFR-04 的"随输入总量线性或接近线性"目标相符。趋势基线已可重复生成；是否将某个比率上界升级为阻断门禁，留待多主机/多次实测稳定性评估后另行决策。

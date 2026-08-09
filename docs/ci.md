# CI 质量门禁

Morva 已在本地提交中配置 GitHub Actions 的 [Test Pipeline](../.github/workflows/test.yml)，用于验证 `main` push、pull request、手动运行和每周定时运行。截至 2026-08-10 本次核对，该提交尚未推送，工作流尚未由 GitHub 托管运行，`Quality gate` 也尚未设为 branch protection 必需检查；推送或配置保护规则后必须同步更新本节。工作流只需要仓库读权限，不使用 secret、外部服务或部署凭据。

## 门禁结构

- `Format and Clippy`：执行 `cargo fmt --all -- --check` 和严格 workspace Clippy。
- `Test (*)`：按 core language、core simulation 和 CLI process 三个职责分片并行运行全部 Rust 测试。
- `Executable examples`：顺序运行 `check` / `parse` / `inspect` / `simulate` 四命令闭环。
- `Burn-in`：仅在每周定时或手动触发时连续执行三次 workspace tests，首次失败立即终止。
- `Quality gate`：无论上游成败都生成 step summary，任一必需 job 非 `success` 则失败。

项目使用 Rust 内建 test harness，当前不产生 JUnit/HTML 报告，因此不上传空 artifact。失败证据保留在具名 matrix job 日志和最终 summary 中。不配置自动重试；burn-in 的每次失败都直接失败，避免掩盖真实问题。

## 本地对齐

提交前执行：

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p morva-cli -- check examples/order.morva
cargo run --locked -p morva-cli -- parse examples/order.morva
cargo run --locked -p morva-cli -- inspect examples/order.morva
cargo run --locked -p morva-cli -- simulate examples/order.morva NormalConfirmation
```

CI 运行过一次后，应在 GitHub branch protection 中把 `Quality gate` 设为必需检查。工作流固定 `actions/checkout` 的 commit SHA；升级时应同时更新 SHA 与版本注释。

# 贡献指南

感谢参与 CTPBuddy。开始前先读 [README](README.md) 的「它解决什么问题 / 当前边界」与 [DESIGN.md §0.6](DESIGN.md)（当前实现与设计差距的集中登记）。

## 开发环境

- Rust：见 [rust-toolchain.toml](rust-toolchain.toml) 钉定的 stable（MSRV 1.89，写在 `core/Cargo.toml`）；组件含 rustfmt、clippy。
- Python：uv workspace，`.python-version` 选择 3.13，运行时支持回退到 3.9；**运行时零第三方依赖**，仅 e2e 的 ZMQ 套件可选装 `pyzmq`。

```bash
uv sync --locked --all-packages
cargo build --manifest-path core/Cargo.toml
```

## 提交前必跑

```bash
# Rust：格式、lint、全量单测、确定性 fuzz
cargo fmt --manifest-path core/Cargo.toml --all -- --check
cargo clippy --locked --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings
cargo test --locked --workspace --manifest-path core/Cargo.toml

# Python：单测 + lint + 契约向量
uv run ruff check .
uv run python tests/test_py.py
uv run python tests/test_journal_contract.py

# 端到端（每套自起 server；需要外部真实数据的套件缺源时以退出码 3 SKIP）
uv run python tests/e2e/m1_smoke.py
uv run python tests/e2e/review_regressions.py
```

改动了什么面，就必须跑对应门禁：撮合回报 → 相应 exchange layout 的 e2e 顺序断言；账本 → query 投影 + journal + 日结测试；Shim → `m1_shim_e2e.py` 与 `shim_hardening.py`；Web → Host/Origin/CSRF、路径穿越、重复键、body 上限相关单测；CI 矩阵见文档站[「测试与贡献」页](docs-site-oink/content/development/testing-ci-contributing.md)。

## 项目约定（容易踩的）

1. **不要手改生成产物**。新增 wire 消息先改 `core/ctpbuddy-wire/src/msgs.rs` 的 id 指派与生成器（`shim/codegen/`、`registry.json`），重新生成三端镜像；CI 用 `git diff --exit-code` 卡生成物漂移。
2. **Python 不实现撮合与账本**，Rust 不碰 SQLite：写路径一律经 ADMIN 进 core；Python 层只做校验、投影、控制面。
3. **错误码 / 错误串逐字对齐官方 error.xml**，不凭记忆写；新增状态要同步更新 `docs/错误码全集.md`（`tools/fill_errorcode_status.py --check` 可校验）。
4. **诚实的 skip**：无外部数据/依赖时退出码 3 显式 SKIPPED，绝不冒充 pass；真实数据审计报告只写匿名化结果。
5. **行为保持类重构**要保证帧字节、journal JSON、错误串、断言集合与 CI 覆盖集合不变；大改动先写 spec（参考 `docs/specs/`）。
6. 金额相关逻辑使用 `ctpbuddy-ledger` 的定点 `Money`，不要引入新的 f64 求和路径。
7. 提交信息引用真实源码路径、测试命令与边界；中文或英文均可。

## 文档

- 设计口径变化追加到 DESIGN 对应章节与 §0.6，不静默改写历史落地口径（重大决策走 ADR，见 `docs/adr/`）。
- 统一术语见 [GLOSSARY.md](GLOSSARY.md)；不要把未运行的验收、未完成的功能写成完成。

## 许可

提交即表示你同意你的代码以项目 [MIT 许可证](LICENSE) 发布。

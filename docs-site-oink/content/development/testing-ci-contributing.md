---
title: 测试、CI 与贡献入口
linkTitle: 测试与贡献
weight: 60
description: 从测试目录、GitHub Actions 和源码反向映射说明如何验证修改。
---

测试入口来自 `tests/test_py.py`、`tests/e2e/`、`core/**/src` 单测以及 `.github/workflows/core-tests.yml`、`docs-site.yml`。主要来源提交为 `b9b8fbd`、`12d55e8`、`6d2afe2` 和 `4d2af40`。

## 本地验证层级

```bash
python tests/test_py.py
cargo test --locked --workspace --manifest-path core/Cargo.toml
cargo build --locked --bin ctpbuddy-server --manifest-path core/Cargo.toml
python tests/e2e/m1_smoke.py
python tests/e2e/m2_book.py
python tests/e2e/m2_scenario.py
python tests/e2e/m2_journal.py
python tests/e2e/m3_settlement.py
python tests/e2e/m3_fee_cffex.py
python tests/e2e/m3_order_sysid.py
python tests/e2e/m4_real_replay.py
```

M1 覆盖登录/订阅/报单穿透；M2 覆盖限价簿、价格时间优先、FAK/FOK、场景、journal；M3 覆盖 RefData、detail/FIFO、显式日结、OrderSysID、SQLite 投影；M4 真实回放只验收受控普通期货子集。没有外部真实导出时，真实回放按源码约定 SKIP，不应改成“全绿即全覆盖”。

## CI 实际矩阵

`core-tests.yml` 在 Windows 上使用 Python 3.9 和 stable Rust。unit job 安装 `./py`、运行 `ctpbuddy --version` 与 assertions help、Rust workspace test/build、Python 单测。e2e matrix 当前包含 `m1_smoke`、`m2_scenario`、`m2_journal`、`m3_bootstrap`、`m3_5_settlement_info`、`m3_6_projection`、`m3_settlement`、`m3_fee_cffex`、`m3_order_sysid`、`m4_real_replay`、`settings`；每项自建本地 core，不依赖官方 SDK 或外部行情。

`docs-site.yml` 使用 Hugo Extended 0.167.0，执行 `--gc --minify --printPathWarnings --panicOnWarning`，再检查 index、CSS 和未跟踪的 public/resources。CI 不是 Docker 或 Windows Shim 真实发布验收的替代物。

## 贡献规则

新增 wire 消息先改 `msgs.rs`/生成器和三端 registry，不直接手改 `generated/`；改撮合回报要补对应 exchange layout 与 e2e 顺序断言；改账本要同时核对 query projection、journal 和日结测试；改 Web 要覆盖 Host/Origin/CSRF、路径穿越、重复键、body 上限和错误码。

提交说明应引用真实源码路径、测试命令和边界；真实审计必须匿名化并区分 `source alignment`、`controlled replay`、`core ledger replay`。不要复制官方 HTML 全集到 OINK，也不要把未运行的 Docker、未取得的夜盘日历或未完成的完整账本写成完成。

## 源码反向索引

- 协议：`core/ctpbuddy-wire/src/frame.rs`、`msgs.rs`、`py/ctpbuddy/wire.py`。
- 场景时间：`core/ctpbuddy-market/src/lib.rs`、`transform.rs`、`py/ctpbuddy/scenario.py`。
- 撮合回报：`core/ctpbuddy-matching/src/engine.rs`。
- 账本日结：`core/ctpbuddy-ledger/src/lib.rs`、`core/ctpbuddy-server/src/admin.rs`。
- 查询审计：`py/ctpbuddy/sdk/client.py`、`store.py`、`tools/audit_query_expectations.py`。
- Web：`py/ctpbuddy/web.py`；Docker/发布：`tools/docker_acceptance.py`、`tools/release_package.py`。

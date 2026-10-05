# 变更与进度日志

> 自 README「当前进展」迁出。里程碑清单保留原文，含 CI run id / 提交号等历史记录；新条目请追加在最上方。

## 2026-10（本轮）

- 新增 Linux Shim（`.so`）、Linux CI 作业；连接数上限（`CTPBUDDY_MAX_CONNS`）与写超时；可选 `CTPBUDDY_TD_TOKEN`。
- 账本金额改定点数 `Money(i64)`，JSON 换 `serde_json`，ledger/handlers/engine 拆分；journal fsync 策略与 `--recover` 快照恢复；恢复时 journal 写入 `ledger_recovered`（含 `working_orders_voided`）审计标记。
- 新增 wire 解码与 World 的确定性 fuzz 测试；无深度行情上的限价单仅在最新价穿越限价时成交；可选 `CTPBUDDY_MAKER_AT_LIMIT=1` 让挂单按自身限价成交。

## 历史里程碑



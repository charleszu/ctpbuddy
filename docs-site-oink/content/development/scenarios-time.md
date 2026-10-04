---
title: 场景、行情源与虚拟时间
linkTitle: 场景与虚拟时间
weight: 30
description: 说明 CSV tick、Playback、scenario DSL 和日历边界如何连接到 Rust 世界循环。
---

本页对应 `core/ctpbuddy-market/src/lib.rs`、`transform.rs`、`core/ctpbuddy-server/src/scenario.rs` 与 `py/ctpbuddy/scenario.py`。主要来源提交为 `10e0543`（场景 DSL 与播放控制）和 `4d2af40`（日历与 Web 约束）。

## CSV 到 Tick

`CsvSource::load(path)` 把 CSV 转为 `Tick`。Tick 暴露 `virtual_ms()`、`best_bid()`、`best_ask()` 和 `to_depth_md()`；播放顺序是文件顺序，源码不会按墙钟重新排序。Python `ctpbuddy.sources.csv_source.validate_scenario()` 负责检查场景文件，`write_canonical()` 写回规范化字段。

`transform.rs` 的 `Transform` 通过 `apply_all(ticks, transforms)` 生成测试行情变化。它是行情注入层，不感知 CTP 会话或账户账本；缺失/非法源文件应在场景校验阶段失败，而不是由撮合器猜测。

## Playback API

`Playback::new(ticks, speed)` 创建状态，`speed=0` 表示尽可能快。实际可用方法：

- `pause()` / `resume()`：暂停或恢复。
- `step()`：设置一次性释放标志；下一次 `poll()` 最多返回一个 tick。
- `seek(target_ms)`：定位到第一个 `virtual_ms >= target_ms` 的 tick，跳过的 tick 不会补发。
- `set_speed(0..=1000)`、`set_loop(bool)`、`progress()`、`virtual_time()`、`finished()`。
- `poll(now: Instant)`：按速度释放到期 tick；同一时间窗仍按文件顺序返回。

`core/ctpbuddy-matching/src/engine.rs:ClockCtx` 只接收 `trading_day` 和 `now_ms`，撮合器不读取系统时钟。没有场景时，server 才会使用 `dtime.rs:today_trading_day()` 的 UTC 日期规则；20:00 以后滚到下一自然日只是无场景兜底，不是完整交易所日历。

## ADMIN 控制

```python
with Admin() as a:
    a.start_scenario("scenarios/rb_demo", paused=True, speed=0)
    a.seek("09:00:00")
    a.step()
    a.set_speed(10)
    a.loop(False)
```

Rust `start_scenario` 要求目录存在，可接收已规范化 `spec`；`seek` 未加载场景时报错。循环只重置行情游标和虚拟时间，**不重置撮合器、账户或账本**；循环不提供状态隔离；重新加载的初仓约束与显式 reset 的影响必须分别核对，不能把 seek 或 loop 当作账户重置。日结后 `Playback::advance_trading_day(next_day)` 丢弃旧日行情、回到暂停、关闭循环，避免旧行情污染新日。

## 日历边界

`py/ctpbuddy/calendar.py:TradingCalendar` 只读取用户提供的 JSON 快照，可校验固定 SHA256；`Admin.settle_day()` 省略 `next_trading_day` 时调用 `calendar.next_trading_day(current)`。仓库生产样本明确是 2026 日盘候选，`scope: day-session-only`，不是六所完整 TradingDay 数据库；夜盘未知边界不得推断。文档和测试应把“离线日盘快照”写清楚，不能把它描述成交易所官方全集。

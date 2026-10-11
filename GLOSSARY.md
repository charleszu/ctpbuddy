# CTPBuddy 术语表

本文统一 Rust core、Python control plane、journal 和 Web read model 使用的领域词汇。

| 术语 | 约定含义 |
|---|---|
| **自然日 / calendar date** | 交易所所在地的公历日期，格式为 `YYYY-MM-DD`。它描述行情实际发生的日期。 |
| **TradingDay** | CTP 交易日，格式为 `YYYYMMDD`。夜盘通常归属于下一交易日；它是结算、账本和 journal 的业务日。 |
| **ActionDay** | 行情实际打印所在自然日，格式为 `YYYYMMDD`。缺省时 canonical tick 将其视为 TradingDay；夜盘数据应显式提供。 |
| **virtual time / 虚拟时间** | 回放时由 tick 的 `update_time` 和 `update_millisec` 映射出的交易日内时间轴。它不等同于机器墙上时钟。 |
| **canonical tick** | 由行情源适配器输出的固定列布局 tick。CSV、Parquet 或插件都必须先转换到这个形状，核心只消费该形状。 |
| **canonical journal contract** | journal 事件名称、envelope、必填字段、规范化、hash、replay 和投影规则的共同契约，规范入口见 `core/ctpbuddy-server/src/journal_contract.rs`。 |
| **journal event** | 已写入 append-only JSONL 的业务事实。事件序号 `seq` 单调递增，不能用实时查询结果反推。 |
| **replay** | 根据 journal 中可重放的输入事件重新驱动核心。不可重放的 ADMIN 输入必须显式计数并标记 `not_replayed`。 |
| **projection / read model** | 从 journal 构建的查询模型，例如 SQLite。它可能落后 live core，必须带自己的 sequence、virtual time 和来源元数据。 |
| **provider** | 把外部参考数据或行情数据转换为项目 canonical contract 的适配器。provider 不拥有核心业务状态。 |
| **provenance** | 用 version、SHA256 和 source 描述离线日历或参考数据的来源和内容身份。只写一个版本字符串不足以证明内容一致。 |
| **World** | Rust server 的单线程交易编排状态，持有 engine、ledger、journal、playback 和连接生命周期；外部协议应通过较窄的 handler/adapter seam 访问它。 |

# 变更与进度日志

> 自 README「当前进展」迁出。里程碑清单保留原文，含 CI run id / 提交号等历史记录；新条目请追加在最上方。

## 2026-10（本轮）

- **中金所平今费时间序池落地**（知识库 §10.4 #10 修正，生产数据 81/81 判别）：`ExchangeRules.fee_close_pool`（仅 CFFEX）+ `Position.fee_open_pool`——开仓逐笔累池、平仓取 `min(手数, 池)` 作平今手数，与明细的先开先平消耗正交（昨仓被平收平今费、池尽后平今仓收平昨费）；日结随 `today_position` 同点清池；快照 v2 携带池状态，恢复兼容 v1 并按交易日重建当日开仓池。锁定：ledger 单测（三段分叉 + DCE 对照 + 日结清池）、e2e `m3_fee_cffex.py`（bootstrap 昨仓 + scenario 自带 `refdata/commission_rates.jsonl` 走完整 server）、`tools/audit_real_accounts.py` 复跑全绿（459/459、976/983、64/64）。
- 日结后持仓保证金按结算价重估（昨仓恒用昨结算价，逐明细按剩余手数重算）；交易所差异集中到 `exchange_rules.rs` 规则表；`SubscribePrivateTopic` 的 RESTART/RESUME 经 AUTH 的 `private_resume` 生效，登录后重放私有流。

- 新增 Linux Shim（`.so`）、Linux CI 作业；连接数上限（`CTPBUDDY_MAX_CONNS`）与写超时；可选 `CTPBUDDY_TD_TOKEN`。
- 账本金额改定点数 `Money(i64)`，JSON 换 `serde_json`，ledger/handlers/engine 拆分；journal fsync 策略与 `--recover` 快照恢复；恢复时 journal 写入 `ledger_recovered`（含 `working_orders_voided`）审计标记。
- 新增 wire 解码与 World 的确定性 fuzz 测试；无深度行情上的限价单仅在最新价穿越限价时成交；可选 `CTPBUDDY_MAKER_AT_LIMIT=1` 让挂单按自身限价成交。

## 历史里程碑

M1（核心闭环 + Shim 全链路）已完成：

- [x] 仓库骨架、设计文档（含存储分层与表结构定稿）
- [x] 头文件 codegen（ctpsdk 6.7.13 → Rust/Python/C++ 三端结构体镜像）
- [x] Rust 核心：wire 帧 / market（CSV 源 + 虚拟时钟）/ matching（即时成交 + 限价簿）/ ledger / server（TCP 传输占位 + JSONL journal）
- [x] Python 包：wire / SDK / CLI / 场景源（SDK 对查询流控 90 透明重试）
- [x] e2e 冒烟测试（SDK 登录 → 订阅 → 报单穿透 → 断言成交与资金扣减）
- [x] C++ Shim：139+14 个纯虚方法全 override、519 项编译期布局校验、真实下游 demo（stock 厂商头文件 + 链接 Shim import lib）全链路通过 `tests/e2e/m1_shim_e2e.py`——含 CTP 两类查询流控真实触发（在途 -2 + 每秒 QryFreq 90 重试）
- [ ] ZeroMQ 传输适配（当前为 TCP 占位，帧协议一致）
- [x] 任务41最小初仓闭环：真实 core 启动导入逐笔 SHFE 昨仓、静态 `YdPosition`、平昨保持静态值、零余量 detail 过滤、无初仓流水；SHFE/INE 查询按逐笔 detail 年龄桶聚合，非 SHFE 单行
- [x] 任务42第一阶段显式日结：ADMIN `settle_day` 接收用户供给的 `settlement_prices` 与严格递增 `next_trading_day`；要求 playback 暂停且无活动订单；账本先暂存校验再原子替换，最终按供给结算价盯市，动态权益滚存至 `PreBalance`，清零当日资金/盈亏/手续费/冻结，今仓转静态昨仓，清理当日订单成交与旧确认；不自动按固定时刻触发、不复用 `reset_account`、不伪造 SettlementInfo 查询
- [x] 任务43 `OrderSysID`：内部订单号继续用于引擎/账本/journal关联；首条对外 OrderSysID 为空，按交易所布局在 accepted 边界后填充，交易所拒单全程为空；Trade/QryOrder/撤单使用最终非空系统号，覆盖 SHFE GFD、拒单、DCE/FAK 布局与最终关联
- [x] M3-5（本地）：结算单原始 GBK 供给与 `ReqQrySettlementInfo` 分段查询；`tests/e2e/m3_5_settlement_info.py` 已加入 CI 矩阵，本地通过，远端 CI 已核实成功（37157525169 / 3d42505、37124374367 / 12d55e8、37124336391 / 394bff9）
- [x] M3-6（本地 + 已验证远端）：JSONL journal 的 SQLite 投影、原子 rebuild 与只读 CLI/Web 查询；设置页已增加账户/持仓/资金快照/订单/成交/审计/结算报告只读浏览、broker/investor/day 筛选与分页；Web API 采用白名单、textContent 防注入、非实时快照边界和路径安全错误；仅开放带确认/CSRF/严格校验/审计的 `settlement_report`、`settle_day` 写入口，不暴露 reset/shutdown/任意 admin/SQL；`tests/e2e/m3_6_projection.py` 已加入 CI 矩阵，本地通过，远端 CI `37157525169`（`3d42505`）、`37124374367`（`12d55e8`）、`37124336391`（`394bff9`）均 success
- [ ] M4 交付：断言 CLI、OINK 文档站、Shim真实构建基础、真实成交受控回放、三表源对齐第一阶段已有；完整账户Core回放仍受期权/RefData/初始结算状态建模限制，三渠道发布与完整发布包仍未完成，不将 M4 总项标为完成

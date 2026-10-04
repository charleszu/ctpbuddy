# CTPBuddy

[![repo](https://img.shields.io/badge/repo-github.com%2Fcharleszu%2Fctpbuddy-blue)](https://github.com/charleszu/ctpbuddy)

**本地 / 私有部署的 CTP 兼容仿真交易环境** —— 为 CTP 下游系统（策略、交易终端、条件单）提供确定性、可注入、可共享的测试基础设施。

> 通用 CTP 开发指南见 [`docs/CTP开发知识库/README.md`](docs/CTP开发知识库/README.md)；它用于分层检索和工程入门，**不是官方权威**。官方 HTML 与项目考证 notes 的优先级、适用基线及真实样本边界见该 README；项目进度仍以本文件和 [`DESIGN.md`](DESIGN.md) 为准。

> 完整设计见 [DESIGN.md](DESIGN.md)。**CTP 语义知识库（流控/生命周期/会话/报单回报时序/状态机/资金持仓/保证金/行情/结算）见 [docs/CTP语义知识库.md](docs/CTP语义知识库.md)**，含 M2 实现清单；深度原始笔记在 `docs/notes/`。官方资料可读版：SDK《6.7.13_API接口说明》HTML 版（405 页干净 HTML，剔除 CHM 主题框架、保留表格/代码/内嵌图片，页间链接与 `anchor-id-*` 锚点均已校验；另有 1 页目录漏收附录、8 个官方附件与 84 条官方源死链/1 条悬空锚点的公示）在 [docs/api-doc-html/](docs/api-doc-html/)，error.xml 错误码全集（299 条，逐条标注「已实现 19 / 可落地 51 / 暂不可达 229」+ 推送面）在 [docs/错误码全集.md](docs/错误码全集.md)，双推送面口径见 [docs/notes/09](docs/notes/09-错单推送面与错误码对账.md)；FAK 部成部撤的**三所分流**回报（上期所/大商所+广期所/郑商所三种不同形状）见 [docs/notes/10](docs/notes/10-FAK回报按交易所分流.md)；程序化交易入门系列 17 份客户端实操资料（连接认证/穿透式监管、行情现手开平、报撤单成交回报、查询流控与持仓更新）的整理与实现影响清单见 [docs/notes/11](docs/notes/11-入门系列-连接认证与穿透式监管.md)~[docs/notes/14](docs/notes/14-入门系列-查询流控与持仓查询更新.md)（汇总登记在知识库 §10.4）。CTPBuddy 与上海期货信息技术有限公司无任何隶属关系；本项目不附带任何官方 SDK 文件，`ctpsdk/` 目录中的头文件由使用者自备、禁止入库与分发。

## 它解决什么问题

SimNow / openctp 都是远程 CS 模式：网络绑死、无法注入极端行情、不可复现、无管理面。CTPBuddy 把柜台搬到本地：替换同名 DLL 即可接入，行情可回放、场景可注入、账户可管理、团队可共用。

## 架构速览

```
下游系统 ──(同名 DLL 替换)── CTPBuddy Shim (C++) ──ZeroMQ/帧协议── Rust 核心服务
                                                                    ├─ 行情回放引擎（源由用户提供）
                                                                    ├─ 撮合引擎（默认单线程确定性）
                                                                    └─ 账户账本（每账户单写者）
Python 层 (pip install ctpbuddy): CLI / SDK 断言 / FastAPI Web 后台 / 行情源插件
```

## 当前进展

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
- [ ] M4 交付：断言 CLI 有本地实现和单测；当前只完成部分 e2e CI，三渠道发布与文档站仍未完成，不将 M4 总项标为完成

## 快速开始

```bash
# 1. 构建核心服务（Rust 1.98+）
cd core && cargo build

# 2. 安装 Python 层（开发模式）
pip install -e ./py

# 3. 跑测试（自动拉起 server；Shim e2e 需先构建 Shim）
python tests/test_py.py          # Python 侧单测：帧编解码 / 结构体布局 / 场景校验
cd core && cargo test            # Rust 侧单测：场景 DSL / 行情 transform / FAK 三所回报布局
cd .. && python tests/e2e/m1_smoke.py     # 端到端：登录→订阅→报单穿透→断言成交与资金
python shim/build_msvc.py --demo # MSVC 构建 Shim DLL + 真实下游 demo（Windows）
python tests/e2e/m1_shim_e2e.py  # 真实 CTP 应用经 Shim 打穿核心的全链路

# M2 全量 e2e（每套自起 server，互不依赖）
python tests/e2e/m2_book.py      # 限价簿撮合：价格/时间优先、FAK/FOK、冻结闭环
python tests/e2e/m2_scenario.py  # 场景 DSL 管道与播放控制
python tests/e2e/m2_journal.py   # journal 录制/重放与确定性 core hash
python tests/e2e/m2_flow.py      # 报单流控 + 订单状态机（双服务器）
python tests/e2e/m2_surface.py   # 错单双推送面 + 流控面（--order-freq 2）
python tests/e2e/m2_ioc.py       # FAK 部成部撤的三所分流回报（官方场景 8/9/10）
python tests/e2e/m3_refdata.py    # 四张费率查询 + 与账本 CurrMargin/Commission 交叉核对
python tests/e2e/m3_detail.py     # 持仓明细：逐笔盈亏 + 先开先平
python tests/e2e/m3_settlement.py # 显式日结：结算价、今仓转昨仓、账户滚存与确认门禁
python tests/e2e/m3_order_sysid.py # 任务43：OrderSysID 接受边界、拒单空号、Trade/QryOrder最终关联

# M3-6：从 Rust 写入的 JSONL journal 构建 Python 标准库 SQLite 查询投影
ctpbuddy journal rebuild ./data/journal --db ./data/ctpbuddy.db
ctpbuddy journal query account --db ./data/ctpbuddy.db --broker 8888
ctpbuddy journal query order_record --db ./data/ctpbuddy.db --investor test01

# 4. 手动起一套玩玩（示例场景 rb_demo）
ctpbuddy serve --scenario scenarios/rb_demo --data-dir ./data
ctpbuddy status                  # 另开一个终端
```

真实柜台数据对账（可选，需自行提供导出目录）：

```
python tools/audit_real_accounts.py    # 资金恒等式逐项核对真实账户日与结算单
```

读取期货公司的账户导出（order/trade/account 三表）与盯市结算单，逐项核对
`Balance` / `Available` / 结算单权益恒等式、盯市盈亏的逐笔求和、平今平昨盈亏口径。
目录可用 `CTPBUDDY_EXPORT_DIR` / `CTPBUDDY_SETTLEMENT_DIR` 指定；未提供则跳过，
不影响上面的回归。口径细节见 DESIGN §8.7.1。

M3-6 存储投影：Rust 核心只写 `data/journal/*.jsonl`，Python 标准库 `sqlite3` 通过 `ctpbuddy journal rebuild` 原子生成 `data/ctpbuddy.db`；数据库可以直接删除后重建，重建依据 journal 顺序与 full hash，结果确定。`journal query` 和配置了 `--db` 的 Web `/api/projection?table=...` 只读查询账户、持仓/资金快照、订单、成交、审计及用户供给的结算报告；快照非实时，缺失字段不编造。SQLite 损坏时不要修复数据文件，删除后重新 rebuild。

核心支持 `--qry-freq <n>`（env `CTPBUDDY_QRY_FREQ`）调前置每秒查询预算：超过即回 `OnRspError[90]`「CTP：查询未就绪，请稍后重试」，与真实 CTP 前置一致（DESIGN §8.8）。除行情与 RefData 外，可用本地 Web 后台配置已实现的柜台参数：`ctpbuddy web --host loopback --port 8080 --admin 127.0.0.1:5561`。可配置 `qry_freq`、`order_freq`（报单/撤单独立额度）、`max_user_sessions`（0 为关闭限制）、`settlement_required`（默认开启，未确认报单返回官方 42「CTP:结算结果未确认」）、`initial_funds`（仅首次开户）。Web 通过 ADMIN 单一真相读写，绑定回环地址、同源/CSRF、严格 schema 校验；设置写入 `data_dir/settings.json` 并原子替换，已有账户资金不改。启动覆盖优先级为 CLI > env > 持久化 > 默认，覆盖字段会明确显示；不可用 data_dir 时更新拒绝。

## 参考数据（合约 / 保证金 / 手续费）

费率与合约参数**一律由使用者提供，核心不内置编造值**。字段集合直接以官方查询返回结构体为依据（`ReqQryInstrument` + 三张费率表 + `ReqQryBrokerTradingParams`），核心用同一份数据既算账也应答查询——客户端拿 `ReqQryInstrumentMarginRate` 与自己的 `ReqQryTradingAccount.CurrMargin` 交叉核对时，数字必然一致。

```bash
# 内置快照：789 个真实期货合约（六所全覆盖）+ 公司保证金率
ctpbuddy refdata show refdata

# 从你自己的数据导出（厂商表格 → CTP 词汇，编码/列名由 provider 负责翻译）
ctpbuddy refdata export --kind csv --path ./desk_export --out ./myrefdata
ctpbuddy serve --scenario scenarios/rb_demo --refdata ./myrefdata --data-dir ./data
```

provider 是鸭子类型的：实现任意子集表方法即可，**没实现的方法视为「本 desk 无此规则」而非错误**（没配手续费就是零手续费，这是合法柜台配置）。随包快照**刻意不含手续费表**——编造的手续费比没有更糟。

取用优先级：`--refdata` / `CTPBUDDY_REFDATA` > `<scenario>/refdata/` > 随包快照。口径详见 DESIGN §8.6.1。

## 期货交易日历（Python，离线）

`ctpbuddy.calendar.TradingCalendar` 只读取用户提供的 JSON 快照，运行时不访问网络、不安装第三方依赖。快照必须带固定 `version`、`source`、`scope: futures` 和 `days`；GitHub 项目只能作为人工/离线生成快照的数据源，不能把股票交易所日历直接当作期货日历。

```python
from ctpbuddy.calendar import TradingCalendar
from ctpbuddy.sdk import Admin

calendar = TradingCalendar.from_file("calendar.json", expected_sha256="...")
with Admin(calendar=calendar) as admin:
    admin.settle_day({"rb2601": 3501.0})  # 从 status 当前 TradingDay 推导下一期货交易日
```

最小快照格式如下（**虚构测试 fixture，不代表真实休市安排**）：

```json
{
  "schema": "ctpbuddy.trading-calendar/v1",
  "version": "fixture-r1",
  "source": {"kind": "fixture", "name": "offline-demo", "revision": "r1", "license": "test-only", "scope": "futures"},
  "days": [
    {"date": "2026-10-02", "is_trading_day": true, "trading_day": "20261002", "exchanges": {"SHFE": {"night_action_day": "20261002", "night_trading_day": "20261005"}}},
    {"date": "2026-10-03", "is_trading_day": false},
    {"date": "2026-10-04", "is_trading_day": false},
    {"date": "2026-10-05", "is_trading_day": true, "trading_day": "20261005"}
  ]
}
```

`ctpbuddy calendar validate calendar.json` 输出规范化内容 SHA256 和实际覆盖范围；再用 `--sha256 <固定值>` 校验。SHA256 不包含 JSON 缩进差异，但包含版本、来源和全部映射。来源元数据只是可追溯声明，结构校验不能证明市场数据权威性，真实快照需由用户核验。GitHub 来源需 `kind: github`、`name: owner/repo`、`revision: <40位commit SHA>`、明确 `license` 和 `scope: futures`。官方来源需 `kind: official`、固定公告 URL 和固定摘录 revision。

自然日使用 `YYYY-MM-DD`，期货 `TradingDay` 使用 `YYYYMMDD`；`next_trading_day` 只返回快照明确标记的期货交易日。夜盘必须在 `days[].exchanges[EXCHANGE]` 中显式标记：开放时同时提供 `night_action_day` 和 `night_trading_day`，官方明确关闭时使用 `{"status":"closed"}`；缺失时拒绝查询，绝不从周末、股票休市表或交易所名称推断。显式传入 `Admin.settle_day(..., next_trading_day="YYYYMMDD")` 仍兼容旧调用。

生产目录 `calendar/production/` 当前只提交了 `shfe-2026-new-year.sample.json` 和 `MANIFEST.json`：其固定来源为上期所 2025-12-17〔2025〕157号公告，覆盖 2025-12-31 至 2026-01-05 的元旦样本及 2025-12-31 夜盘关闭边界，快照 SHA256 为 `3a1929f644c903ffaea2f8751e9f998a2be8cd71d9937c0e321f1aa9b768b67a`。这不是完整 2026 生产日历；manifest 明确记录了缺口，不能把缺失日期解释为休市，也不能离线自动补全。

用户覆盖通过 `calendar.with_overrides(user_snapshot)` 或 `load_calendar(path, override=...)` 完成，按自然日整条替换并重新计算快照 SHA256。推荐在 CI 中固定并校验 SHA256；不要把未核验的外部数据写入仓库。

已核查的外部数据源示例：`gerrymanoim/exchange_calendars`，Apache-2.0，固定 commit `bbda29fed902374bdb75acab008f421fbd567823`。其 README 明确定位为证券交易所日历、日历由用户贡献维护，并将常规交易时段外（含盘前/盘后/竞价/午休）视为关闭；仓库包含上海证券交易所 XSHG，但不提供 CTP 期货夜盘 ActionDay/TradingDay 语义。因此本项目不在运行时依赖它，也不将其数据直接作为期货快照。

## 结构

```
ctpsdk/                  # 使用者自备的官方 SDK（禁止入库，见 .gitignore）
docs/                    # 设计文档 + CTP 语义知识库 + 官方 API 文档/错误码全集（CHM 与 error.xml 的可读化）
shim/                    # C++ Shim（DLL/so 同名替换）+ codegen
core/                    # Rust workspace：wire / market / matching / ledger / server
py/                      # Python 包 ctpbuddy
scenarios/               # 示例场景与 tick 数据
tests/e2e/               # 端到端测试
docker/                  # 团队部署
```

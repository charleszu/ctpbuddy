# CTPBuddy

[![repo](https://img.shields.io/badge/repo-github.com%2Fcharleszu%2Fctpbuddy-blue)](https://github.com/charleszu/ctpbuddy)

**本地 / 私有部署的 CTP 兼容仿真交易环境** —— 为 CTP 下游系统（策略、交易终端、条件单）提供确定性、可注入、可共享的测试基础设施。

> 通用 CTP 开发指南见 [`docs/CTP开发知识库/README.md`](docs/CTP开发知识库/README.md)；它用于分层检索和工程入门，**不是官方权威**。官方 HTML 与项目考证 notes 的优先级、适用基线及真实样本边界见该 README；项目进度仍以本文件和 [`DESIGN.md`](DESIGN.md) 为准。

> 柜台运行所需的外部数据、目录格式、加载优先级和客户端挂载链路见 [`docs/CTP柜台外部数据与挂载说明.md`](docs/CTP柜台外部数据与挂载说明.md)。

> 完整设计见 [DESIGN.md](DESIGN.md)。源码反向映射索引见 [`docs-site-oink/content/architecture/source-layout.md`](docs-site-oink/content/architecture/source-layout.md)，按协议、场景时间、撮合回报、账本日结、查询审计、Web 安全、Docker/Shim 发布和测试 CI 指向实际模块路径。**CTP 语义知识库（流控/生命周期/会话/报单回报时序/状态机/资金持仓/保证金/行情/结算）见 [docs/CTP语义知识库.md](docs/CTP语义知识库.md)**，含 M2 实现清单；深度原始笔记在 `docs/notes/`（编号 01~05、09~14；06~08 跳号：06「API 文档 Markdown 转换」与 07「错误码全集」已分别上移为 `docs/api-doc-html/` 与 `docs/错误码全集.md`（提交 `17e2f8e`），08 从未发布，编号不回收）。官方资料可读版：SDK《6.7.13_API接口说明》HTML 版（405 页干净 HTML，剔除 CHM 主题框架、保留表格/代码/内嵌图片，页间链接与 `anchor-id-*` 锚点均已校验；另有 1 页目录漏收附录、8 个官方附件与 84 条官方源死链/1 条悬空锚点的公示）在 [docs/api-doc-html/](docs/api-doc-html/)，error.xml 错误码全集（299 条，逐条标注「已实现 23 / 可落地 47 / 暂不可达 229」+ 推送面，由 `tools/fill_errorcode_status.py` 全量重算、`--check` 可校验）在 [docs/错误码全集.md](docs/错误码全集.md)，双推送面口径见 [docs/notes/09](docs/notes/09-错单推送面与错误码对账.md)；FAK 部成部撤的**三所分流**回报（上期所/大商所+广期所/郑商所三种不同形状）见 [docs/notes/10](docs/notes/10-FAK回报按交易所分流.md)；程序化交易入门系列 17 份客户端实操资料（连接认证/穿透式监管、行情现手开平、报撤单成交回报、查询流控与持仓更新）的整理与实现影响清单见 [docs/notes/11](docs/notes/11-入门系列-连接认证与穿透式监管.md)~[docs/notes/14](docs/notes/14-入门系列-查询流控与持仓查询更新.md)（汇总登记在知识库 §10.4）。CTPBuddy 与上海期货信息技术有限公司无任何隶属关系；仓库**不包含** `ctpsdk/*/` 下的官方二进制 SDK（`.dll`/`.lib`，由使用者自备，`.gitignore` 禁止入库与分发），但 `docs/api-doc-html/files/` 附带了由官方 CHM 文档转换得到的头文件、`error.xml` 与 PDF，仅供文档交叉引用（许可与来源说明见下文「许可」）。

## 它解决什么问题

SimNow / openctp 都是远程 CS 模式：网络绑死、无法注入极端行情、不可复现、无管理面。CTPBuddy 把柜台搬到本地：替换同名 DLL 即可接入，行情可回放、场景可注入、账户可管理、团队可共用。

> **当前边界（请先读）**：传输为明文 TCP，ADMIN 与交易端口**无认证**（TD 登录不校验密码）；核心拒绝把 ADMIN 绑到非回环地址（`CTPBUDDY_ALLOW_REMOTE_ADMIN=1` 才放行）。Shim 支持 Windows DLL 与 Linux `.so`（`shim/build_linux.py`，WSL 已验证）；前置地址需指向核心，或用环境变量 `CTPBUDDY_ADDR=tcp://host:port`（可选加固：`CTPBUDDY_TD_TOKEN=<密钥>` 设置后，Shim 握手 AUTH 的 auth_code 必须相等，仅覆盖 AUTH；`CTPBUDDY_MAX_CONNS` 限制并发连接，默认 256，写超时 10 秒） 重定向。ZeroMQ、多 Broker、登录后快照补发/私有流重传尚未实现，详见 DESIGN §0.6。「团队共用」目前仅适用于受信内网并自行做网络隔离。

## 架构速览

```
下游系统 ──(同名 DLL 替换)── CTPBuddy Shim (C++) ──ZeroMQ/帧协议── Rust 核心服务
                                                                    ├─ 行情回放引擎（源由用户提供）
                                                                    ├─ 撮合引擎（默认单线程确定性）
                                                                    └─ 账户账本（每账户单写者）
Python 层 (pip install ctpbuddy): CLI / SDK 断言 / 本机 Web 后台 / 行情源插件
```

## 当前进展

### 运行与录制边界

核心服务需要 Rust 1.89 或更新版本构建。Python SDK 的连接超时与请求等待超时不再作为空闲连接的存活期限；真正断连后，请求立即失败。

`--data-dir DIR` 的当前录制位于 `DIR/journal/`。复用该目录重启时，旧录制整体保存在 `DIR/journal-run-<时间戳>-<进程号>/journal/`，新录制的序号从 1 开始。每份录制应单独验证、重放或重建投影，不要合并不同运行的事件；此行为不是账本崩溃恢复，重启仍按启动配置建立账本。设置和已供给结算报告保持原有持久化方式。数据目录有进程级文件锁，不能同时启动两个 journal 写者；锁随进程退出释放，无需删除锁文件。

Wire v1 保持现有小端 C 布局和字段偏移，Rust 编解码按字段处理并将 padding 清零。报单价格中的 NaN 和无穷值在进入冻结及撮合前拒绝。

当前里程碑摘要（完整清单与 CI 记录见 [CHANGELOG.md](CHANGELOG.md)）：

- M1 核心闭环 + Shim 全链路、M2 撮合/流控、M3 参考数据/显式日结/OrderSysID/结算单/journal 投影：已完成。
- 未完成：ZeroMQ 传输适配（当前 TCP 占位）、M4 三渠道发布与完整发布包；详见 DESIGN §0.6。

## 快速开始

```bash
# 1. 构建核心服务（Rust 1.89+）
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
python tools/audit_query_expectations.py --limit 20 --report C:/temp/query-audit.json
python tests/e2e/m4_real_replay.py --report C:/temp/core-query-subset-audit.json
```

M4 真实回放只验收 `controlled futures subset`：动态选择 RefData 覆盖交易日的普通期货、`OffsetFlag=0`、`HedgeFlag=1`、空投资单元、唯一关联且一笔完全成交的真实 order/trade。回放在临时目录和匿名隔离账户中使用原 order 的 LimitPrice/VolumeTotalOriginal/TimeCondition/VolumeCondition/MinVolume；卖单 bid 使用真实成交价，买单 ask 使用真实成交价。报告仅保存匿名计数与 skip 原因，不生成真实正文；未提供外部目录时打印 SKIPPED 并以退出码 3 退出（传 `--allow-skip` 才为 0，CI 需显式传入），有数据但无合法候选时失败。它不声称真实账户前日权益或完整账户重演，初仓 empty 仅指该隔离 controlled open 子集。

`audit_query_expectations.py` 从严格同交易日、同账号的 `order.csv` / `trade.csv` 与结算单构建
`ReqQryOrder`、`ReqQryTrade`、`ReqQryInvestorPosition`、`ReqQryInvestorPositionDetail` 的可核期望。
订单使用 `CombOffsetFlag`，成交使用 `OffsetFlag`；`OrderSysID` 只作 trim 后关联键，`OrderRef` 兜底必须包含
`FrontID`/`SessionID`，多候选一律 ambiguity，不取 first candidate。持仓数量以**前一结算持仓 + 当日成交**为基线，
并与同日结算单核对；非 SHFE/INE 的平仓年龄/FIFO 不完整时只记 ambiguity，不从今平量猜昨仓。期权仅做数量变化校验，
不将权利金或期权保证金转换为期货资金。没有真实查询回报时状态为 `not_evaluated`，不是通过。
支持 `--date`、`--investor`、`--limit`、`--report`；报告只写匿名 hash、计数、差异和 skip/fail 理由，不保存真实正文。
坏数据返回非零；缺源显式 skip；缺同日/前日结算单的账户日记为 `not_evaluated`（与 `audit_three_way.py` 口径一致，不算通过）；同一 (结算日, 账号) 出现多份结算单计入 `duplicate_settlement`、列入报告并按坏数据判失败；CSV 先按 UTF-8(BOM) 再按 GBK 解码。该脚本是 source alignment / query expectations，不代表 Core 账本重演已完成。
目录可用 `CTPBUDDY_EXPORT_DIR` / `CTPBUDDY_SETTLEMENT_DIR` 指定；未提供则跳过，不影响上面的回归。
口径细节见 DESIGN §8.7.1。

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

`calendar/production/cn-futures-day-2026.snapshot.json` 是项目离线快照：完整覆盖 2026 自然日的“周一至周五且不在六所同日公开节假日区间”的日盘候选，并将其明确标为 `scope: day-session-only`；它不是六所官方逐日 TradingDay 数据库，也不把民用调休工作日直接当期货交易日。六所来源、提取 revision、事实 hash、DCE 动态参考页及夜盘缺口均登记在 `calendar/production/MANIFEST.json`。夜盘只固化公告明确的 closed 边界，未知 open/closed 一律拒绝推断；生产使用前须复核公告并固定 SHA256。`build_2026.py` 仅是可审计的离线生成器，不访问网络。

生产目录仍保留 `shfe-2026-new-year.sample.json`：固定来源为上期所 2025-12-17〔2025〕157号公告，覆盖 2025-12-31 至 2026-01-05 的元旦样本及 2025-12-31 夜盘关闭边界，SHA256 为 `3a1929f644c903ffaea2f8751e9f998a2be8cd71d9937c0e321f1aa9b768b67a`。新增全年日盘快照 SHA256 为 `b9217ede09a806e99d3819331367b76731bf4407fade96f2a02ed3123fdb71e3`，覆盖 365 自然日、24 条显式夜盘关闭记录；完整逐日夜盘仍未取得，manifest 明确记录缺口。不能把缺失夜盘解释为关闭或开放。

Web 启动可指定 `ctpbuddy web --workspace . --db data/ctpbuddy.db`；目录与详情为只读，场景加载只接受 `scenarios/<name>` 和 `confirmed=true`。加载前必须暂停已有回放、无活动订单且无持仓保证金，不自动重置账户；账户实时摘要、初始持仓与历史投影各有独立状态标识。页面提供场景、账户/持仓、订单/成交、结算/审计和回放导航，不实现 Core 不存在的功能。

用户覆盖通过 `calendar.with_overrides(user_snapshot)` 或 `load_calendar(path, override=...)` 完成，按自然日整条替换并重新计算快照 SHA256。推荐在 CI 中固定并校验 SHA256；不要把未核验的外部数据写入仓库。

已核查的外部数据源示例：`gerrymanoim/exchange_calendars`，Apache-2.0，固定 commit `bbda29fed902374bdb75acab008f421fbd567823`。其 README 明确定位为证券交易所日历、日历由用户贡献维护，并将常规交易时段外（含盘前/盘后/竞价/午休）视为关闭；仓库包含上海证券交易所 XSHG，但不提供 CTP 期货夜盘 ActionDay/TradingDay 语义。因此本项目不在运行时依赖它，也不将其数据直接作为期货快照。

## Docker 验收与发布包

Linux 容器验收只覆盖不依赖 Windows ABI 的边界：Python wheel/CLI、Rust `ctpbuddy-server` build/run、随包 `refdata`，以及 Web/OINK（镜像没有 Hugo 二进制时明确 `SKIP`，可在独立 Hugo 环境构建）。容器**绝不运行 Windows Shim**，也不复制 `ctpsdk/`、Windows DLL/EXE 或真实账户/行情数据。

```bash
python tools/docker_acceptance.py       # 无 Docker 或 daemon 不可用时打印 SKIPPED、退出码 3；--allow-skip 才为 0
# CI / 本机 Docker：
docker compose -f docker/compose.yml build
docker compose -f docker/compose.yml run --rm acceptance
```

Windows Shim 发布包只从本机已构建的 `shim/bin/` 白名单复制 `thosttraderapi_se.dll`、`thostmduserapi_se.dll`，以及存在时的 `demo_td.exe`：

```bash
python tools/release_package.py          # 输出系统临时目录中的 dist
python tests/e2e/release_package.py      # 临时 PE fixture 必跑；真实产物段未启用时 SKIPPED、退出码 3（--allow-skip 为 0）
CTPBUDDY_TEST_REAL_RELEASE=1 python tests/e2e/release_package.py
```

包内有 `manifest.json`（version、commit、逐文件 SHA256、PE machine、architecture、SDK `6.7.13`）、`LICENSE`、`INSTALL.txt` 和 `shim/ctpbuddy-shim-manifest.json`。脚本不读取/复制原始 `ctpsdk`，不读取/复制 `data`，不构建或执行 Shim；真实产物验收必须在 Windows 上单独完成。

## 许可

本项目代码以 [MIT](LICENSE) 许可发布。以下随仓库附带的数据**不在 MIT 范围内**，各自按来源声明使用：

- `docs/api-doc-html/files/` 中由官方 CHM 转换得到的头文件、`error.xml` 与 PDF 属上期技术所有，仅供文档交叉引用，不构成再分发授权；`ctpsdk/*/` 二进制 SDK 不入库。
- `calendar/production/` 下的交易日历快照取自六所公开休市公告的事实摘录（来源、提取 revision 与事实 hash 登记在 `calendar/production/MANIFEST.json`），快照 `source.license` 自述「原网页再分发许可未确认」——即这些数据**不随 MIT 授权、再分发许可状态未确认**，生产使用前须自行复核公告并固定 SHA256。
- `refdata/` 随包合约快照源自 LocalCTP 参考实现的 `instrument.csv` 导出，见 DESIGN §8.6.1。

## 结构

```
ctpsdk/                  # 使用者自备的官方 SDK 二进制（禁止入库，见 .gitignore）
docs/                    # 设计文档 + CTP 语义知识库 + 官方 API 文档/错误码全集（CHM 与 error.xml 的可读化；api-doc-html/files/ 附带 CHM 转出的 .h/error.xml/PDF）
shim/                    # C++ Shim（DLL/so 同名替换）+ codegen
core/                    # Rust workspace：wire / market / matching / ledger / server
py/                      # Python 包 ctpbuddy
scenarios/               # 示例场景与 tick 数据
tests/e2e/               # 端到端测试
docker/                  # Linux 验收镜像与 compose
```

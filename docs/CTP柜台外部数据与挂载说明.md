# CTP 柜台外部数据与挂载说明

本文说明 CTPBuddy 运行一个本地 CTP 兼容柜台时，需要由外部提供哪些数据，以及这些数据如何进入核心服务。

这里的“外部数据”指行情、合约参数、费率、账户初始状态、交易日历和结算资料等由使用者或数据供应方提供的内容。CTPBuddy 不连接真实期货公司柜台自动拉取这些数据，也不在核心中编造生产费率。

## 1. 总体数据流

```text
厂商导出 / 历史行情 / 结算资料
        │
        ├─ 参考数据 provider → JSONL → Rust Catalog / RefData
        ├─ 行情数据 → scenario/ticks.csv → Playback
        ├─ 交易日历 → Python TradingCalendar → ADMIN settle_day
        └─ 结算报告 → ADMIN settlement_report → data/settlement_reports

CTP 策略或终端
        │ CTP Trader/Md API
        ▼
同名 Shim DLL/SO
        │ TCP TD 端口，默认 127.0.0.1:5560
        ▼
Rust 核心：会话、行情、撮合、账本、结算、查询和推送

Python CLI / SDK / Web
        │ ADMIN JSON，默认 127.0.0.1:5561
        ▼
Rust 核心控制面
```

Shim 只负责 CTP ABI 适配、帧封装和 SPI 分发；文件数据的加载、撮合、资金和账本计算都在 Rust 核心完成。

## 2. 外部数据清单

| 数据类别 | 主要内容 | 是否必需 | 进入方式 |
|---|---|---:|---|
| 柜台连接与身份 | `BrokerID`、TD 地址、ADMIN 地址、客户端 `UserID/InvestorID` | 是 | 启动参数、环境变量、CTP 登录请求 |
| 行情 | 合约、交易所、交易日、时间、最新价、成交量、昨结、涨跌停、五档价格和数量 | 回放行情时是 | `scenario/ticks.csv` |
| 合约目录 | 合约代码、交易所、乘数、最小变动价位、交易日期、交易状态、报单量限制等 | 是；无自定义数据时使用随包快照 | `instruments.jsonl` |
| 公司保证金率 | 多空按金额/按手保证金率 | 否 | `margin_rates.jsonl` |
| 手续费率 | 开仓、平昨、平今的按金额/按手费率 | 否；缺失时按零手续费处理 | `commission_rates.jsonl` |
| 报单/撤单费 | 申报费相关字段 | 否 | `order_comm_rates.jsonl` |
| 柜台交易参数 | `MarginPriceType`、币种、可用资金口径等 | 否，有默认值 | `trading_params.jsonl` |
| 初始账户状态 | 初始资金、初始持仓明细、开仓价、开仓日期、昨结算价 | 否 | `scenario.yaml` / `scenario.json` |
| 交易日历 | 自然日、期货 `TradingDay`、夜盘 `ActionDay/TradingDay` 映射 | 仅日结推导下一交易日时需要 | Python `TradingCalendar` |
| 日结输入 | 合约结算价、下一交易日 | 跨交易日时需要 | ADMIN `settle_day` |
| 结算单正文 | 真实或外部生成的结算单内容 | 查询真实结算单时需要 | ADMIN `settlement_report` |

客户端在运行过程中发送的登录、订阅、查询、报单和撤单请求不属于预加载文件，而是通过 Shim 实时进入核心。

## 3. 行情数据如何挂载

### 3.1 场景目录

当前核心实际消费 CSV 行情。典型场景目录如下：

```text
scenarios/<name>/
  scenario.yaml       # 编写格式，可选
  scenario.json       # Python 编译后的核心消费格式
  ticks.csv           # 行情源
  refdata/            # 场景专属参考数据，可选
```

`ticks.csv` 的规范字段由 `core/ctpbuddy-market` 定义，当前为 40 个字段：

```text
instrument, exchange, trading_day, update_time, update_millisec,
last_price, volume, turnover, open_interest,
pre_settlement, settlement, pre_close, open, high, low, close,
upper, lower, pre_open_interest, average,
bid1..bid5, ask1..ask5, bidvol1..bidvol5, askvol1..askvol5
```

夜盘数据可以额外提供 `action_day` 列；缺失时核心按 `trading_day` 处理。源数据应按虚拟时间顺序排列，核心不会替源数据重新排序，也不会访问网络补行情。

### 3.2 加载管道

```text
scenario.yaml
    → Python 受限 YAML 解析、校验和归一化
scenario.json
    → Rust 解析场景
ticks.csv
    → CsvSource
    → freeze / gap / liquidity 等确定性变换
    → 虚拟时钟 Playback
    → 行情推送 + 撮合触发 + 账本盯市
```

常用命令：

```bash
ctpbuddy scenario validate scenarios/dsl_demo
ctpbuddy scenario compile scenarios/dsl_demo
ctpbuddy serve --scenario scenarios/dsl_demo --data-dir ./data
```

当前 `source.kind` 只接受 `csv`；Parquet、数据库或实时行情插件属于扩展方向，不能直接作为当前 Rust 核心的场景源。

如果行情只有 `last_price` 而没有五档深度，撮合只能退化为按最新价触发的即时成交模式。需要测试限价簿、FAK/FOK 或价格穿越时，应提供完整五档价格和数量。

## 4. 参考数据如何挂载

### 4.1 标准 JSONL 目录

参考数据 provider 可以读取 CSV、厂商表格或其他数据源，然后输出核心直接消费的 JSONL：

```text
myrefdata/
  instruments.jsonl
  margin_rates.jsonl
  commission_rates.jsonl
  order_comm_rates.jsonl
  trading_params.jsonl
```

表与 CTP 查询的对应关系如下：

| 文件 | 对应查询 | 作用 |
|---|---|---|
| `instruments.jsonl` | `ReqQryInstrument` | 合约目录、乘数、最小变动价位和交易属性 |
| `margin_rates.jsonl` | `ReqQryInstrumentMarginRate` | 公司保证金率，实际用于资金冻结和占用 |
| `commission_rates.jsonl` | `ReqQryInstrumentCommissionRate` | 开仓、平昨、平今手续费 |
| `order_comm_rates.jsonl` | `ReqQryInstrumentOrderCommRate` | 报单和撤单申报费 |
| `trading_params.jsonl` | `ReqQryBrokerTradingParams` | 保证金计价方式等柜台参数 |

`instruments.jsonl` 是唯一必填表。其他表可以为空；“没有这条规则”是合法柜台配置，不会被自动填入虚构费率。

### 4.2 从 CSV 或厂商导出转换

```bash
ctpbuddy refdata export \
  --kind csv \
  --path ./desk_export \
  --out ./myrefdata
```

输入 CSV 的列名应使用 CTP 字段对应的 snake_case 名称，例如 `instrument_id`、`exchange_id`、`volume_multiple`、`price_tick`。如果厂商列名不同，应由 provider 负责翻译；核心不会猜测列含义。

也可以实现 Python provider 的五个表方法：

```python
instruments()
margin_rates()
commission_rates()
order_comm_rates()
trading_params()
```

先通过 `ctpbuddy refdata export` 固化为 JSONL，再交给 Rust 核心，避免热路径依赖 Python 解释器。

### 4.3 加载优先级

核心按以下顺序选择参考数据：

```text
--refdata DIR
或 CTPBUDDY_REFDATA
        > scenarios/<name>/refdata/
        > 仓库自带 refdata/
```

显式指定的 `--refdata` 加载失败会直接退出，不会静默回退到其他目录。未指定自定义目录时，随包数据提供合约和公司保证金率快照，但不提供手续费表。

启动示例：

```bash
ctpbuddy serve \
  --scenario scenarios/dsl_demo \
  --refdata ./myrefdata \
  --broker-id 8888 \
  --data-dir ./data
```

参考数据加载后只形成一份核心 `Catalog/RefData`，同时服务于撮合、账本和 `ReqQryInstrument*Rate` 查询，保证查询结果与资金计算使用同一套参数。

## 5. 账户初始状态如何挂载

场景可以配置自动开户资金：

```yaml
accounts:
  - investor: "demo001"
    balance: 800000
```

也可以导入逐笔初始持仓：

```yaml
accounts:
  - investor: "demo001"
    balance: 800000
    positions:
      - instrument: rb2601
        exchange: SHFE
        direction: long
        open_date: "20261001"
        trade_id: T001
        open_price: 3500
        volume: 2
        pre_settlement: 3490
```

场景账户在加载时写入账本；已经存在的账户不会被重复导入初始持仓。未在场景中列出的用户，第一次成功登录时自动开户，初始资金取自：

```text
CLI --initial-funds
→ 环境变量 CTPBUDDY_INITIAL_FUNDS
→ data/settings.json
→ 默认值
```

使用 `--recover` 时，核心从 `data/state/ledger.json` 恢复账本；工作订单不会恢复，服务会将其视为已作废。

## 6. 交易日历和结算资料如何挂载

### 6.1 交易日历

交易日历由 Python 侧加载离线 JSON 快照，核心运行时不联网：

```python
from ctpbuddy.calendar import TradingCalendar
from ctpbuddy.sdk import Admin

calendar = TradingCalendar.from_file("calendar.json")
with Admin(calendar=calendar) as admin:
    admin.settle_day({"rb2601": 3501.0})
```

日历的职责是根据当前 `TradingDay` 推导 `next_trading_day`。它不替代行情中的 `trading_day`，也不应从周末规则或股票交易日历推断夜盘映射。

### 6.2 日结价和结算单

日结必须由外部明确提供每个合约的结算价及下一交易日：

```text
Admin.settle_day(
    settlement_prices={"rb2601": 3501.0},
    next_trading_day="20261005",
)
```

执行前必须暂停回放，且不能有活动订单。日结完成后，核心推进交易日、滚存账户和持仓状态，并清理当日订单/成交流。

如果需要让 `ReqQrySettlementInfo` 返回真实或外部生成的结算单，则通过 `Admin.settlement_report()` 写入。正文以原始 GBK 字节保存，持久化目录为：

```text
data/settlement_reports/
```

没有外部正文时，`settle_day` 会生成标记为 `modeled_ledger_minimal` 的最小模拟报告，不能把它当作真实期货公司结算单。

## 7. 客户端实时接入

客户端不需要把行情或费率再通过 CTP API 上传一次；这些数据已经由核心从文件或 ADMIN 加载。客户端只需要连接 TD 前置并发送标准 CTP 请求：

```text
CTP 应用
  → 同名 Trader/Md DLL 或 SO
  → Shim
  → TD TCP 端口
  → Rust 核心
```

默认 TD 地址为 `127.0.0.1:5560`。客户端可以在 `RegisterFront()` 中直接使用核心地址，或通过 `CTPBUDDY_ADDR` 重定向。

登录时提供 `BrokerID` 和 `UserID`。当前实现中：

- 一个核心实例只服务一个 `BrokerID`；
- `UserID` 同时作为 `InvestorID` 使用；
- `Password` 会被接收，但当前柜台不校验密码；
- 设置 `CTPBUDDY_TD_TOKEN` 后，Shim AUTH 的 `auth_code` 必须匹配该 token；
- 认证和登录成功后，客户端可以订阅行情、查询合约/资金/持仓/费率、报撤单并接收订单和成交回报。

Python CLI、SDK 和 Web 使用 ADMIN 端口控制场景、回放、设置、日结和结算报告，默认地址为 `127.0.0.1:5561`。

## 8. 不属于核心运行时输入的数据

真实柜台导出的 `order.csv`、`trade.csv`、`account.csv` 和真实结算单目录，主要供 `tools/audit_*.py` 做来源对齐和对账审计。它们不会自动挂载成核心的实时账户或行情；若要进入仿真环境，必须先转换为：

- `ticks.csv`；
- 参考数据 JSONL；
- 场景账户/初仓；
- `settlement_prices` 或 `settlement_report`。

## 9. 实现入口

- 核心配置、场景和参考数据加载：[`core/ctpbuddy-server/src/lib.rs`](../core/ctpbuddy-server/src/lib.rs)
- 场景 JSON 解析：[`core/ctpbuddy-server/src/scenario.rs`](../core/ctpbuddy-server/src/scenario.rs)
- 参考数据 provider 和 JSONL 导出：[`py/ctpbuddy/refdata/__init__.py`](../py/ctpbuddy/refdata/__init__.py)
- 场景 YAML 校验和编译：[`py/ctpbuddy/scenario.py`](../py/ctpbuddy/scenario.py)
- Python ADMIN 控制：[`py/ctpbuddy/sdk/admin.py`](../py/ctpbuddy/sdk/admin.py)
- Python CTP 客户端示例：[`py/ctpbuddy/sdk/client.py`](../py/ctpbuddy/sdk/client.py)
- Shim 连接、认证和 CTP API 转发：[`shim/src/api_core.cpp`](../shim/src/api_core.cpp)
- 结算报告持久化和查询：[`core/ctpbuddy-server/src/settlement.rs`](../core/ctpbuddy-server/src/settlement.rs)


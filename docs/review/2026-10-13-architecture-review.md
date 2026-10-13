# CTPBuddy 架构与实现全面评审

- 日期:2026-10-13
- 评审基线:main 分支 `d4fdeae`(feat: model exchange-specific night calendar mapping)
- 评审方式:通读 `DESIGN.md`(全文)、`docs/adr/0001`、`docs/operations.md`、`docs/journal-upgrade-and-flow-replay.md`、`GLOSSARY.md`、`CHANGELOG.md`;逐行精读 Rust core 承重模块(`lib.rs`、`journal.rs`、`journal_contract.rs`、`handlers.rs`、`handlers/order.rs`、`engine.rs`、`ledger/lib.rs`、`ledger/settlement.rs`、`settlement.rs`、`auto_settlement.rs`、`calendar.rs`、`flow.rs`、`transport_zmq.rs`、`frame.rs`、`msgs.rs`、`market/lib.rs`、`main.rs` 及其余模块头);由三个独立子评审分别通读 C++ shim 全部源码、Python 控制平面全部源码、测试/工具/CI/数据链路;子评审结论经本人交叉验证后合并(修正了其中一处误判,见 §6.2)。

---

## 1. 总体结论

**这是一份成熟度显著超出其 v0.1 版本号的架构。** 三层拆分(C++ shim / Rust core / Python 控制面)职责边界清晰且被代码真实执行;确定性目标(§7.6)被工程化为可验证的 hash 契约;CTP 语义还原深度(错误码逐条对账、按所回报形状分流、平今费时间序池)在同类开源项目中罕见;文档-实现-测试三者形成了罕见的三向闭环,且文档的诚实度(差距登记、审计边界、skip 不冒充 pass)本身就是质量资产。

**主要风险集中在三处,而非近期热点:**

1. **Shim 的失败路径健壮性**(§4.2):无心跳/超时、发送队列满时静默丢帧、载荷尺寸不符返回"成功+NULL"。这些不影响正常路径,但直接决定下游在异常场景下是"收到 OnFrontDisconnected"还是"永久挂起"——而后者的排查成本会记在 CTPBuddy 头上。
2. **journal 写失败的静默分叉**(§4.1):账本先应用、journal 后落盘的设计下,I/O 失败只打一行 stderr,live 状态与 journal 从此分叉,直到下次重放校验才会暴露。
3. **World 协调器仍在膨胀轨道上**(§3.2):`lib.rs` 2756 行 + `admin.rs` 1224 行 + `query.rs` 1297 行,seam 化(client_route、decode seam、两阶段 dispatch)方向正确但只完成一半;样板重复的根源是自研 `json::Value` 没有 derive,这一代价在 admin 命令面上被放大。

上述三点均有明确、局部的修法,不动架构。**本评审未发现需要推翻任何既有架构决策(ADR/技术选型)的证据。**

---

## 2. 架构评价:分层与深模块盘点

### 2.1 分层结构(经源码证实)

```
下游 CTP 应用 ── CTP ABI ──> C++ Shim(薄生成壳 + ApiCore 深核心)
                                │ CB 帧(raw struct,GBK 字段)
                                v
              Rust core ── World 单线程世界循环(唯一写者)
               ├─ transport: TCP / ZMQ(纯 Rust zeromq)/ ZMQ(原生 libzmq+CURVE)
               ├─ matching:  限价簿引擎(两时刻撮合)
               ├─ ledger:    盯市账本(冻结/成交/日结)
               ├─ market:    canonical tick + 虚拟时钟回放
               └─ journal:   append-only JSONL(唯一权威事件流)
                                │ journal_contract.json(编译期嵌入,双语言 vectors)
                                v
              Python 控制面 ── 校验/重放/SQLite 投影/Web/CLI/日历
```

各层职责与 DESIGN §4.1 表格逐条相符:shim 不做业务判断(证实于 `api_core.cpp`,全部判断在 server);Python 不做撮合账本(证实于 `py/ctpbuddy/*`,写路径全部经 ADMIN 进 core);Rust 不碰 SQLite(证实于 Cargo.toml 与代码)。

### 2.2 深模块(小接口、大行为——项目的承重墙)

| 模块 | 接口 | 藏在接口后面的复杂度 | 评价 |
|---|---|---|---|
| `Ledger` | `freeze` / `freeze_close_position` / `on_fill` / `unfreeze_order` / `settle_trading_day` / `mark_to_market` | 逐日盯市恒等式、先开先平逐明细盈亏、平今/平昨分腿计费、中金所时间序费池、品种内大单边聚合、跨品种显式 relief、定点 Money、staged-clone 原子日结 | **最深的一个**。调用方只需知道"报单前冻结、成交时结算、终态解冻";单条 `Close` 吃 2 昨 + 1 今时按两腿分别计价这类语义,外部完全不可见(`ledger/lib.rs:658-941`) |
| `MatchingEngine` | `submit` / `cancel` / `on_tick` | 价格时间优先簿、五档消耗跟踪、FOK/FAK lookahead、FAK 成交缓冲后按 `IocLayout` 三所形状铺开、`internal_sys_id`/`order_sys_id` 双号边界 | 撮合只发生在"报单到达"与"tick 到达"两处,resting 单互不撮合——确定性之根,注释把官方场景依据写全(`engine.rs:35-37, 182`) |
| journal contract | `journal_contract.json` + `validate_event` | envelope/必填/类型/嵌套校验、版本兼容边界、canonical hash 规则 | ADR-0001 完整落地:单一 JSON 契约、build.rs 编译期嵌入(`build.rs:13-33`)、Rust/Python 跑同一份 vectors(`journal_contract.rs:683-705`)。契约漂移在构建期和测试期都会被抓住 |
| `Journal` 写者 | `record` / `flush` | 单实例锁、按 run 归档、崩溃半行容忍、seq 断链 fail-closed、三级 sync 档位 | fail-closed 彻底:契约校验失败不推进 seq(`journal.rs:240-243`,有专门单测),恢复时 `seq != expected` 即拒(`journal.rs:97-103`) |
| `flow::WindowBudget` | `take(now, limit)` | 1 秒墙钟窗口重置 | 54 行,但被 qry/order/exchange/FTD/connect 五处闸门共用,语义单一 |
| shim `ApiCore` | 生成类回调表 | 连接管理、GBK 转换、pending 配对、SPI 线程契约 | "薄生成壳 + 一个深核心":900 行 `api_td_reqs.cpp` 由 codegen 约 20 行表驱动生成;回调持锁外派发,严格满足"SPI 不在应用栈上触发" |

**删除测试(deletion test)全部通过**:删掉上述任一模块,复杂度会原样重新出现在 N 个调用方处,而不是消失。

### 2.3 浅/上帝模块(复杂度在接口面上)

1. **`World`(`lib.rs` 2756 行)**。DESIGN §0.6 已自我登记为"仍由 World 统一协调"。正面:seam 化在真实推进——`client_route` 纯函数(`handlers.rs:104-129`)、`decode_order_insert`/`decode_order_action` 纯命令 seam(`handlers/order.rs:612-717`,有单测证明无 world 副作用)、engine 事件拆成 state-apply 与 notification-fanout 两阶段(`lib.rs:1844-1847`)。剩余问题:`Conn` 结构体 15 个字段把传输、会话、流控、订阅四类关注点捏在一起;`replay_archived_tail` 的恢复事件分发(`lib.rs:1369-1430`)是又一张按字符串分发的事件表,与 Python `replay.py` 的 walk 分发同构——同一份契约的**第三份**手写消费逻辑(前两份是 Rust 校验器与 Python 校验器,好在均由 vectors 钉住,这一份没有)。
2. **`admin.rs`(1224 行)**。每个命令手写"未知/重复字段 + `matches!` 白名单"样板,同一模式重复约 15 次(`admin.rs:44-48, 557, 630, 726, 900` 等)。根源是自研 `json::Value` 没有 serde derive——这是"控制面 stdlib-only"决策的隐性代价,集中体现在这一个文件。
3. **`handlers/query.rs`(1297 行)**。查询投影样板,与 admin 同源(手写字段组装 + 白名单校验)。CTP 查询面天然是宽表,此处浅是可以接受的——深度的补偿在于数据来源(账本/catalog)已深。

### 2.4 Seam 的真实性检验

- **传输 seam:真**(三个 adapter:TCP `accept_loop`、纯 Rust `transport_zmq`、原生 CURVE `transport_zmq_native`,共用 `WorldMsg`/`Frame` 接口;`transport_zmq.rs:35-57` 展示 CURVE 时无缝切换)。
- **journal 契约 seam:真**(Rust 校验器与 Python 校验器两个 adapter,同一 vectors)。
- **行情源 seam:半**(canonical tick 形状统一,但源目前只有 CSV;Parquet/插件在 §0.6 已登记未实现——诚实,但"一个 adapter = 假想 seam")。
- **provider seam(refdata):真**(鸭子类型 Python provider + 规范 JSONL,`--refdata`/scenario/bundled 三级取用,显式失败为硬错误,`lib.rs:651-664`)。

### 2.5 并发与运行时

世界循环是同步 std::thread + mpsc,唯一例外是 ZMQ 传输线程内嵌 current-thread tokio runtime(`transport_zmq.rs:61-68`),跨边界用 std mpsc(入)+ tokio mpsc(出,有界 1024、try_send 丢帧计数)。这带来两个观察:

- tokio/zeromq/serde_json/libloading 使 server crate 不再"零依赖"(见 §6.1 漂移),但换来了异步 ZMQ 与原生 CURVE 能力,且 runtime 被完全隔离在传输线程内,世界循环的可推理性未受影响——**取舍合理,只是文档没更新**。
- PUB 广播是 best-effort(丢帧计数进 admin status),私有流回退 DEALER 会话——与文档承诺一致。

---

## 3. 各子系统评审

### 3.1 Rust core(本人精读)

**正确性要点(正面)**:

- 报单路径的闸门次序与文档一致:字段校验 → 结算确认(42)→ 报单流控(116,insert/cancel 双流)→ 交易所额度(41)→ 静态校验 → 资金/持仓冻结 → 引擎;每步失败都先 journal 拒单记录再回应,回滚路径完整(`handlers/order.rs:68-377`)。交易所额度超限还合成终态 `'5'/'4'` 订单事件进 `orders_today`,保证 OrderRef 不被复用(`handlers/order.rs:162-196`)——这是对 §8.11"拒付的报单 '4'"口径的忠实执行。
- 冻结按**原始估算额** pro-rata 释放的设计(`FrozenEst`,`ledger/lib.rs:77-99`)从类型上消除了"多段成交后冻结残留"这类 off-by-proportion 缺陷,`on_fill` 里 `est.total = fill.volume_total_original` 的对齐(`ledger/lib.rs:682-685`)闭环了 SHFE `CancelFirst` 先撤后成的乱序路径。
- 日结的 staged-clone 模式:全部校验通过后 clone 账本、在副本上完成盯市/转昨/重估/清零,再整体替换(`ledger/settlement.rs:70-169`),"任一账户失败则全批不变"不是口号而是结构保证。
- 快照次序契约(journal flush → snapshot)有注释和代码双重保证(`lib.rs:1661-1669`),恢复时先快照后 durable tail,历史泛型 `settle`/`admin` 事件 fail-closed 拒恢复(`lib.rs:1421-1425`)。
- 敌意输入防御:4000 帧随机 fuzz 进 world loop 不 panic 的单测(`lib.rs:2120-2163`)、`ensure_admin_loopback` 的绑定前校验(`lib.rs:296-318`)、常量时间比较的 token 校验(`lib.rs:358-361`)。
- `PatientReader` 处理 Windows 上 shutdown 不唤醒阻塞 recv 的问题(`lib.rs:463-546`),连接限额/写超时都有资源回收测试。

**正确性问题(按严重度)**:

| # | 位置 | 问题 | 影响 |
|---|---|---|---|
| R1 | `journal.rs:245-256` | journal 写失败(I/O 错误)时事件被静默丢弃:仅 `eprintln`,`seq` 不推进,世界循环继续。而此时账本已经应用了该事件(`apply_engine_event` 先改状态、`journal_record_*` 后落盘) | live 状态与 journal 分叉;崩溃恢复/replay 将回放出不同状态。重放 core hash 比对**最终会**发现分叉,但实时没有任何告警面,分叉窗口可能长达数小时。这是"journal 是唯一权威事件流"承诺下最值得堵的洞 |
| R2 | `handlers.rs:500-515` | `on_logout` 不校验请求的 BrokerID/UserID 与登录会话一致,任意身份的 logout 都会清空该连接的 investor 绑定并 journal 该 (broker,user) 的 logout | 语义偏松(真实 CTP 校验一致才允许);单连接内影响有限,但 journal 的 session_logout 记录可被污染 |
| R3 | `engine.rs:437-448` | 涨跌停校验仅在"存在 tick"时执行;首个 tick 到达前,任何限价都通过 | 有记录的仿真边界(§8.10 未列入但与"深度为空的源退化"同族),建议至少在文档口径中登记 |
| R4 | `lib.rs:1327-1363` | run_loop 每次 `recv_timeout(wait)` 只处理**一条**消息后重算 pulse;高请求压力下消息处理与 pulse 交替,吞吐有上界 | 单线程设计的已知代价,PULSE 语义(真定时器)优先于吞吐,是正确取舍;列于此仅为完整性 |

### 3.2 C++ shim(子评审 + 本人抽查)

架构正面已列 §2.2。健壮性问题是 shim 的主要短板:

| # | 位置 | 问题 | 影响 |
|---|---|---|---|
| S1 | `api_core.hpp:50-51`(PING/PONG 定义未使用)、`api_core.cpp:529-562`(connect_front 未设 SO_KEEPALIVE) | **无心跳、无请求超时**。server 静默死亡(无 FIN,如 kill -9 后 NAT/虚拟机挂起)时,客户端收不到任何回调,pending 永不完成 | 下游表现为"策略卡死无异常"。DESIGN §5.5 承诺"不匹配时明确报错,绝不静默"——版本不匹配路径做到了,半开连接路径没做到 |
| S2 | `api_core.cpp:751` | ZMQ 发送队列 ≥1024 时 **pop_front 丢最老帧**;被丢的 REQ 其 pending 永不完成;若是查询,`qry_in_flight_` 永久卡在 1,后续所有 `ReqQry*` 永远返回 -2 | 无恢复路径的单点故障。慢消费者场景(远端团队模式)可触发 |
| S3 | `dispatch_md.cpp:19-24` 等生成的 `payload_as` | 载荷尺寸与结构体不符时回调"成功 + p=nullptr"(ErrorID=0) | 客户端把 NULL 当成功解引用即崩溃(demo 自己的 `demo_td.cpp:250` 都无保护)。版本错配/协议漂移时会放大成崩溃而非报错 |
| S4 | `api_core.cpp:780-799, 1070-1072` | auth 的 JSON 手工拼接不过滤 >0x7F 字节;on_auth_rsp 用 `find("\"ok\":true")` 子串匹配解析 | AuthCode/AppID 含 GBK 字节时产出的 JSON 非 UTF-8,Rust 侧 serde 解析失败 → 认证永无响应;子串匹配对 server 消息格式变化脆弱 |
| S5 | `api_core.hpp:150-152`;`dispatch_md.cpp:49-56` | SUB_MD 的 expected_responses 按发送合约数计数,server 少答/去重时 `bIsLast` 永不置真 | 订阅挂起;当前 server 恰好逐个应答,风险处于潜伏态 |
| S6 | `api_core.hpp:247` | `RegisterSpi` 无锁写 `spi_`;Init 后再注册与 reader 线程读存在数据竞争 | 厂商契约要求 Init 前注册,属边缘违约场景 |
| S7 | `api_core.cpp:621` | 每次收帧分配 ~1MB 缓冲再 resize | 高频行情路径的无谓分配 churn(功能正确,性能债务) |

设计债务:硬编码 `BrokerID=11/UserID=16` 魔法数(`api_core.cpp:811-818`,应取 `sizeof` 成员);`pending_` 无容量上限且每条目内嵌三个大结构;TCP 帧头解析与 `decode_frame_bytes` 两份重复解码;`GetApiVersion` 返回 `"CTPBuddy/0.1.0"` 与厂商版本串格式不同,解析版本号的下游可能异常;`build_msvc.py:107` 每次构建重跑 codegen 并回写仓库其他文件,构建副作用偏大。

亮点(值得保持):Release-from-callback 的自删除处理(detach + delete_on_exit)规避自 join 死锁;`GetTradingDay` 用 thread_local 拷贝解决悬垂指针;GBK 截断按字符边界且保 NUL,iconv 回退 '?' 与 Windows 行为一致;codegen 的 fail-loud 校验(纯虚函数数不符即退出)+ ctypes 交叉验证布局。

### 3.3 Python 控制平面(子评审)

正面:cli.py 是薄委托而非上帝模块(命令函数 10-30 行、延迟 import);journal/store/replay 三者通过共享契约显式对齐,`CONTRACT_SHA256` 随投影元数据与 replay 结果传播,契约漂移可被 upgrade/projection 校验捕获;`not_replayed` 语义严谨(分类计数、双侧从 core hash 剔除、永不误报 FAIL);确定性重放的工程深度(front_id 用 dummy 连接对齐全局计数器、`_step_until` 的防过冲逻辑及其注释);web 安全面完整(双重回环强制、CSRF compare_digest、Host/Origin/Sec-Fetch-Site 三重校验、路径穿越多重防御、SQL 参数化 + 表白名单、CSP)。

正确性问题:

| # | 位置 | 问题 | 影响 |
|---|---|---|---|
| P1 | `sdk/client.py:281-284, 342-344` | `_request`/`_query_once` 仅捕 OSError;`p.q.get` 超时抛出的 `queue.Empty` 直接漏给调用方(`_md_request` 却转换为 CTPError) | SDK 的异常契约违约:核心无响应时用户拿到的是 `queue.Empty` 而非文档承诺的 `CTPError` |
| P2 | `sdk/client.py:256-257` | `_read_loop` 把 `(OSError, ValueError)` 一律视为干净关闭,而 ValueError 包含 bad magic/版本不符/截断 | 协议漂移与对端关闭不可区分,排障困难(与 S3 叠加) |
| P3 | `store.py:316-322` | 投影 order_update 缺 order_sys_id/order_key 时按"inserts seq >= update seq 就近匹配"回填 | 同账户同 order_ref 跨日复用可能错配;投影可丢弃可重建,但应显式收紧或文档化 |
| P4 | `web.py:26, 228-229, 295` | `MAX_SETTLEMENT_CONTENT=512KB` 实际被 `MAX_BODY=8192` 压制,结算正文永远到不了声明上限 | 常量自相矛盾,校验形同虚设且误导维护者(注意:core 侧 `settlement.rs:81` 允许 512KB——两端上限不一致本身就是漂移) |
| P5 | `replay.py:105-110, 122` | `free_port` 的 bind-close-reuse TOCTOU;`_connect_admin` 用 `time.time` 而 client 用 monotonic | 低概率端口竞态;系统时钟回拨时超时行为异常 |

设计债务:`Client._wake_all` 死代码;`store.py` 全无类型注解(与 journal/replay/client 的全注解风格割裂);`walk()` 以字符串字面量分发事件类型;`Admin` 非线程安全且构造期做网络 I/O(当前"每请求新连接"用法规避了);`_step_until` 每 watermark 多次 admin.status 轮询(大录制 O(N) RTT);`assets/settings.js` 为无来源说明的压缩产物。

### 3.4 测试、CI 与交付(子评审)

测试金字塔形态健康:Rust 全 workspace 单测(双平台)→ 跨语言契约 vectors(Rust/Python 双端跑同一份 journal_vectors.json)→ 21 套 e2e 矩阵。e2e 是**真语义断言**(精确回报序列、maker/taker 共享 TradeID、三所 FAK 形状互不相容、平均成本法会算错的数字锁先开先平、双跑同 hash + 篡改必败的负控制);`review_regressions.py` 把历次评审发现固化为回归,是少见的好实践。

缺口:

| # | 问题 | 证据 |
|---|---|---|
| T1 | 无任何 Python lint(无 ruff/flake8,workflow 无对应 step),6567 行 e2e + 1771 行单测纯靠自觉 | `.github/workflows/core-tests.yml` 全文 |
| T2 | CI Python 全部 3.9,本地 `.python-version` 为 3.13,单版本矩阵 | `core-tests.yml:22,71,105,143` vs `.python-version` |
| T3 | `m4_install_shim.py` 未进任何 CI job(进程级 CLI 安装/还原路径无门禁,仅 test_py.py 覆盖同域单测) | matrix 与 linux 循环均无该文件 |
| T4 | e2e harness(find_core/free_port/wait_port/drain/Feed)在 8+ 个套件中各复制一份,无共享模块 | `m2_book.py:66-132`、`m2_flow.py:82-227`、`settings.py:14-23` 等 |
| T5 | `test_py.py` 用手写 main() 注册测试,新增 `test_` 忘记注册会静默不跑 | `tests/test_py.py:1741-1767` |
| T6 | 报单流控 e2e 依赖真实墙钟窗口 `sleep(1.05s)` | `m2_flow.py:73-75` |
| T7 | fuzz 仅是确定性 #[test](2 万次),无 coverage 引导长跑、无 cargo audit/deny、无定期 schedule | `core/ctpbuddy-wire/tests/fuzz.rs:32` |

交付面(release_package)安全细节扎实:来源锁死白名单且拒链接/重解析点、PE machine 校验、复制后复验 sha256、manifest 自认"commit 不证明二进制构建来源"——诚实,但也说明发布流水线(构建、签名、tag)尚未存在。

### 3.5 数据链路:calendar 与 refdata(子评审)

- `calendar/production/build_2026.py` 基本落实 provenance:六所来源带 URL/文号/正文 sha256/verified 分级;snapshot sha256 + facts_sha256 把来源、节假日、夜盘事实整体入 hash;MANIFEST 如实声明 gaps。风险:"统一六所日盘"契约实质由三所正文 + 摘录外推(CZCE/DCE/GFEX 无固定正文);manifest version 硬编码与重跑日期解耦。
- refdata 弱一档:789 行 instruments.jsonl 无快照日期/来源/sha256 登记(对比 calendar 的做法)。好在有 manifest 校验单测与行数一致性检查。
- Rust 侧 `calendar.rs` 的读取校验严格(schema/version/重复日/休市日不得带 trading_day/夜盘映射枚举闭集),缺失即拒绝——与 Python 授权侧"缺失夜盘映射也不允许推断"的口径一致。
- 审计闭环好:`audit_real_accounts.py` 用真实柜台导出验证资金恒等式/大边保证金/逐笔盯市;`audit_three_way.py` 严格区分 fail/not_evaluated 且禁 first-candidate。真实数据私有不入 CI 是诚实取舍。

---

## 4. 风险清单(跨子系统汇总,按严重度)

**P1——会在异常场景下造成下游"无告警挂起"或状态分叉,应优先处理:**

1. Shim 无心跳/请求超时,半开连接永久挂起(S1)。
2. Shim ZMQ 发送队列满丢帧 → pending 永不完成 → 查询闸门永久 -2(S2)。
3. journal 写失败静默丢弃事件,账本与 journal 分叉无实时告警(R1)。
4. `payload_as` 尺寸不符返回成功+NULL,协议漂移放大为下游崩溃(S3,与 P2 叠加)。

**P2——契约违约或排障性缺陷:**

5. SDK 超时抛 `queue.Empty` 违反 CTPError 异常契约(P1)。
6. SDK 把协议错误(ValueError)与对端关闭混同为干净关闭(P2)。
7. auth JSON 非 ASCII 未转义 + 子串匹配解析(S4)。

**P3——正确性边角与文档漂移:**

8. web/core 两侧结算正文字节上限不一致(8KB vs 512KB)(P4)。
9. `on_logout` 不校验身份(R2);无 tick 时涨跌停不设防(R3)。
10. 投影 order_update 就近匹配回填可能跨日错配(P3)。

**P4——工程卫生:**

11. CI 无 Python lint、Python 版本矩阵漂移(T1/T2)、m4_install_shim 无 CI 门禁(T3)。
12. e2e harness 八处复制、test_py.py 手写注册(T4/T5)。

---

## 5. 文档与实现一致性(专项核对)

DESIGN §0.6 的差距登记表经抽查**与实现相符**——这张表本身就是该项目最重要的导航资产。发现的漂移:

1. **§11.1 "保持零外部 crate 依赖"已过时**。事实:`ctpbuddy-wire`/`market`/`matching` 零外部依赖,`ctpbuddy-ledger` 有 serde_json,`ctpbuddy-server` 有 tokio/zeromq/serde_json/libloading/bytes/futures(`core/ctpbuddy-server/Cargo.toml`)。"离线可构建、单二进制分发"仍成立(Cargo.lock 锁定、纯 Rust 依赖),但"零外部依赖"的表述应改为"域 crate 零外部依赖,server 引入异步传输栈"。
2. **§6.4 "四处必须同步"低估了既有机制**。`gen_shim.py:660-679` 实际会回写 `msgs.rs` 与 `wire.py` 的 id 格式,且 CI 有 `git diff --exit-code` 门禁(`core-tests.yml:113-116`);真正的手工面只剩"id 的指派"本身。文档应更新为描述该 codegen + 门禁机制,而不是四处置人工同步的警示。(注:本项修正了 Python 子评审"wire.py 无兜底漂移"的判断——漂移敞口比其报告的小。)
3. **web 常量自漂移**(§3.3 P4):同文件内声明上限与实际上限矛盾,属于代码内文档失效。
4. 抽查一致项:帧格式(§6.2 vs `frame.rs`)、消息号分配(§6.3 vs `msgs.rs`)、流控分工表(§8.8 vs `order_gate`/`qry_gate`/FTD/connect 实现)、结算确认前置(§8.7 vs `handlers/order.rs:118-126`)、夜盘时间轴(§7.4 vs `session_ms`)——均相符。

---

## 6. 亮点(评审确认,非客套)

1. **文档诚实度是系统性资产**:0.6 差距登记、审计报告区分 fail/not_evaluated/skip 且 skip 用退出码 3 不冒充 pass、operations.md 如实记录"远端 CI 本轮未核验""独立子代理审查未能执行"。
2. **真实数据对账方法**(§8.7.1):459/976 份真实结算单逐项复核;"审计脚本自身的取样方式也是被审计对象的一部分"的教训(2024 子目录漏读后从 os.listdir 改 os.walk)被写进设计文档。
3. **错误码全集 299 条逐条对账** + `fill_errorcode_status.py --check` 幂等生成状态列,使"推送面"与"错误码"两个正交维度都可回归。
4. **错单双推送面的忠实复刻理由**(§8.12):"推错推送面比推错错误码更隐蔽"——只挂 OnRspOrderInsert 的客户端会在真实 CTP 上漏掉全部交易所层拒单,CTPBuddy 拒绝掩盖此类下游缺陷。这是仿真环境的价值主张,实现做到了。
5. **冻结按原始估算额 pro-rata + 终态统一解冻**的类型化设计,把一类比例缺陷挡在结构之外。
6. **review_regressions.py**:把评审发现固化为回归测试,形成"评审→测试"的飞轮。

---

## 7. 改进建议(优先级排序)

1. **Shim 心跳与超时(修 S1/S2)**:实现 PING/PONG(常量已在),DEALER 侧空闲 N 秒发 PING、M 秒未收 PONG 即触发 OnFrontDisconnected + 释放全部 pending;发送队列满时改为"拒绝新请求(-1/-2)而非丢弃最老帧",让背压可见。这是单项收益最大的改动。
2. **journal 写失败上浮(修 R1)**:`Journal` 增加连续失败计数/last_error,进 admin `status`;超过阈值可选择 fail-fast(停止世界循环)——"权威事件流"承诺下,宁可停机也不静默分叉。
3. **SDK 异常契约修复(修 P1/P2)**:超时统一转 `CTPError`;`_read_loop` 区分协议错误(raise/回调)与干净关闭。
4. **payload_as 防御(修 S3)**:尺寸不符时回调携带明确 ErrorID(如 -1 协议错误)而非成功+NULL。
5. **CI 补门禁**:ruff(pylint 类)+ Python 3.9/3.13 矩阵 + m4_install_shim 入 matrix。
6. **e2e harness 收敛(修 T4/T5)**:抽 `tests/e2e/harness.py`;test_py.py 改自动发现(或 unittest/pytest 结构)。
7. **文档漂移修正**:§11.1 依赖表述、§6.4 同步机制描述、web 结算上限常量统一(8KB 或 512KB 二选一并两端一致)。
8. **World 继续下沉(机会非债务)**:admin.rs 命令表驱动化(命令名 → (schema, handler) 注册表)消除白名单样板;`Conn` 按关注点拆分;`replay_archived_tail` 的事件处理下沉给 ledger/engine 自身的 `apply_event` 方法,消除第三份字符串分发。
9. **refdata manifest 补来源登记**:对齐 calendar 的 facts_sha256 做法,给 789 合约快照补来源/日期/哈希。
10. **on_logout 身份校验**(R2)与 SUB_MD 计数防御(S5):小改动,顺手修。

---

## 8. 结语

本项目最大的架构优势不是某个具体设计,而是**决策依据的可追溯性**:每个语义口径都能指到官方文档、真实数据对账或明确的取舍记录;每个未实现项都登记在案而不是被掩盖。这使得本次评审能够把注意力集中在真正的问题上——而真正的问题(shim 失败路径、journal 静默分叉、CI 卫生)都是局部的、可修的。按 §7 的优先级推进,不需要动任何架构决策。

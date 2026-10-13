# CTPBuddy 冗余与多余设计全面评审（简化视角）

- 日期：2026-10-13
- 评审基线：`review-hardening` 分支 `453f5b6`（vendor the pinned libzmq source tarball），工作树干净
- 评审方式：由六个独立子评审分别通读 Rust server、Rust wire/market/matching、Rust ledger、C++ shim 第一方全部源码、Python 包全部源码、tests/tools/CI/数据链路；主评审通读 `DESIGN.md`（含 §0.6 差距登记）、ADR-0001/0002 与既有评审 [`2026-10-13-architecture-review.md`](2026-10-13-architecture-review.md)，并对子评审的头条主张逐一抽查复核（registry.hpp 死枚举、docs 头文件 md5 双份、ledger 无用依赖、`qry_gate`×11、`.iter().find()` 样板、`free_port`×10、`WEB_TABLES`/`iter_frames`/`_wake_all` 死代码、CI 子集重跑、两处劣化 find_core 变体、6 个抽查结构体零引用、`Catalog` 转发壳、`virtual_ms` 逐次解析——全部坐实）
- 铁律：**行为保持**。所有提案不改变任何对外可观测输出（帧字节、回报顺序、错误码与错误串、journal JSON、查询行、测试断言集合、CI 覆盖集合）；性能项单列并要求输出逐位不变。

---

## 1. 总体结论

**手写代码的"多余"是有限且高度集中的；最大的冗余不在手写代码里，而在生成面与拷贝面。** 全仓可归为六块，按体量排序：

| # | 冗余块 | 量级 | 性质 |
|---|---|---|---|
| 1 | Rust 生成结构体 485/519 零引用（`core/ctpbuddy-wire/src/generated/structs.rs` 45,473 行中约 42,500 行） | **≈ -42,500 行** | codegen 无过滤全量发射（§3.1 WM-G1） |
| 2 | shim 死枚举 `registry.hpp` StructId 519 项零消费者 | **-515 行** | 第三份 id 拷贝，真正被消费的只有同文件的 static_assert（§4.1 SH-F9） |
| 3 | CTP ABI 头文件在 docs 下字节级双份入库（md5 已证相同） | **-693KB / -5 文件** | chm_to_html 产物与 CHM 原件并存（§4.2 SH-F13） |
| 4 | e2e harness 成片复制 + 3 处已劣化变体 | **≈ -800 行（测试侧）** | `free_port`×10、`find_core`×8、`drain`×7、启停样板×16 文件（§6.1 TT-N1/N3） |
| 5 | server 控制面手工样板（admin 白名单×13、query 前导×12、journal_contract 手查 Map×20+） | **≈ -600 行** | 自研 `json::Value` 无 derive 的既知代价的具体化（§2.1 SV-R1/R2/R3） |
| 6 | 热路径纯冗余分配（逐 tick 解析时间串、tick 双克隆、无订阅者仍整帧克隆、无 journal 仍建 payload、零积压仍排序全连） | LOC 中性，**稳态分配大幅下降** | 全部零行为变化（§7） |

**账面合计**（相互独立、均含行为保持论证）：

- 手写代码净删 ≈ **-2,100 行**（不含下条搬移）；
- 纯搬移（结算算法从 admin.rs 归位 settlement.rs）**-440 行体积**（LOC 0）；
- 生成物 ≈ **-43,000 行**（第 1、2 项）；Python 侧生成物建议**保留全集**（§5 N19，理由见该节）；
- 仓库体积 **-693KB**（第 3 项）。

**同样重要的负面清单**：子评审逐项核验了既往评审誉为"深模块"的领域复杂度（费用双腿、双轨盈亏、冻结比例释放、先开先平正交轴、IocLayout 三所形状、exchange_rules 表驱动、FAK 缓冲、staged-clone 原子结算、Money 顺序无关求和），**全部为必要设计而非冗余**，见 §8 防误删清单。本次未发现任何需要推翻的架构决策；所有提案都是局部收敛。

**既往（同日架构评审）简化类建议的现状**：正确性修复已全部落地（心跳、队列拒绝、payload 防御、auth 转义、flush 记账等，见 65f9aa2/8ca25c9）；但简化类条目全部仍在——admin 样板精确计数 13 处、`replay_archived_tail` 第三份字符串分发未动、`_wake_all` 死代码仍在（client.py:288）、`walk()` 字面量分发仍在、`settings.js` 仍无源码、e2e harness 比当时认知的"8+ 套件"更严重且已产生 3 处劣化变体。本文为这些条目给出精确计数与可执行方案。

---

## 2. Rust server（`core/ctpbuddy-server`，子评审 + 主评审抽查）

事实基线：库面唯一消费者是 `src/main.rs`；近四个提交未触及下述样板。

### 2.1 重复样板（高价值）

| ID | 位置 | 内容 | 方案 | LOC |
|---|---|---|---|---|
| SV-R1 | `journal_contract.rs`（47-52、68-69、77-78、109-126、133-151、176-200 等 20+ 处） | `map.iter().find(\|(k,_)\| k=="x")` 手工查 `serde_json::Map`，而同文件 40 行已在用 `map.get("events")`——两种写法并存；6 个合法类型字面量另重复 3 份（167、217、286-295），`matches_data_type`(570) 已是判别函数 | 全部换 `map.get(k)`；类型清单提 `known_type()`；`data_has_required_fields`(434-498) 三级链式导航提为一次取 `spec` | **≈ -130** |
| SV-R2 | `handlers/query.rs`（qry_gate 前导 11 处：558、577、611、653、689、711、798、829、852、881、909；handlers.rs:538）+ 手写"逐行发帧 + QRY_LAST"尾部 6 处 | `if !self.qry_gate(...) { return; }` + session 解包（-3 错误面）逐字重复；`send_rate_rows<T>`(784-795) 已是泛型收尾却只有 4 个查询在用 | 提 `qry_session()`（gate+session+错误面复用）；6 处尾部改调 `send_rate_rows` | **≈ -70** |
| SV-R3 | admin 面字段校验 13 处（HashSet+白名单 9 处：admin.rs:649-656、743-752、919-924，settings.rs:45-52、178-183，live.rs:19-28，settlement.rs:21-37，auto_settlement.rs:63-71；exact-shape `matches!` 3 处：admin.rs:44-49、578，flow_replay.rs:260） | 既往评审"约 15 处"的精确化；错误文案"含未知或重复字段"6 处相同 | `json.rs` 加 `exact_keys`/`reject_unknown_dup` 两个纯函数，命令改 `const CMD_KEYS` 表驱动；错误串逐字保留 | **≈ -80** |
| SV-R5 | `settings.rs:67-71` vs `156-168` | 同一张 6 键 min/max 数字表写两遍（patch 校验表 + schema 展示表），纯靠人肉同步 | 单一 `SETTING_DEFS` 常量表双驱动，消除两表漂移面 | **≈ -40** |
| SV-R6 | `main.rs:146-182` vs `279-302`（另 130-145 vs 259-278） | env 块与 CLI 块对同一批设置重复解析（三键 settings + qry/order-freq） | 提 `apply_flag(cfg, key, raw)`，错误路径由调用方定（exit vs Err） | **≈ -35** |
| SV-R7 | 杂项（各 2-14 处）：admin 回复样板 ×14、账户排序 ×3（admin.rs:183-188、786-791，settlement.rs:173-178）、`auth_code_ok`(handlers.rs:69-77) 与 `constant_time_eq`(lib.rs:353-356) 同构、env 布尔解析 ×3、login/logout 身份校验 ×2、`is_finite ? n : Null` ×4、"查询未就绪"字面量 ×2 | 小型重复成片 | `ok_cmd()`/`accounts_sorted()`/`env_flag()`/`conn_identity()`/`json::num_or_null()`/常量化 | **≈ -60** |

### 2.2 死代码与不可达（全仓 grep 核验）

| ID | 位置 | 内容 | LOC |
|---|---|---|---|
| SV-D1 | `state.rs:31-44、83-91` | `write/write_with_flow/read/read_with_flow` 四个 pub 包装零调用方（lib.rs 只用 `_meta` 版） | -25 |
| SV-D2 | `calendar.rs:176-248` | `night_session`/`night_status` 仅自测试使用；连带 `NightPolicy` 读路径（264-386）无生产消费者。**注意**：d4fdeae 刚加的 exchange-specific 夜盘建模可能是为后续接线预留；保守方案只把两个查询函数 `#[cfg(test)]`；删除解析会让畸形 `night_policy` 从启动报错变静默接受，属行为变化，**需作者确认路线图后再定** | -75（保守档） |
| SV-D3 | `journal_contract.rs:583-624` | `canonical_event` 仅本文件 vectors 测试使用 | 0（`#[cfg(test)]`） |
| SV-D4 | `dtime.rs:10-37` | `now_wall` 4 元组 3 个值被弃（journal.rs:245 唯一消费者），`civil_from_unix` 只服务被弃部分 | -20（并消除每条 journal 记录 2 次多余 `format!`） |
| SV-D5 | `security.rs:25`（`load(_endpoint)` 死参数）、`flow.rs:17`（`origin();` 无必要调用，OnceLock 惰性初始化已保证） | 零散 | -2 |
| SV-V1 | `admin.rs:753-764` → `1001-1006` | settle_day 结算价被校验两遍，第二处错误分支在第一处之后不可达 | -6 |
| SV-V2 | `handlers.rs:607-614` | `on_settle_confirm` 同帧两次 `session()`（各 2 次 String clone） | -3 |

明确**不建议删**的"看似冗余"：journal 写时校验 + 恢复时再校验（journal.rs:261 + lib.rs:1366）是文件可能被外部改写前提下的 fail-closed 双保险（既往评审同结论）。

### 2.3 结构归位（纯搬移）

- **SV-S1**：结算域算法（`settle_day` admin.rs:714-909、`stage_expired_liquidations` 958-1143、`record_forced_liquidations` 1145-1168，共 ~440 行）错放在 admin.rs；`settlement.rs` 名义结算模块却只装报告解析。三函数逐字搬到 settlement.rs，admin.rs 回到纯命令分发（1272 → ~830 行）。风险低（机械 move）。
- **SV-S2（明确结论）**：settlement.rs vs auto_settlement.rs **不合并**——报告解析 vs 墙钟调度器，职责正交。flow.rs vs private_flow.rs 同理**不冗余**（五闸门共享的 WindowBudget vs CTP 私有流重放，仅命名相近）。
- **SV-S4**：live.rs 分页半用半弃（`page()` 只服务 2 处，orders/trades 手写 skip/take，live.rs:97-100、145-148）——统一二选一。
- **SV-S5**：`Conn` 16 字段（lib.rs:588-610）中四个流控字段 + 订阅集可收成子结构，构造点仅 2 处；建议随其他 Conn 改动顺手做，不单独立项。
- **SV-F1（与 ledger 报告合并）**：`mark_to_market`（lib.rs:1821-1831）与 `mark_to_market_now`（query.rs:508-517）函数体逐行相同——删一份统一命名。

---

## 3. Rust wire / market / matching（子评审 + 主评审抽查）

### 3.1 生成面（本次评审最大单项）

**WM-G1｜Rust 生成结构体 485/519 零引用。** `structs.rs` 519 个 `pub struct`，全仓（含测试）实际引用仅 34 个（子评审逐类清点：银行/银期 ~40、期权组合 ~60、SPBM/RCAMS ~40、Sync* ~45、Qry* 镜像 ~110、管理档案 ~70、其他回执 ~80；主评审抽查 6 个"未用"结构体全部 0 引用）。生成器 `gen_structs.py:198` `for name in sorted(structs)` 无过滤全量发射，而 `MESSAGE_STRUCTS`（gen_structs.py:60-110）已是现成的白名单机制。**方案**：`gen_rust` 增加发射白名单（= MESSAGE_STRUCTS.values() ∪ 34 个在用）；`registry.json` 的 0x2000+ 全量索引 id 分配**保持不变**（只过滤 Rust 输出），Python/C++ 侧不动。**≈ -42,500 行**（45,473 → ~3,000）。代价：未来新增查询面需扩白名单（一次性编辑，与 MESSAGE_STRUCTS 维护同型）。

### 3.2 重复逻辑

| ID | 位置 | 内容 | LOC |
|---|---|---|---|
| WM-D1 | `engine/emit.rs`（54-127、233-246、262-280 共 12 处） | `next_notify_seq → 盖戳 → events.push(Order(...))` 三行式重复 12 处；`push_transition` 与 `emit_fill` 前态推送是同一段代码的手工内联 | 提 `push_order_row()`，-33 |
| WM-D2 | `refdata.rs`（1009-1021/1055-1067/1103-1115 去重检测；554-579 写入分流；649-673 broker 回退；709-727 排序扫描） | 三张费率表的加载/写入/查找/排序四组逻辑各 ×3 份，逐行同构只差表名 | 泛型 `insert_rate<T>`/`lookup_for<T>`，错误串以表名参数化逐字保留，-80 |
| WM-D3 | `engine.rs:827-859` vs `873-908` | `available_depth` 与 `best_counterpart` 的簿侧扫描谓词、tick 深度消耗循环双写 | 提公共迭代器（注意 break→take_while 等价），-12 净 |
| WM-D4 | `refdata/jsonl.rs:31-94` | 五个访问器各自重写 `iter().find` 键查找 | 提 `get()`，-17 净 |
| WM-D5 | `refdata.rs:1210-1234`（两张同构 match 表）、`catalog.rs:291-331`（向上找 refdata 双循环）、`engine.rs:462`/`emit.rs:291`（`format!("{:010}")` ×2）、`matching/lib.rs:156-173`（`from_utf8_lossy` 与 wire `cstr` 两套定长→String 约定并存，字段全源自 `set_cstr` ASCII） | 零散 | -44 |

### 3.3 跨 crate 重复

| ID | 位置 | 内容 | LOC |
|---|---|---|---|
| WM-F1 | server `handlers/order.rs:660-680` 手写 4 份 `{let mut v=[0u8;N]; set_cstr(...)}` vs matching `lib.rs:31-35` 的 `to_fixed`（ledger tests 已在用） | 4 处改调 `to_fixed::<N>()`，风险零 | -14 |
| WM-F2 | `order.rs:167-195` 流控拒绝路径手建 22 字段 OrderField vs `engine.rs:1110-1155` `build_order_field` | matching 暴露 `order_field_from_intent()`，注意保留 server 侧 CombHedgeFlag 覆写层 | -5 净 |
| WM-F4 | `market/lib.rs:336-354` `valid_compact_date` vs server `calendar.rs:405-414` `valid_ymd` 同一闰年/月天数表 | 保留 market 侧（更底层），server 改调 | -12 |
| WM-F5 | matching `engine.rs:60,62` 死错误码 30/31 与 ledger `lib.rs:30-31` 同值双定义 | matching 侧零引用（见 WM-G3），删除即消重 | -6 |
| WM-F3 | `jsonl.rs:117-251` 手写 JSON 解析器（含完整转义表）vs server `json.rs`（serde 驱动） | 两套转义语义并存；零运行时依赖约束下最小方案是把 jsonl.rs 搬到 wire（或微 crate）共用，**不动** server 的 serde 路径。涉及 crate 拓扑，需与 DESIGN §13 口径对齐，列后续独立 PR | -180 净 |

### 3.4 死代码 / 无谓抽象

- **WM-A1**：`Catalog` 是 `RefData` 的纯转发壳——`struct Catalog { inner: RefData }`，31 个方法约 27 个一行转发（catalog.rs:40-254，crate 注释自认"thin name"）；非转发逻辑只有 `estimated_commission(_for)` 与 `bundled`。方案：`pub type Catalog = RefData;` + 三个真方法移入 RefData impl，调用点零改动。**-140**。
- **WM-G2**：matching/catalog 生产零调用的 pub 面整表（逐个 grep 核验）：`catalog_mut`（engine.rs:331）、`refdata_mut`（catalog.rs:67）、`maker_fill_at_limit()` getter（engine.rs:301）、`Catalog::estimated_commission/margin/commission`（非 `_for` 版，catalog.rs:157-226）、`OrderCommRate::insert_fee/action_fee`（refdata.rs:448，申报费目前只投影不收取）、`cross_margin_rules()` getter（refdata.rs:603）、三个未排序 `*_rows`（679-695，注释称 query 用、实际走 `*_for`）、`*_rows_sorted` 仅测试用（709-727）、`Direction::opposite/label`、`OffsetFlag::label`（lib.rs:60-115）、`Catalog::is_empty`（catalog.rs:85）。合计 **-110**。
- **WM-G3**：死常量 `ERR_NO_POSITION/ERR_INSUFFICIENT_MONEY/ERR_NO_COUNTERPARTY`（engine.rs:60-65，拒绝实际由 ledger 侧承担）；`frame.rs:30` `encoded_len` 降私有。`msgs.rs:34-35` `LOGOUT/LOGOUT_RSP` 属跨语言 id 表成员（Rust 侧未路由），**保留**、加注释防误判。
- **WM-V1**：一笔 insert 上静态校验最多跑两遍——`order.rs:200` `engine.check()` 后 `order.rs:293` `submit()` 首行再 `check()`（engine.rs:431），两次之间 catalog/intent 均未变，结果恒等。方案：engine 加 `submit_prevalidated()`（server 已 check 过），或删 server 侧 decode 的 finite 预检（拒绝码与消息一致）。**不建议强改**（收益小），登记为顺手项。

### 3.5 明确"非冗余"（matching/market/wire 覆盖声明，防误删）

exchange_rules 四字段各有独立消费者；IocLayout 三变体三所形状确实互异（tests 锁定）；FAK 缓冲是"后知剩余量"语义需要；`gbk::encode_with_bounds` 的 (bytes, ends) 是字符边界截断必需；frame 层校验只在 `read_from` 一处（TCP/zmq/replay 复用，无双重校验）；msgs id 表是 gen_shim 单一事实源的三语言镜像；Playback 锚点三字段各有测试；build.rs 契约代码生成是 ADR-0001 落地；server 未平行实现 market 的时间/帧逻辑。

---

## 4. C++ shim 第一方（子评审 + 主评审抽查）

### 4.1 死表面

- **SH-F9｜`registry.hpp` StructId 枚举 519 项零消费者**（registry.hpp:8-528；全仓 grep 仅 gen_structs.py:376 发射处）。api_core.cpp include 该文件只为 531 行起的 static_assert（真正有价值部分）。方案：codegen 停发枚举（或仅发 static_asserts）重新生成。**-515 行**（1051 → ~535），零风险。id 权威表是 registry.json，C++ 枚举是无人消费的第三份拷贝。
- **SH-F10**：`msgs::PONG/LOGOUT/LOGOUT_RSP/ADMIN_REQ/ADMIN_RSP` 在 shim 内零引用（api_core.hpp:52、102-105）。**手删会被下次 regen 加回**（gen_shim.py:663-682 只追加），要么接受（同步机制代价，api_core.hpp:88-89 注释已声明"三处同号"是有意设计），要么同步时过滤（-5，可校验性略降，取舍项）。
- **SH-F11**：gen_shim.py 死分支（472-499 两条永不执行的 errrtn 分支，当前所有 errrtn 行都设 `payload_input: True`）、死助手 `cstr()`（269-271）、gen_structs.py:367 死形参 `skipped`。生成输出逐字节不变。-33。

### 4.2 拷贝冗余

- **SH-F13｜CTP ABI 头文件双份入库（-693KB）**：`docs/api-doc-html/files/` 与 `docs/notes/assets/compiled_html/` 下 ThostFtdcUserApiStruct.h、ThostFtdcUserApiDataType.h、ThostFtdcTraderApi.h、ThostFtdcMdApi.h、DataCollect.h **md5 完全一致**（主评审已复算）。前者是 chm_to_html.py:657-668 从后者复制的产物，CI 借用作 SDK（core-tests.yml:111、152 `--sdk docs/api-doc-html/files`）。方案（a）：CI 与文档改指 `--sdk docs/notes/assets/compiled_html`（字节相同 → 构建产物相同），chm_to_html 停止复制 .h；方案（b）：保留现状、两目录互指 README。**倾向 (a)**：消除"改一份忘另一份"的漂移面。注意 TGate*.h 仅 compiled_html 有，不可反向合并。
- **SH-F1｜帧头 13 字节编解码三处重复**：`Frame::encode`（api_core.hpp:119-137）、`decode_frame_bytes`（api_core.cpp:526-536）、reader_main TCP 分支第三次手写（879-899），magic/version/长度校验各一份。抽 `decode_header()`，保持 TCP 路径"先收头、len 越界即断线"的顺序语义。-25。
- **SH-F2｜`is_query_msg`（api_core.cpp:1061-1079）手工 12 项清单 100% 复制 dispatch 表 qry 列**（dispatch_td.cpp:501-512），两份数据无交叉校验。改为 `find_row_by_req(msg) && qry_last != nullptr`（同为线性扫，返回集合逐项相等）。-19，消除一处会漂移的双份真相。
- **SH-F3｜auth 两条路径近似重复**（api_core.cpp:741-755 vs 757-817）：JSON 前缀与 in-flight 状态四连各一份；BrokerID=11/UserID=16 魔数 ×4（744-751；对照 772-775 已正确用 `sizeof`——既往评审项**仍未修**）。抽 `auth_json_prefix()` + `bind_auth_locked()`，尺寸取自 `sizeof`。-15。
- **SH-F7**：PUB endpoint 归一化（418-422）与 `parse_address`（252-262）同构。-4。**SH-F8**：`GetApiVersion` 版本串两份提常量（两 exports 文件**不合并**——dllexport 工厂符号必须分落两个 DLL）。

### 4.3 构建脚本

- **SH-F6｜build_msvc.py / build_linux.py 平台无关部分 ~80-100 行逐行同构**：ZMQ 版本/SHA/URL 常量、tarball 获取/校验/解压、cmake 关闭项清单、sdk_include_dirs、targets 源表。抽 `build_common.py`，两脚本只留平台驱动（vcvars/cl/`/LD /Fe` vs argv/g++/soname——真实差异不强行合一）。-80。
- **SH-F5｜build_msvc.py:193 每次构建重跑 codegen 并无条件回写仓库文件**（gen_shim.py:693-728 无内容比较总是重写，还会触碰 msgs.rs/wire.py），与 build_linux 的 `--regen` 可选不对称（既往评审项仍未修）。最小修法：gen_shim 写前先读旧内容、相同则跳过（write-if-changed，+6 行），CI `git diff --exit-code` 门禁继续兜底。
- **SH-F4｜ZMQ 收帧每帧全新 ~1MB 缓冲**（既往评审项仍在）：`zmq_recv_part` 每调用 `resize(WIRE_MAX_PAYLOAD+...)`，`read_zmq_frame` 循环内 `first`/`frame`/`ignored` 按值新建——收一个 13 字节 PONG 也分配 ≥1MB，空闲心跳周期同样反复进入。把缓冲提升为 reader 线程私有成员复用（reader 单线程独占；`decode_frame_bytes` 以长度为准，复用安全）。TCP 读路径无此问题（栈缓冲头 + 按需 payload），形成对照。

负面清单（勿动）：TD/MD 生成代码同型函数是双 DLL/双 SPI 的 ABI 形态必需；`unsupported()` 蹦床覆盖全部纯虚是 vtable 二进制兼容硬性要求；demo 自带 GBK 转码是"零 CTPBuddy 特定代码"的 e2e 前提；msgs 四处镜像是单一来源+同步机制的设计选择；`utf8_to_gbk` 每次开/关 iconv 是线程安全取舍（仅错误路径）。

---

## 5. Python 包（`py/ctpbuddy`，子评审 + 主评审抽查）

### 5.1 既往项现状

`Client._wake_all` 仍死（client.py:288-291，`close()` 内联同款逻辑）；`walk()` 仍字面量分发（replay.py:611-629）；`_step_until` 仍 O(N) 全量 status 轮询（replay.py:224-251，且 on_watermark 先 poll 一次、`_step_until` 立即再 poll 一次）；`assets/settings.js` 仍是无源码压缩产物（tests 直接对产物断言）。

### 5.2 死代码（全部 grep 核验，零风险删除）

| ID | 位置 | 内容 |
|---|---|---|
| PY-N9 | wire.py:137-143 | `iter_frames` 零引用 |
| PY-N7 | store.py:37-38 | `WEB_TABLES` 零引用（web.py:28 的 `WEB_PROJECTION_TABLES` 是内容相同的工作副本——二选一：删 store 侧或 web 改 import，保单一真相） |
| PY-N10 | client.py:32-33 | `LOGOUT/LOGOUT_RSP` 导入未用 |
| PY-N11 | sdk/__init__.py:4,8 | calendar 再导出无消费者（顶层包仍导出，公共入口不受影响） |
| PY-N12 | calendar.py:338-340 | `load_calendar` 零调用（测试用 `TradingCalendar.from_file`） |
| PY-N13 | admin.py:156-163 | `settle_day` 的日历推导分支仓库内不可达（replay 总传 next_trading_day、web 走原始 cmd）；删除为语义收缩（错误来源变化），**需一次决策** |
| PY-N14/15/16 | refdata `csv_dir` 工厂、`_call_flexible` 单行包装与 `_instantiate` 重复分支（`type` 分支是 `callable` 分支真子集）、scenario `_scalar/_split_flow` 死参数 | -12 |

### 5.3 重复逻辑

- **PY-N1｜client.py 三份相同的 pending-socket 管道**（`_request` 293-328 / `_query_once` 349-385 / `_md_request` 508-553）：注册→closed 检查→send_lock 发送→队列/超时转 CTPError→finally pop，逐字重复三份。提 `@contextmanager _slot()` + `_send()`。-40。
- **PY-N2｜web.py 重复实现 `Admin.settlement_report` 的 GBK 归一化**（web.py:279-285 vs admin.py:134-144，且 314 行同请求内第二次 encode）。改调 `client.settlement_report`（与 replay.py:518 一致），长度校验复用一次编码。-8。
- **PY-N3｜核心进程 argv 构建样板**：cli.py:141-175 vs replay.py:701-722。提 `core_argv()`。-12。
- **PY-N5/N6**（登记）："64 位 hex sha256" 校验 4 处（大小写策略各异，不强统一）；JSON 重复键 hook 3 份（refdata 与 web 两份逐字等价可合一，calendar 份异常类型不同保留）。
- **PY-N8**（登记，不建议立即合并）：scenario.py YAML/JSON 双轨校验器平行 ~120 行；**不能**简单合一——两轨行为有差异（YAML 轨不拒未知字段），合并即改报错集合。仅抽共有小片段。
- **PY-N17**：`verify_events`（journal.py:169-173）重复 contract 已做的 seq 整数/vt_ms 有限性检查；删除后畸形输入的 problems 列表会变短（退出码与 hash 路径不变）——若"诊断消息逐字不变"也算行为，则降级为登记。

### 5.4 生成面

**PY-N19｜`generated/structs.py` 93% 无 py 侧引用（481/519）**，但**建议保留全集（方案 b）并在 DESIGN 注明**：它是 gen_structs 单一来源的全集镜像，零维护成本、与 shim codegen 同源；白名单裁剪（-1,440 行）有漏名即运行时 KeyError 的风险，且 `verify_layout` 测试（test_py.py:55-56 断言 n==519）依赖全集。与 WM-G1 的区别：Rust 侧是编译产物、无消费者即死重（裁剪零风险）；Python 侧是运行时按名查询的表（`SIZES` 动态查），裁剪收益/风险比不划算。

---

## 6. tests / tools / CI / 数据链路（子评审 + 主评审抽查）

### 6.1 e2e harness（既往项的精确化，**劣化已实际发生**）

逐字节比对（去空行归一化）的精确清单：

| 助手 | 份数 | 位置 |
|---|---|---|
| `free_port` | 10（7 相同 + 3 变体改名 `port()`） | py/replay.py:105 + 7 个 e2e 相同拷贝 + m3_order_sysid:27、m3_bootstrap:50、m4_real_replay:144、settings:14 |
| `find_core` | 8（7 字节级相同） | py/replay.py:85 + m1_smoke:63、m2_book:66、m2_flow:82、m2_ioc:70、m2_journal:57、m2_scenario:53、m3_refdata:63 |
| `wait_port`（+`wait_admin` 变体） | 7 + 3 变体 | m1_smoke:83 等 6 处相同 + m3_order_sysid:35（timeout 硬编码）等 |
| `drain` | 7 | m2_book:156 等（仅 quiet 秒数 0.2/0.3/0.35 不同） |
| `st`/`tr`/`rows`/`close`/`wait_idx` | 5/5/2/6/8 | m2_* 系列成片 |
| `Feed` 类 | 3（m2_book ⊃ m2_flow 逐行相同核心） | m2_book:197-227、m2_flow:205-227、m2_scenario:114-133 |
| 启停样板（Popen+shutdown+communicate+`"---- server log ----"`） | ~16 文件 | 仓库已有正确形状：operations.py:38-61 的 `@contextmanager server()` |

**三处劣化变体（复制已经造成实际代价）**：settings.py:12 内联 find_core 只找 `debug` 无 release/Linux 名（该套件进不了 linux job 的直接原因）；m3_order_sysid.py:29-31 硬编码 `target/task43/debug`（遗留任务期 profile）；m3_bootstrap.py:164 硬编码 `REPO/target/task41/...` 且目录漏了 `core/`（主评审已复核原文）。另 `TIME_SCALE` 旋钮块在 m2_flow/m2_surface 复制，flow_replay.py:74/81 的同类墙钟等待却**没有**旋钮（慢机调不了——复制不一致造成的真实缝隙）。

**方案（TT-N1/N2/N3/N4）**：新建 `tests/e2e/harness.py` 收编恒等助手（`drain` 参数化、`Feed` 取 m2_book 超集、`prepare_scenario` 取 m2_scenario 超集 + `with_compile` 参数）；启停样板泛化 `harness.server()`（以 operations.py 的现成 ctx manager 为蓝本）；`TIME_SCALE` 入 harness 并给 flow_replay 接入。现有 import 枢纽（m1_smoke/m3_refdata/m2_flow 被 9 个文件 import）保留套件专属常量。**净 ≈ -780 行**；迁移一次一个套件、每个 matrix 项独立验证；CI 调用点与断言集合逐项不变。

### 6.2 test_py.py 注册机制（TT-A2）

25/25 全注册、当前无漏（AST 比对），风险是潜在的（忘写 main() 即静默不跑）。**最小方案**：main() 顶部加 4 行漂移哨兵（`defined == listed` 集合断言），调用序列原样保留——输出逐字节不变，忘注册从静默变响亮失败。真自动发现会改输出顺序/格式，不满足行为保持，仅登记。

### 6.3 CI（TT-N5/N6/N7/N8）

- **N6**：unit job 内 `cargo test -p ctpbuddy-server journal_contract`（core-tests.yml:44）是同 job 上一行 `cargo test --workspace`（L41）的子集重跑——删 cargo 行，保留该步独有价值的 `python tests/test_journal_contract.py`。主评审已复核原文。
- **N7**：e2e 套件清单双维护（matrix L63 21 项 vs linux 循环 L164 19 项）且已漂移（`settings`/`m4_real_replay` 仅 Windows、`shim_hardening` 仅 Linux；settings 缺 Linux 的根因即 6.1 劣化 find_core）。方案：单一 `suites` 准备 job 输出 JSON，两平台共用 + 显式排除名单（排除理由注释固化）。
- **N8**：4 个 job 重复 checkout+setup-python+rust+pip 四件套 → 本地 composite action。
- **N5**：`m4_install_shim.py` 仍无任何 CI 门禁（既往 T3，spec 已标"单独批次"）——纯增量 +2 行 yml。
- 已核**非冗余**：双平台 19 套重跑（.exe/so/路径/socket 差异）、unit 与 linux 的 cargo/projection 双跑、docker 与 linux 不重叠、m1_shim_e2e ×2 是双模式。

### 6.4 tools 与数据链路

- **TT-N9**：m4_real_replay 与 audit_three_way 三重重叠——`trim` 逐字节相同却本地复制（已在 import 路径上）；`option_like` 双份；real_replay_audit 与 m4_real_replay 是两个并行"可回放性扫描器"（合并需保留两套判据独立出口，进阶档谨慎）。
- **TT-N10**：tools 间 CSV 解码阶梯三份实现且**编码顺序互不一致**（audit_real_accounts 的 gbk-first 是对其数据形态的刻意选择）——参数化共享，**禁止**顺手统一顺序。
- **TT-N11**：audit_query_expectations 被 audit_three_way 取代仍并列维护（docs-site 自认"更早的子集脚本"）——标注 superseded，中期并入后退役。
- **TT-N12/N14**：`gen_gbk_table.py` 与 `calendar/production/build_2026.py` 产物已入库但无再生成一致性门禁（对照 shim codegen 的 `git diff --exit-code` 模式）——可选加 `--check`。
- 已核非孤儿：scenarios/ 两目录（README/Dockerfile/测试断言锁定）、registry.json（codegen 消费）、calendar/production 五文件互相咬合。

---

## 7. 热路径性能冗余（全部零行为变化，输出逐位不变）

按"每秒发生次数 × 单次浪费"排序：

| ID | 位置 | 浪费 | 修法 |
|---|---|---|---|
| WM-H1 | `market/lib.rs:102-104` → `parse_time_ms`(356-365) | `Tick::virtual_ms()` 每次调用重新解析时间字符串（1 次堆分配 + 3 次 parse）；poll while 条件、advance、seek、transforms、`last_md_time` 全量扫各调一次——回放百万 tick = 每 tick 2-4 次解析 | `Tick` 加非序列化 `update_ms` 缓存（或 OnceCell），提供单一构造入口防测试旁路 |
| WM-H2 | `market/lib.rs:516、548` + `engine.rs:738-739` | 每 tick 两次整 Tick 深拷贝（5 String + 4 数组）：poll 释放即 clone（server 只读遍历），on_tick 再 clone 存 `last_md` | poll 返回区间引用；`last_md` 改轻量 MdSnapshot（只存 engine 真正读的 5 个字段）。借用重构，中风险，tests 全绿为门槛 |
| SV-H4 | `lib.rs:1836-1839、1924-1927、1985-1992` | 默认部署（未设 PUB 端点，broadcast_tx=None）下每个行情 tick 仍整帧深拷贝 + topic `format!` 然后整包丢弃；私有通知同 | publish 前判 `broadcast_tx.is_some()` |
| SV-H1 | `lib.rs:2029-2096` | `data_dir` 为空（无 journal）的场景运行中，每笔 order/trade 事件仍构建完整 JSON 树 + day 克隆后丢弃 | 三个 `journal_record_*` 开头 `if self.journal.is_none() { return; }` |
| SV-H3 | `lib.rs:1745-1746` | `drain_ftd` 每 10ms 脉冲收集并排序全部 conn id，`ftd_pending` 绝大多数时刻为空 | 先判 `any(!ftd_pending.is_empty())` 再排序 |
| SV-H2 | `lib.rs:1821-1831`（PULSE=10ms） | 每 10ms 无条件构建 last_prices + pre_settlements 两个 HashMap（即使无新 tick）；position_detail 查询一次构建 3 个 map | 最小档：查询路径复用 mark_to_market_now 已建的 map；盯市时机变化（缓存版本号）牵动时点余额，**勿轻动** |
| WM-H3 | `engine.rs:1112-1117`、`emit.rs:148-153` | 每行 order/trade 回报 12 次 GBK 往返（`cstr→String→set_cstr`，ASCII 输入下严格恒等，每对 1 个 String + 两次码表查换） | `copy_fixed::<N>` 直拷 + debug_assert 往返恒等（已核无绕过 set_cstr 的写入路径） |
| WM-H4 | `engine.rs:1147-1150`、`emit.rs:159` | 每行回报 3-4 次 `format_hhmmss` 字符串分配 | `write_hhmmss(buf, ms)` 直写定长字段 |
| SH-F4 | `api_core.cpp:544-560、590-598` | shim ZMQ 每帧（含 13 字节 PONG、空闲心跳）全新 ~1MB 缓冲 ×1-3 次 | 缓冲提升为 reader 线程私有成员复用 |
| PY-NP1 | `replay.py:766-768` | 回放成功路径对两条流各再做一遍完整 core 规范化 + canonical 序列化（hash 相等 ⇒ diff 必为 None） | `diff = first_core_diff(...) if recorded_core != replay_core else None` |
| PY-NP3 | `store.py:252-254` | `_project` 每事件同一 data JSON 序列化两次 | 复用一次结果（重建提速近一倍于该点） |
| 零散 | SV-H5（每报单 3 次 `vt_day()` 克隆、session 双 String 克隆）、SV-H6（journal 每条记录 3 个被弃时间值 + 契约线性扫描可预编译）、SV-H7（orders_today push 后回读拷贝大 POD）、WM-H5（FAK 缓冲 rec.clone()×3、submit 克隆整 Tick 绕借用）、WM-H6（行情扇出每 tick topic format + 每订阅者整帧 clone）、WM-H7（`Frame::write_to` 先整帧 encode 再写）、LD-F4（成交热路径为纯查询数据无效重算交易所费率桶，可后移到查询点） | | |

---

## 8. 防误删清单（核验为"非冗余"，勿动）

**领域语义（既往评审"深模块"结论经本次逐项复核维持）**：开/平昨/平今三段计费与今昨双轴（IM2410 生产数据锁定）；`pnl`/`trade_pnl` 双轨（CTP 两个必产出）；`FrozenEst` 按原始估算额比例释放（类型化消除 off-by-proportion）；`take_details_filtered` 今昨过滤与先开先平正交；平仓后成本从存活明细重算而非减法（昨仓昨结转valued 的刻意选择）；`detail_bucket_field` 25 字段清零重算（SHFE 双行防双计）；staged-clone 原子结算与结算末尾冻结残留检查（money authority 的 belt-and-suspenders，建议保留 Err 形态）；snapshot 手写序列化（v1→v2 迁移与位级 roundtrip）；`Money` 定点与顺序无关求和。

**工程结构**：json.rs 自研 Value 门面（插入序 + NaN→null 是设计决策）；build.rs 双语言契约嵌入（ADR-0001）；`PatientReader` 拆分（Windows 关停语义）；`accept_loop_with` 测试 seam；`decode_order_insert`/`validate_order_action` 分离（两个可独立测试的错误面）；settings.rs 的 MoveFileExW FFI（Windows 原子替换所需）；flow_replay 接受 `"curve"`（ADR-0002 后兼容旧 journal 的刻意保留）；server/ledger 结算双层校验（JSON 重复键检测是 ledger 无法承担的传输层职责）；`apply_withdraw` 的 Account 克隆（拒绝路径原子契约，有测试锁定）。

**Python**：journal.py 类型再导出 facade（"单一词汇表"文档化设计）；`_connect_admin` 不做探测连接（conn id 对齐——不能与 wait_port 合并）；`_drain`（非阻塞单遍）与 e2e `drain`（静默期判定）语义不同；refdata 字段清单与 contract 双份 + 启动断言（fail-fast）；`verify_layout` 全集依赖。

**shim/生成**：见 §4 负面清单；TD/MD 同型生成函数、`unsupported()` 蹦床、msgs 四处镜像、demo 自带转码。

**测试/CI**：双平台矩阵；各套件 `make_scenario`（测试数据非重复代码）；release_package 测试与被测关系；真实数据审计工具不入 CI（登记的手动工具）。

---

## 9. 落地批次建议（均为行为保持；回归门槛：cargo test workspace + engine/refdata vectors + 21 套 e2e + review_regressions）

| 批次 | 内容 | 净变化 | 风险 |
|---|---|---|---|
| 0（生成/拷贝面，收益最大） | WM-G1（Rust structs 白名单发射，**-42,500**）+ SH-F9（registry.hpp 停发枚举，**-515**）+ SH-F13（CI 指向 compiled_html，**-693KB**） | **≈ -43,000 行 / -700KB** | 低（生成物无消费者；md5 已证等价；registry.json 不动；一次 codegen 重跑 + git diff 门禁验证） |
| 1（零风险机械删除） | SV-D1/D4/D5、SV-V1/V2、WM-G2/G3、WM-A1、WM-F1、LD-B1~B7、PY-N7/N9~N16、SV-R7、LD-A1~A7、SH-F11、TT-N6 | ≈ **-450** | 极低 |
| 2（样板收敛） | SV-R1/R2/R3/R5/R6、WM-D1/D2/D4、PY-N1/N2/N3、LD-C1/C3、WM-F4/F5、SV-S4 | ≈ **-520** | 低（错误串/帧序列逐字保持，vectors 钉死） |
| 3（结构归位） | SV-S1（settle_day 搬家，admin.rs -440 体积）、SV-F1、SH-F1/F2/F3、SH-F6 | ≈ -150 + 搬移 | 低-中 |
| 4（测试与 CI） | TT-N1/N2/N3/N4（harness.py，一次一个套件）、TT-A2 哨兵、TT-N7/N8、TT-N5、TT-N9 最小档/N10 | ≈ **-780** | 中低（逐套件迁移 + matrix 验证） |
| 5（热路径） | §7 全表（先 SV-H1/H3/H4、WM-H1、SH-F4、PY-NP1/NP3 这些低风险项；WM-H2/H5 借用重构放最后） | LOC 中性，稳态分配显著下降 | 低→中 |
| 6（需作者决策/登记） | SV-D2（夜盘读路径是否近期接线——d4fdeae 刚加，疑似预留）、PY-N13/N17（公共面/诊断输出变化）、WM-F3（crate 拓扑）、SH-F10（镜像完整性 vs 少量常量）、PY-N8/N19（保留全集并注明）、TT-N11（superseded 标注） | — | 各含一次决策 |

**一句话总结**：这个仓库的问题不是"设计过度"而是"生成物无过滤 + 复制后各自演化"——批次 0 三个改动合计约 -66,500 行（占全仓追踪行数 142,029 的 ~47%）与最大的一块漂移面，且不碰任何运行时行为；手写侧的收敛（批次 1-4）合计再减约 -1,900 行，把既往评审登记的样板债变成可执行的表驱动/搬移方案；批次 5 用 LOC 中性的改动换取每 tick/每报单数量级更少的分配。领域复杂度本身（§8）经逐项核验是干净的，不是本次对象。

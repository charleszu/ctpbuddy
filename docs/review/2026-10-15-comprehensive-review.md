# CTPBuddy 全面评审：设计 / 文档 / 代码 / 工程化

- 日期：2026-10-15
- 评审基线：`main` 分支 `3bca91a`（merge review-hardening + simplification-landing），工作树干净
- 评审范围：架构设计与演进（git 全历史 188 提交）、Rust core / Python 控制面 / C++ Shim 三端代码、文档体系（DESIGN / README / ADR / docs-site / notes / 知识库）、测试与 CI、构建打包与工程卫生
- 评审方式：通读 DESIGN.md（全文结构与承重章节）、README、CHANGELOG、GLOSSARY、ADR 0001/0002、operations.md、两份 2026-10-13 评审与两份落地 spec；抽读全部 Rust crate 的承重模块与 Python 包关键模块、Shim 失败路径；全量静态扫描（unwrap/unsafe/TODO/裸 except/硬编码路径）；**在本机实跑验证**：`cargo test --workspace`（157 测试全绿）、`cargo clippy -D warnings`（零告警）、`cargo fmt --check`（通过）、`tests/test_py.py`（PASS）、`tests/e2e/review_regressions.py`（PASS）
- 与既有评审的关系：本仓库已有两份高质量评审（[架构评审](2026-10-13-architecture-review.md)、[简化评审](2026-10-13-simplification-review.md)）及两轮共 30 张落地工单。本报告不重复其论述，重点是**在 `3bca91a` 上独立复核前轮结论的闭环情况**，并给出前轮未覆盖的新发现。

---

## 1. 总体结论

**CTPBuddy 是一个工程成熟度显著高于 v0.1 版本号的项目。** 12 天、188 个提交内，从 M1 骨架演进到包含确定性回放、定点账本、三所回报分流、多 Broker、双传输、Web 控制面、完整 CI 的仿真柜台，且最近两天主动用两轮正式评审 + 30 张行为保持工单完成自我硬化——这种"评审 → 工单 → 回归测试固化"的飞轮在个人项目中罕见。

| 维度 | 评级 | 一句话 |
|---|---|---|
| 架构设计 | **A−** | 分层被代码真实执行；契约先行、单一写者、定点 Money 是先进且自洽的取舍；唯一结构性债是 World 协调器仍偏大 |
| 代码实现 | **A** | Rust 端纪律极强（生产代码仅 4 处可论证的 unwrap、唯一手写 unsafe 是一处 FFI）；Python/Shim 良好；前轮 P1 风险已逐条闭环 |
| 测试与 CI | **A−** | 语义级 e2e + 跨语言契约向量 + 确定性 fuzz 是业内上好水平；缺口在 Python lint、3.13 矩阵、依赖审计与覆盖率 |
| 文档 | **B+** | 覆盖深度与诚实度罕见（差距登记、skip 不冒充 pass、错误码 299 条对账）；但 DESIGN §9 是未标注的过时草图、docs-site 个别页已漂移 |
| 工程卫生 | **A−** | 零运行时第三方依赖、生成物 git-diff 门禁、发布包白名单都很严谨；发布前需解决许可元数据不一致与少量社区/可复现性缺口 |

**没有发现任何需要推翻既有架构决策（含两份 ADR）的问题。** 本次新发现全部是局部的文档漂移和工程卫生项，无高危代码缺陷。按 §7 清单逐项处理即可，不需要架构返工。

---

## 2. 演进脉络（git 历史）

- **时间跨度**：2026-10-02 ～ 2026-10-15（约 12 天，188 提交，单一主作者）。
- **阶段划分清晰，提交信息可直接当发布说明读**：
  1. **M1 骨架**（`2ee8b7e`）：wire/market/matching/ledger/server 五环 + Shim 全链路；
  2. **M2 语义核心**：限价簿（价格/时间优先 + FAK/FOK）→ 场景 DSL → journal 录制/重放与确定性 hash → 流控 + 订单状态机 + 错误码全集对账；
  3. **CTP 语义深挖**：#42 错单双推送面、#43 FAK 三所分流、真实 SDK 一请求多帧竞态；
  4. **M3 账本与参考数据**：provider 机制、先开先平、显式日结、OrderSysID、SQLite 投影、结算单供给；
  5. **M4 运营化**：ADMIN 认证、墙钟自动日结/到期强平、实时 Web 后台、FTD/连接/交易所三限流、ZeroMQ 全家族（ROUTER/DEALER/PUB/SUB/IPC）、多 Broker；
  6. **数据严谨性**：交易日历统一 contract、官方公告快照 + manifest/SHA256、真实柜台导出三表/四查询对账（459/976/64 份结算单复算）；
  7. **质量收口（10-13 两轮评审）**：8 张失败路径硬化工单 + 22 张行为保持简化工单，合并提交 `3bca91a`。
- **演进中的判断力（正面）**：
  - 每轮语义都有官方出处（CHM/error.xml/真实柜台数据），决策记录在 DESIGN 落地口径段落与 notes，不凭印象写仿真；
  - 主动做**减法**：生成结构体白名单一次净删 41.5k 行、CURVE 整条技术栈删除并落 ADR 0002、头文件双份去重 −693KB——项目变大后仍有删除勇气；
  - merge 提交保留评审分支上下文，CHANGELOG 按轮次记录，历史可追溯性好。
- **演进中的代价（可理解）**：DESIGN 正文随实现快速迭代，靠 §0.6 差距表兜底，但表只登记"功能差距"，没有覆盖"章节级草图过时"（见 D-1）。

---

## 3. 架构设计评审

### 3.1 分层：边界被代码执行，而非文档口号

```
下游 CTP 应用 ──同名 DLL/so ABI──> C++ Shim（薄生成壳 + ApiCore）
                                   │ CB 帧（raw repr(C) 结构，GBK 字段）
                                   v
                 Rust core：World 单线程世界循环（唯一写者）
                   wire / market / matching / ledger + journal(JSONL)
                                   │ 同一契约的跨语言 vectors
                                   v
                 Python：CLI / SDK / Web / 场景编译 / journal 投影（stdlib-only）
```

- Shim 不做业务判断、Python 不实现撮合账本、Rust 不碰 SQLite——三条边界本次逐一复核仍然成立（写路径全部经 ADMIN 进 core；[pyproject.toml](../../py/pyproject.toml) `dependencies = []`；core 无任何 SQLite 依赖）。
- crate 依赖方向无环且单向：`wire ← market ← matching ← ledger ← server`（[core/Cargo.toml](../../core/Cargo.toml) workspace 成员），server 是唯一组合层。

### 3.2 先进且自洽的设计决策

1. **确定性是被工程化的契约，不是口号**：journal 是唯一权威事件流；canonical hash 契约（[ADR 0001](../adr/0001-canonical-journal-contract.md)）编译期嵌入 build.rs，Rust/Python 跑同一份 `journal_vectors.json`；重放连 front_id 空洞都用 dummy 连接对齐（[DESIGN §11.2](../../DESIGN.md)）。这是事件溯源 + golden-vector 契约测试的教科书级落地。
2. **定点货币**：[money.rs](../../core/ctpbuddy-ledger/src/money.rs) 全部入账额用 `Money(i64)`（1e-6 元），累加顺序无关——因为账本遍历 HashMap，f64 求和在跨进程间不可复现；pro-rata 释放走 i128 中间值并 clamp；NaN/Inf 在入口归零并有单测。价格/比率保留 f64、入口一次性舍入的边界划得准确。
3. **冻结按原始估算额比例释放 + 终态统一解冻**：从类型上消除"多段成交后冻结残留"类缺陷；日结用 staged-clone（副本上完成全部校验后整体替换），"任一账户失败全批不变"是结构保证而非注释承诺。
4. **单一写者 + 传输隔离**：World 循环是 std::thread + mpsc，tokio runtime 被关在 ZMQ 传输线程内（[transport_zmq.rs](../../core/ctpbuddy-server/src/transport_zmq.rs)），世界状态无锁可推理。10ms PULSE 给出吞吐上界，但换来了真定时器语义（脉冲不被请求饿死，是重放正确性的前提）——取舍正确且已文档化。
5. **代码生成单一事实源**：[registry.json](../../registry.json) 人工评审、id 一经分配不变更；gen_shim 回写三语言镜像，CI 以 `git diff --exit-code` 门禁防漂移（[core-tests.yml](../../.github/workflows/core-tests.yml) L112-115）。
6. **安全模型诚实**：不假装加密（CURVE 删除有 ADR），ADMIN/Web 双回环强制、token 常量时间比较、Host/Origin/CSRF/重复头/CSP 多重校验（[web.py:120-140](../../py/ctpbuddy/web.py)）；威胁边界（TD 不校验密码、跨机必须走隧道）在 README 显著声明。
7. **仿真保真取向正确**：错单双推送面、FAK 三所形状、平今费时间序池、交易所错误码逐字复刻——"推错推送面比推错错误码更隐蔽"，项目拒绝帮下游掩盖缺陷，这正是仿真设施的价值主张。

### 3.3 仍然成立的结构性债（非新问题，登记在案）

- **World 协调器偏大**：[lib.rs](../../core/ctpbuddy-server/src/lib.rs) 2654 行（生产段 2112 行）、[query.rs](../../core/ctpbuddy-server/src/handlers/query.rs) 1155 行、[admin.rs](../../core/ctpbuddy-server/src/admin.rs) 827 行。seam 化方向真实（`client_route` 纯函数、decode seam、两阶段 dispatch、结算算法已从 admin 搬到 [settlement.rs](../../core/ctpbuddy-server/src/settlement.rs)），但 Conn 仍有 16 个字段捏合传输/会话/流控/订阅四类关注点（[lib.rs:595-617](../../core/ctpbuddy-server/src/lib.rs)）；journal 恢复事件的字符串分发表（第三份契约消费逻辑）与 Python `walk()`（[replay.py:584-641](../../py/ctpbuddy/replay.py)）仍同构。建议作为下一轮"World 下沉"专项，不急。
- **server 内并存两套 JSON 栈**：自研 [json.rs](../../core/ctpbuddy-server/src/json.rs)（保插入序、NaN→null）与 serde_json 并存。这是控制面样板偏多的根源；既有评审已论证其为刻意选择（journal 规范化语义需要），维持，但新人需要文档提示二者分工。
- **PULSE 10ms 单线程吞吐上界**：仿真场景足够，接真实行情压力前需重新评估（已在 §0.6 式诚实风格中可补一条）。

---

## 4. 代码实现评审

### 4.1 Rust core

**正面（本机实测/逐处复核）**：

- 测试基线：workspace 157 个单测全绿（ledger 28 / market 12 / matching 37 / server 71 / wire 5 / fuzz 4），clippy `-D warnings` 零告警，rustfmt 通过。
- panic 面极小：全 workspace 生产代码中 [lib.rs](../../core/ctpbuddy-server/src/lib.rs) 2112 行生产段仅 4 处 unwrap，且均为逻辑前置守卫后的取值（非空队列 pop、contains 后 get）；390 处 unwrap 绝大多数在测试与 generated。
- 全仓**唯一手写 unsafe** 是 [settings.rs:212](../../core/ctpbuddy-server/src/settings.rs) 的 `MoveFileExW` FFI（Windows 原子替换，标志位正确）；其余 unsafe 仅生成代码的 `mem::zeroed()`（repr(C) POD，健全）。
- 敌意输入防御成体系：wire 确定性 fuzz（2 万次/用例，随机字节 + 变异帧不 panic、超长长度先于分配拒绝，[fuzz.rs](../../core/ctpbuddy-wire/tests/fuzz.rs)）、world loop 4000 帧 fuzz 不 panic、GBK 解码失败替换字符、帧 padding 清零。
- journal fail-closed 彻底：契约校验失败不推进 seq、恢复时 seq 断链即拒、半行容忍、单实例文件锁；写失败健康状态上浮 ADMIN status（[journal.rs:277-310](../../core/ctpbuddy-server/src/journal.rs)）。
- 错误码/错误串逐字对齐官方 error.xml（[engine.rs:46-66](../../core/ctpbuddy-matching/src/engine.rs) 注释自曝"本 pass 之前 12 个常量错了 11 个"，并锁定官方来源）。

**问题（全部低危）**：

| ID | 位置 | 问题 | 建议 |
|---|---|---|---|
| R-1 | [engine.rs:424](../../core/ctpbuddy-matching/src/engine.rs) | 涨跌停校验仅在存在 tick 时执行；首个 tick 前任意限价通过。前轮评审已建议登记口径，§0.6 至今未收录 | 在 §0.6 增加一行仿真边界声明（先于代码改动） |
| R-2 | [calendar.rs:176-386](../../core/ctpbuddy-server/src/calendar.rs) | `NightPolicy` 解析读路径无生产消费者（简化评审 SV-D2 决策项），畸形 night_policy 的行为价值待路线图确认 | 接线或 `#[cfg(test)]`，二选一落子 |
| R-3 | lib.rs / query.rs / admin.rs 体量 | 见 §3.3 | 下一轮专项，非阻塞 |

### 4.2 Python 控制面

**正面**：零第三方运行时依赖（实测 `dependencies = []`，pyzmq 仅在 PEP 735 e2e 组）；无裸 `except:`；子进程启停/清理在 finally 中闭环（[replay.py:825-840](../../py/ctpbuddy/replay.py)）；SQLite 全参数化 + 只读 URI + 表白名单（[store.py:446-500](../../py/ctpbuddy/store.py)）；Web 路径穿越多重防御；SDK 异常契约已修复（超时统一 `CTPError`、协议错误带原因，[client.py:223-268](../../py/ctpbuddy/sdk/client.py)），三份 pending 管道已收敛为 `_slot()`/`_send_frame()`。

**问题（全部低危）**：

| ID | 位置 | 问题 | 建议 |
|---|---|---|---|
| P-1 | [assets/settings.js](../../py/ctpbuddy/assets/settings.js) | 10.9KB 单行压缩产物，无源码/source map，测试直接对产物断言；无法 review、无法 lint | 保留构建步骤并提交源文件（或明确标注生成方式与工具版本） |
| P-2 | [store.py](../../py/ctpbuddy/store.py) | 模块级函数（`_json/_validate/_project/rebuild/upgrade` 等）无类型注解，与全包注解风格割裂 | 补注解（CI 加 ruff 后可顺带门禁） |
| P-3 | [harness.py:35-40](../../tests/e2e/harness.py) | `free_port` bind-close-reuse TOCTOU（前轮 P5，低概率）；`m3_order_sysid.py:28-44`、`m4_real_replay.py:145` 仍保留本地副本，wave 2 迁移不彻底 | 端口改由内核分配后传给 server（或接受现状并删除剩余副本） |
| P-4 | replay.py `walk()` / `_step_until` | 事件类型仍按字符串字面量分发；watermark 轮询为 O(N) RTT（前轮已登记，未在 22 工单内） | 维持登记；录制变大后再优化 |
| P-5 | [audit_query_expectations.py](../../tools/audit_query_expectations.py) | 简化评审 TT-N11 已认其被 audit_three_way 部分取代，README/docs-site 仍并列当主力文档 | 文件头标注 superseded 与替代关系 |
| P-6 | web 运行时 | 标准库 `ThreadingHTTPServer` 非生产级——但绑定回环、信号量限并发、定位就是本地面板，**接受** | 无需改；文档继续禁止远程暴露 |

### 4.3 C++ Shim

**正面（前轮四项 P1 已逐条在代码中核实，质量高于 spec 最低要求）**：

- 心跳：5s 空闲 PING、15s 无帧判死并走既有断链清场，环境变量可调、0 关闭（[api_core.cpp:1034-1048](../../shim/src/api_core.cpp)）；
- ZMQ 队列满返回 -2 拒新不丢帧，AUTH/PING 控制帧绕过容量（避免握手被数据队头阻塞，[api_core.hpp:314-315](../../shim/src/api_core.hpp)）；
- 载荷尺寸不符 → ErrorID=-1 + 明确文本，经该行错误面回调，不再"成功+NULL"（[api_core.cpp:1002-1026](../../shim/src/api_core.cpp)）；
- auth JSON 对所有非 ASCII 字节 `\u00XX` 转义，`ok:false` 显式失败不等待超时；
- `is_query_msg` 改为从 dispatch 表派生（消除双份清单）；ZMQ 收帧缓冲已复用（消除每帧 ~1MB churn）；registry.hpp 515 行死枚举已删，只留有真实价值的 static_assert。

**残留（低危，均为前轮已登记项）**：

| ID | 位置 | 问题 |
|---|---|---|
| S-1 | [api_core.cpp:1077-1080](../../shim/src/api_core.cpp) | auth 响应用 `find("\"ok\":true")` 子串匹配；转义修复后协议已合法，但对服务端消息格式演化仍脆弱，建议后续改最小 JSON 判定 |
| S-2 | SUB_MD 按发送合约数计 `expected_responses`（前轮 S5）、`RegisterSpi` 无锁写（S6，违约边缘场景） | 小修，顺手可做 |
| S-3 | build_msvc.py / build_linux.py 平台无关段仍有同构（简化评审 SH-F6，未入选 22 工单） | 抽 build_common.py，收益中等 |

---

## 5. 文档评审

### 5.1 覆盖面（优秀）

DESIGN.md（137KB，含 §0.6 活的差距登记表）+ README 26KB（运行边界、数据许可、Docker/发布全口径）+ 2 份 ADR + GLOSSARY + operations 运维手册 + 14 篇考证 notes（跳号原因显式说明，编号不回收）+ 17 篇 CTP 开发知识库 + 错误码全集 299 条（脚本可重算、`--check` 可校验）+ 405 页干净 HTML 官方文档 + OINK 文档站 28 页。**一个人维护的项目里，这个文档密度和溯源纪律是顶尖的。** CHANGELOG 区分"本轮"与"历史里程碑"，不把未完成项勾掉（M4 总项明确不标完成）。

### 5.2 发现的漂移（本次新核实，按影响排序）

| ID | 位置 | 问题 | 严重度 |
|---|---|---|---|
| D-1 | [DESIGN §9.1/§9.2](../../DESIGN.md)（L740-766） | CLI/SDK 章节仍是 M1 草图且未标注：`ctpbuddy up/down` 不存在（实际是 `serve` + `web` 两个命令，无 `down`）；`Client("ipc://ctpbuddy")` 与实际签名 `Client(addr="127.0.0.1:5560")`（[client.py:99](../../py/ctpbuddy/sdk/client.py)）不符；`wait_order_filled`、`Scenario(...).run(assert_all=True)`、`c.market(...)` 在代码中均不存在；示例 broker 用 9999 而默认是 8888。§0.6 差距表只管功能、没标这一节。新人按 §9 入门会立刻踩空 | **中** |
| D-2 | [testing-ci-contributing.md](../../docs-site-oink/content/development/testing-ci-contributing.md) L30, L45 | 文档站已漂移：CI 矩阵写成 11 个套件（实际 21 个，缺 m2_book/flow/ioc/surface、m3_detail/margin/refdata、review_regressions/operations/flow_replay）；未提 rustfmt/clippy 门禁（实际已有）；源码索引把"账本日结"指向 `admin.rs`（结算算法已按 SV-S1 搬到 `settlement.rs`） | **中** |
| D-3 | [DESIGN L797](../../DESIGN.md) | §11.1 仍把 `libloading` 列为 server 依赖；ADR 0002 已随 CURVE 一起删除它（[Cargo.toml](../../core/ctpbuddy-server/Cargo.toml) 实测只剩 serde/bytes/futures/tokio/zeromq）。同节 §13 决策表"传输=ZeroMQ"也未补"TCP 现为默认"的后案 | 低 |
| D-4 | [DESIGN L1087](../../DESIGN.md) | §14 测试策略的 wheel 冒烟仍写不存在的 `ctpbuddy up` | 低 |
| D-5 | DESIGN 文头 | 仍标注"v0.1（草案）· 2026-10-02 · 状态：待评审"，与已完成 M1-M3、两轮评审后的事实不符 | 低 |
| D-6 | [DESIGN §6.1](../../DESIGN.md) | Socket 拓扑图只有 ZMQ 形态（admin 还画成 `ipc://buddy-admin`），实际 ADMIN 是 TCP :5561；正文已澄清，图未更新 | 低 |

一致性良好的抽查项（不再展开）：帧格式/消息号、流控分工表、结算确认前置、CURVE 移除（operations.md L11、§0.6、ADR 三处口径一致，L61 的 CURVE 为带日期历史记录，合理）、许可归因（MPL/官方文档/日历快照三组例外清楚）、README 引用的文档路径全部存在。

---

## 6. 测试、CI 与工程化

### 6.1 测试体系（A−）

- **金字塔健康**：157 Rust 单测（含大量语义锁定：FAK 三所形状互不相容、先开先平、冻结比例释放、journal fail-closed）→ 跨语言契约 vectors → 27 个 e2e 脚本（每套自起 server、临时目录隔离）。
- **断言是真语义**：精确回报序列、maker/taker 共享 TradeID、双跑同 hash + 篡改必败的负控制、真实柜台数据逐项资金恒等式。
- **诚实的 skip 惯例**：无外部数据/无 Docker/pyzmq 缺失时退出码 3 显式 SKIPPED，`--allow-skip` 才归零——skip 永不冒充 pass。
- [review_regressions.py](../../tests/e2e/review_regressions.py) 把评审发现固化成回归（本机实测 PASS），是评审飞轮的实体。
- 缺口：无覆盖率度量；fuzz 是固定种子确定性长跑、无 schedule 任务；e2e 流控套件依赖墙钟窗口（已有 `CTPBUDDY_E2E_SLOW` 旋钮缓解）。

### 6.2 CI（A−，缺口明确）

[core-tests.yml](../../.github/workflows/core-tests.yml) 五 job（unit / e2e matrix / shim / linux / docker）+ docs-site 站：双平台、真 Shim e2e、生成物一致性门禁、release package 真实产物门禁都有。缺口：

| ID | 问题 | 建议 |
|---|---|---|
| C-1 | 无 Python lint/格式门禁（61 个 py 文件纯自觉） | 加 ruff（check + format），初始规则可只开 E/F 子集 |
| C-2 | CI 全部 Python 3.9，本机 `.python-version` 是 3.13——3.13 上零验证（兼容下限有人测，开发版本没人测） | unit job 加 3.13 到矩阵 |
| C-3 | e2e 套件清单双维护：matrix 21 项 vs linux 循环 20 项硬编码两处 | 用单一 suites 准备 job 产出 JSON + 平台排除名单（简化评审 TT-N7） |
| C-4 | 无 `cargo audit`/`cargo-deny`、无许可证策略检查；Rust 用 stable 浮动、无 `rust-toolchain.toml`，Docker 基础镜像浮动 tag | 发布前三渠道流程里一并补（钉工具链、审计依赖、镜像 digest） |
| C-5 | 远端 CI 实际运行状态文档未闭环（operations.md 自述"本轮暂不核验"） | 下次有真实 run id 时补登 |

### 6.3 工程卫生

- **依赖面克制**：Rust 域 crate（wire/market/matching）零外部依赖；Python 零运行时依赖；uv workspace + lock；libzmq 源码 tarball 入库（SHA256 互锁、首次构建零网络）。
- **发布链路严谨**：release package 白名单复制、拒链接/重解析点、PE machine 校验、复制后复验 sha256、manifest 自认"commit 不证明二进制来源"——诚实但也说明签名/tag 发布流水线尚未建（M4 已知项）。
- **许可元数据不一致（发布前应修）**：

  | 位置 | 声明 |
  |---|---|
  | [LICENSE](../../LICENSE) | MIT |
  | [py/pyproject.toml](../../py/pyproject.toml) | **未声明 license 字段** |
  | [core/Cargo.toml:15](../../core/Cargo.toml) | `MIT OR Apache-2.0`，但仓库没有 Apache-2.0 许可证文本 |

  三渠道发布前需统一（要么补 Apache-2.0 文本与 NOTICE，要么 Rust 改 `MIT`，Python 补 `license = "MIT"`）。
- **Docker**（[Dockerfile](../../docker/Dockerfile)）：多阶段、wheel 安装后删源码，良好；残留：最终镜像以 root 运行、基础镜像浮动 tag（验收镜像可接受，生产化时收紧）。
- **仓库内容**：40MB 追踪文件，最大头均有正当理由（vendored OINK 第三方 JS、zmq tarball、官方 HTML 两份形态）；`.scratch/` 30 个工单文件被追踪——过程可追溯但目录名暗示临时，建议发布前归档到 docs 或移除跟踪。
- 面向开源发布尚缺 `SECURITY.md`（安全问题上报渠道）与根 `CONTRIBUTING.md`（文档站有贡献页但仓库根无入口）。
- **refdata provenance 弱于 calendar**：[refdata/manifest.jsonl](../../refdata/manifest.jsonl) 只有格式契约，789 合约快照无来源/日期/内容 sha256（calendar/production 有完整 MANIFEST + facts hash 可对标；前轮建议第 9 条，未落地）。

---

## 7. 问题清单（按优先级）

**发布前应做（都是小改动）**

1. **D-1**：重写 DESIGN §9.1/§9.2 为实际 CLI/SDK 表面，或整节标注"历史草图，以 README/`ctpbuddy --help` 为准"。
2. **许可元数据统一**（§6.3）：LICENSE / py project / Cargo workspace 三处对齐。
3. **D-2**：更新文档站 testing-ci-contributing 页（21 套件、clippy/fmt 门禁、settlement.rs 指针）。
4. **D-3/D-4**：删 DESIGN 的 libloading 残留与两处 `ctpbuddy up`；§13 传输行补 TCP 默认后案。
5. 补 `SECURITY.md` 与根 `CONTRIBUTING.md`；钉 `rust-toolchain.toml` 与（可选）镜像 digest。

**下一轮工程化（收益/成本比高）**

6. CI 加 ruff + Python 3.13 矩阵（C-1/C-2）；e2e 清单单一来源（C-3）；cargo audit/deny（C-4）。
7. refdata 快照补来源 manifest（对标 calendar）；settings.js 提交可编辑源；store.py 补类型注解。
8. R-1：首个 tick 前涨跌停边界登记进 §0.6（先文档后代码）。

**维持登记、择机处理（不阻塞）**

9. World 下沉专项：admin 命令注册表化、Conn 子结构、恢复事件第三份字符串分发表（§3.3）。
10. 前轮遗留低危项：S-1/S-2/S-3、P-3/P-4/P-5、R-2、harness 剩余副本、`.scratch` 归档。

---

## 8. 前轮评审闭环核对（独立复核结论）

- **架构评审 4 项 P1 风险——全部闭环且质量达标**：Shim 心跳/断链清场（S1）、队列满拒新（S2）、载荷防御（S3）、auth 转义（S4）、journal 健康上浮 + `CTPBUDDY_JOURNAL_HARD_FAIL`（R1）、logout 身份校验（R2，[handlers.rs:497-533](../../core/ctpbuddy-server/src/handlers.rs)）、SDK 异常契约（P1/P2）、Web 512KB 单一上限（P4）——均在代码中逐处核实，并有 `shim_hardening.py` / `review_regressions.py` 固化。
- **架构评审工程项**：CI clippy/rustfmt 门禁✓、m4_install_shim 入 CI✓、harness 收敛✓（有少量残留副本，P-3）、test_py 注册哨兵✓（[test_py.py:1807-1837](../../tests/test_py.py)）；ruff（T1）与 3.13 矩阵（T2）**未做**；refdata provenance **未做**。
- **简化评审 22 张工单——全部落地**：抽验 structs.rs 33 个镜像（CHANGELOG 声称 33，实测 33）、registry.hpp 仅余 static_assert、结算归位 settlement.rs、SETTING_DEFS 单表、热路径 guard、tick 时间戳缓存、harness 两波、回放 diff 短路——与提交历史和代码现状一致。批次 6 的"需决策项"（night_policy、jsonl 解析器跨 crate 合并、Python structs 全集保留、TT-N11 等）维持原状是**符合 spec 约定**的，不是遗漏。
- 既有评审的深模块/防误删清单（费用双腿、冻结比例释放、staged-clone、Money 等）本次复核仍为必要设计，未被简化波误伤。

---

## 9. 结语

CTPBuddy 最稀缺的资产不是某个模块，而是**整条"官方语义 → 设计口径 → 代码 → 测试 → 对账数据"的可追溯链**，以及项目对自身边界的诚实：未完成就登记、跳过就退出码 3、不编造费率、不冒充真实柜台回放、评审发现回灌成回归测试。本次全面复核没有找到推翻性问题，反而在最新的合并点上验证了前轮修复的真实性——这说明团队（即使是一个人）已经具备稳定的自我纠错回路。

当前最该做的不是继续加功能，而是花半天清掉 §7 前 5 项文档/许可漂移：它们体量小，但处于新人入门路径（DESIGN §9）、发布合规（license）和外部门面（docs-site）上，与代码的真实质量不匹配。修完之后，这个仓库从"代码 A、门面 B+"会变成内外一致的 A。

---

## 附录 A：2026-10-15 优化落地记录（当日实施）

§7 清单已按"逐板块、逐条目"实施；除明确标注"维持登记"的条目外全部完成。

### A.1 发布前项（§7 前 5 组）——全部完成

| 评审条目 | 落地内容 | 落点 |
|---|---|---|
| D-1 | DESIGN §9.1/§9.2 按真实 CLI/SDK 表面重写（`serve`/`web`/`install-shim`/`journal`/`replay`/`refdata`/`calendar`/`assertions`；Client/Admin 方法清单；断言 DSL 改为场景 YAML one-shot），章首标注替代关系 | DESIGN §9 |
| 许可不一致 | Cargo workspace 改 `MIT`；Python 包补 MIT classifier（弃用需 setuptools 77 的 PEP 639 字段，兼容 3.9 旧构建链）；wheel METADATA 已验证 | `core/Cargo.toml`、`py/pyproject.toml` |
| D-2 | 文档站测试页改为真实 21 套件矩阵、ruff/fmt/clippy 门禁、audit/docker/shim job；账本日结指针从 admin.rs 改为 settlement.rs/auto_settlement.rs | docs-site testing-ci-contributing.md |
| D-3/D-4/D-5/D-6/R-1 | §11.1 删除 libloading；§14 wheel 冒烟改 `serve`/`web`；文头状态改「持续维护」；§6.1 拆 TCP 默认 + 可选 ZMQ 两张图；§0.6 增首个 tick 前涨跌停边界行 | DESIGN.md |
| 社区/可复现 | 新增 `SECURITY.md`（私下报告渠道 + 设计安全边界）、`CONTRIBUTING.md`（环境/必跑门禁/七条约定）、`rust-toolchain.toml`（1.98.1 + rustfmt/clippy；MSRV 1.89 保留在 Cargo.toml） | 仓库根 |

### A.2 工程化项（§7 第二组）——完成

| 评审条目 | 落地内容 |
|---|---|
| C-1 ruff | 根 pyproject 配置保守规则集（E4/E9/F），per-file 例外仅两处刻意模式（journal.py 契约再导出、e2e 的 E402 引导）；全仓 59 个存量发现清零（49 安全自动修复 + 10 人工） |
| C-2 3.13 矩阵 | unit job 矩阵 3.9/3.13 |
| C-3 套件双维护 | 新增 `suites` 准备 job 单一产出 windows(21)/cross(20) 两组 JSON，matrix 与 linux 循环都消费它 |
| C-4 依赖审计 | 新增 `audit` job（taiki-e 装 cargo-audit，`--file core/Cargo.lock`）；本地因网络无法拉 advisory-db，未本机验证 |
| refdata provenance | 新增 `refdata/MANIFEST.json`（schema `ctpbuddy.refdata-provenance/v1`，4 个文件 SHA256 + 行数 + 来源说明）；`JsonlDirProvider.verify_provenance()` 在 `refdata show`/validate 中强制校验（未知文件/缺文件/哈希/行数四类失败）；`.gitattributes` 强制 `*.jsonl`/`MANIFEST.json` LF 保证跨平台哈希一致；新增篡改单测并注册哨兵 |
| P-1 settings.js | settings.js/live.js 去压缩可读化（jsbeautifier 一次性格式化，验证括号平衡、幂等、禁用 API 不变量），文件头声明直接维护/无构建/安全约束 |
| P-2 store.py | 19 个函数/方法全部补齐类型注解 |
| P-3/P-5 | harness 残留副本继续收编：m2_flow/m2_scenario 改用共享 Feed（harness 增 `md()` 超集）、m3_order_sysid/m4 删本地 free_port、m3_refdata 删重复 wait_idx；audit_query_expectations 标注 superseded |

### A.3 计划外但被门禁发现并修复的真实缺陷

1. **`tools/audit_cffex_close_fee.py:75` 变量名拼错**（`SETTLEMENT_DIR` vs 实际定义的 `SETTLE_DIR`）：目录存在时必然 NameError——脚本最后一次成功复跑后引入的潜伏回归，ruff F821 直接命中。
2. **`tests/test_query_expectations.py` 基线即红**：wave2 起 `e2e.m4_real_replay` 经包路径导入时 `harness` 不在 sys.path（stash 回退验证：`3bca91a` 同样失败）；远端 CI 当轮未核验故未暴露。已补 sys.path 引导。
3. **settings e2e 在代理环境必挂**：裸 urllib 会走 `HTTP(S)_PROXY`（本机 7897 代理对回环地址回 502），改为安装空 ProxyHandler 的 opener；CI 无代理变量但本地开发不再有假失败。

### A.4 其余落地

- Docker 验收镜像非 root 化（uid 10001，chown /app 保留可选 Hugo 写 public/resources 的能力；临时数据走 /tmp）。
- `.scratch/` 30 张已完工单 `git mv` 到 `docs/tickets/2026-10-13-{review-hardening,simplification-landing}/`，新增批次索引 README；CHANGELOG 与 hardening spec 的指针同步更新。

### A.5 验证证据（2026-10-15 本机，Windows / Rust 1.98.1 / Python 3.13.15 venv）

- `cargo fmt --check` 通过；`cargo clippy --workspace --all-targets -D warnings` 零告警；`cargo test --workspace` **157 测试全绿**。
- `ruff check .` 全绿；`tests/test_py.py`（含新增 provenance 测试与注册哨兵）、`test_journal_contract.py`、`test_projection_upgrade.py`、`test_query_expectations.py`（13 测试）、`m4_install_shim.py` 全绿。
- e2e 本机实跑 20 套全绿：m1_smoke、m2_book/flow/ioc/surface/scenario/journal、m3_detail/margin/refdata/fee_cffex/bootstrap/5/6/settlement/order_sysid、settings、review_regressions、operations、flow_replay；m4_real_replay 以 `--allow-skip` 验证导入与跳过路径。
- wheel 重新构建成功，METADATA 含 MIT classifier，assets 完整。
- **未本机覆盖**：`cargo audit`（本机无法访问 advisory-db）、Shim e2e / shim_hardening（MSVC + libzmq 全量构建耗时长；m1_shim_e2e 仅有一处 ruff 删除未用导入，CI shim/linux job 覆盖）、Hugo 站构建（本机无 hugo；正文改动无 front matter 变化）。

### A.6 明确不做（与评审"维持登记、择机处理"一致，附理由）

- **World 下沉专项**（评审 §7 第 9 条）：lib.rs/query.rs/admin.rs 体量与第三份字符串分发表是真实结构债，但属需独立 spec 的大规模行为保持重构，不应搭车文档/工程化批次；建议像 10-13 两轮一样走"评审 → spec → 工单 → 回归"。
- **Shim S-1/S-2/S-3**：auth 子串匹配、SUB_MD 计数防御、RegisterSpi 锁、build_common 抽取——需要 MSVC 全量构建 + libzmq 编译验证，单独立项。
- **P-4**（`walk()` 字符串分发改注册表、`_step_until` O(N) 轮询）：录制规模变大后再优化。
- **R-2**（calendar NightPolicy 读路径）：接线或 `#[cfg(test)]` 取决于夜盘路线图，需产品决策。
- **free_port TOCTOU**：bind-close-reuse 是测试辅助的固有竞态，真正消除需端口由内核分配后传给 server（CLI 不支持 fd 传递），收益不抵改动。


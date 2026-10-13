# Spec: 评审落地第二批——冗余消除与行为保持简化

- Triage: `ready-for-agent`
- 来源: `docs/review/2026-10-13-simplification-review.md`（批次 0/1/2/3/4/5；批次 6 与其余"需作者决策/本地不可验证"项明确排除，见 Out of Scope）
- 日期: 2026-10-13
- 铁律: **行为保持**——不改任何帧字节、错误码与错误串、journal 行、测试断言集合、CI 覆盖集合。每张工单动手前必须先实地复核评审论断在当前 HEAD 是否仍成立（评审基线 453f5b6），有偏差先重新勘察并修订工单，再实施。

## Problem Statement

简化评审（2026-10-13）确认 CTPBuddy 的领域设计与深模块是干净的，但存在三类系统性冗余，持续产生维护成本与漂移风险：

1. **生成面无过滤**：codegen 把 CTP 6.7.13 头文件的 519 个结构体全量发射进三个语言镜像，其中 Rust 侧 485 个零引用（45,473 行生成物中约 42,500 行是死重）；`shim/generated/registry.hpp` 里 519 项 StructId 枚举零消费者（真正被消费的只有同文件的 static_assert 布局门禁）；CTP ABI 头文件在 `docs/api-doc-html/files/` 与 `docs/notes/assets/compiled_html/` 下 md5 级双份入库（-693KB）。
2. **复制后各自演化**：e2e harness 助手（`free_port`×10、`find_core`×8、`drain`×7、启停样板×16 文件）成片复制且已产生 3 处劣化变体（settings.py 只找 debug 路径导致该套件进不了 linux job；m3_order_sysid/m3_bootstrap 硬编码 task41/task43 遗留 profile 路径）；server 控制面手工样板（admin 白名单×13、query 前导×12、journal_contract 手查 Map×20+）；shim 帧头编解码三处重复、`is_query_msg` 手工复制 dispatch 表查询列。
3. **热路径纯冗余分配**：`Tick::virtual_ms()` 每次调用重新解析时间字符串；每 tick 两次整 Tick 深拷贝的子集、无 PUB 订阅者仍整帧克隆、无 journal 仍构建 JSON 树、零积压仍排序全部连接、shim ZMQ 每帧全新 ~1MB 缓冲、回放成功路径多做两遍规范化。

## Solution

按评审批次 0→5 顺序落地全部"行为保持"提案：批次 0 给 codegen 加 Rust 发射白名单（= MESSAGE_STRUCTS 值集 ∪ 实际引用集，白名单在实施时以全仓 grep 机械推导而非照抄评审清单）、停发 registry.hpp 死枚举（static_assert 全部保留）、消除 docs 头文件双份（CI 改指 compiled_html，同步 DESIGN/README 合规表述）；批次 1 删除全部已核验的死代码/死包装/不可达分支；批次 2 收敛样板（表驱动 admin 字段校验、`qry_session` 前提取、journal_contract 改 `Map::get`、refdata 三表泛型化等）；批次 3 结构归位（结算算法从 admin.rs 搬 settlement.rs、shim 帧头/查询流控单一真相）；批次 4 测试与 CI（`tests/e2e/harness.py` 收编恒等助手、逐套件迁移、test_py 注册哨兵、CI 子集重跑删除）；批次 5 低风险热路径修复（journal 空判、PUB 空判、drain_ftd 空判、virtual_ms 缓存、ZMQ 缓冲复用、回放 diff 短路）。全部改动以仓库既有门禁为验收：cargo workspace 测试 + clippy + codegen git-diff 门禁 + Python 单测 + e2e 套件。

## User Stories

1. 作为核心维护者，我希望 Rust 侧生成结构体只发射实际使用的白名单集合，以便 45k 行生成物缩到 ~3k 行、编译时间下降、新增结构体的噪音消失，而 registry.json 的 id 分配与 Python/C++ 镜像完全不动。
2. 作为核心维护者，我希望 registry.hpp 不再发射零消费的 StructId 枚举而保留全部 static_assert，以便 shim 构建仍有完整布局门禁，且少维护一份会漂移的 id 拷贝。
3. 作为仓库管理员，我希望 CTP ABI 头文件在 git 里只有一份，以便"改了一份忘另一份"的漂移面消失、仓库瘦身 ~700KB。
4. 作为核心维护者，我希望 server/ledger/matching 的死 pub 面、死常量、不可达分支、纯转发壳被删除，以便 API 面反映真实用途、`cargo doc` 与补全不再给出假入口。
5. 作为核心维护者，我希望 admin 字段校验、query 前导、journal_contract Map 访问、refdata 三表加载等样板收敛为表驱动/泛型助手，以便新增命令/查询/表时只改一处数据。
6. 作为核心维护者，我希望 settings 的 min/max 表与 schema 展示表共用单一 `SETTING_DEFS` 常量，以便两表永不漂移。
7. 作为核心维护者，我希望 main.rs 的 env 块与 CLI 块共用同一 `apply_flag` 解析，以便同一设置只写一次解析逻辑。
8. 作为核心维护者，我希望结算域算法（settle_day/stage_expired_liquidations/record_forced_liquidations）从 admin.rs 归位到 settlement.rs，以便 admin.rs 回到纯命令分发、结算逻辑有单一居所。
9. 作为 shim 维护者，我希望帧头编解码只有一份实现、`is_query_msg` 由 dispatch 表推导、auth 两条路径共用前缀构造且魔数取自 `sizeof`，以便协议与流控的真相各只有一份。
10. 作为测试工程师，我希望 e2e 恒等助手（find_core/free_port/wait_port/drain/st/tr/Feed/启停样板）收编进 `tests/e2e/harness.py`，以便新增套件不再复制、劣化变体（settings 的单路径 find_core、task41/task43 硬编码）被修复且该类缺陷不再复发。
11. 作为测试工程师，我希望 test_py.py 有"定义集 == 注册集"漂移哨兵，以便新增测试忘写 main() 时响亮失败而非静默跳过。
12. 作为 CI 维护者，我希望 unit job 删除已被 workspace 测试覆盖的子集重跑、m4_install_shim 进入门禁，以便 CI 时间不浪费在重复执行、覆盖缺口补上。
13. 作为回放用户，我希望成功路径不再对两条流各多做一遍完整规范化+canonical 序列化（hash 相等即短路 diff），以便大录制的回放明显提速且结果逐位不变。
14. 作为场景运行用户，我希望无 journal 的运行不再为每笔事件构建 JSON 树、无 PUB 订阅端点不再为每个 tick 深拷贝整帧、零 FTD 积压不再每 10ms 排序全部连接，以便稳态内存分配显著下降而输出不变。
15. 作为行情回放用户，我希望 `Tick::virtual_ms()` 不再每次解析时间字符串，以便百万级 tick 回放的每-tick 开销下降一个分配+解析量级。
16. 作为 shim 用户，我希望 ZMQ 读路径复用线程私有缓冲而非每帧全新 ~1MB，以便高频行情与空闲心跳下的分配 churn 消失。
17. 作为 SDK 用户，我希望 client.py 的三份 pending-socket 管道收敛为一个上下文管理器，以便锁序与异常路径的行为有单一实现点（对外契约逐字不变）。
18. 作为 Web 后台开发者，我希望 settlement 报告的 GBK 归一化只有 Admin 一份实现、web 侧同请求内不做两次编码，以便行为单一且省一半编码开销。
19. 作为贡献者，我希望 DESIGN/README 的合规表述在头文件去重后同步更新，以便文档与仓库结构继续一致。
20. 作为项目维护者，我希望本批落地的每个行为保持论断都被既有测试（cargo workspace、codegen git-diff、Python 单测、e2e、review_regressions）钉住，以便后续简化有可回归的基线。

## Implementation Decisions

- **白名单机械推导**：Rust 发射白名单不得照抄评审的 34 个清单，实施时以"全仓非 generated Rust 源（含 tests）中出现的 CThostFtdc* 名字"机械推导，并 ∪ MESSAGE_STRUCTS 值集；`cargo build/test` 通过即覆盖证明。registry.json、Python structs.py（保留全集，评审 §5.4 方案 b）、C++ 镜像不动。
- **registry.hpp**：gen_structs 的 C++ 发射只去掉枚举段，static_assert 段逐字节保留；重新生成后 shim 构建必须仍过全部布局断言。
- **头文件去重带验证门**：删除 5 个 .h 前先 grep 非生成消费方（预期仅 CI `--sdk` 与 DESIGN/README 表述）；CI 两平台改指 `docs/notes/assets/compiled_html`；chm_to_html.py 停止复制 .h 并同步其索引链接；`error.xml`/PDF 不动；DESIGN §2/§15 与 README 的 `api-doc-html/files/` 表述改为指向唯一份。若发现隐身消费方，降级为方案 b（保留双份 + 互指 README），不强行删除。
- **样板收敛不引新依赖**：admin 字段校验用 `json.rs` 内新增纯函数 + `const CMD_KEYS` 表；错误消息逐字保留（错误串是测试面）。`qry_session` 复用 -3 错误面；查询尾部统一走既有 `send_rate_rows`。
- **结算归位为纯搬移**：三个函数逐字搬到 settlement.rs，调用点改路径；不做任何逻辑修改；与 replay 侧结算序列的进一步下沉（单一 `apply_settlement`）不在本批。
- **shim 单一真相**：`decode_header` 抽取须保持 TCP 路径"先收头、len 越界即断线"的顺序语义；`is_query_msg` 改为 `find_row_by_req(msg) && qry_last != nullptr`；auth 前缀/状态置位抽助手且尺寸取 `sizeof` 成员。
- **harness 收编逐套件迁移**：`tests/e2e/harness.py` 只收恒等助手（`drain` 参数化 quiet、`Feed` 取超集、`prepare_scenario` 带 `with_compile` 开关、`TIME_SCALE` 旋钮入 harness 并给 flow_replay 接入）；现有 import 枢纽（m1_smoke/m3_refdata/m2_flow）保留套件专属常量；一次迁移一个套件、迁移后本地跑该套件；三处劣化变体随收编修复（find_core 统一 debug/release/Linux 名）。
- **热路径修复零输出变化**：journal/PUB/drain_ftd 三处为"空则提前返回"守卫；`virtual_ms` 用缓存字段并提供单一构造入口防测试旁路；shim ZMQ 缓冲提升为 reader 线程私有成员（单线程独占，decode 以长度为准）；回放 diff 短路依赖既有 sha256 抗碰撞假设。
- **每工单先验证后动手**：工单模板含"验证步"——按评审 file:line 复核现状（grep/读码），偏差则重新勘察修订工单再实施；这是本批的硬性流程要求。

## Testing Decisions

- 好的测试是既有外部行为门禁，不新增测试面（本批为行为保持重构，新增测试反而扩大面）；每个行为保持论断由以下既有 seam 钉住：
  1. `cargo test --locked --workspace --manifest-path core/Cargo.toml` + `cargo clippy -D warnings` + `cargo fmt --check`（server/matching/ledger/wire 全部单测与 vectors）；
  2. codegen 确定性门禁：重跑 gen_shim/gen_structs 后 `git diff --exit-code`（既有 CI 模式），钉住 G1/SH-F9 的生成物形状；
  3. `python tests/test_py.py`（含 `verify_layout` n==519 的 Python 全集断言，证明 py 镜像未受 Rust 白名单影响）+ `test_journal_contract.py` + `test_projection_upgrade.py` + `test_query_expectations.py`；
  4. e2e：每次 server/shim 改动后本地跑受影响套件（m1_smoke、m2_book、m2_flow、operations、flow_replay、review_regressions 为基线集；harness 迁移逐套件跑该套件）；
  5. MSVC shim 构建 `python shim/build_msvc.py --demo --sdk <目录>`（SH 项的布局断言与 demo 链接）。
- Prior art：`tests/e2e/review_regressions.py`（评审发现固化为回归）、journal contract 双语言 vectors、`shim_hardening.py` 4 探针。
- 快照对比：harness 迁移与样板收敛前后，对代表性套件的 stdout 做逐行对比（套件自身的 `---- server log ----` 与断言输出不变）。

## Out of Scope

- 评审批次 6 全部（需作者决策）：SV-D2 夜盘读路径去留（d4fdeae 疑似预留）、PY-N13 settle_day 日历推导分支、PY-N17 verify_events 重复校验（诊断输出微变）、WM-F3 jsonl 解析器 crate 归位、SH-F10 msgs 镜像常量过滤、PY-N8 scenario 双轨校验器合并、PY-N19 Python 生成物裁剪（保留全集）、TT-N11 audit_query_expectations 退役。
- 本地不可验证项：SH-F6 build 脚本公共模块抽取（Linux 侧无法本地验证、gh 失效无法盯 CI）、TT-N7 CI 动态 matrix、TT-N8 composite action（同因）、TT-N10 tools CSV 解码阶梯参数化（依赖真实导出数据）。
- 高风险性能重构：WM-H2/H5（借用重构）、WM-H3/H4/H6/H7、SV-H2 盯市时机、SV-H5/H6、WM-V1 submit_prevalidated、LD-F4 费率桶后移、LD-E1 in-place 结算（评审自评倾向 keep）。
- LD-C3 日期校验跨 crate 统一（对 <1900 年输入属收紧、评审标注需显式选择）。
- 任何对外可观测行为变化（错误串、帧序、CLI 输出格式、退出码）。

## Further Notes

- 评审与实施的间隔内仓库可能前进：所有工单以"验证步先行"消化漂移；若某论断已失效（如已被他人修复），工单标记 obsolete 并跳过。
- 预期总量：手写净删约 -1,900 行、纯搬移 -440 行体积、生成物 -43,000 行、仓库 -693KB（SH-F13 顺利时）。
- 分支策略：在 `review-hardening` 之上新开 `simplification-landing` 分支逐工单推进（由用户决定合并节奏；本流程不自动合并/推送）。

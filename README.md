# CTPBuddy

[![repo](https://img.shields.io/badge/repo-github.com%2Fcharleszu%2Fctpbuddy-blue)](https://github.com/charleszu/ctpbuddy)

**本地 / 私有部署的 CTP 兼容仿真交易环境** —— 为 CTP 下游系统（策略、交易终端、条件单）提供确定性、可注入、可共享的测试基础设施。

> 完整设计见 [DESIGN.md](DESIGN.md)。**CTP 语义知识库（流控/生命周期/会话/报单回报时序/状态机/资金持仓/保证金/行情/结算）见 [docs/CTP语义知识库.md](docs/CTP语义知识库.md)**，含 M2 实现清单；深度原始笔记在 `docs/notes/`。官方资料可读版：SDK《6.7.13_API接口说明》HTML 版（405 页干净 HTML，剔除 CHM 主题框架、保留表格/代码/内嵌图片，页间链接与 `anchor-id-*` 锚点均已校验；另有 1 页目录漏收附录、8 个官方附件与 84 条官方源死链/1 条悬空锚点的公示）在 [docs/api-doc-html/](docs/api-doc-html/)，error.xml 错误码全集（299 条，逐条标注「已实现 19 / 可落地 51 / 暂不可达 229」+ 推送面）在 [docs/错误码全集.md](docs/错误码全集.md)，双推送面口径见 [docs/notes/09](docs/notes/09-错单推送面与错误码对账.md)；FAK 部成部撤的**三所分流**回报（上期所/大商所+广期所/郑商所三种不同形状）见 [docs/notes/10](docs/notes/10-FAK回报按交易所分流.md)。CTPBuddy 与上海期货信息技术有限公司无任何隶属关系；本项目不附带任何官方 SDK 文件，`ctpsdk/` 目录中的头文件由使用者自备、禁止入库与分发。

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

# 4. 手动起一套玩玩（示例场景 rb_demo）
ctpbuddy serve --scenario scenarios/rb_demo --data-dir ./data
ctpbuddy status                  # 另开一个终端
```

核心支持 `--qry-freq <n>`（env `CTPBUDDY_QRY_FREQ`）调前置每秒查询预算：超过即回 `OnRspError[90]`「CTP：查询未就绪，请稍后重试」，与真实 CTP 前置一致（DESIGN §8.8）。

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

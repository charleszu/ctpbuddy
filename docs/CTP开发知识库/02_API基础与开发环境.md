# 02 API基础与开发环境

> 见《13_API方法速查表》《14_错误码速查表》《15_枚举常量速查表》《16_结构体字段速查表》。

## 1. 要点速查表

（待填充）

## 2. 详细说明

### 2.1 动态库/头文件/平台

**发布包文件（以 Windows C++ 为例）**

| 文件 | 说明 |
|---|---|
| ThostFtdcMdApi.h | 行情接口头文件（含 CThostFtdcMdApi / CThostFtdcMdSpi） |
| ThostFtdcTraderApi.h | 交易接口头文件（含 CThostFtdcTraderApi / CThostFtdcTraderSpi） |
| ThostFtdcUserApiDataType.h | 业务数据类型定义（TThostFtdc...Type、枚举字符常量） |
| ThostFtdcUserApiStruct.h | 业务数据结构定义（CThostFtdc...Field） |
| thostmduserapi.dll / .lib | 行情动态库 / 导入库（Linux 为 `.so`） |
| thosttraderapi.dll / .lib | 交易动态库 / 导入库（Linux 为 `.so`） |
| error.xml / error.dtd | 错误代码与错误信息列表（见《14_错误码速查表》） |

- 证券（SSE）版本库文件名类似，文件名中带“SSE”标识；看穿式/穿透式监管版本、TGate 网关另有独立库（见 §API 家族，《01》）。
- 平台：Windows(32/64)、Linux(32/64)、Android、iOS（C++ 接口；新版另有 Windows/Linux 64 位）；其他语言（Python/Go/C# 等）均为对 C++ 动态库的封装（工程建议：封装层同样必须遵守回调线程规则）。
- **内存对齐**：库按标准 **8 字节对齐**编译；自行声明结构体或跨语言映射时须保持一致。
- **进程内连接数上限**：Linux 单进程约 180–200 个 API 连接，Windows 约 400 个；更多连接须多开进程。
- 每个 API 实例会创建流文件，无法关闭；多实例须留意操作系统文件句柄限制（Linux `ulimit -n`）。
- 生产/评测版本：`CreateFtdcTraderApi(pszFlowPath, bIsProductionMode=true)`；`true` 使用生产版 API，`false` 使用测评版 API（看穿式监管版本新增参数，原型由 `CreateFtdcTraderApi(const char*pszFlowPath="")` 改来）。

### 2.2 Api/Spi 类模型

- 交易：`CThostFtdcTraderApi`（请求函数：Req*）+ `CThostFtdcTraderSpi`（回调：OnRsp*/OnRtn*/OnErrRtn*/OnFront*）。
- 行情：`CThostFtdcMdApi` + `CThostFtdcMdSpi`。
- 二者封装 FTD 协议。应用通过 Api 发请求，通过**继承 Spi 并重载回调**接收响应与回报。Spi 的回调均有空实现，只需重载关心的方法。
- Api 对象由静态工厂 `CreateFtdcTraderApi` / `CreateFtdcMdApi` 创建，**不能用 new/delete**，用 `Release()` 销毁。
- Spi 实例由用户创建，通过 `RegisterSpi(pSpi)` 注册（每个 Api 一个）。

### 2.3 线程模型

- 客户端至少两个线程：**应用主线程**与 **API 工作线程**。与后台的通讯由 API 工作线程驱动。
- Api 请求接口**线程安全**，可多线程同时发起；Spi 回调全部由 API 工作线程触发（Api/Spi 在不同线程，与平台无关，Linux/Windows 一致）。
- **回调中不得阻塞**：回调阻塞即阻塞 API 工作线程，与后台通讯停止（包括心跳超时断线）。回调应迅速返回，将数据放入缓冲队列（或用 Windows 消息机制）交由业务线程处理。**回调中不建议发送请求或做其他重逻辑，也不要同步等待另一个响应**（等待响应需要工作线程继续运行，必然死锁）。（工程建议：回调内只做拷贝入队；务必深拷贝，指针参数在回调返回后失效。）
- `Init()` 与 `Release()` **非线程安全**，多线程使用须加锁。
- API 请求的输入指针参数**不能为 NULL**。
- 请求函数返回值：0 表示请求已成功发出；非 0 表示错误（详见 §2.5）。

### 2.4 命名规则

| 前缀/形式 | 方向 | 含义 | 示例 |
|---|---|---|---|
| `Req*` | 客户端→后台 | 请求 | ReqUserLogin、ReqOrderInsert、ReqOrderAction |
| `ReqQry*` | 客户端→后台 | 查询请求 | ReqQryInstrument、ReqQryInvestorPosition |
| `OnRsp*` | 后台→客户端 | 请求对应响应 | OnRspUserLogin、OnRspOrderInsert |
| `OnRspQry*` | 后台→客户端 | 查询对应响应 | OnRspQryInstrument |
| `OnRtn*` | 后台→客户端 | 主动回报/通知（私有流/公共流） | OnRtnOrder、OnRtnTrade、OnRtnInstrumentStatus |
| `OnErrRtn*` | 后台→客户端 | 错误回报（交易所/柜台拒绝） | OnErrRtnOrderInsert、OnErrRtnOrderAction |
| `OnFront*` | 连接事件 | 连接建立/断开 | OnFrontConnected、OnFrontDisconnected |
| `OnHeartBeatWarning` | 连接事件 | 心跳超时警告 | |
| `OnRspError` | 通用错误 | 前置地址无法识别、请求/功能无法识别、无此功能、无权限等 | |
| `Subscribe*/UnSubscribe*` | 行情订阅 | 订阅/退订 | SubscribeMarketData、SubscribeForQuoteRsp |

完整方法列表见《13_API方法速查表》。本章是通用指南；官方行为以 [`../api-doc-html/`](../api-doc-html/) 对应页面为准，项目状态以根 README/DESIGN 为准。

### 2.5 nRequestID / IsLast / RspInfo / ErrorID 约定

**请求函数返回值**

| 返回值 | 含义 |
|---|---|
| 0 | 请求发送成功（**仅代表已发出，不代表业务成功**） |
| -1 | 网络原因导致发送失败 |
| -2 | 未处理请求队列总数量超限（查询流控：在途 1 笔） |
| -3 | 每秒发送请求数量超限（查询流控：1 笔/秒） |
| -4 | （TGate 等）API 验证失败（连到非 TGate 地址） |

> 发送不成功可等待一会重发；应做超时重发机制（工程建议：请求队列 + 返回值判断）。

**响应回调参数**

- `nRequestID`：响应对应的请求编号，由请求方指定，用于请求/响应匹配（通常自增，不强制唯一，工程建议保证递增）。
- `bIsLast`：本次响应是否最后一条（一个请求可对应多条响应记录，多次回调，最后一次为 true）。**查询无数据时也会回调一次，数据指针为 NULL、bIsLast=true**。
- `pRspInfo`：处理结果；`ErrorID == 0` 成功，非 0 失败，`ErrorMsg` 为错误描述（GBK 编码）。**多次回调时，第一次之后的 pRspInfo 可能为空（NULL）**，访问前必须判空。（工程建议：`if (pRspInfo && pRspInfo->ErrorID != 0)`。）
- 业务数据指针 `pXxx` 在出错时可能为 NULL。
- 错误码含义见 `error.xml` / 《14_错误码速查表》。

**查询流控速览**：见 §2.12。

### 2.6 流文件（.con）目录与多账号隔离

- 交易 API 实例生成：`DialogRsp.con`、`Private.con`、`Public.con`、`QueryRsp.con`、`TradingDay.con`；行情 API 实例生成：`DialogRsp.con`、`QueryRsp.con`、`TradingDay.con`。
- 存放路径由 `CreateFtdcTraderApi(pszFlowPath)` / `CreateFtdcMdApi(pszFlowPath)` 指定，默认空串为当前目录。例如 `CreateFtdcTraderApi(".\\flow\\")`。
- **目录必须事先创建**，否则运行即报 `RuntimeError: can not open CFlow file in line 279 of file ...ThostFtdcUserApiImplBase.cpp`。
- 若报同类错误 line 338（`CThostUserFlow::OpenFile`）并生成 core 文件：可能是 `ulimit` 的 open files 过小，开不了更多文件/线程。
- 路径末尾可加文件名前缀以区分会话，如 `".//flow/a_"` 将生成 `a_DialogRsp.con` 等。
- 流文件**不可随意修改或删除**，记录了各数据流已接收的序号与交易日信息，对 Resume 续传至关重要；客户端无法决定是否生成。
- **多实例、多账号不得共用流文件/目录**：共用会造成数据流紊乱或缺失（如收不到报单回报、Resume 参考了另一个 DLL 写的进度）。
- 同一目录下同时用行情 API 与交易 API，二者会互相覆盖同名 `.con`；应为每个实例指定不同目录（如 `flow\\md\\`、`flow\\trade\\01\\`）。
- 一个交易日内换交易日会重置进度（`TradingDay.con`）。（工程建议：每个账号、每种 API、每个进程使用独立目录，如 `flow/<broker>_<user>_<td|md>/`。）

### 2.7 前置地址格式（RegisterFront / RegisterNameServer）

格式 `protocol://ipaddress:port`：

| 类型 | 格式 | 示例 |
|---|---|---|
| TCP IPv4 | `tcp://ip:port` | `tcp://192.168.0.1:41205` |
| TCP IPv6 | `tcp6://[ipv6]:port` | `tcp6://fe80::20f8:aa9b:7d59:887d:35001` |
| SSL 加密前置 | `ssl://ip:port` | `ssl://192.168.0.1:41205` |
| SOCKS 代理 | `socks4://`/`socks4a://`/`socks5://代理ip:端口/user:pass@目标ip:端口` | `socks5://代理地址:端口/user:pass@127.0.0.1:10001` |
| 域名 | 可用域名代替 IP | `tcp://domain:port` |
| UDP 行情 | 仍写 `tcp://行情前置:port` | 见下 |

- 可**多次调用 RegisterFront** 注册多个地址实现冗余；断线时 API 自动从地址池**择优**（最先建立 TCP 连接者）重连。同一时间 CTP 只允许用户在一个交易中心有交易权限，故注册的多个地址必须属于**同一中心**；跨中心切换见《01》FENS/TGate。
- 交易和行情端口填反会报“CTP:无此功能”（OnRspError）；对无权限用户报单报“无此权限”。
- `RegisterNameServer`：通过名字服务器自动选取前置，不再直接 RegisterFront（与 FENS 配合，见《01》）。
- **UDP 行情**：`CreateFtdcMdApi(pszFlowPath, bIsUsingUdp, bIsMulticast, bIsProductionMode)`：

| 行情类型 | bIsUsingUdp | bIsMulticast |
|---|---|---|
| TCP 行情（默认） | false | false |
| UDP 单播行情 | true | false |
| 组播行情（仅内网，需确认系统支持） | true | true |

 无论 TCP 还是 UDP，注册地址一律 `RegisterFront("tcp://行情前置:port")`；因 UDP 不可靠，登录、订阅与第一次行情接收仍走 TCP，UDP 使用相同地址与端口，不需额外配置节点。普通行情前置均为 TCP，UDP 须向期货公司申请且仅限专线/内网。mdfront 的 ini 中 `ThostChannelModel`/`ThostUsingMulticast` 决定支持模式：`tcp/空→TCP`；`udp/no→TCP+UDP`；`udp/yes→组播`。

### 2.8 生命周期 Create / RegisterSpi / Subscribe / Register / Init / Join / Release

| 方法 | 说明 |
|---|---|
| `CreateFtdcTraderApi(flowPath, bIsProductionMode)` | 创建实例；多实例须用不同 flow 目录 |
| `RegisterSpi(pSpi)` | 注册回调；可传 NULL 取消 |
| `SubscribePrivateTopic(type)` / `SubscribePublicTopic(type)` | 仅交易 API，须在 Init 前；类型见《01》§订阅模式 |
| `RegisterFront(addr)` / `RegisterNameServer(addr)` / `RegisterFensUserInfo(pInfo)` | 注册连接地址，须在 Init 前 |
| `Init()` | 启动 API 工作线程并开始连接，成功后回调 OnFrontConnected；前面只是准备，此处才真正开始工作；非线程安全 |
| `Join()` | 阻塞等待 API 工作线程结束，返回 int；通常主线程用其保活 |
| `Release()` | 销毁接口对象本身，回收资源并断开连接；非线程安全 |
| `GetTradingDay()` | 登录成功后获取当前交易日（如 20150422） |
| `GetApiVersion()` | 获取 API 版本号字符串（如 `v6.3.6_20141230`），静态方法，可在 Init 前后调用 |
| `GetFrontInfo(pFrontInfo)` | 获取已连前置信息（新版） |
| `RegisterUserSystemInfo` / `SubmitUserSystemInfo` | 看穿式监管终端信息上报（见《01》API 家族） |

**销毁顺序与 Release 注意**：
- 推荐顺序：`pApi->RegisterSpi(NULL); pApi->Release(); pApi=NULL; delete pSpi;`（先注销 Spi 再 Release，最后再删除 Spi 实例）。否则 Release 时回调线程仍引用 Spi 易崩溃/死机。
- 建议用 `ReqUserLogout` 登出，等自动重连后再重新登录，复用 API 实例；**不建议直接 Release API 实例**反复重建。
- 不要在 Spi 回调线程内调用 `Release()`（工程建议：会造成工作线程自等待）。

**初始化标准 5 步**（交易）：创建实例并 RegisterSpi → SubscribePrivateTopic → SubscribePublicTopic → RegisterNameServer 或 RegisterFront → Init；连接成功回调 OnFrontConnected → （如需）ReqAuthenticate → ReqUserLogin → ReqSettlementInfoConfirm。

行情 API 为：CreateFtdcMdApi → RegisterSpi → RegisterFront → Init → OnFrontConnected → ReqUserLogin → SubscribeMarketData。

### 2.9 GetApiVersion / GetFrontInfo

- `GetApiVersion()`：返回版本字符串（如 `v6.3.6_20141230`、`6.7.13`系列），用于日志和问题定位。
- `GetFrontInfo(CThostFtdcFrontInfoField*)`：字段 `FrontAddr`（前置地址）、`QryFreq`（查询流控）、`FTDPkgFreq`（FTD 流控）。连接成功后可取得正确前置地址，**登录成功后**才能取得正确的查询流控和 FTD 流控值。`QryFreq` 对操作员不受限，返回极大值。
- `GetTradingDay()`：登录后返回交易日；登录前不可靠。
### 2.10 数据类型与编码
### 2.11 编译链接与环境

## 3. 标准流程/时序
## 4. 代码范例
## 5. 常见坑与排查
## 6. 检查清单
## 7. 关键词索引



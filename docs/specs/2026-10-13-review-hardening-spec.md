# Spec: 评审落地第一批——失败路径健壮性硬化

- Triage: `ready-for-agent`
- 来源:`docs/review/2026-10-13-architecture-review.md` §4 P1/P2 风险与 §7 建议 1/2/3/4/7(部分)/10
- 日期:2026-10-13

## Problem Statement

架构评审确认 CTPBuddy 的正常路径质量上乘,但异常路径存在四类会直接伤害下游用户的缺陷:

1. **下游策略会无告警挂起**:Shim 没有心跳与请求超时,核心静默死亡(无 FIN)时客户端收不到 `OnFrontDisconnected`,挂起请求永不完成;ZeroMQ 发送队列满时丢弃最老帧,被丢请求的 pending 永不完成,若是查询则查询闸门(`qry_in_flight_`)永久卡死,之后所有 `ReqQry*` 永远返回 -2,无恢复路径。
2. **协议漂移被放大为下游崩溃**:响应帧载荷尺寸与结构体不符时,Shim 回调"成功 + NULL 数据"(ErrorID=0),把 NULL 当成功的客户端(包括项目自己的 demo)解引用即崩溃,而不是收到明确错误。
3. **权威事件流静默分叉**:journal 是唯一权威事件流,但写失败只打一行 stderr;账本已应用事件、journal 未落盘时,世界继续运行,live 状态与恢复/重放结果分叉且无任何告警面。
4. **Python SDK 违反自身异常契约**:请求超时抛裸 `queue.Empty` 而非承诺的 `CTPError`;协议错误(bad magic/版本不符/截断)与对端干净关闭混同为同一路径,排障时无法区分"核心挂了"与"协议坏了"。

另有三处小修正随批落地:logout 不校验身份、Web 结算正文上限常量与实际限制矛盾(声明 512KB、实际 8KB,且与核心侧 512KB 不一致)、两处文档漂移(依赖面表述、"四处人工同步"低估了 codegen+CI 门禁)。

## Solution

按"异常路径与正常路径同等可观测"的原则逐项修复:Shim 实现 PING/PONG 心跳与 pong 截止期,半开连接在秒级触发 `OnFrontDisconnected` 并清空挂起请求;发送队列满时拒绝新请求(厂商语义 -2)而非丢弃旧帧;载荷尺寸不符改为携带明确 ErrorID 的错误回调;journal 写失败状态上浮进 ADMIN status 并提供停机兜底开关;SDK 超时统一转 `CTPError` 并区分协议错误与干净关闭;auth JSON 拼接对非 ASCII 字节转义,杜绝"发出非法 JSON 后认证永无响应"。所有修复不改变任何正常路径的可观测行为与 wire 契约。

## User Stories

1. 作为下游策略开发者,我希望核心进程静默死亡(无 FIN、NAT 挂起)时 Shim 在数秒内回调 `OnFrontDisconnected`,以便我的重连逻辑接管而不是策略卡死。
2. 作为下游策略开发者,我希望断链时所有挂起请求立刻以错误回调收尾(而非等待超时),以便我的挂单/查询状态机进入一致的"未知态"处理分支。
3. 作为下游策略开发者,我希望 ZeroMQ 发送队列满时新请求同步返回 -2"未处理请求超过许可数",以便我按厂商契约做退避重试,而不是请求被静默吞掉后永久等待。
4. 作为下游策略开发者,我希望查询闸门永远不会因一次静默丢帧而永久卡死,以便长时间运行的策略在瞬时拥塞后仍能继续查询。
5. 作为下游策略开发者,我希望响应帧与预期结构体尺寸不符时收到 `OnRspError`/错误回调(带明确 ErrorID 与 ErrorMsg),以便版本错配表现为可诊断的错误而不是 NULL 解引用崩溃。
6. 作为 desk 运维,我希望 ADMIN `status` 能看到 journal 的健康状态(最近写失败原因、连续失败计数、当前 seq/pending),以便我在巡检时发现权威事件流已分叉,而不是数天后重放校验才暴露。
7. 作为 desk 运维,我希望可以选择"journal 写失败即停机"的严格模式,以便对一致性要求高的场景宁可停机也不静默分叉(默认保持宽容模式,行为不变)。
8. 作为 SDK 使用者,我希望任何请求超时都抛出 `CTPError` 并带超时上下文,以便我的异常处理只面对一种异常类型。
9. 作为 SDK 使用者,我希望连接因协议错误(bad magic/版本不符/截断)断开时异常信息明确说明协议错误,以便我区分"核心挂了"与"版本不匹配"并采取不同处置。
10. 作为 SDK 使用者,我希望行情订阅请求的超时与交易请求的超时行为一致(都转 `CTPError`),以便统一处理。
11. 作为核心维护者,我希望 auth 载荷中的非 ASCII 字节被合法转义,以便 AuthCode/AppID 含 GBK 字节时认证仍能得到服务端响应(成功或失败),而不是永远无响应。
12. 作为核心维护者,我希望 auth 响应中显式的失败(`ok:false`)被识别为认证失败回调,而不是等到超时,以便下游更快得到失败信号。
13. 作为核心维护者,我希望 `ReqUserLogout` 校验 BrokerID/UserID 与连接身份一致后才清会话并记 journal,以便 journal 的 `session_logout` 事实不被任意身份污染。
14. 作为 Web 后台用户,我希望结算单正文的真实上限(512KB,与核心一致)在声明与校验中一致,以便大结算单能实际提交成功,维护者也不会被假常量误导。
15. 作为项目贡献者,我希望 DESIGN 文档的依赖面表述与"消息号同步机制"描述反映真实机制(codegen 回写 + CI 门禁),以便新人按文档理解时不会得到过时的危险信号或虚假的手工负担。
16. 作为测试工程师,我希望上述每个行为都有一个不依赖真实核心崩溃的测试入口(假 CB server / 故障注入 / 假 ADMIN),以便回归可自动化。

## Implementation Decisions

### Shim 心跳与超时(S1)

- 心跳在 Shim 侧单边实现,不改核心:核心对 PING 的回显 PONG 既有行为(handlers 的 Ping 路由原样回 echo)即为心跳应答,Wire 契约零变更。
- reader 线程维护"最近收到任意帧"时间戳;空闲超过 `PING_IDLE`(默认 5 秒)发送一个 PING 帧(TCP 走既有写路径,ZMQ 入 `zmq_outbound_` 并 flush);自最近收帧起超过 `PONG_DEADLINE`(默认 15 秒)未收到任何帧,判定连接死亡:走既有断链路径(关 socket、触发 `OnFrontDisconnected`、清空全部 pending 并对每个挂起请求按其 dispatch 行的 err 面补错误回调、进入既有重连退避)。
- TCP 读必须以有界超时轮询(或 SO_RCVTIMEO 分块)实现空闲检测,不得阻塞在无超时的 read 上;ZMQ 用 zmq_poll 超时。
- 两个阈值经环境变量可调(`CTPBUDDY_SHIM_PING_IDLE_SEC` / `CTPBUDDY_SHIM_PONG_DEADLINE_SEC`),默认值写入 shim README;设为 0 关闭心跳(兼容旧行为)。

### Shim 发送队列满拒新(S2)

- `zmq_outbound_` 达到容量(默认 1024,经 `CTPBUDDY_SHIM_ZMQ_SEND_QUEUE` 可调)时,`send_request` 在锁内同步返回 **-2**(厂商语义"未处理请求超过许可数"),**不**再 pop_front 丢弃最老帧;已入队帧保证最终发出。
- 断链清场路径同样清空 `zmq_outbound_`,不残留对已死连接的积压。

### Shim 载荷尺寸防御(S3)

- 生成器的 dispatch 模板改为三分支:尺寸等于 `sizeof(S)` 走正常类型化回调;尺寸为 0 保持现状(合法的"无数据"成功面,如空载成功响应);**尺寸非 0 且不等于 `sizeof(S)`** 不再走"成功 + NULL",改为经该行的 err 面(或 push 行的 `on_rsp_error_fallback`)回调 `ErrorID=-1`、ErrorMsg 载明"payload size mismatch (got N, expected M)"。
- 复合载荷行(input ++ RspInfo 的错单回报面)按其既有切分逻辑不变;本决策只影响单结构体行。
- 该改动落在 codegen 模板,重新生成后依赖既有 CI"生成物与仓库一致"门禁防漂移。

### auth 载荷转义与失败识别(S4)

- `json_string` 对所有 `<0x20` 与 `>0x7F` 的字节输出 `\u00XX` 转义,保证产物是合法 JSON(核心侧 serde 可解析);正常 ASCII auth 场景字节级不变。
- `on_auth_rsp` 显式识别 `"ok":false` 走认证失败回调(带 error 文本),只有既非 true 也非 false 的意外形状才留给超时兜底。

### journal 写失败上浮(R1)

- Journal 写者增加健康状态:最近一次写/同步失败的原因与时间、连续失败计数;成功写入或 flush 即清零。
- ADMIN `status` 新增 `journal` 对象(seq、pending、last_error、consecutive_failures),字段缺失语义为"journal 未启用"。
- 新增环境变量 `CTPBUDDY_JOURNAL_HARD_FAIL=1`:写失败时世界循环置 shutdown(停机保一致),默认关闭保持现行为。状态上浮与停机兜底都只依赖既有世界循环单写者路径,不新增并发面。
- canonical journal contract 不变:不新增事件类型(写失败时本来就写不进 journal),健康状态是进程内观测面。

### logout 身份校验(R2)

- `ReqUserLogout` 的 BrokerID/UserID 与连接已认证身份不一致时拒绝(错误码与登录侧身份不一致同族),成功路径才清 investor 绑定并记 `session_logout` journal 事件;未登录连接的 logout 维持现状回应(不引入新的状态假设)。

### SDK 异常契约(P1/P2)

- 所有请求等待路径(交易、查询、行情订阅)超时统一抛 `CTPError`,错误信息标明超时与等待的消息类型/req_id。
- 读循环区分退出原因:协议错误(bad magic、版本不符、截断等解码 ValueError)记录为连接级错误状态并让后续等待立刻以携带原因的 `CTPError` 失败;干净关闭保持现有静默收尾语义。
- 不改变任何成功路径与帧序行为。

### Web 结算上限统一(P4)

- 结算报告提交端点的真实 body 上限取 512KB(与核心侧结算报告载荷上限一致),其余端点维持现有通用上限;两处常量改为单一来源引用,消除"声明 512KB 实际 8KB"的自相矛盾。

### 文档漂移修正

- DESIGN §11.1:依赖面表述改为"域 crate 零外部依赖;server 引入异步传输栈(tokio/zeromq/libloading 等),Cargo.lock 锁定、离线可构建"。
- DESIGN §6.4:消息号同步描述改为反映实际机制——id 在 wire 模块手工指派为唯一源,codegen 回写各语言镜像,CI 以"生成物与仓库一致"门禁强制;人工面只剩 id 指派本身。

## Testing Decisions

- **好测试只断言外部可观察行为**:回调序列、返回码、status 字段、异常类型;不断言内部结构/线程布局。
- **Shim 行为(心跳、队列拒绝、尺寸防御)**:以真实 shim 二进制 + Python 假 CB server 为 seam——前者是既有最高 seam(m1_shim_e2e 先例),后者是一个只做 accept/应答/静默的最小 TCP 服务。新增 e2e:静默 server 下 demo 在 PONG_DEADLINE 内收到 `OnFrontDisconnected`;小容量队列下第三次请求返回 -2;错误尺寸帧触发错误回调而非 NULL 成功。demo 作为测试载体允许增加受入口参数控制的最小断言模式(仅测试用,不进发布叙事)。
- **journal 健康(R1)**:沿用 journal.rs 既有的故障注入先例(把目标文件替换为目录使写失败):断言失败后状态字段、清零行为;停机模式断言世界循环退出。admin status 新字段在既有 operations 断言风格下补充。
- **logout 校验(R2)**:沿用 handlers.rs 既有 world 直驱单测先例(手工开连接、喂帧、断言回应帧)。
- **SDK(P1/P2)**:沿用 test_py.py 既有假 server 单测先例:超时断言异常类型与消息、协议错误断言连接级失败原因。
- **回归面**:全部既有 Rust 单测、Python 单测、shim e2e、m2/m3 套件必须保持绿;生成物再生成后必须与仓库零差异(CI 门禁的本地等价检查)。

## Out of Scope

- CI 门禁补齐(Python lint、Python 版本矩阵、m4_install_shim 入口)——属工作流变更,单独批次。
- e2e harness 收敛与 test_py.py 自动发现——纯重构,单独批次。
- World 下沉(admin 命令表驱动、Conn 拆分、恢复事件处理下沉)——架构演进,单独 spec。
- refdata manifest 来源登记、SUB_MD 计数防御(S5)、RegisterSpi 锁(S6)、TCP 路径双份解码合并、~1MB 收帧缓冲 churn——登记为后续小额工单。
- 无 tick 时涨跌停不设防(R3):属仿真边界口径,需先定文档口径再动代码。

## Further Notes

- 全部修复遵循"正常路径零行为变化"约束:心跳只在空闲时发送;队列拒绝只在满时发生;尺寸防御只命中协议错配帧;SDK 变化只影响异常面。
- 心跳默认参数(5s/15s)取"远小于人工感知阈值、远大于正常抖动"的保守值;阈值环境变量名进 shim README 与 operations 文档。
- 本 spec 的工单拆分见 `docs/tickets/2026-10-13-review-hardening/`（to-tickets 产物，2026-10-15 自 `.scratch/` 归档）；gh 令牌失效期间以文件形式承载 `ready-for-agent` 标注，恢复认证后可平移到 GitHub Issues。

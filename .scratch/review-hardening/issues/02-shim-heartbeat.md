# 02: Shim PING/PONG 心跳与半开连接断链清场

**What to build:** 核心静默死亡(无 FIN)时,Shim 在 PONG_DEADLINE 内回调 `OnFrontDisconnected`,全部挂起请求立刻以错误回调收尾,随后进入既有重连退避;核心侧零改动(PING 已有 PONG 回显)。

**Blocked by:** 01(同一 reader/清场管线的先序改动)

**Status:** done

- [x] reader 线程维护"最近收帧"时间戳;TCP 读以有界超时轮询,ZMQ 以 poll 超时,不得无超时阻塞
- [x] 空闲超过 `CTPBUDDY_SHIM_PING_IDLE_SEC`(默认 5)发送 PING;自最近收帧起超过 `CTPBUDDY_SHIM_PONG_DEADLINE_SEC`(默认 15)未收任何帧判定死亡
- [x] 死亡路径:关 socket、`OnFrontDisconnected`、清空全部 pending 并按各 dispatch 行 err 面补错误回调、清空发送队列、进入既有重连退避
- [x] 两个环境变量设为 0 关闭心跳(兼容旧行为);默认值写入 shim README 与 docs/operations.md
- [x] e2e:Python 假 CB server(accept 后静默) + 真实 demo,小阈值下断言在 deadline 内收到 `OnFrontDisconnected` 且进程不挂起
- [x] 既有 shim e2e 保持绿(默认阈值不影响正常流程)

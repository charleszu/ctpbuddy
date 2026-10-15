# 01: Shim ZeroMQ 发送队列满时拒新请求而非丢旧帧

**What to build:** 下游在 ZeroMQ 慢消费者场景下,发送队列达到容量时新的 `Req*` 调用同步返回 -2(厂商语义"未处理请求超过许可数"),已入队帧保证不丢;查询闸门不再可能因丢帧永久卡死。队列容量经环境变量可调以便测试注入小容量。

**Blocked by:** None (can start immediately)

**Status:** done

- [x] 队列达到容量时 `send_request` 在锁内返回 -2,不再 pop_front 丢弃最老帧
- [x] 队列容量默认 1024,环境变量 `CTPBUDDY_SHIM_ZMQ_SEND_QUEUE` 可调(正整数,非法值回落默认并告警)
- [x] 断链清场路径清空发送队列
- [x] e2e:pyzmq 假 ROUTER(bind 后不读) + demo 以小容量队列连续请求,断言达到容量后的请求返回 -2 且无请求被静默吞掉
- [x] 既有 shim e2e(m1_shim_e2e / curve 真实 shim 路径)保持绿

> 落地备注(2026-10-13 更新): shim 静态链接 libzmq(ADR 0002)后本机 e2e 已完整覆盖——`shim_hardening.py` 实测 accepted=2 refused=1198,零丢帧。

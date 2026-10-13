# 21: shim ZMQ 读缓冲复用（消除每帧 ~1MB 分配）

**What to build:** ZMQ 读路径的接收缓冲提升为 reader 线程私有复用，空闲心跳与 13 字节 PONG 不再触发 ~1MB 级分配；帧解析语义不变。评审项 SH-F4（§4.3、§7）。

**Blocked by:** 15（api_core.cpp 同文件先落单一真相重构）。

**Status:** done (landed as cdca95d)

- [ ] 验证步：复核 zmq_recv_part 每次 resize ~1MB、read_zmq_frame 循环内 first/frame/ignored 逐帧新建；确认 reader 单线程独占与 decode 以长度为准（复用安全性依据）；确认所有 early-return/continue 路径不复用脏数据
- [ ] 缓冲提升为线程私有成员（或循环外复用）；TCP 读路径不动
- [ ] `python shim/build_msvc.py --demo` 构建通过；m1_smoke、m1_shim_e2e、shim_hardening（flood 探针）通过

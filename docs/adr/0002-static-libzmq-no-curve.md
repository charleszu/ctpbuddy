# ADR 0002:Shim 静态链接 libzmq,移除 CURVE 传输加密

- 状态:已接受
- 日期:2026-10-13
- 范围:shim 构建(build_msvc/build_linux)、shim ZeroMQ 接入、core ZMQ 传输(security/transport_zmq_native)、运维文档与 CI
- 替代关系:取代 `docs/operations.md` 与 DESIGN 中"可选 CURVE/ZAP 加密"的原设计;journal 契约与 CB 帧协议不受影响

## 背景

原实现里 shim 通过运行时 dlopen 加载 libzmq,核心侧为 CURVE 再以 libloading 加载原生 libzmq(第三个 ZMQ 实现)。架构评审(2026-10-13)指出:运行时加载使 `zmq://` 部署破坏 shim"下游只换单个 DLL"的产品承诺;CURVE 是核心侧原生库依赖的主要动因。评估结论:本系统两侧均为运营者自己的程序,仿真报文不敏感,传输安全的正确位置在部署层(VPN/SSH 隧道/网络隔离);真正敏感的 ADMIN token 与静态结算数据各有独立控制面。CURVE 的密钥分发、ZAP 白名单管理与原生库依赖是净成本。

## 决策

1. **libzmq 静态链接进 shim**:构建脚本从锁定版本(当前 4.3.5,SHA256 固定)的官方源码,用与 shim 相同的工具链(Windows 同一 vcvars 的 cl, Linux 同一 g++)经 CMake 编译为静态库,链入两个 `thost*_se` 二进制。DLL/so 保持厂商单文件形态,无运行时 libzmq 依赖。
2. **编译期 `ZMQ_CURVE=OFF`**(同 `-DENABLE_CURVE=OFF -DWITH_LIBSODIUM=OFF`):shim 与核心都不再提供 CURVE 能力面;`apply_curve_options`、`transport_zmq_native.rs`(libloading 适配)与全部 `CTPBUDDY_ZMQ_CURVE_*` 环境变量删除。
3. **传输安全交部署层**:跨机部署使用 wireguard/SSH 隧道或隔离内网;ADMIN token 仍是控制面身份校验,ADMIN 通道自身的加固(如 TLS)留待独立决策。
4. 核心 ZMQ 路径统一由纯 Rust `zeromq` crate 承载(ROUTER/DEALER + 可选 PUB/SUB),不再有第二个实现。

## 约束

- libzmq 源码版本与 SHA256 在构建脚本中固定;升级版本必须同时更新脚本、提交新版 tarball 并走正常评审。
- **锁定版本的源码 tarball 随仓库分发**(`shim/third_party/src/`,2.5MB):首次构建零网络依赖,内容身份由 git 历史与脚本内 SHA256 双重锁定;解压目录与编译产物保持 ignore。
- MPL-2.0 归属:静态链接的 libzmq 源码(即仓库内 tarball)属 ZeroMQ 项目,发布物需保留来源与版本声明(见 DESIGN §15)。
- 旧 journal 中 `flow_input` 的 `curve` scope 标签仍可被重放器接受(历史录制兼容),但不再有产生它的代码路径。
- 环境变量 `CTPBUDDY_LIBZMQ_DIR` 允许复用预构建安装前缀(CI 缓存场景);除此之外无任何外部库开关。

## 后续工作

- 构建 CI 对 `shim/third_party/` 缓存做 key(源 tarball + 静态库),减少每次构建的编译时间。
- 若未来出现真实的跨不可信网络部署需求,重新评估传输安全(连同 ADMIN 通道一起),另行 ADR。

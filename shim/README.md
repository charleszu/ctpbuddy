# CTPBuddy Shim（CTP ABI 同名替换 DLL）

`thosttraderapi_se.dll` / `thostmduserapi_se.dll` 的 drop-in 替换：vtable 与 CTP 6.7.13
官方头文件逐项一致，请求结构体按原始字节经 winsock TCP 送到 `ctpbuddy-server`，
响应帧由 reader 线程派发为 SPI 回调。

## 构建

```
python shim/build_msvc.py [--demo] [--sdk DIR]
```

- 需要 MSVC（VS 2022 任一版本或 Build Tools；自动探测 `vcvars64.bat`，也可用
  `VCVARS64` 指定）。
- `--sdk` / `CTPBUDDY_SDK`：CTP 6.7.13 头文件目录，默认 `ctpsdk/6.7.13_20260225`
  （不入库）。接受厂商布局 `<sdk>/td/win64` + `<sdk>/md/win64`，或平铺目录；
  `docs/api-doc-html/files/` 已含同版本头文件（仅换行符不同），CI 即用它：
  `python shim/build_msvc.py --demo --sdk docs/api-doc-html/files`。
  Shim 只需头文件，不链接厂商 `.lib`。
- 构建前会自动运行 `codegen/gen_shim.py` 重新生成 `src/generated/`；生成产物入库，
  CI 校验回写后 `git diff` 为空。

产物：`bin/*.dll`、`lib/*.lib`、`bin/demo_td.exe`（`--demo`）。

## 代码生成

- `codegen/gen_shim.py`：解析 `CThostFtdcTraderApi` / `CThostFtdcMdApi` 全部纯虚函数
  （先归一化空白，支持折行签名；任一 `virtual` 未匹配即报错退出），生成
  `api_*.hpp`、`api_*_reqs.cpp`、`dispatch_*.cpp`。`MSG` 表是消息编号的唯一来源，
  新编号会同步追加到 `msgs.rs` / `api_core.hpp` / `wire.py`（只追加，不改号）。
- `codegen/gen_structs.py`：结构体镜像（Rust / Python / C++ static_assert /
  `registry.json`）；消息表 `MESSAGES` 从 `gen_shim.MSG` 导入，本脚本只补充各消息
  的 payload 结构体列。

## 线程与并发契约

- 应用线程调用 `Req*` / `Register*` / `Init` / `Release`；**唯一** reader 线程负责
  连接、读帧和**全部** SPI 回调。
- `pending_`（req_id → 请求缓存）只在持 `mu_` 下访问：`send_request` 在应用线程
  插入，派发行在 reader 线程持锁取出后**释放锁再回调**（回调里可再次调用 API）。
- 本地应答的回调（未实现的 `Req*` → `OnRspError(-1)`、重复登录、`ReqAuthenticate`
  参数错误、MD 询价桩）**不在应用线程直接回调**：经 `ApiCore::post_callback`
  投递到 reader 线程队列，reader 在两帧之间（≤100ms，`select` 超时）与重连退避
  期间排空。未连通时这些请求与厂商 API 一致返回 `-1`（网络连接失败），不入队。
- 未 `RegisterSpi`（SPI 为空）时所有回调静默丢弃，但 pending 仍被消费，不泄漏。
- 返回码遵循 CTP：`0` 已发送；`-1` 网络未连通 / 参数空指针；`-2` 已有在途查询
  （查询流控，仅在帧真正发出后才置位）。
- `GetTradingDay()` 持锁拷贝到 thread_local 缓冲后返回：对调用线程有效至该线程
  下一次调用；重登录后新交易日立即可见。
- 两个 DLL 各执行一对 `WSAStartup` / `WSACleanup`，Winsock 按进程引用计数，成对即可。

## 验证

```
cargo build --bin ctpbuddy-server --manifest-path core/Cargo.toml
set PYTHONUTF8=1
python tests/e2e/m1_shim_e2e.py
set CTPBUDDY_SHIM_MODE=auth-check && python tests/e2e/m1_shim_e2e.py
```

`demo_td.exe` 输出统一为 UTF-8（CTP `ErrorMsg` 为 GBK，demo 打印前转码），
harness 需在 `PYTHONUTF8=1` 下运行（CI 已设置）。

## Linux / WSL

```n python3 shim/build_linux.py --demo        # g++ -> shim/bin/thosttraderapi_se.so, thostmduserapi_se.so, demo_td
 CTPBUDDY_CORE=<ctpbuddy-server> PYTHONUTF8=1 python3 tests/e2e/m1_shim_e2e.py
```n
- 头文件取 `ctpsdk/.../td/linux64`（亦接受 win64 / 平铺目录）；产物名与厂商 .so 相同，符号 mangling 一致（已用 nm 对照）。
- POSIX 路径：poll、MSG_NOSIGNAL（对端消失不触发 SIGPIPE）、iconv 做 UTF-8↔GBK。
- 前置地址支持主机名；环境变量 `CTPBUDDY_ADDR=tcp://host:port` 覆盖应用注册的所有前置。
- WSL 下 cargo 请设 `CARGO_TARGET_DIR` 到 Linux 文件系统，避免与 Windows 的 core/target 混用。

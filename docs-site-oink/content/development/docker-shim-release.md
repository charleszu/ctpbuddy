---
title: Docker、Shim 与发布工具
linkTitle: Docker 与发布
weight: 50
description: 说明 Linux Docker 验收、Windows Shim 构建和白名单发布包的真实行为。
---

工具入口为 `tools/docker_acceptance.py`、`docker/compose.yml`、`shim/build_msvc.py` 和 `tools/release_package.py`。主要来源提交为 `4d2af40`。本页描述工具实际做什么，不把工具存在写成发布已经完成。

## Docker 验收

```bash
python tools/docker_acceptance.py
# 或
 docker compose -f docker/compose.yml build
 docker compose -f docker/compose.yml run --rm acceptance
```

脚本先检查 docker CLI 和 `docker info`。CLI 缺失或 daemon 不可用时安全返回 `SKIP`，不会伪造通过；因此本次源码文档构建不运行 Docker。容器覆盖 Linux 下 Python wheel/CLI、Rust server build/run、refdata 和可选 Web/OINK 检查，**不运行 Windows ABI Shim**，不复制官方 `ctpsdk/`、Windows DLL/EXE、真实账户或行情数据。

## Shim 构建

`python shim/build_msvc.py --demo` 是 Windows/MSVC 构建入口，面向使用者自备 SDK 头文件。Shim 负责同名 CTP ABI、结构体布局、帧收发和 SPI 分发，不承载撮合/账本业务。Linux Docker 不验证这条 ABI 链路；真实下游 demo 需要 Windows 和匹配的运行库。

## 发布包

```bash
python tools/release_package.py
python tests/e2e/release_package.py
CTPBUDDY_TEST_REAL_RELEASE=1 python tests/e2e/release_package.py
```

`build_package()` 来源固定为 `<repo>/shim/bin`，只复制 `thosttraderapi_se.dll`、`thostmduserapi_se.dll` 和存在时的 `demo_td.exe`。它拒绝 symlink/reparse point，读取项目版本和当前 commit，检查 PE machine、DLL 标记、统一架构，并为每个文件记录 SHA256/大小/架构。输出含 `manifest.json`、`shim/ctpbuddy-shim-manifest.json`、`LICENSE`、`INSTALL.txt`。

脚本不构建、不执行 Shim，不读取/复制原始 SDK、`data` 或真实数据；manifest 的 commit 只是打包时源码状态，不能证明二进制构建来源。安装工具另有 dry-run/`--apply` 和恢复路径，不能对真实生产客户端默认操作。

## 当前发布状态

项目有本地发布包和 Docker 验收工具，但 M4 三渠道发布、完整发布流程和 Windows 真实产物验收仍不能统称完成。文档必须保留这些条件与 SKIP 结果。

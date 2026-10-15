# 安全策略

## 报告漏洞

请通过 GitHub 仓库的 **Privately report a vulnerability**（Security → Report a vulnerability）私下报告，不要公开 issue。复现步骤、影响版本与建议修复方式越具体越好；本项目目前为维护者业余响应，没有付费悬赏计划。

## 安全边界（设计如此，不是漏洞）

CTPBuddy 是**本地 / 受信内网**的仿真柜台，不是面向公网的生产交易系统：

- **TD 交易前置不校验密码**，只做身份绑定（BrokerID/UserID 一致性）。不要把 5560 端口暴露到不可信网络。
- **ADMIN 控制面默认只绑回环**。远程绑定必须同时满足 `CTPBUDDY_ALLOW_REMOTE_ADMIN=1` 与非空 `CTPBUDDY_ADMIN_TOKEN`；跨机部署要求 wireguard/SSH 隧道或隔离内网。
- **传输层不加密**：CURVE/ZAP 已按 [ADR 0002](docs/adr/0002-static-libzmq-no-curve.md) 明确移除，传输机密性由部署层（VPN/SSH 隧道）提供。
- **Web 后台仅回环**：Host/Origin/`Sec-Fetch-Site` 校验、CSRF token（`secrets.compare_digest`）、只允许回环 ADMIN 上游、写操作白名单。它不是远程管理面板。
- **权威数据不编造**：合约/费率/日历均由使用者提供；随仓快照的来源与许可边界见 README「许可」与各 MANIFEST。官方 SDK 二进制（`.dll`/`.lib`）不在仓库内。
- 随仓 `docs/` 下的官方 CHM 反编译 HTML 仅供文档交叉引用，属上期技术所有。

## 发布完整性

- Shim 发布包（`tools/release_package.py`）只从白名单复制文件，带逐文件 SHA256、PE machine/architecture 与 commit 记录，并在安装/恢复前做备份。manifest 中的 commit 不证明二进制构建来源——正式发布的签名与 tag 流程尚在路线图（DESIGN §12）。

## 支持版本

0.1.x 初始开发阶段：只对 `main` 最新提交提供修复，不维护旧版本安全分支。

# 13: py 结构收敛——client 管道 / web GBK 归一 / core argv

**What to build:** SDK client 的三份 pending-socket 管道收敛为上下文管理器+发送助手；web 的结算报告 GBK 归一改调 Admin 单一实现且同请求只编码一次；核心进程 argv 构建样板共用。锁序、异常路径、异常契约、生成的命令行逐项不变。评审项 PY-N1/N2/N3（§5.3）。

**Blocked by:** 07（client.py 同文件先清死导入与 _wake_all）。

**Status:** done (landed as eadb2dd)

- [ ] 验证步：复核三处管道逐字重复（注册/closed 检查/send_lock/finally pop）；web 与 admin 的 GBK 归一化逐字等价 + web 同请求二次 encode；cli 与 replay 的 argv 拼接重叠
- [ ] 提 `@contextmanager _slot()` 与 `_send()`；web 分支改调 `client.settlement_report`（与 replay 一致）；提 `core_argv()`（argv 序列保持原序）
- [ ] test_py.py 全绿（SDK 生命周期/超时契约断言）；m3_5_settlement_info（web/结算路径）、flow_replay、operations 通过
**落地备注（终审追认）:** PY-N2（web 结算报告 GBK 归一改调 Admin.settlement_report）经实地勘察放弃——web 走自有 invoke/admin_cmd 通道，调用点合并属重构而非去重，理由记录于提交 eadb2dd。

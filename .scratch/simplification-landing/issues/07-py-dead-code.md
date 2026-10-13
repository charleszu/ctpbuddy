# 07: Python 包死代码清理

**What to build:** py 包的零引用函数/常量/导入/再导出被删除，公共 CLI 与 SDK 入口行为不变。评审项 PY-N7、N9~N12、N14~N16、O-1（§5.1、§5.2）。

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] 验证步：逐项复核——store.WEB_TABLES 全仓零引用（web 侧是内容相同的 WEB_PROJECTION_TABLES）；wire.iter_frames 零引用；client 的 LOGOUT/LOGOUT_RSP 导入未用；sdk/__init__ calendar 再导出无消费者（顶层包仍导出同名项）；calendar.load_calendar 零调用；refdata.csv_dir 零调用；_call_flexible 单行体与 _instantiate 的 type⊂callable 重复分支；scenario 两个死参数；Client._wake_all 仍死（O-1，主评审已复核）
- [ ] 删除上述死项；WEB_TABLES 二选一收敛为单一真相（推荐 web 改 import）；_instantiate 合并分支、注释保留
- [ ] `python tests/test_py.py`、`test_journal_contract.py`、`test_projection_upgrade.py`、`test_query_expectations.py` 全绿；e2e 抽查 m3_refdata 通过
- [ ] 不碰：PY-N13（settle_day 推导分支，批次 6）、PY-N17（verify_events 重复校验，批次 6）

# 16: test_py 注册哨兵 + CI 子集重跑删除 + m4_install_shim 门禁

**What to build:** test_py.py 获得"定义集==注册集"漂移哨兵（忘注册从静默跳过变启动即失败）；unit job 删除被 workspace 测试覆盖的 cargo 子集重跑；m4_install_shim 进入 CI。输出格式、测试顺序、覆盖集合不变（新增门禁为纯增量）。评审项 TT-A2、TT-N6、TT-N5（§6.2、§6.3）。

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] 验证步：AST 比对当前 25/25 全注册、定义序≠调用序（哨兵须保持调用序列原样）；复核 CI unit job 内 `-p ctpbuddy-server journal_contract` 是 workspace 测试的子集、该步 python vectors 检查独有价值保留；复核 m4_install_shim 仍无任何 CI 入口且本机可独立运行
- [ ] main() 顶部加集合断言哨兵（+4 行），调用序列不动；CI 删 cargo 子集行、步名保留；m4_install_shim 加入 unit job（pip install ./py 之后）
- [ ] `python tests/test_py.py` 输出与改前逐行一致；临时注释掉一个注册验证哨兵确实触发 AssertionError 后还原

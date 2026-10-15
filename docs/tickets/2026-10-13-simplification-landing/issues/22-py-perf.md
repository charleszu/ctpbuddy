# 22: py 性能——回放 diff 短路 + store 单次序列化

**What to build:** 回放成功路径在 core hash 相等时跳过 first_core_diff 的完整规范化+canonical 序列化（sha256 抗碰撞已是既有 ok 判定假设）；投影重建不再对同一事件序列化两次 JSON。回放结果、rebuild 产物逐位不变。评审项 PY-NP1、PY-NP3（§7）。

**Blocked by:** 13（replay.py 同文件先落 core_argv 收敛）。

**Status:** done (landed as 433f274)

- [ ] 验证步：复核 hash_stream(core=True) 与 first_core_diff 的重复规范化路径；复核 store._project 的 `_json(d)` 与 `_json(e["data"])` 双序列化及 data=None 时 `"{}"`/`"null"` 差异
- [ ] `diff = first_core_diff(...) if recorded_core != replay_core else None`；store 复用一次序列化结果并保持 None→"{}" 语义
- [ ] test_py.py（replay/store 断言）、test_projection_upgrade、m3_6_projection、flow_replay 通过；对一个既有录制目录做迁移前后 replay 输出逐行对比

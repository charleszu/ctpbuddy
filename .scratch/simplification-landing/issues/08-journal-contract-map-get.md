# 08: journal_contract 改用 Map::get（-130 行样板）

**What to build:** journal contract 校验器全部键访问改用标准 Map 取值，类型判别收敛为单一函数；校验结论与错误串逐字不变。评审项 SV-R1（§2.1）。

**Blocked by:** 04（同 crate 先清死代码，含本文件 canonical_event 的 cfg(test) 处理）。

**Status:** ready-for-agent

- [ ] 验证步：复核 `.iter().find(键相等)` 跨行样板仍遍布（主评审已复核 20+ 处，同文件 40 行已在用 `map.get`）；类型字面量 3 份与 matches_data_type 判别函数并存；data_has_required_fields 三级链式导航
- [ ] 全部换 `map.get(k)`；类型清单提 `known_type()`；链式导航提为一次取 spec
- [ ] 双语言 vectors（cargo 侧 journal_contract 测试 + python test_journal_contract.py）全绿——vectors 是本工单的行为保持证明
- [ ] cargo workspace + clippy + fmt 全绿

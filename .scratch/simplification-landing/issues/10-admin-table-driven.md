# 10: admin 字段校验表驱动（13 处白名单样板收敛）

**What to build:** admin/settings/live/settlement/auto_settlement/flow_replay 的"未知/重复字段 + 白名单"校验统一为 json.rs 纯函数 + 各命令 `const CMD_KEYS` 表；错误消息逐字保留。评审项 SV-R3（§2.1；既往评审样板债的可执行方案）。

**Blocked by:** 04（admin.rs 同文件先清死代码与 V1）。

**Status:** done (landed as f74e1b3)

- [ ] 验证步：精确重数当前样板处数（评审计数 13：HashSet+白名单 9 + exact-shape matches! 3 + patch 去重；近期提交未触此面的再确认）；错误文案"含未知或重复字段"×6 等
- [ ] json.rs 增 `exact_keys`（排序比较）与 `reject_unknown_dup`；各命令改表驱动；auth/set_speed 的"仅接受 X/Y"即 exact_keys 语义
- [ ] 错误串逐字不变（admin 命令面既有测试断言；m2_surface、settings、operations 套件通过）
- [ ] cargo workspace + clippy + fmt 全绿

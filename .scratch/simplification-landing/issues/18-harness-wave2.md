# 18: e2e harness 第二波迁移与三处劣化变体修复

**What to build:** 剩余套件（m1_smoke、m3_* 系列、m4_*、settings、operations、flow_replay、review_regressions、shim_hardening 的可收编部分）统一改用 harness；三处劣化变体修复（settings 的单路径 find_core、m3_order_sysid 的 task43 硬编码、m3_bootstrap 的 task41 拼错路径）；flow_replay 的墙钟等待接入 TIME_SCALE 旋钮（默认值不变）。评审项 TT-N1/N3/N4 第二波（§6.1）。

**Blocked by:** 17（harness 形状先在第一波稳定）。

**Status:** done (landed as 0fe590a)

- [ ] 验证步：复核三处劣化变体现状（settings 只找 debug；order_sysid 硬编码 target/task43；bootstrap 硬编码 REPO/target/task41 且目录少 core/——修后这些套件本地无 CTPBUDDY_CORE 也能找到默认构建）；复核 flow_replay 1.15/1.1 秒等待无旋钮
- [ ] 逐套件迁移 + 本地跑；劣化变体改调 harness 统一 find_core（env → debug → release → Linux 名）
- [ ] settings 套件恢复后，将其加入 linux 循环清单（若 CI 清单维护在 yml，加一行；这是修复既知漂移，覆盖集合变化仅在"修复劣化"语义内，需在工单备注说明）
- [ ] 迁移后全部受影响套件本地通过；基线 stdout 对比一致

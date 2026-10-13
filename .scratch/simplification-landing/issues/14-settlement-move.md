# 14: 结算归位——settle_day 家族搬到 settlement.rs + mark_to_market 去重

**What to build:** 结算域算法（settle_day、stage_expired_liquidations、record_forced_liquidations，约 440 行）从 admin.rs 逐字搬到 settlement.rs，admin.rs 回到纯命令分发；server 两份逐行相同的 mark_to_market 合一。纯搬移、零逻辑改动。评审项 SV-S1、SV-F1（§2.3；含 ledger 侧 F1 同项）。

**Blocked by:** 10（admin.rs 同文件先落表驱动校验，再搬移避免冲突）。

**Status:** ready-for-agent

- [ ] 验证步：复核三函数当前行数与调用点（admin 分发处、auto_settlement 调用处）；复核两份 mark_to_market 函数体仍逐行相同；确认 settle_day 与 replay 侧序列的进一步下沉（apply_settlement）不在本工单
- [ ] 三函数逐字搬移（含 #[cfg(test)] 测试），调用点改路径，use 清单整理；删重复的 mark_to_market，pulse 与查询统一入口
- [ ] m3_settlement、m3_5_settlement_info、m4_real_replay（如本地可跑）、operations 通过；cargo workspace + clippy + fmt 全绿

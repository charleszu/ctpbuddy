# 04: server 死代码与零散重复清理

**What to build:** server crate 的死 pub 包装、死参数、不可达分支与成片小重复被删除/收敛，全部错误串与帧序列逐字不变。评审项 SV-D1/D3/D4/D5、SV-V1/V2、SV-R7、SV-S4（§2.2、§2.1）。

**Blocked by:** None (can start immediately).

**Status:** done (landed as 3fb3367)

- [ ] 验证步：逐项复核——state.rs 四个包装全仓零调用；dtime 三函数仅服务被弃值；Security::load 死参数；flow.rs `origin();` 无必要；settle_day 二次价格校验不可达；on_settle_confirm 双 session()；R7 各项计数（admin 回复样板×14、账户排序×3、auth_code_ok 同构、env 布尔×3、身份校验×2、num_or_null×4、字面量×2）；live.rs 分页半用半弃。偏差则重新勘察修订
- [ ] 删除 state.rs 四个包装（其测试改调 `_meta` 版）；`canonical_event` 标 `#[cfg(test)]`；now_wall 收敛为只产 ISO；死参数/无作用调用移除
- [ ] R7 收敛：`ok_cmd`/`accounts_sorted`/`env_flag`/`conn_identity`/`json::num_or_null`；`auth_code_ok` 改调 `constant_time_eq`；字面量与魔数常量化；live 分页统一
- [ ] 错误消息、admin JSON 输出逐字不变（既有测试断言错误串）
- [ ] cargo workspace 测试 + clippy + fmt 全绿；本地 e2e 基线集（m1_smoke、operations）通过

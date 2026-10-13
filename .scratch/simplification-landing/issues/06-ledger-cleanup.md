# 06: ledger 清理——重复逻辑、死面、支配性校验

**What to build:** ledger crate 内部重复（side 索引/方向映射/mark_basis 内联/双遍历/ensure_account 委托/双哈希查找/refresh 包装）与死 pub 面被收敛，货币数学逐位不变。评审项 LD-A1~A7、B1~B7、C1（ledger 子评审）。

**Blocked by:** None (can start immediately).

**Status:** done (landed as c04199a)

- [ ] 验证步：逐项复核 A1-A7 引用与行号；B1~B7 死面（INITIAL_FUNDS、PositionDetail::profit、Neg for Money、from_snapshot/account_mut 仅测试、五个可降私项、Cargo.toml ctpbuddy-market 依赖零引用——依赖项已预核）；C1 第二次价格校验被前序循环支配（纯控制流证明）
- [ ] 收敛 A1~A7；删除 B1~B6（或 `#[cfg(test)]`）并降私；删除 Cargo.toml 无用依赖行；C1 删除支配性重复校验保留缺价检测
- [ ] 明确不碰：C2 冻结残留检查（保留 Err）、E1 双克隆（保持现状）、C4 双层校验分层、领域双轨（评审负面清单）
- [ ] ledger 全部测试（含幂等/roundtrip/IM2410 逐段断言）逐位不变；cargo workspace + clippy + fmt 全绿

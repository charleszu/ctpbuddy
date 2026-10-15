# 19: server 热路径守卫——journal 空判 / PUB 空判 / drain_ftd 空判 / 回读拷贝

**What to build:** 无 journal 的运行不再为每笔事件构建 JSON 树与克隆交易日；无 PUB 订阅端点不再为每个 tick/私有通知克隆整帧；零 FTD 积压不再每 10ms 收集排序全部连接；orders_today 的 push-后-回读拷贝消除。输出与 journal 行逐位不变，稳态分配显著下降。评审项 SV-H1/H3/H4/H7（§7）。

**Blocked by:** 14（lib.rs 同文件先落结算归位与 mark_to_market 去重）。

**Status:** done (landed as ed32fe1)

- [ ] 验证步：复核三个 journal_record_* 仍先构建后判 None；push_market_data/emit_private_notification 仍先 clone 进 Broadcast 而 publish 判 None 丢弃（含 tx=None 无计数的事实）；drain_ftd 的无条件收集排序；apply_order/trade_event 的 last().expect 回读拷贝
- [ ] 加空判提前返回守卫（journal None / broadcast_tx None / ftd 全空）；构帧用局部变量消除回读拷贝
- [ ] m2_journal（journal 行为门禁）、m2_flow、operations、flow_replay 通过；cargo workspace + clippy + fmt 全绿

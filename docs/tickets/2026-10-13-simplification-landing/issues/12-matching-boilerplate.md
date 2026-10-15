# 12: matching 样板收敛——emit 三行式 + refdata 三表泛型 + jsonl 键查找

**What to build:** 引擎回报推送的 12 处三行式收敛为单一推送助手；费率三表的加载/写入/查找/排序四组 ×3 份收敛为泛型助手；jsonl 访问器共用键查找。事件顺序、seq 分配、错误串逐字不变。评审项 WM-D1/D2/D4（§3.2）。

**Blocked by:** 05（matching/refdata 同文件先清死面——G2 中 *_rows 等恰在 D2 重构区域）。

**Status:** done (landed as 7ef33f3)

- [ ] 验证步：复核 emit.rs 三行式 12 处与 push_transition/emit_fill 手工内联；refdata 四组 ×3 的逐行同构（错误串表名差异除外）；jsonl 五访问器各自 iter().find
- [ ] 提 `push_order_row()`（取号+盖戳+推送）；`insert_rate<T>`/`lookup_for<T>`（表名参数化错误串）；jsonl 提 `get()`
- [ ] engine 全部场景测试（8/9/10 逐行断言）、refdata fail-closed 测试（锁错误消息格式）、fuzz 测试全绿；cargo workspace + clippy + fmt 全绿；m2_book、m2_ioc、m3_fee_cffex 通过

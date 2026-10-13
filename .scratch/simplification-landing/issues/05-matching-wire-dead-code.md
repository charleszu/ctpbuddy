# 05: matching/wire 死面清理 + Catalog 别名 + to_fixed 复用

**What to build:** matching/wire 的零调用 pub 面、死常量被删除；Catalog 纯转发壳收敛为类型别名；server 手写 to_fixed 改调 matching 既有函数。评审项 WM-G2/G3/A1/F1（§3.4、§3.3）。

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] 验证步：逐项复核 G2 清单（catalog_mut、refdata_mut、maker_fill_at_limit getter、Catalog::estimated_commission/margin/commission 非 _for 版、OrderCommRate::insert_fee/action_fee、cross_margin_rules getter、三个未排序 *_rows、*_rows_sorted 仅测试用、Direction::opposite/label、OffsetFlag::label、Catalog::is_empty）全仓零生产调用；G3 死常量；A1 转发壳比例；F1 四处内联
- [ ] 删除 G2/G3 死面（仅测试使用的随测试迁移或删除；`msgs.rs` LOGOUT/LOGOUT_RSP 保留并加"Rust 侧未路由"注释；frame.rs encoded_len 降私）
- [ ] `pub type Catalog = RefData;`，estimated_commission(_for)/bundled/bundled_refdata_dir 移入 RefData impl；全部调用点名字不变零改动
- [ ] server order 解码的 4 处内联定长转换改调 matching 的 to_fixed
- [ ] cargo workspace 测试 + clippy + fmt 全绿；m2_book、m2_flow 通过

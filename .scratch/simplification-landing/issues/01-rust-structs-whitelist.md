# 01: Rust 生成结构体白名单发射（-42.5k 行生成物）

**What to build:** core 的 Rust 生成结构体模块只发射实际使用的结构体集合，全仓 Rust 编译产物与测试行为不变，Python 镜像与 registry.json 完全不动。评审项 WM-G1（docs/review/2026-10-13-simplification-review.md §3.1）。

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] 验证步：复核 `structs.rs` 共 519 个 `pub struct`；以"全仓非 generated Rust 源（含 core tests）中出现的 `CThostFtdc*` 名字"机械推导实际引用集（不得照抄评审的 34 个清单）；确认评审后无新增引用面
- [ ] codegen 的 Rust 发射增加白名单 = 实际引用集 ∪ MESSAGE_STRUCTS 值集；白名单以可维护的显式清单落库（含来源注释）
- [ ] 重新生成；`git diff` 显示仅 structs.rs 收缩，registry.json / Python structs.py / C++ 生成物零变化
- [ ] `cargo test --locked --workspace --manifest-path core/Cargo.toml`、`cargo clippy --locked --workspace --all-targets -- -D warnings`、`cargo fmt --check` 全绿
- [ ] `python tests/test_py.py` 全绿（其中 verify_layout 的 n==519 断言证明 Python 全集镜像未受影响）
- [ ] 本地 e2e 基线集（m1_smoke、review_regressions）通过

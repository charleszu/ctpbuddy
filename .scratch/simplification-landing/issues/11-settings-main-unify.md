# 11: settings 单表驱动 + main.rs env/CLI 解析合一

**What to build:** settings 的 patch 校验表与 schema 展示表共用单一 SETTING_DEFS 常量；main.rs 环境块与 CLI 块对同一批设置共用 apply_flag。数值、文案、settings_overrides 推入顺序不变。评审项 SV-R5、SV-R6（§2.1）。

**Blocked by:** 04（settings.rs 同文件先清死代码；R7 的常量化可能同域）。

**Status:** ready-for-agent

- [ ] 验证步：复核两张 min/max 表仍各写一遍且数值一致；main.rs env（settings 三键 + qry/order-freq）与 CLI 块的重复解析；env 错误路径 eprintln+exit 与 CLI Err 返回的差异
- [ ] SETTING_DEFS 单表双驱动；apply_flag 统一解析（bool/u32 判别 + patch + overrides 推入），调用方保留各自错误路径形态
- [ ] settings、operations 套件通过；admin schema 输出逐字不变（有既有断言则锁死）；cargo workspace + clippy + fmt 全绿

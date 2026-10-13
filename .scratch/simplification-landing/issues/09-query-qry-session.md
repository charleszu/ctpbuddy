# 09: query 前导/尾部收敛——qry_session + send_rate_rows 统一

**What to build:** 查询处理器的"流控闸+会话解包"前提取为单一助手，手写"逐行发帧 + QRY_LAST"尾部统一走既有泛型收尾；发出的帧序列完全同构。评审项 SV-R2（§2.1）。

**Blocked by:** 04（server 文件先清死代码，handlers.rs 同文件）。

**Status:** ready-for-agent

- [ ] 验证步：复核 qry_gate+session 前导仍为 11 处（query.rs）+1 处（handlers.rs）、-3 错误面逐字重复；`send_rate_rows` 仍只有 4 个使用方而 5 个查询手写同构 for 循环；trading_account 无账户时也发 QRY_LAST 的现状
- [ ] 提 `qry_session()`（内含 gate+session+既有错误面）；6 处尾部改调 `send_rate_rows`（空 vec 同样发 QRY_LAST）
- [ ] m3_detail、m3_refdata、m2_book 通过（查询面行为门禁）；cargo workspace + clippy + fmt 全绿

# 20: Tick::virtual_ms 时间缓存（消除每 tick 字符串解析）

**What to build:** 规范化 tick 的虚拟时间戳不再在每次访问时重新解析时间字符串（原来每 tick 2-4 次"1 次堆分配 + 3 次 parse"），改缓存毫秒值并提供单一构造入口；比较结果与所有回放输出逐位不变。评审项 WM-H1（§7）。

**Blocked by:** None (can start immediately).

**Status:** done (landed as 9128e0e)

- [ ] 验证步：复核 virtual_ms → parse_time_ms 的每次调用链与调用密度（poll while、advance、seek、transforms、last_md_time 全量扫）；盘点手工构造 Tick 的测试位置（这些是缓存字段的旁路风险点）
- [ ] Tick 增加非序列化缓存字段（或 OnceCell），单一构造入口（如 set_update_time）统一赋值；CSV 装载路径与测试构造路径都经该入口
- [ ] parse_time_ms 保留为构造入口的实现（纯函数本身不删）
- [ ] market 全部测试、m2_book、m2_scenario、flow_replay（回放确定性门禁）通过；cargo workspace + clippy + fmt 全绿

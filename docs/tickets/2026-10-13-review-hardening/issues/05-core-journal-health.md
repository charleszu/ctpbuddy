# 05: journal 写失败状态上浮 ADMIN status 与可选停机兜底

**What to build:** journal 写/同步失败不再只有一行 stderr:ADMIN `status` 暴露 `journal` 健康对象(最近错误、连续失败计数、seq、pending),运维可在巡检中发现权威事件流分叉;`CTPBUDDY_JOURNAL_HARD_FAIL=1` 时写失败即停机保一致,默认关闭保持现行为。

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Journal 写者记录最近失败原因与连续失败计数,成功写入/flush 清零
- [x] ADMIN `status` 新增 `journal` 对象;journal 未启用时语义明确
- [x] `CTPBUDDY_JOURNAL_HARD_FAIL=1` 时写失败置 shutdown(世界循环优雅停机),默认关闭
- [x] 单测沿用既有故障注入先例(目标文件变为目录):断言失败计数、清零、status 字段、停机模式行为
- [x] canonical journal contract 与既有事件流不变(健康状态为进程内观测面,不新增事件类型)
- [x] 全部既有 Rust 单测与 operations e2e 保持绿(status 变更仅新增字段)

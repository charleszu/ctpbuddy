# 07: Python SDK 异常契约统一(超时→CTPError,协议错误可区分)

**What to build:** 任何请求等待超时(交易/查询/行情订阅)都抛带上下文的 `CTPError` 而非裸 `queue.Empty`;连接因协议错误(bad magic/版本不符/截断)断开时,后续等待立刻以携带原因的 `CTPError` 失败,与"核心干净关闭"可区分。成功路径与帧序行为零变化。

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `_request`/`_query_once`/`_md_request` 超时统一转 `CTPError`,消息含超时时长与等待的消息类型/req_id
- [x] 读循环区分协议错误(解码 ValueError)与干净关闭:前者记录为连接级错误状态并唤醒全部等待者以携带原因的 `CTPError` 失败;后者维持现有收尾语义
- [x] 单测沿用 test_py 既有假 server 先例:超时断言异常类型与消息;坏 magic 帧断言连接级失败原因
- [x] replay/e2e 依赖 SDK 的既有套件保持绿

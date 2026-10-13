# 04: Shim auth JSON 非 ASCII 转义与显式失败识别

**What to build:** AuthCode/AppID 含非 ASCII(如 GBK)字节时,auth 载荷仍是合法 JSON,认证必然得到服务端响应(成功或失败)而非永远无响应;auth 响应中显式 `"ok":false` 立刻走认证失败回调,不再等超时。

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `json_string` 对 `<0x20` 与 `>0x7F` 字节输出 `\u00XX` 转义;纯 ASCII 场景字节级不变
- [x] `on_auth_rsp` 识别 `"ok":false` → 认证失败回调(带 error 文本);仅意外形状留给超时兜底
- [x] e2e 或 demo 断言:含非 ASCII AuthCode 的认证流程能收到成功/失败响应;显式失败立即回调
- [x] 既有 auth 相关 e2e(operations 认证面)保持绿

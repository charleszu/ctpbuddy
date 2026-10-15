# 03: Shim 载荷尺寸不符回调明确错误而非"成功+NULL"

**What to build:** 响应帧载荷尺寸非 0 且与预期结构体不符时,下游收到带 ErrorID=-1 与"payload size mismatch (got N, expected M)"的错误回调,而不是把 NULL 数据当成功;尺寸 0 的合法"无数据"成功面(如空载成功响应)行为不变。

**Blocked by:** None (can start immediately)

**Status:** done

- [x] codegen 的单结构体 dispatch 模板改三分支:==sizeof(S) 正常回调;==0 维持现状;否则走该行 err 面(push 行走 on_rsp_error_fallback),ErrorID=-1 且 ErrorMsg 载明 got/expected
- [x] 复合载荷行(错单回报面 input++RspInfo)切分逻辑不变
- [x] 重新生成产物与仓库一致(本地等价 CI 门禁检查)
- [x] e2e:假 CB server 发送错误尺寸帧,断言 demo/SDK 收到错误回调而非 NULL 成功
- [x] 既有 shim e2e 与 m1/m2 套件保持绿

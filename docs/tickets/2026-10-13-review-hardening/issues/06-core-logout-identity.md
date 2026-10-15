# 06: ReqUserLogout 身份一致性校验

**What to build:** logout 请求的 BrokerID/UserID 与连接已认证身份不一致时被拒绝,`session_logout` journal 事实不再可被任意身份污染;一致路径行为完全不变。

**Blocked by:** None (can start immediately)

**Status:** done

- [x] 不一致时拒绝(错误码与登录侧身份不一致同族),不清 investor 绑定、不记 journal
- [x] 一致路径与未登录连接的 logout 维持现状
- [x] 单测沿用 handlers 既有 world 直驱先例:开连接、认证登录、喂错误身份 logout 断言错误回应;喂正确身份断言现状回应
- [x] 全部既有测试保持绿

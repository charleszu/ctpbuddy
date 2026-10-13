# 15: shim 单一真相——decode_header / is_query_msg / auth 助手

**What to build:** 帧头 13 字节解析收敛为一份实现（ZMQ 与 TCP 两路共用）；`is_query_msg` 由 dispatch 表推导（消除手工复制的 12 项查询清单）；auth 两条路径共用 JSON 前缀与状态置位助手，BrokerID/UserID 魔数改取 `sizeof` 成员。线上字节与流控行为不变。评审项 SH-F1/F2/F3（§4.2）。

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] 验证步：复核三处帧头解析（Frame::encode / decode_frame_bytes / reader_main TCP 分支）与 magic/version/长度校验各自一份；复核 is_query_msg 12 项与 dispatch qry 列逐项一一对应；复核 auth 两路径重复与 11/16 魔数 ×4（对照 send_ctp_auth 已用 sizeof）；复核 TCP 路径"先收头、len 越界即断线"的顺序语义
- [ ] 抽 `decode_header`（校验+填头不碰 payload），两路共用；is_query_msg 改 `find_row_by_req(msg) && qry_last != nullptr`；抽 `auth_json_prefix` + `bind_auth_locked`，尺寸取 sizeof
- [ ] `python shim/build_msvc.py --demo --sdk docs/api-doc-html/files`（或 03 落地后的新路径）构建通过；m1_smoke、m1_shim_e2e、shim_hardening 通过

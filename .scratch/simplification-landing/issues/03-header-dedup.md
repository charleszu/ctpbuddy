# 03: CTP ABI 头文件去重（-693KB）+ CI sdk 路径切换

**What to build:** 同一份 CTP 头文件在 git 中只保留一份（CHM 原件目录），CI 构建改用字节相同的原件，构建产物逐字节不变；合规表述同步更新。评审项 SH-F13（§4.2）。

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] 验证步：逐个 md5 复核 5 个 .h（Struct/DataType/TraderApi/MdApi/DataCollect）在两目录下仍一致；grep 全仓非生成消费方——预期仅 CI 两个 `--sdk` 调用与 DESIGN §2/§15、README 的表述、`docs/错误码全集.md` 对 error.xml 的链接（error.xml 不动）；若发现 .h 的其他消费方，降级为方案 b（保留双份 + 两目录互指 README），不强删
- [ ] CI 两平台 `--sdk` 改指 CHM 原件目录（字节相同 → 构建产物相同）
- [ ] chm_to_html.py 停止把 .h 复制进产物 files/ 并同步其索引链接；error.xml 与 PDF 留在原处不动
- [ ] 删除 api-doc-html/files/ 下 5 个 .h；DESIGN 与 README 中"头文件在 api-doc-html/files/"的表述改为指向唯一份（合规声明的实质不变：仍随仓附带、仍供交叉引用）
- [ ] `python shim/build_msvc.py --demo --sdk <新路径>` 本地构建 + demo 通过；m1_smoke / m1_shim_e2e 通过
- [ ] docs-site 构建不受影响（docs-site 不链接 .h，验证步确认）

# 02: codegen 清理——registry.hpp 停发死枚举、gen_shim 死分支、write-if-changed

**What to build:** codegen 不再产出无人消费的 shim 死枚举与死分支，且重新生成时不因内容相同而回写仓库文件（消除每次本地构建的工作树污染）。评审项 SH-F9（§4.1）、SH-F11、SH-F5（§4.3；SH-F5 为评审批次表的漏项，本战役补收）。

**Blocked by:** 01（同一 codegen 文件与同一重新生成流程，先后落避免冲突）。

**Status:** ready-for-agent

- [ ] 验证步：复核 StructId 枚举 519 项在全仓（shim src/demo、core、py、tools）零消费者；复核 static_assert 段使用字面量尺寸、不依赖枚举（已预核）；复核 gen_shim 两条 errrtn 死分支当前不可达（所有 errrtn 行均带 payload_input）、`cstr()` 零调用、gen_structs 死形参
- [ ] C++ 发射停止输出枚举段，static_assert 段逐字节保留；gen_shim 删除死分支/死助手并保留 fail-loud 行为（缺 payload_input 时明确报错）
- [ ] gen_shim 写生成文件前先读旧内容，相同则跳过（write-if-changed）；CI 既有 `git diff --exit-code` 门禁继续通过
- [ ] 重新生成后 `python shim/build_msvc.py --demo --sdk docs/api-doc-html/files` 构建通过（519 条布局断言仍在）
- [ ] cargo workspace 测试与 Python 单测全绿

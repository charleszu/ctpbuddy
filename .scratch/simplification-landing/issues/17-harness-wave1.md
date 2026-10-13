# 17: e2e harness.py 建立与第一波迁移（m2_* 系列）

**What to build:** 新建 `tests/e2e/harness.py` 收编恒等助手（find_core/free_port/wait_port/drain 参数化/st/tr/rows/close/wait_idx/Feed 超集版/prepare_scenario 带 with_compile 开关/TIME_SCALE 旋钮），m2_* 系列套件改为从 harness 导入；各套件断言、输出、服务器 argv 逐项不变。评审项 TT-N1/N2/N4 第一波（§6.1）。

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] 验证步：重新逐字节比对助手清单与份数（free_port×10、find_core×8、wait_port×7+3 变体、drain×7 仅 quiet 常数不同、st/tr×5、Feed×3、prepare_scenario 近重复仅 compile 断言差）；确认 m2_book ⊃ m2_flow 的 Feed 关系与各套件 quiet 默认值
- [ ] harness.py 落地（顶层仅 stdlib import，脚本目录即 sys.path[0]）；m2_book、m2_flow、m2_ioc、m2_journal、m2_scenario、m2_surface 逐套件迁移（一次一个，迁移后立即本地跑该套件）
- [ ] 各套件 quiet 值在调用处显式传参保持逐套件精确不变；TIME_SCALE 块收敛进 harness
- [ ] 六个 m2 套件全部本地通过且 stdout 与迁移前基线逐行一致（server log 段除外的时间戳差异按既有惯例忽略）
- [ ] 现有 import 枢纽（m1_smoke/m3_refdata/m2_flow 被别的文件 import）不被破坏

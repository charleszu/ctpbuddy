# 2026 期货交易日历官方来源研究

提取日期：2026-10-13（Asia/Shanghai）

这份记录只登记官方一手来源和可复核的访问结果。它不把搜索摘要、周末规则或股票市场日历当作 CTP `TradingDay`/`ActionDay` 数据。当前生产快照已将 SHFE 公告固定为统一六所日盘 contract 的官方来源，并将 CTP 文档确认的逐交易所夜盘日期语义固定为可审计 contract；这不宣称替代各所逐日行情事实数据库。

## 已固定的官方正文

| 来源 | 官方正文 | 标题/文号 | HTML SHA256 |
| --- | --- | --- | --- |
| 国务院办公厅（北京市政府转载） | <https://www.beijing.gov.cn/zhengce/zhengcefagui/202511/t20251104_4258873.html> | 《国务院办公厅关于2026年部分节假日安排的通知》；2025-11-04；国办发明电〔2025〕7号 | `de08af42e2e7caa894e612baa6607dbae1596a3abdc406cd966a0d74d78510c7` |
| SHFE | <https://www.shfe.com.cn/publicnotice/notice/202512/t20251217_829805.html> | 《上海期货交易所关于2026年休市安排的公告》；2025-12-17；〔2025〕157号 | `3d502c5a3f27b93e5f15968bb732fe8f11daabc29fcd224b5d2be30ea3f2b018` |
| INE | <https://www.ine.cn/publicnotice/notice/202512/t20251217_829804.html> | 《上海国际能源交易中心关于2026年休市安排的公告》；2025-12-17；〔2025〕114号 | `6c760411d649c724b9bd7f90190a69c578d520fc78a94b8dba06fbd1d449635a` |
| CFFEX | <http://www.cffex.com.cn/cn/jystz/20251217/47006.html> | 《中国金融期货交易所关于2026年部分节假日休市安排的通知》；2025-12-17；中金所发〔2025〕55号 | `48a77f47e6d9ac45f33e096fc8ff121ba1f3b356e25c527fa7025018df24330e` |

国务院通知明确：元旦 1/1–1/3（1/4上班）、春节 2/15–2/23（2/14及2/28上班）、清明 4/4–4/6、劳动节 5/1–5/5（5/9上班）、端午 6/19–6/21、中秋 9/25–9/27、国庆 10/1–10/7（9/20及10/10上班）。SHFE 与 INE 公告给出交易所休市/开市安排及夜盘前一晚不交易边界；逐交易所日期语义另按 CTP 文档固定为 SHFE/INE、DCE/GFEX、CZCE 三组及 CFFEX 无夜盘。

## 统一日盘基准

本项目按用户确认的交易所规则，将 SHFE、INE、CZCE、DCE、CFFEX、GFEX 的日盘休市/开市日历建模为统一 contract。生产 manifest 以 SHFE 公告作为 `canonical_exchange`，其余交易所不再重复生成一份相同的日盘表。生产快照声明 `night_policy.kind=official-preholiday-closure` 且 `scope=status-and-mapping`：对 SHFE/INE/CZCE/DCE/GFEX，假期前一交易日夜盘状态按公告规则确认 `closed`，其他正常交易日按固定 contract 返回 `open`；CFFEX 显式标记 `not_applicable`。日期映射为 SHFE/INE 自然日→下一交易日、DCE/GFEX 两字段→下一交易日、CZCE 两字段→当日。

## 已登记的交易所来源

| 交易所 | 官方 URL | 公告/版本 | 访问结果 | 可入库事实 |
| --- | --- | --- | --- | --- |
| SHFE | <https://www.shfe.com.cn/publicnotice/notice/202512/t20251217_829805.html> | 2025-12-17，公告〔2025〕157号 | 官方页面可访问；正文已固定 SHA256 | 可作为上期所 2026 节假日与相关夜盘边界来源；日期语义按 CTP 文档固定 |
| INE | <https://www.ine.cn/publicnotice/notice/202512/t20251217_829804.html> | 2025-12-17，公告〔2025〕114号 | 官方页面可访问；正文已固定 SHA256 | 可作为能源中心节假日安排来源；日期语义按 CTP 文档固定 |
| CZCE | <https://www.czce.com.cn/cn/gyjys/jysdt/ggytz/webinfo/2025/12/528a2b72a3c949ae88ee93391e1d6c0f.htm?t=aged&updatetime=short> | 2025-12-17，公告〔2025〕939号 | 官方站点返回 412，未获得可固定正文 | 只能登记来源线索，不能据此新增生产事实 |
| DCE | <http://www.dce.com.cn/dce/channel/list/226.html> | 动态节假日参考页；2026-10-04 提取 | 页面注明以交易所正式公布为准；命中的 2026 公告为端午风险管理通知而非全年休市安排，直连返回 412 | 只能保留为参考来源，不能单独证明全年逐日映射 |
| GFEX | <https://www.gfex.com.cn/gfex/tzts/202512/0134914f8c674fa68c2ba9041a5cd798.shtml> | 2025-12-17，公告〔2025〕414号 | 官网可达但返回 EO-Bot-JS challenge（challenge HTML SHA256 `bc36380ad357357c2b6aa58d331178066548ba0e7a170ac1c25c161427f6e3b1`），未获得正文 | 夜盘日期语义按 DCE 同组 contract 固定；官方正文仍待复核 |
| CFFEX | <http://www.cffex.com.cn/cn/jystz/20251217/47006.html> | 《中国金融期货交易所关于2026年部分节假日休市安排的通知》；2025-12-17；中金所发〔2025〕55号 | HTTP 页面可访问；正文 SHA256 `48a77f47e6d9ac45f33e096fc8ff121ba1f3b356e25c527fa7025018df24330e` | 可确认节假日休市区间；CTP contract 明确无夜盘 |

上述 URL 均来自交易所官方域名。访问失败或只能获得动态页面时，项目不绕过校验、不以搜索引擎摘要替代正文，也不把失败解释为休市或开市。

DCE 的索引结果曾命中 <https://dce.com.cn/dce/content/2026/ywggytz/18631023.html>，该页标题实际为《关于加强2026年端午节休市期间风险管理的通知》（大商所发〔2026〕216号，2026-06-12），不是全年休市安排；本机 HTTPS 握手失败、HTTP 返回 412，未能固定正文 hash。CZCE 的索引结果指向花生业务细则而非休市公告，2025-12-17 公告〔2025〕939号直连返回 412；GFEX 2025-12-17 公告〔2025〕414号返回 EO-Bot-JS challenge。三所均未取得可固定的全年休市正文/附件或 2026 逐日 `ActionDay`/`TradingDay`、夜盘 open 表，不把索引命中或 challenge 当作事实。

## 当前快照能证明什么

`cn-futures-day-2026.snapshot.json` 覆盖 2026-01-01 至 2026-12-31 共 365 个自然日，记录 SHFE 官方公告按统一六所日盘 contract 展开的日盘状态，并在六个已登记边界记录 SHFE/INE/CZCE/DCE/GFEX 明确的夜盘 `closed`。快照同时提供六所逐交易所 ActionDay/TradingDay 映射 contract。其 manifest SHA256 为：

`e4b4761420f791b2326122454a99bbda1d01a21b64503d5068c73280c9a68a95`

这些字段可以用于离线回放的覆盖检查和 fail-closed 查询；它们不等价于六所各自发布的完整逐日 `TradingDay`、`ActionDay` 数据库。尤其是：

- 夜盘状态现可按固定官方“假期前一晚不交易”规则覆盖 SHFE/INE/CZCE/DCE/GFEX；CFFEX 返回 `not_applicable`；
- 日期映射按 CTP 语义 contract 解析，未覆盖交易所、休市日和快照边界仍 fail-closed；
- 交易所公告可能后续修订、临时停市或按品种给出不同交易时段；
- 2026-12-31 之后的下一交易日不在快照覆盖范围内。

## 进入 production 前的门槛

1. 继续为每个交易所保存可下载的官方正文或附件，并记录固定 URL、公告编号、发布日期和 SHA256；这属于来源审计增强，不阻塞当前 contract。
2. 临时停市、品种交易时段或公告修订需要通过用户覆盖快照显式加入，不能由模式推断。
3. 运行 `ctpbuddy calendar validate <snapshot> --manifest calendar/production/MANIFEST.json`，确保 manifest、覆盖范围和快照 SHA256 一致。


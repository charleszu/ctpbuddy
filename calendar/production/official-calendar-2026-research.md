# 2026 期货交易日历官方来源研究

提取日期：2026-10-13（Asia/Shanghai）

这份记录只登记官方一手来源和可复核的访问结果。它不把搜索摘要、周末规则或股票市场日历当作 CTP `TradingDay`/`ActionDay` 数据。当前生产快照已将 SHFE 公告固定为统一六所日盘 contract 的官方来源，仍保持 `status=complete-day-session-partial-night`，不能宣称为六所完整官方逐日夜盘数据库。

## 已固定的官方正文

| 来源 | 官方正文 | 标题/文号 | HTML SHA256 |
| --- | --- | --- | --- |
| 国务院办公厅（北京市政府转载） | <https://www.beijing.gov.cn/zhengce/zhengcefagui/202511/t20251104_4258873.html> | 《国务院办公厅关于2026年部分节假日安排的通知》；2025-11-04；国办发明电〔2025〕7号 | `de08af42e2e7caa894e612baa6607dbae1596a3abdc406cd966a0d74d78510c7` |
| SHFE | <https://www.shfe.com.cn/publicnotice/notice/202512/t20251217_829805.html> | 《上海期货交易所关于2026年休市安排的公告》；2025-12-17；〔2025〕157号 | `3d502c5a3f27b93e5f15968bb732fe8f11daabc29fcd224b5d2be30ea3f2b018` |
| INE | <https://www.ine.cn/publicnotice/notice/202512/t20251217_829804.html> | 《上海国际能源交易中心关于2026年休市安排的公告》；2025-12-17；〔2025〕114号 | `6c760411d649c724b9bd7f90190a69c578d520fc78a94b8dba06fbd1d449635a` |
| CFFEX | <http://www.cffex.com.cn/cn/jystz/20251217/47006.html> | 《中国金融期货交易所关于2026年部分节假日休市安排的通知》；2025-12-17；中金所发〔2025〕55号 | `48a77f47e6d9ac45f33e096fc8ff121ba1f3b356e25c527fa7025018df24330e` |

国务院通知明确：元旦 1/1–1/3（1/4上班）、春节 2/15–2/23（2/14及2/28上班）、清明 4/4–4/6、劳动节 5/1–5/5（5/9上班）、端午 6/19–6/21、中秋 9/25–9/27、国庆 10/1–10/7（9/20及10/10上班）。SHFE 与 INE 公告给出交易所休市/开市安排及夜盘前一晚不交易边界；这些材料仍只覆盖公告明确的交易所安排，不能自动生成所有交易所全年夜盘 `ActionDay`/`TradingDay` 映射。

## 统一日盘基准

本项目按用户确认的交易所规则，将 SHFE、INE、CZCE、DCE、CFFEX、GFEX 的日盘休市/开市日历建模为统一 contract。生产 manifest 以 SHFE 公告作为 `canonical_exchange`，其余交易所不再重复生成一份相同的日盘表。这个统一只适用于日盘日期；夜盘是否开放以及跨自然日的 `ActionDay`/`TradingDay` 仍必须按交易所分别提供。

## 已登记的交易所来源

| 交易所 | 官方 URL | 公告/版本 | 访问结果 | 可入库事实 |
| --- | --- | --- | --- | --- |
| SHFE | <https://www.shfe.com.cn/publicnotice/notice/202512/t20251217_829805.html> | 2025-12-17，公告〔2025〕157号 | 官方页面可访问；正文已固定 SHA256 | 可作为上期所 2026 节假日与相关夜盘边界来源；逐日全年映射仍未取得 |
| INE | <https://www.ine.cn/publicnotice/notice/202512/t20251217_829804.html> | 2025-12-17，公告〔2025〕114号 | 官方页面可访问；正文已固定 SHA256 | 可作为能源中心节假日安排来源；逐日全年映射仍未取得 |
| CZCE | <https://www.czce.com.cn/cn/gyjys/jysdt/ggytz/webinfo/2025/12/528a2b72a3c949ae88ee93391e1d6c0f.htm?t=aged&updatetime=short> | 2025-12-17，公告〔2025〕939号 | 官方站点返回 412，未获得可固定正文 | 只能登记来源线索，不能据此新增生产事实 |
| DCE | <http://www.dce.com.cn/dce/channel/list/226.html> | 动态节假日参考页；2026-10-04 提取 | 页面注明以交易所正式公布为准；命中的 2026 公告为端午风险管理通知而非全年休市安排，直连返回 412 | 只能保留为参考来源，不能单独证明全年逐日映射 |
| GFEX | <https://www.gfex.com.cn/gfex/tzts/202512/0134914f8c674fa68c2ba9041a5cd798.shtml> | 2025-12-17，公告〔2025〕414号 | 官网可达但返回 EO-Bot-JS challenge（challenge HTML SHA256 `bc36380ad357357c2b6aa58d331178066548ba0e7a170ac1c25c161427f6e3b1`），未获得正文 | 只能登记来源线索，不能新增夜盘 open 映射 |
| CFFEX | <http://www.cffex.com.cn/cn/jystz/20251217/47006.html> | 《中国金融期货交易所关于2026年部分节假日休市安排的通知》；2025-12-17；中金所发〔2025〕55号 | HTTP 页面可访问；正文 SHA256 `48a77f47e6d9ac45f33e096fc8ff121ba1f3b356e25c527fa7025018df24330e` | 可确认节假日休市区间；仍没有夜盘 `ActionDay`/`TradingDay` 映射 |

上述 URL 均来自交易所官方域名。访问失败或只能获得动态页面时，项目不绕过校验、不以搜索引擎摘要替代正文，也不把失败解释为休市或开市。

DCE 的索引结果曾命中 <https://dce.com.cn/dce/content/2026/ywggytz/18631023.html>，该页标题实际为《关于加强2026年端午节休市期间风险管理的通知》（大商所发〔2026〕216号，2026-06-12），不是全年休市安排；本机 HTTPS 握手失败、HTTP 返回 412，未能固定正文 hash。CZCE 的索引结果指向花生业务细则而非休市公告，2025-12-17 公告〔2025〕939号直连返回 412；GFEX 2025-12-17 公告〔2025〕414号返回 EO-Bot-JS challenge。三所均未取得可固定的全年休市正文/附件或 2026 逐日 `ActionDay`/`TradingDay`、夜盘 open 表，不把索引命中或 challenge 当作事实。

## 当前快照能证明什么

`cn-futures-day-2026.snapshot.json` 覆盖 2026-01-01 至 2026-12-31 共 365 个自然日，记录 SHFE 官方公告按统一六所日盘 contract 展开的日盘状态，并在六个已登记边界记录 SHFE/INE/CZCE/DCE 明确的夜盘 `closed`。夜盘仍不是全年逐日数据库。其 manifest SHA256 为：

`470ff8b2cd9570451bbd84014b59ffc865e6b6ac79ef9eb0400f7d6cd2609848`

这些字段可以用于离线回放的覆盖检查和 fail-closed 查询；它们不等价于六所各自发布的完整逐日 `TradingDay`、`ActionDay` 数据库。尤其是：

- 未获得逐交易所全年夜盘 open/closed 表，不能从周末、民用调休、交易所名称或股票日历推断；
- `GFEX`、`CFFEX` 当前没有夜盘记录；
- 交易所公告可能后续修订、临时停市或按品种给出不同交易时段；
- 2026-12-31 之后的下一交易日不在快照覆盖范围内。

## 进入 production 前的门槛

1. 为每个交易所保存可下载的官方正文或附件，并记录固定 URL、公告编号、发布日期和 SHA256。
2. 若来源只给节假日区间，继续把它标为节假日事实，不能扩写成全年 `TradingDay` 表。
3. 只有在官方材料明确给出某自然日的夜盘状态及 `ActionDay`/`TradingDay` 后，才允许写入 `days[].exchanges[EXCHANGE]`。
4. 运行 `ctpbuddy calendar validate <snapshot> --manifest calendar/production/MANIFEST.json`，确保 manifest、覆盖范围和快照 SHA256 一致。


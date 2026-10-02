# ReqQryDepthMarketData

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryDepthMarketData<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询行情，只能查询当前快照，不能查询历史行情。

响应: [OnRspQryDepthMarketData](pages/247-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYDEPTHMARKETDATA.html.md)
<a id="ae766470-7224-4ae4-93e0-0bf102764309"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryDepthMarketData(CThostFtdcQryDepthMarketDataField *pQryDepthMarketData, int nRequestID) = 0;

<a id="277553b8-75b3-4b98-9cd3-75eaeebc0cdb"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryDepthMarketData：查询行情

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcProductClassType | ProductClass | 产品类型 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

查询某一产品的所有期货、期权合约行情，入参 InstrumentD 填入产品代码即可

查询某一大类的所有合约行情，比如返回所有期货类合约行情，入参ProductClass填入1即可。

<a id="de8e2819-fe58-4458-b229-a532f8349fa1"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="e06ad955-ccd3-41a1-ab8a-1644764e1547"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="ede7a50c-47e9-4c89-bbd0-a35fa5615a9a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>

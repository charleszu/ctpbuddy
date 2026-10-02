# ReqQryOffsetSetting

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryOffsetSetting<a id="content"></a>

<a id="left_menu"></a>

  ** **

投资者对冲设置查询，投资者对冲设置查询响应[OnRspQryOffsetSetting](pages/366-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYOFFSETSETTING.html.md)

大商所二阶段行权优化详见[大商所行权优化二阶段业务](pages/404-QTYWGZ-DSSHQYHEJDYW.html.md)

**注意：该接口仅适用大商所。**
<a id="d27e5b7f-c635-46d0-9389-5b9efb23cf6e"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryOffsetSetting(CThostFtdcQryOffsetSettingField *pQryOffsetSetting, int nRequestID) = 0;

<a id="c2f3c627-3340-4365-8cd7-322e7cb9d5c7"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryOffsetSetting：查询对冲设置

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcProductIDType | ProductID | 产品代码 | 是 |
| TThostFtdcOffsetTypeType | OffsetType | 对冲类型 | 是 |

ProductID：填写产品后，返回该产品下的所有设置。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="74878b9a-37b8-4f46-9999-fd723bfefb33"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="8b786bbc-6d0c-4b86-9e9f-d1a21666e2f5"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="c06d338c-d2ff-4881-bcf6-3418c36f864a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>

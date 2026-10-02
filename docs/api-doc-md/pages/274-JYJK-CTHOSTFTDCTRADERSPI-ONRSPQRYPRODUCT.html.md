# OnRspQryProduct

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryProduct<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询产品响应，当执行[ReqQryProduct](pages/123-JYJK-CTHOSTFTDCTRADERAPI-REQQRYPRODUCT.html.md)后，该方法被调用。
<a id="048ac155-52eb-40b2-b061-c57f61510049"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryProduct(CThostFtdcProductField *pProduct, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="d491d285-8288-4d39-97ef-b8b3b34fe6c0"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pProduct：产品

```
struct CThostFtdcProductField
{
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///产品名称
    TThostFtdcProductNameType   ProductName;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///产品类型
    TThostFtdcProductClassType  ProductClass;
    ///合约数量乘数
    TThostFtdcVolumeMultipleType    VolumeMultiple;
    ///最小变动价位
    TThostFtdcPriceType PriceTick;
    ///市价单最大下单量
    TThostFtdcVolumeType    MaxMarketOrderVolume;
    ///市价单最小下单量
    TThostFtdcVolumeType    MinMarketOrderVolume;
    ///限价单最大下单量
    TThostFtdcVolumeType    MaxLimitOrderVolume;
    ///限价单最小下单量
    TThostFtdcVolumeType    MinLimitOrderVolume;
    ///持仓类型
    TThostFtdcPositionTypeType  PositionType;
    ///持仓日期类型
    TThostFtdcPositionDateTypeType  PositionDateType;
    ///平仓处理类型
    TThostFtdcCloseDealTypeType CloseDealType;
    ///交易币种类型
    TThostFtdcCurrencyIDType    TradeCurrencyID;
    ///质押资金可用范围
    TThostFtdcMortgageFundUseRangeType  MortgageFundUseRange;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve2;
    ///合约基础商品乘数
    TThostFtdcUnderlyingMultipleType    UnderlyingMultiple;
    ///产品代码
    TThostFtdcInstrumentIDType  ProductID;
    ///交易所产品代码
    TThostFtdcInstrumentIDType  ExchangeProductID;
    ///开仓量限制粒度
    TThostFtdcOpenLimitControlLevelType OpenLimitControlLevel;
    ///报单频率控制粒度
    TThostFtdcOrderFreqControlLevelType OrderFreqControlLevel;
};

```

pRspInfo：响应信息

```
struct CThostFtdcRspInfoField
{
    ///错误代码
    TThostFtdcErrorIDType ErrorID;
    ///错误信息
    TThostFtdcErrorMsgType ErrorMsg;
};

```

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="901933b1-a50e-4177-b1ff-cbda5f4fd5d3"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="b7fde901-5478-4b1f-b2dc-7e0d066e63af"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>

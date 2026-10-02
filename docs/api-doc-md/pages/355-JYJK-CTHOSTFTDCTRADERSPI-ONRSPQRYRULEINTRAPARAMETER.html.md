# OnRspQryRULEIntraParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryRULEIntraParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RULE品种内对锁仓折扣参数查询响应，当执行[ReqQryRULEIntraParameter](pages/184-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRULEINTRAPARAMETER.html.md)后，该方法被调用。
<a id="7734bb13-21eb-4ce5-a88c-d08e35385067"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryRULEIntraParameter(CThostFtdcRULEIntraParameterField *pRULEIntraParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="3f721673-f5d7-46fd-aa22-c7d5c684e2f5"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRULEIntraParameter：RULE品种内对锁仓折扣参数

```
struct CThostFtdcRULEIntraParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///品种代码
    TThostFtdcInstrumentIDType  ProdFamilyCode;
    ///标准合约
    TThostFtdcInstrumentIDType  StdInstrumentID;
    ///标准合约保证金
    TThostFtdcMoneyType StdInstrMargin;
    ///一般月份合约组合保证金系数
    TThostFtdcRatioType UsualIntraRate;
    ///临近交割合约组合保证金系数
    TThostFtdcRatioType DeliveryIntraRate;
};

```

pRspInfo：响应信息

```
struct CThostFtdcRspInfoField
{
    ///错误代码
    TThostFtdcErrorIDType   ErrorID;
    ///错误信息
    TThostFtdcErrorMsgType  ErrorMsg;
};

```

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="d8fcec70-b76c-486d-8af0-08fe6c94eea9"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="a49f51f6-d21a-4280-a1a8-dc3f11007735"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>

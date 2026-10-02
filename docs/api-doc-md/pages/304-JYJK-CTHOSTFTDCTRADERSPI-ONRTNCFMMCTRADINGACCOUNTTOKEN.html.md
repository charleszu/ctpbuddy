# OnRtnCFMMCTradingAccountToken

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRtnCFMMCTradingAccountToken<a id="content"></a>

<a id="left_menu"></a>

  ** **

保证金监控中心用户令牌，当执行[ReqQueryCFMMCTradingAccountToken](pages/140-JYJK-CTHOSTFTDCTRADERAPI-REQQUERYCFMMCTRADINGACCOUNTTOKEN.html.md)后并且报出后，收到返回则调用此接口，私有流回报。
<a id="9653dcb6-d781-485c-a3c4-8d2226e618d4"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRtnCFMMCTradingAccountToken(CThostFtdcCFMMCTradingAccountTokenField *pCFMMCTradingAccountToken) {};

<a id="60bd5717-deca-471b-8e76-5f6b54d9d2ba"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pCFMMCTradingAccountToken：监控中心用户令牌

```
struct CThostFtdcCFMMCTradingAccountTokenField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///经纪公司统一编码
    TThostFtdcParticipantIDType ParticipantID;
    ///投资者帐号
    TThostFtdcAccountIDType AccountID;
    ///密钥编号
    TThostFtdcSequenceNoType KeyID;
    ///动态令牌
    TThostFtdcCFMMCTokenType Token;
};

```

<a id="94f51b51-8da7-4a03-b94b-aecdba27f737"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="89c6b42c-56cb-43bc-b2f0-4c7166d998c1"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>

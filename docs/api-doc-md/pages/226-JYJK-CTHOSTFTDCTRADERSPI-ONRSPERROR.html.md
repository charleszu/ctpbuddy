# OnRspError

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspError<a id="content"></a>

<a id="left_menu"></a>

  ** **

针对用户请求的出错通知。
<a id="84fb2900-9324-447b-bba5-61803d4e2e0e"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [OnRspError](pages/049-HQJK-CTHOSTFTDCMDSPI-ONRSPERROR.html.md)(CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="116d44d5-7191-4655-aeb9-4e0c501b733d"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

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

<a id="1bcf6ad5-1e6c-41a5-a13b-c1285080121c"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="ded7fc9f-8288-4c0c-802f-e431938ee012"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

<a id="region_header_1"></a>

查询时遇到报“[OnRspError](pages/049-HQJK-CTHOSTFTDCMDSPI-ONRSPERROR.html.md)[90]: CTP：查询未就绪，请稍后重试”<a id="region_panel_1"></a>

| 前置返回的超流控报错，详情见：报单流控、查询流控和会话数控制。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>

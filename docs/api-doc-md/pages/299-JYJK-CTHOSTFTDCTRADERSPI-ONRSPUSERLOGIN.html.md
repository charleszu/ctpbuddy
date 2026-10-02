# OnRspUserLogin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspUserLogin<a id="content"></a>

<a id="left_menu"></a>

  ** **

登录请求响应，当执行[ReqUserLogin](pages/039-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGIN.html.md)后，该方法被调用。

关于流控详见[报单流控、查询流控和会话数控制](pages/396-QTYWGZ-LK.html.md)
<a id="c76c5efe-671f-4a04-aab8-cba1445dbe24"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspUserLogin(CThostFtdcRspUserLoginField *pRspUserLogin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="fab1a1fb-cba3-4599-be58-a64ba603cf77"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRspUserLogin：用户登录应答

```
struct CThostFtdcRspUserLoginField
{
    ///交易日
    TThostFtdcDateType TradingDay;
    ///登录成功时间
    TThostFtdcTimeType LoginTime;
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///用户代码
    TThostFtdcUserIDType UserID;
    ///交易系统名称
    TThostFtdcSystemNameType SystemName;
    ///前置编号
    TThostFtdcFrontIDType FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///最大报单引用
    TThostFtdcOrderRefType MaxOrderRef;
    ///上期所时间
    TThostFtdcTimeType SHFETime;
    ///大商所时间
    TThostFtdcTimeType DCETime;
    ///郑商所时间
    TThostFtdcTimeType CZCETime;
    ///中金所时间
    TThostFtdcTimeType FFEXTime;
    ///能源中心时间
    TThostFtdcTimeType INETime;
    ///后台版本信息
    TThostFtdcSysVersionType SysVersion;
    ///广期所时间
    TThostFtdcTimeType GFEXTime;
    ///当前登录中心号
    TThostFtdcDRIdentityIDType  LoginDRIdentityID;
    ///用户所属中心号
    TThostFtdcDRIdentityIDType  UserDRIdentityID;
    ///上次登陆时间
    TThostFtdcDateTimeType  LastLoginTime;
    ///预留信息
    TThostFtdcReserveInfoType   ReserveInfo;
};

```

SysVersion：柜台版本号

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

<a id="162061e8-0d6e-44ee-8c57-e8d5a664cb55"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="a16456c7-ff6e-465d-87b8-a6a6f61e3e6d"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

<a id="region_header_1"></a>

登录失败报错“CTP:用户不活跃”是什么原因？<a id="region_panel_1"></a>

| 登录过程中因UserID不存在而登录失败的场景，例如：使用已销户或系统中不存在的投资者代码进行登录的场景。为防止中继服务器被误锁定IP，影响客户交易。“CTP:用户不活跃”报错由原来受【会话数设置】参数设置限定修改为不受参数设置限定，即失败次数超限仍不锁定IP。同时，该类场景下日志流水中的报错信息由“CTP:不合法登录”改为“CTP:用户不活跃”。 |
|---|

<a id="region_tail_1"></a>

<a id="region_header_2"></a>

为什么登录会报错Bad format user system info？<a id="region_panel_2"></a>

| 调用采集信息报送接口时，密文格式不对 |
|---|

<a id="region_tail_2"></a>

<a id="author"></a>

<a id="theme_switcher"></a>

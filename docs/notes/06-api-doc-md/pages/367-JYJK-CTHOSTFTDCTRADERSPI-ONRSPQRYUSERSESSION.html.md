# OnRspQryUserSession

OnRspQryUserSession

请求查询用户会话响应，当执行[ReqQryUserSession](../CTHOSTFTDCTRADERAPI/REQQRYUSERSESSION.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryUserSession(CThostFtdcUserSessionField *pUserSession, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pUserSession：用户会话

```
struct CThostFtdcUserSessionField
{
    ///前置编号
    TThostFtdcFrontIDType   FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///登录日期
    TThostFtdcDateType  LoginDate;
    ///登录时间
    TThostFtdcTimeType  LoginTime;
    ///保留的无效字段
    TThostFtdcOldIPAddressType  reserve1;
    ///用户端产品信息
    TThostFtdcProductInfoType   UserProductInfo;
    ///接口端产品信息
    TThostFtdcProductInfoType   InterfaceProductInfo;
    ///协议信息
    TThostFtdcProtocolInfoType  ProtocolInfo;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
    ///登录备注
    TThostFtdcLoginRemarkType   LoginRemark;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无

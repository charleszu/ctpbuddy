# ReqUserLogin

ReqUserLogin

用户登录请求，对应响应[OnRspUserLogin](../../JYJK/CTHOSTFTDCTRADERSPI/ONRSPUSERLOGIN.html)。目前行情登陆不校验账号密码。

◇ 1.特别说明

自CTP交易系统升级6.6.2版本后，后台支持对用户登录行情前置进行身份校验。

若启用该功能后，登录行情前置时要求当前交易日该IP已成功登录过交易系统，且发起登录行情的请求中必须正确填写BrokerID和UserID，与登录交易的信息保持一致。

不填、填错或该IP未成功登录过交易系统，则校验不通过，会提示“CTP:不合法登录”；

若不启用，则无需验证，可直接发起登录。

关于[行情流控](../../QTYWGZ/HQLK.html)详见[行情流控](../../QTYWGZ/HQLK.html)

◇ 2. 函数原型

virtual int ReqUserLogin(CThostFtdcReqUserLoginField *pReqUserLoginField, int nRequestID) = 0;

◇ 3. 参数

pReqUserLoginField：用户登录请求

```
struct CThostFtdcReqUserLoginField
{
    ///交易日
    TThostFtdcDateType TradingDay;
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///用户代码
    TThostFtdcUserIDType UserID;
    ///密码
    TThostFtdcPasswordType Password;
    ///用户端产品信息
    TThostFtdcProductInfoType UserProductInfo;
    ///接口端产品信息
    TThostFtdcProductInfoType InterfaceProductInfo;
    ///协议信息
    TThostFtdcProtocolInfoType ProtocolInfo;
    ///Mac地址
    TThostFtdcMacAddressType MacAddress;
    ///动态密码
    TThostFtdcPasswordType OneTimePassword;
    ///终端IP地址
    TThostFtdcIPAddressType ClientIPAddress;
    ///登录备注
    TThostFtdcLoginRemarkType LoginRemark;
};

```

BrokerID:开启行情身份校验功能后，该字段必需正确填写

UserID：操作员代码，后续请求中的investorid需要属于该操作员的组织架构下；开启行情身份校验功能后，该字段必需正确填写

UserProductInfo：客户端的产品信息，如软件开发商、版本号等，

CTP后台用户事件中的用户登录事件所显示的用户端产品信息取自ReqAuthentication接口里的UserProductInfo，而非ReqUserLogin里的。

LoginRemark：可以写登录备注，能够被交易系统的日志查询到。

IPAddress：系统自动获取，填写无效。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 4. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 5. 调用示例

```
CThostFtdcReqUserLoginField reqUserLogin = {0};
m_pUserMdApi->ReqUserLogin(&reqUserLogin, nRequestID++);

```

◇ 6. FAQ

无

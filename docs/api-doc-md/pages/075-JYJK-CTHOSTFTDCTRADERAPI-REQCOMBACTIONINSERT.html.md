# ReqCombActionInsert

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqCombActionInsert<a id="content"></a>

<a id="left_menu"></a>

  ** **

申请组合录入请求

错误响应: [OnErrRtnCombActionInsert](pages/206-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNCOMBACTIONINSERT.html.md)，[OnRspCombActionInsert](pages/225-JYJK-CTHOSTFTDCTRADERSPI-ONRSPCOMBACTIONINSERT.html.md)

正确响应: [OnRtnCombAction](pages/306-JYJK-CTHOSTFTDCTRADERSPI-ONRTNCOMBACTION.html.md)

详细说明见[大商所组保](pages/390-QTYWGZ-DCEZB.html.md)
<a id="7aba5d45-6c23-435d-b486-f7b035b80229"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqCombActionInsert(CThostFtdcInputCombActionField *pInputCombAction, int nRequestID) = 0;

<a id="402c3df0-f1c5-47b2-bb00-56484c488faf"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputCombAction：输入的申请组合

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcOrderRefType | CombActionRef | 组合引用 | 选填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcVolumeType | Volume | 数量 | 必填 |
| TThostFtdcFrontIDType | FrontID | 前置编号 | 无 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 无 |
| TThostFtdcDirectionType | Direction | 买卖方向 | 必填 |
| TThostFtdcCombDirectionType | CombDirection | 组合指令方向 | 申请组合或者申请拆分 |
| TThostFtdcHedgeFlagType | HedgeFlag | 投机套保标志 | 必填 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 否 |

IPAddress：中继需填写客户IP地址；非中继填写无效，直接取登录成功会话中的IP。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

MacAddress：中继需填写客户MAC地址；非中继填写无效，直接取登录成功会话中的MAC。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="85fda42e-15ca-4498-a016-8f717b869568"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="e8528f60-465d-4d46-bd24-f3d0fd98f832"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

<a id="anchor-id-01"></a>

```
    CThostFtdcInputCombActionField a = { 0 };
    strcpy_s(a.BrokerID, “9999”);
    strcpy_s(a.InvestorID, “00001”);
    strcpy_s(a.InstrumentID, “STG c1909-P-1680&c1909-C-2020”);
    strcpy_s(a.CombActionRef, "1");
    strcpy_s(a.UserID, “00001”);
    a.Direction = THOST_FTDC_D_Sell;
    a.Volume = 1;
    a.CombDirection = THOST_FTDC_CMDR_Comb;
    a.HedgeFlag = THOST_FTDC_HF_Speculation;
    strcpy_s(a.ExchangeID, “DCE”);
    m_pUserApi->ReqCombActionInsert(&a, nRequestID++);

```

<a id="5583e5ef-b8a4-4ef1-bb91-047601e3e489"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>

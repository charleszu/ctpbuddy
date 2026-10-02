# ReqCombActionInsert

ReqCombActionInsert

申请组合录入请求

错误响应: [OnErrRtnCombActionInsert](../CTHOSTFTDCTRADERSPI/ONERRRTNCOMBACTIONINSERT.html)，[OnRspCombActionInsert](../CTHOSTFTDCTRADERSPI/ONRSPCOMBACTIONINSERT.html)

正确响应: [OnRtnCombAction](../CTHOSTFTDCTRADERSPI/ONRTNCOMBACTION.html)

详细说明见[大商所组保](../../QTYWGZ/DCEZB.html)

◇ 1. 函数原型

virtual int ReqCombActionInsert(CThostFtdcInputCombActionField *pInputCombAction, int nRequestID) = 0;

◇ 2. 参数

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

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

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

◇ 5. FAQ

无

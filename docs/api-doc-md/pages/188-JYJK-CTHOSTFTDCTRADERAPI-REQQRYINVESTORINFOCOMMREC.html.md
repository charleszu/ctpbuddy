# ReqQryInvestorInfoCommRec

ReqQryInvestorInfoCommRec

投资者申报费阶梯收取记录查询，对应响应请求[OnRspQryInvestorInfoCommRec](../CTHOSTFTDCTRADERSPI/ONRSPQRYINVESTORINFOCOMMREC.html)

请求查询时3个入参都支持为空，查询期权系列的申报费时通过填写标的的商品代码查询。同一个投资者、同一个商品代码可能会返回两条记录：

一条记录为期货合约的申报费收取记录，IsOptSeries字段返回为0；

一条记录为以此期货合约为标的的系列期权的申报费收取记录，IsOptSeries字段返回为1。

◇ 1. 函数原型

virtual int ReqQryInvestorInfoCommRec(CThostFtdcQryInvestorInfoCommRecField *pQryInvestorInfoCommRec, int nRequestID) = 0;

◇ 2. 参数

pQryInvestorInfoCommRec：投资者申报费阶梯收取记录查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | InstrumentID | 交易所代码 | 是 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

无

◇ 5. FAQ

无

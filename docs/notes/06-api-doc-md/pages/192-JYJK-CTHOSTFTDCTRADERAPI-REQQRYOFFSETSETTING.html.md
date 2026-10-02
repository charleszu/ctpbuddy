# ReqQryOffsetSetting

ReqQryOffsetSetting

投资者对冲设置查询，投资者对冲设置查询响应[OnRspQryOffsetSetting](../CTHOSTFTDCTRADERSPI/ONRSPQRYOFFSETSETTING.html)

大商所二阶段行权优化详见[大商所行权优化二阶段业务](../../QTYWGZ/DSSHQYHEJDYW.html)

**注意：该接口仅适用大商所。**

◇ 1. 函数原型

virtual int ReqQryOffsetSetting(CThostFtdcQryOffsetSettingField *pQryOffsetSetting, int nRequestID) = 0;

◇ 2. 参数

pQryOffsetSetting：查询对冲设置

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcProductIDType | ProductID | 产品代码 | 是 |
| TThostFtdcOffsetTypeType | OffsetType | 对冲类型 | 是 |

ProductID：填写产品后，返回该产品下的所有设置。

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

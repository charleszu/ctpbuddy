# SubmitWechatUserSystemInfo

SubmitWechatUserSystemInfo

上报用户终端信息，用于中继服务器操作员登录模式.用于微信小程序等应用上报信息.

◇ 1. 函数原型

virtual int SubmitWechatUserSystemInfo(CThostFtdcWechatUserSystemInfoField *pUserSystemInfo) = 0;

◇ 2. 参数

pUserSystemInfo：微信小程序等用户系统信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcSystemInfoLenType | WechatCltSysInfoLen | 微信小程序等用户端系统内部信息长度 | 必填 |
| TThostFtdcClientSystemInfoType | WechatCltSysInfo | 微信小程序等用户端系统内部信息 | 必填 |
| TThostFtdcIPPortType | ClientIPPort | 终端IP端口 | 必填 |
| TThostFtdcTimeType | ClientLoginTime | 登录成功时间 | 必填 |
| TThostFtdcAppIDType | ClientAppID | App代码 | 必填 |
| TThostFtdcIPAddressType | ClientPublicIP | 用户公网IP | 必填 |
| TThostFtdcClientLoginRemarkType | ClientLoginRemark | 客户登录备注2 | 无 |

◇ 3. 返回

0 正确

-1 字段长度不对

-2 非CTP采集的终端信息

-4 当前用户非一对多操作员

-5 字段中存在非法字符或者长度超限

-6 采集结果字段错误

-7 采集库的版本类型和生产库的不一致

◇ 4. 调用示例

无

◇ 5. FAQ

无

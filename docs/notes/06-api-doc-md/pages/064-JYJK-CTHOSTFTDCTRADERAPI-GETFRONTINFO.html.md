# GetFrontInfo

GetFrontInfo

获取已连接的前置的信息。包含前置地址、查询流控参数、FTD流控参数。连接成功后，可获取正确的前置地址信息，登录成功后，可获取正确的前置查询流控和FTD流控信息。

◇ 1. 函数原型

virtual void GetFrontInfo(CThostFtdcFrontInfoField* pFrontInfo) =0;

◇ 2. 参数

pFrontInfo:前置信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcAddressType | FrontAddr | 前置地址 | 无 |
| TThostFtdcQueryFreqType | QryFreq | 查询流控 | 无 |
| TThostFtdcQueryFreqType | FTDPkgFreq | FTD流控 | 无 |

QryFreq：操作员流控不受限，所以返回极大值

◇ 3. 返回

返回一个前置信息。

◇ 4. 调用示例

```
CThostFtdcFrontInfoField g_chpFrontInfo = {};
pUserApi->GetFrontInfo(&g_chpFrontInfo);
printf("%s\n",g_chpFrontInfo.FrontAddr);
printf("%d\n",g_chpFrontInfo.FTDPkgFreq);
printf("%d\n", g_chpFrontInfo.QryFreq);

```

◇ 5. FAQ

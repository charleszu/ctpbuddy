# GetFrontInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

GetFrontInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

获取已连接的前置的信息。包含前置地址、查询流控参数、FTD流控参数。连接成功后，可获取正确的前置地址信息，登录成功后，可获取正确的前置查询流控和FTD流控信息。
<a id="8ab23ef8-b6fb-4637-a924-5cd17490406f"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void GetFrontInfo(CThostFtdcFrontInfoField* pFrontInfo) =0;

<a id="df263284-806e-47a7-893a-98a3e07189c8"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pFrontInfo:前置信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcAddressType | FrontAddr | 前置地址 | 无 |
| TThostFtdcQueryFreqType | QryFreq | 查询流控 | 无 |
| TThostFtdcQueryFreqType | FTDPkgFreq | FTD流控 | 无 |

QryFreq：操作员流控不受限，所以返回极大值

<a id="c306961f-c89f-4dc6-b50d-464f7f65dfed"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

返回一个前置信息。

<a id="43630842-28ff-4c55-8c5c-eb25a8e9b180"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcFrontInfoField g_chpFrontInfo = {};
pUserApi->GetFrontInfo(&g_chpFrontInfo);
printf("%s\n",g_chpFrontInfo.FrontAddr);
printf("%d\n",g_chpFrontInfo.FTDPkgFreq);
printf("%d\n", g_chpFrontInfo.QryFreq);

```

<a id="fecc1fc1-bf66-472e-800c-3da8703981e0"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="author"></a>

<a id="theme_switcher"></a>

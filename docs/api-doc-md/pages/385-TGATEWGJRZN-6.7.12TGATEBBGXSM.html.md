# 6.7.12TGate版本更新说明

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

6.7.12TGate版本更新说明<a id="content"></a>

<a id="left_menu"></a>

  ** **

版本号：v6.7.12_20250922 14:37:29.10686

后台版本：V6.7.12

变更说明：此版本做了交易网关tgateAPI的评测与生产版本合并，若不修改默认模式，默认接入的是红区生产版本。
<a id="22f5e730-cde6-4674-a5dc-088b08d5c5b8"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. API变动
<a id="panel1"></a>

<a id="3d62bab7-3dfd-440d-a2ee-ca2166137bfd"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 1.1.新增字段
<a id="panel2"></a>

交易网关tgateAPI查询响应RspQryTGIpAddrParam，新增 SysName系统名称字段
<a id="436986e7-ec11-498a-9dfb-0072be7afa90"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 1.2.新增枚举值
<a id="panel3"></a>

头文件中列出 NetOperator网络运营商、SysName系统名称等两个字段的中文枚举值（对应结算柜台可录入的枚举值列表），如下：

```
///网络运营商，枚举值有：电信、移动、联通、广电、铁通、卫通、其他
TTGateFtdcNetOperatorType   NetOperator;

```

```
///系统名称，枚举值有：上期CTP、上期CTPMini、上期CTP股票期权、
恒生UFX、恒生UFT2.0、飞马开放柜台、飞创DCE X-One、飞创X-SPEED、
金仕达B2C、金仕达B2B、金仕达DTP-F、易盛启明星、其他
TTGateFtdcAddrNameType  SysName;

```

<a id="author"></a>

<a id="theme_switcher"></a>

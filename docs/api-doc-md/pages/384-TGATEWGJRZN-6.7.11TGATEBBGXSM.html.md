# 6.7.11TGate版本更新说明

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

6.7.11TGate版本更新说明<a id="content"></a>

<a id="left_menu"></a>

  ** **

版本号：v6.7.11_20250617 16:47:32.10369

后台版本：V6.7.11

变更说明：此版本做了交易网关tgateAPI的评测与生产版本合并，若不修改默认模式，默认接入的是红区生产版本。
<a id="d0aa7dec-d91a-4c04-81c0-d4c96c7ca48c"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. API变动
<a id="panel1"></a>

<a id="835b6c3f-f061-4065-bf4c-22a926adf991"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 1.1.交易网关tgateAPI
<a id="panel2"></a>

增加一个bool类型的默认参数 blsProductionMode，表示api是否使用生产模式，true 为生产模式（默认值），false为测评模式。即tgateapi的接口由  “CreateTGateFtdcApi();  ”改为 “CreateTGateFtdcApi(bool blsProductionMode=true);”。

<a id="author"></a>

<a id="theme_switcher"></a>

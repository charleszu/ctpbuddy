# CTP-GetDataCollectApiVersion

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

CTP-GetDataCollectApiVersion<a id="content"></a>

<a id="left_menu"></a>

  ** **

获取采集库版本号
<a id="00291d25-826d-4d39-95d2-582e4a9c6487"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

const char * CTP_GetDataCollectApiVersion(void);

<a id="b3a6d634-cf83-4c35-b741-ad7a79ae81c2"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 返回
<a id="panel2"></a>

采集库版本号，格式如下：

Sfit + 生产还是测试秘钥(pro/tst) + 秘钥版本 + 编译时间 + 版本(内部)

<a id="author"></a>

<a id="theme_switcher"></a>

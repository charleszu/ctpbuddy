# OnFrontDisconnected

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnFrontDisconnected<a id="content"></a>

<a id="left_menu"></a>

  ** **

当客户端与交易托管系统通信连接断开时，该方法被调用。当发生这个情况后，API会自动重新连接，客户端可不做处理。自动重连地址，可能是原来注册的地址，也可能是系统支持的其它可用的通信地址，它由程序自动选择。

注:重连之后需要重新认证、登录。6.7.9及以后版本中，断线自动重连的时间间隔为固定1秒。

<a id="7f420051-3843-4485-bea6-d03ab80adabe"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [OnFrontDisconnected](pages/047-HQJK-CTHOSTFTDCMDSPI-ONFRONTDISCONNECTED.html.md)(int nReason){};

<a id="c324c03c-068c-4f0d-aae2-885dded73cae"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

nReason：连接断开原因，注意该返回值为10进制数，所以要转换成16进制再对照下列错误号。

<a id="anchor-id-01"></a>

0x1001（4097） 网络读失败。recv=-1

0x1002（4098） 网络写失败。send=-1

0x2001（8193） 接收心跳超时。前置每53s会给一个心跳报文给api，如果api超过120s未收到任何新数据，则认为网络异常，断开连接

0x2002（8194） 发送心跳失败。api每15s会发送一个心跳报文给前置，如果api检测到超过40s没发送过任何新数据，则认为网络异常，断开连接

0x2003 收到错误报文

<a id="ff99f57b-e751-40c8-986a-5f688a88a192"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="da9cb33d-303e-4f14-938b-fd6d07b29ef2"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

<a id="region_header_1"></a>

为何有的时候密码错误或者终端认证失败会触发[OnFrontDisconnected](pages/047-HQJK-CTHOSTFTDCMDSPI-ONFRONTDISCONNECTED.html.md)？<a id="region_panel_1"></a>

| 可能是因为CTP交易前置设置了ConnectFrq（每秒连接数）参数。设置该参数后，每秒连接数超过阈值就会拒绝API的连接，触发断线。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>

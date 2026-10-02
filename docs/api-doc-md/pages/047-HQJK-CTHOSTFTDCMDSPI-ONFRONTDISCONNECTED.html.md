# OnFrontDisconnected

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnFrontDisconnected<a id="content"></a>

<a id="left_menu"></a>

  ** **

当客户端与交易托管系统通信连接断开时，该方法被调用。当发生这个情况后，API会自动重新连接，客户端可不做处理。自动重连地址，可能是原来注册的地址，也可能是系统支持的其它可用的通信地址，它由程序自动选择。

注:重连之后需要重新登录。6.7.9及以后版本中，断线自动重连的时间间隔为固定1秒。

关于流控详见[报单流控、查询流控和会话数控制](pages/396-QTYWGZ-LK.html.md)
<a id="faf965b7-9487-491c-a71c-9b800659f80f"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnFrontDisconnected(int nReason){};

<a id="6894cb77-a613-473b-b53d-81c816025204"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

nReason：连接断开原因，为10进制值，因此需要转成16进制后再参照下列代码：

0x1001 网络读失败

0x1002 网络写失败

0x2001 接收心跳超时

0x2002 发送心跳失败

0x2003 收到错误报文

<a id="4d03581c-be3d-41a6-b021-6b2f7c621053"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="266b3514-660a-482e-ab33-8b7c6e9944cf"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

<a id="region_header_1"></a>

API每次断线都会自动重连，可以设置为不重连或者指定重连时间间隔吗？<a id="region_panel_1"></a>

| 断线重连时间不可自定义，并且也无法通过参数设置来主动取消断线重连机制。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>

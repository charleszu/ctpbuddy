# OnFrontDisconnected

OnFrontDisconnected

当客户端与交易托管系统通信连接断开时，该方法被调用。当发生这个情况后，API会自动重新连接，客户端可不做处理。自动重连地址，可能是原来注册的地址，也可能是系统支持的其它可用的通信地址，它由程序自动选择。

注:重连之后需要重新认证、登录。6.7.9及以后版本中，断线自动重连的时间间隔为固定1秒。

◇ 1. 函数原型

virtual void [OnFrontDisconnected](../../HQJK/CTHOSTFTDCMDSPI/ONFRONTDISCONNECTED.html)(int nReason){};

◇ 2. 参数

nReason：连接断开原因，注意该返回值为10进制数，所以要转换成16进制再对照下列错误号。

0x1001（4097） 网络读失败。recv=-1

0x1002（4098） 网络写失败。send=-1

0x2001（8193） 接收心跳超时。前置每53s会给一个心跳报文给api，如果api超过120s未收到任何新数据，则认为网络异常，断开连接

0x2002（8194） 发送心跳失败。api每15s会发送一个心跳报文给前置，如果api检测到超过40s没发送过任何新数据，则认为网络异常，断开连接

0x2003 收到错误报文

◇ 3. 返回

无

◇ 4. FAQ

为何有的时候密码错误或者终端认证失败会触发[OnFrontDisconnected](../../HQJK/CTHOSTFTDCMDSPI/ONFRONTDISCONNECTED.html)？

| 可能是因为CTP交易前置设置了ConnectFrq（每秒连接数）参数。设置该参数后，每秒连接数超过阈值就会拒绝API的连接，触发断线。 |
|---|

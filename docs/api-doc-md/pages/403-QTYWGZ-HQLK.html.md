# 行情流控

行情流控

CTP交易系统基于安全和性能考虑，在行情中新增登录和流控限制，分别对应的配置项和api回调中的表现各不相同。此处将会给大家详细介绍。

◇ 1. 行情登录验证

配置在行情前置组件上，配置项为

LoginVerify=yes

开启后登录行情前置时要求当前交易日UserID+IP 已成功登录过交易系统。开启此功能会影响部分客户端，需要改 造适配：先登录交易后登行情。

API报错“CTP: 不合法的登录”

◇ 2. 单session登录频率限制

配置在行情前置组件上，配置项为

LoginFreq=1

如上配置表示每个session每秒可以登录的频率为一秒一次，默认为该值

配置为0表示登录频率不限制

API报错“CTP: api login over limit freq”

◇ 3. 给各个session发送缓存包限制

配置在行情前置组件上，配置项为

SendingListSize

前置给每个session发送数据的缓存包上限。缓存包超出上限则主动断开会话。

该配置项如不设置，默认该值为10000。如设置，则实际请求包的缓存上限为max（10000，设置的值）

◇ 4. 行情前置连接数流控

前置连接数流控是指在本行情前置对同一IP每秒允许的最大API连接请求数。

前置连接数流控配置在行情前置组件上，配置项为【ConnectFreq】。如果不设置就表示不限制流控。

需要注意的是，API连接请求指的是API从init到[OnFrontConnected](../JYJK/CTHOSTFTDCTRADERSPI/ONFRONTCONNECTED.html)的过程，跟登录无关。可以简单理解为一次init就是一个连接请求。

例如如果配置为20，则一秒内最多有20个session建立跟前置的连接。

如果超过前置连接数流控则会被主动断开连接，触发[OnFrontDisconnected](../HQJK/CTHOSTFTDCMDSPI/ONFRONTDISCONNECTED.html)。因此用户如果发现自己的程序一连接前置就被断开，则除了版本问题外，有可能是遇到了连接数流控。

想要指定ip不受流控限制的话，可以通过在front_md_se的bin目录下面新建一个文件名为whiteiplist的文件，每个ip单独一行。

◇ 5. 行情前置IP最大session连接数限制、最大可订阅合约数限制

ctp系统版本自6.7.2P6开始，行情前置新增配置项。

每个IP上活跃的session个数限制，默认为0不限制

MaxIPSession=10

每个session上订阅合约个数限制，默认为0不限制

MaxSubInstCnt=0

默认不配置，即不限制。

若超 MaxIPSession 限制后，会断开新建 session

若超 MaxSubInstCnt 限制后，会给 API 返回订阅失败，失败原因 6000，原因值“CTP:sub too many insts”

想要指定ip不受以上限制，可以通过在front_md_se的bin目录下面新建一个文件名为whiteiplist的文件，每个ip单独一行。

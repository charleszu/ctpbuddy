# fens连接说明

fens连接说明

fens机制提供了另一种接入CTP的方式。

后续不再维护，请大家使用tgate提供的地址服务，详见[TGate网关接入指南](TGATE.html)

◇ 1. 优势

通常情况下，用户为了实现交易前置接入的冗余备份，会用[RegisterFront](../HQJK/CTHOSTFTDCMDAPI/REGISTERFRONT.html)注册多个地址，当所连前置发生故障时，API会自动连接到其他已注册的可用地址。但是，CTP规定用户同一时间只能在一个中心有交易权限，基于这个前提，所有注册的地址必须是同一中心的，否则API在择优挑选地址时如果连接到其他中心，用户是没有交易权限的；当发生灾备切换时，用户需要从主中心的前置切换到备中心的前置，此时就不得不中断交易，退出程序，重新注册备中心的前置地址进行连接。可见，当发生中心切换时，[RegisterFront](../HQJK/CTHOSTFTDCMDAPI/REGISTERFRONT.html)方式带来的连接恢复成本是很高的。

再比如，一些客户端在打包发布时，往往集成了前置地址列表。但是，如果期货公司更改了前置地址，客户端就不得不重新打包发布。

fens地址就很好的解决了[RegisterFront](../HQJK/CTHOSTFTDCMDAPI/REGISTERFRONT.html)的缺点。用户只要接入fens前置，fens会根据用户所在的中心号自动注册对应中心的地址列表；当发生中心切换时，用户会自动转到另一个中心的前置上，迅速恢复交易，整个过程无需干预。

同样的，对于客户端的打包发布，只需配置fens地址，即便期货公司新增或修改了前置地址，期货公司也不需要通知客户端，客户端也不需要重新打包发布。

另外，fens还能根据用户所在的网段，自定义返回的地址列表。

下文将详细介绍fens的一些配置和使用方法。

◇ 2. fens连接规则

下图是fens.xml示例，该配置在期货公司端：

投资者在调用接口[RegisterFensUserInfo](../HQJK/CTHOSTFTDCMDAPI/REGISTERFENSUSERINFO.html)时需要选择LoginMode参数。

若选择THOST_FTDC_LM_Trade，且投资者是主中心用户，则会连接到sysid="1"的这组地址；

若选择THOST_FTDC_LM_Trade，且投资者是备中心用户，则会连接到sysid="2"的这组地址；

若选择THOST_FTDC_LM_Transfer，且投资者是主中心用户，则会连接到sysid="1"的这组地址；

若选择THOST_FTDC_LM_Transfer，且投资者是备中心用户，则会连接到sysid="1"的这组地址。

◇ 3. 参数说明

THOST_FTDC_LM_Trade模式下，fens会根据用户所在中心，自动切换。

THOST_FTDC_LM_Transfer模式下，fens只连接主中心前置。

◇ 4. 代码示例

交易fens使用方法：

```c++

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
printf(pUserApi->GetApiVersion());
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
CThostFtdcFensUserInfoField pFensUserInfo = { 0 };
strcpy_s(pFensUserInfo.BrokerID, "9999");
strcpy_s(pFensUserInfo.UserID, "1000001");
pFensUserInfo.LoginMode = THOST_FTDC_LM_Trade;
pUserApi->RegisterFensUserInfo(&pFensUserInfo);
pUserApi-> RegisterNameServer (“tcp://127.0.0.1:41205”);
pUserApi->Init();

```

```

行情fens使用方法：

``` c++

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
CThostFtdcFensUserInfoField pFensUserInfo = { 0 };
strcpy_s(pFensUserInfo.BrokerID, g_chBrokerID);
strcpy_s(pFensUserInfo.UserID, g_chUserID);
pFensUserInfo.LoginMode = THOST_FTDC_LM_Trade;
pUserMdApi->RegisterFensUserInfo(&pFensUserInfo, nRequestID++);
pUserMdApi->RegisterNameServer (“tcp://127.0.0.1:41205”);
pUserMdApi->Init();

```

```

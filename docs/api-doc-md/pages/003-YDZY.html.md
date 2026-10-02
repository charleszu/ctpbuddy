# 阅读指引

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

阅读指引<a id="content"></a>

|  | ■ 6.7.13_API接口说明
└◆ 阅读指引 |  |
|---|---|---|

在使用CTP的API前，请务必先了解[看穿式监管数据采集说明](pages/022-CTSJGSJCJJK-_CTSJGSJCJJK.html.md)，因为6月份就开始正式启用看穿式监管，届时只有满足监管要求的客户端才能登录CTP系统做交易。

详见：

[看穿式监管数据采集说明](pages/022-CTSJGSJCJJK-_CTSJGSJCJJK.html.md)

[常见FAQ](pages/023-CTSJGSJCJJK-CJFAQ.html.md)

通过上文我们应该了解了自己要开发的客户端属于哪种模式，并且已经按照该模式成功登录进CTP系统，下面我们就可以正式开始使用CTP的API接口提供的各种功能了。以下两个文档提供了详细代码示例和说明，能够帮助我们快速上手。

详见：

[交易接口](pages/059-JYJK-_JYJK.html.md)

[行情接口](pages/026-HQJK-_HQJK.html.md)

报单和报价涉及到诸多回调函数，如何理清这些回调函数的顺序关系通常是初学者最为困扰的事情。这里例举了一些常用的报单操作所对应的回调顺序，能够帮助我们极大的减轻测试工作。

详见：

[报单回调顺序](pages/389-QTYWGZ-DBHB.html.md)

[报价回调规则](pages/391-QTYWGZ-DJHDGZ.html.md)

如果想要了解做市商相关功能，请参考：

[做市商询价和报价](pages/388-QTYWGZ-BJHXJ.html.md)

如果我们所在的期货公司提供了转发的上期所组播行情，通常这种行情的速度要比CTP普通行情要快，要想使用这种行情，我们可以参考二代[行情接口](pages/026-HQJK-_HQJK.html.md)说明。

详见：

[协议方式接收二代组播行情](pages/392-QTYWGZ-EDHQJR.html.md)

CTP的接口提供了很多丰富的功能，大部分的接口都是易于理解和使用的，但是仍然会有部分接口有其特殊的规则，例如手续费率查询接口并不能一次性返回所有合约的手续费率。如果想要准确正确地使用API，并将其投入到生产中，请务必了解以下这些特殊规则（本文档会持续更新补充）。

详见：

[报单流控、查询流控和会话数控制](pages/396-QTYWGZ-LK.html.md)

[大商所组保](pages/390-QTYWGZ-DCEZB.html.md)

更多内容可以通过左侧菜单树的[其他业务规则](pages/387-QTYWGZ-_QTYWGZ.html.md)找到。

<a id="author"></a>

<a id="theme_switcher"></a>

# CTP-GetSystemInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

CTP-GetSystemInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

获取AES加密和RSA加密的终端信息。该函数来自采集终端信息的动态链接库（安卓版的函数名和使用方法较windows和linux有所区别，请注意阅读API包相关说明文档）。

仅中继模式下的客户端需要调用此函数来采集信息。中继需要将客户端采集到的信息上报给CTP。

直连模式下，登录的时候自动采集并上报终端信息，所以无需调用。

采集库在win、linux、android上不是线程安全的，不要并发调用，ios是线程安全的

采集库暂时不采集IPv6地址

<a id="61593b7c-61e7-47c7-8943-d66011ec5dbf"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

int CTP_GetSystemInfo(char* pSystemInfo, int& nLen);

<a id="e18e389f-1450-4c73-9da4-92a9210168bf"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pSystemInfo：空间需要调用者自己分配至少270个字节。

要注意这并不是一个字符串，而是数组，因为多次加密后可能断串，**使用memcpy拷贝值而不是strcpy**。

nLen：获取到的采集信息的长度。

<a id="50be8f36-dacb-4e68-8d8d-a540e83cfec1"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0 为正确，非0为错误。

具体哪个采集项有问题需要做如下判断：

```
windows返回值定义:
    从低位开始分别标示 终端信息 ->系统盘分区信息
    返回值 & （0x01 << 0） 不为0 表示终端类型未采集到
    返回值 & （0x01 << 1） 不为0 表示 信息采集时间获取异常
    返回值 & （0x01 << 2） 不为0 表示ip 获取失败  （采集多个相同类型信息的场景有一个采集到 即表示采集成功）
    返回值 & （0x01 << 3） 不为0 表示mac 获取失败
    返回值 & （0x01 << 4） 不为0 表示 设备名 获取失败
    返回值 & （0x01 << 5） 不为0 表示 操作系统版本 获取失败
    返回值 & （0x01 << 6） 不为0 表示 硬盘序列号 获取失败
    返回值 & （0x01 << 7） 不为0 表示 CPU序列号 获取失败
    返回值 & （0x01 << 8） 不为0 表示 BIOS 获取失败
    返回值 & （0x01 << 9） 不为0 表示 系统盘分区信息 获取失败
Linux返回值定义：
    从低位开始分别标示 终端信息 -> BIOS信息
    返回值 & （0x01 << 0） 不为0 表示终端类型未采集到
    返回值 & （0x01 << 1） 不为0 表示 信息采集时间获取异常
    返回值 & （0x01 << 2） 不为0 表示ip 获取失败  （采集多个相同类型信息的场景有一个采集到 即表示采集成功）
    返回值 & （0x01 << 3） 不为0 表示mac 获取失败
    返回值 & （0x01 << 4） 不为0 表示 设备名 获取失败
    返回值 & （0x01 << 5） 不为0 表示 操作系统版本 获取失败
    返回值 & （0x01 << 6） 不为0 表示 硬盘序列号 获取失败
    返回值 & （0x01 << 7） 不为0 表示 CPU序列号 获取失败
    返回值 & （0x01 << 8） 不为0 表示 BIOS 获取失败

```

<a id="7f810642-df68-4d6d-823b-0632f188fad3"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
char pSystemInfo[344];
int len;
CTP_GetSystemInfo(pSystemInfo, len);

```

<a id="b22c0026-c61e-46b9-bcf3-fed823a05270"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

终端采集到的信息，保存到数据库里后显示为乱码，怎么办？<a id="region_panel_1"></a>

| 可以使用BASE64转码成可见字符后再存储。 |
|---|

<a id="region_tail_1"></a>

<a id="region_header_2"></a>

在windows下调用该函数后，控制台窗口打印出一些日志，提示找不到wmic之类的信息，这个有关系吗？<a id="region_panel_2"></a>

| 有关系，采集函数需要调用相关windows组件来采集设备信息，如果没有对应组件，则会采集不到相应信息，不符合监管要求。 |
|---|

<a id="region_tail_2"></a>

<a id="region_header_3"></a>

在linux下调用该函数后，控制台窗口会输出一些报错日志，提示没有权限之类的错误，这个有关系吗？<a id="region_panel_3"></a>

| 有关系，采集函数需要调用linux命令来采集设备信息，如果没有对应权限，则会采集不到相应信息，不符合监管要求。 |
|---|

<a id="region_tail_3"></a>

<a id="author"></a>

<a id="theme_switcher"></a>

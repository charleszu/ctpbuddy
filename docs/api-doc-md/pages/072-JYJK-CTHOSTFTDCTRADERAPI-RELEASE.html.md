# Release

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

Release<a id="content"></a>

<a id="left_menu"></a>

  ** **

删除接口对象本身

不再使用本接口对象时,调用该函数删除接口对象

非线程安全，多线程使用请加锁。

建议使用logout函数登出，等自动重连后再重新登录，以实现api实例重复使用。不建议直接release api实例。
<a id="5232c265-1216-4f78-8a2f-49bcca55d515"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void Release() = 0;

<a id="29304c0b-3ed9-4aea-8dba-79f95153eb9b"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

无

<a id="08daeeda-ccdc-4ce2-8f03-065f86982c12"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="007f6e2a-08a1-47e6-88b9-ee38669d5117"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
template <class TUserApi>
void CUserApiEnv<TUserApi>::UnInitialUserApi()
{
    // 释放UserApi
    if (m_pUserApi)
    {
        m_pUserApi->Release();
        m_pUserApi = NULL;
    }
    // 释放UserSpi实例
    if (m_pUserSpiImpl)
    {
        delete m_pUserSpiImpl;
        m_pUserSpiImpl = NULL;
    }
}

```

<a id="0bf41ebd-07c3-49be-bb4e-a83c4afdfd6a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>

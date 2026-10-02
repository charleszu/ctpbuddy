# Release

Release

删除接口对象本身

不再使用本接口对象时,调用该函数删除接口对象

非线程安全，多线程使用请加锁。

建议使用logout函数登出，等自动重连后再重新登录，以实现api实例重复使用。不建议直接release api实例。

◇ 1. 函数原型

virtual void Release() = 0;

◇ 2. 参数

无

◇ 3. 返回

无

◇ 4. 调用示例

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

◇ 5. FAQ

无

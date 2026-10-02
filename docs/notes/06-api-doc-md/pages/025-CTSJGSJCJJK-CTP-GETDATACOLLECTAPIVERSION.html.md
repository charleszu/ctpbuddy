# CTP-GetDataCollectApiVersion

CTP-GetDataCollectApiVersion

获取采集库版本号

◇ 1. 函数原型

const char * CTP_GetDataCollectApiVersion(void);

◇ 2. 返回

采集库版本号，格式如下：

Sfit + 生产还是测试秘钥(pro/tst) + 秘钥版本 + 编译时间 + 版本(内部)

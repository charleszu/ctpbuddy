# ctpsdk

**本目录存放使用者自备的上期技术官方 SDK，只用于本地构建与调试，禁止提交到 git、禁止随项目分发。**

当前放置：`6.7.13_20260225/`（CTP API 6.7.13，含 td / md 的 win64、linux64 头文件与库）。

codegen 默认从环境变量 `CTPBUDDY_SDK` 或本目录下最新版本目录读取头文件：

```bash
export CTPBUDDY_SDK=/path/to/ctpsdk/6.7.13_20260225
python shim/codegen/gen_structs.py
```

合规声明：CTPBuddy 与上海期货信息技术有限公司无任何隶属关系，不分发其任何原始文件。

# 6.7.11TGate版本更新说明

6.7.11TGate版本更新说明

版本号：v6.7.11_20250617 16:47:32.10369

后台版本：V6.7.11

变更说明：此版本做了交易网关tgateAPI的评测与生产版本合并，若不修改默认模式，默认接入的是红区生产版本。

◇ 1. API变动

◇ 1.1.交易网关tgateAPI

增加一个bool类型的默认参数 blsProductionMode，表示api是否使用生产模式，true 为生产模式（默认值），false为测评模式。即tgateapi的接口由  “CreateTGateFtdcApi();  ”改为 “CreateTGateFtdcApi(bool blsProductionMode=true);”。

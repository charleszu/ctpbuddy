# ADR 0001：Canonical journal contract

- 状态：已接受
- 日期：2026-10-11
- 范围：Rust core journal writer、Python journal/replay/store、Web read model

## 背景

Rust 核心负责写 journal，Python 负责校验、重放和 SQLite projection。事件类型、必填字段、可重放性、规范化和 hash 边界如果分别维护，新增一个事件就必须同步修改多处，容易产生“能写但不能重放”或两侧 hash 不一致的状态。

## 决策

以 `core/ctpbuddy-server/src/journal_contract.rs` 的 contract table 作为事件语义的唯一规范入口。每个事件在表中声明：

1. 事件名称和 envelope 必填字段；
2. 是否参与 canonical journal hash；
3. 是否允许 replay，以及 replay 所需的 admin 命令映射；
4. Python 适配器需要保留的身份键和投影字段。

Rust writer 在落盘前执行 contract 校验；Python validator、replay 和 projection 使用同一组事件名、字段规则和 canonicalization vectors 做跨语言回归。contract version 进入 manifest/read-model metadata，语义变化必须递增版本并保留兼容说明。

## 约束

- `ts_wall` 是审计字段，不改变业务事件的 canonical hash；
- 未知事件、缺失必填字段、序列断裂和非法 JSON 都 fail closed；
- `not_replayed` 只表示输入事件不可重新发出，不得把它静默当作 journal 校验成功；
- projection 允许落后 live ADMIN，但必须暴露 journal sequence、virtual time、contract version 和来源。

## 不在本 ADR 中

本决策不规定 Parquet 行情格式、SQLite 表结构或真实柜台的结算细节；这些仍由各自的 provider/ledger contract 负责。

## 后续工作

- 为新增事件维护 Rust/Python golden vector；
- 为 projection hash 定义稳定算法后再加入 metadata；
- 将剩余 admin 输入逐步纳入可重放事件集合。

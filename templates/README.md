# 项目文档模板

模板用于降低启动成本，不是独立规则来源。复制后必须根据项目 profile 删除占位符、填写真实决策，并对不适用项使用 `N/A: <理由>`。

| 模板 | 用途 |
|---|---|
| [`practice.toml`](practice.toml) | 范围、stage、profile 和能力域 |
| [`acceptance.toml`](acceptance.toml) | 预注册 baseline、指标、预算和命令 |
| [`practice-README.md`](practice-README.md) | 实验任务书与项目运行说明 |
| [`ADR.md`](ADR.md) | 架构取舍 |
| [`threat-model.md`](threat-model.md) | 威胁模型、控制和残留风险 |
| [`evaluation-report.md`](evaluation-report.md) | 评测、统计和失败分析 |
| [`data-flow.md`](data-flow.md) | 数据流、信任边界和敏感字段 |
| [`runbook.md`](runbook.md) | 告警、止损、恢复和 rollback 操作 |
| [`incident-review.md`](incident-review.md) | 无责事故复盘和行动项 |
| [`evidence-index.md`](evidence-index.md) | 能力域、行为锚点和原始 artifact 映射 |
| [`blind-task-report.md`](blind-task-report.md) | 盲测角色、隔离、过程和判定记录 |
| [`fault-injection-report.md`](fault-injection-report.md) | 故障注入参数、轨迹、恢复和复验 |
| [`load-test-report.md`](load-test-report.md) | 负载、容量、延迟和成本分析 |

规范阅读路由以 [`AGENTS.md` 的规范性文档表](../AGENTS.md#01-规范性文档)为准。模板使用者必须根据项目 stage、profile 和专项实践读取全部适用规范，不能只读取模板或项目契约。

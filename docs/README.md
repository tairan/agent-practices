# 文档维护说明

## 分层原则

- `AGENTS.md` 只保存仓库级治理、路由和不可降级不变量。
- `docs/learning-path.md` 回答“先练什么、每一步产出什么”。
- `docs/practice-catalog.md` 是题库以及 Subagent、Skill、协议、Gateway 等专项规则的权威来源。
- `docs/project-contract.md` 回答“一个项目如何声明、验收和提交证据”。
- `docs/handbook/` 回答“工程机制必须怎样实现和验证”。
- `docs/technology-baseline.md` 单独管理易变版本。
- `templates/` 提供起点，不定义新规则。
- `assessment/` 管理能力评分、盲测和阶段门禁。

## 维护规则

1. 同一条强制规则只设一个详细权威来源；其他位置使用链接和短摘要。
2. 安全不变量必须在 `AGENTS.md` 可见，详细验证方法放在安全运营手册。
3. 仓库共享的版本、镜像、模型与协议默认基线及核对日期只在技术基线维护；项目 README 只记录实际采用值、项目自有版本、偏离理由和基线链接。
4. 不使用跨文件章节号引用；使用相对链接和稳定标题。
5. 新增 profile 时必须同时更新 `AGENTS.md`、项目契约中的测试矩阵和适用手册。
6. 新增能力域或调整门禁时必须同时更新学习路径和 assessment rubric。
7. 模板与契约冲突时以规范性契约为准；修改契约后必须同步模板。

# Agent Practices

使用 Rust 独立实践 Agent 核心机制、评测、安全、互操作和生产运营能力的实验仓库。

## 从这里开始

1. 阅读 [`AGENTS.md`](AGENTS.md)，确认仓库规则和项目 profile。
2. 按 [`docs/learning-path.md`](docs/learning-path.md) 选择最小实操主线。
3. 从 [`docs/practice-catalog.md`](docs/practice-catalog.md) 选择一个概念。
4. 使用 [`templates/`](templates/README.md) 建立项目声明、验收契约和实验任务书。
5. 按 [`docs/project-contract.md`](docs/project-contract.md) 实现、测试并生成 evidence。
6. 使用 [`assessment/`](assessment/README.md) 执行能力验收，而不是用项目数量推断能力。

## 文档地图

| 目标 | 文档 |
|---|---|
| 理解规则优先级、profile 和隔离边界 | [`AGENTS.md`](AGENTS.md) |
| 选择学习顺序 | [`docs/learning-path.md`](docs/learning-path.md) |
| 查看实践题库 | [`docs/practice-catalog.md`](docs/practice-catalog.md) |
| 编写项目契约和测试 | [`docs/project-contract.md`](docs/project-contract.md) |
| 实现 Model、Prompt、Context、Loop、Tool 与评测 | [`docs/handbook/engineering-contracts.md`](docs/handbook/engineering-contracts.md) |
| 实现安全、身份、数据和生产运营要求 | [`docs/handbook/security-and-operations.md`](docs/handbook/security-and-operations.md) |
| 核对固定技术版本 | [`docs/technology-baseline.md`](docs/technology-baseline.md) |
| 评分、门禁和盲测 | [`assessment/README.md`](assessment/README.md) |

实践项目相互独立。请进入具体 `practices/<concept-name>/` 目录执行其构建、测试和运行命令，仓库根目录不是 Cargo workspace。


# Agent 开发实操路径

本路径面向已经理解 Agent 基本概念、但缺少实现和运行经验的软件工程师。实践目录是题库，不要求按编号全部完成；学习者应选择足以产生能力证据的最小项目集合。

## 实操方法

每个阶段都使用同一闭环：

1. 先实现不使用目标 Agent 机制的 baseline。
2. 在查看最终结果前写好 `acceptance.toml`。
3. 完成最小成功路径。
4. 注入非法输入、超时、重复、取消或权限错误。
5. 对失败分类，修复后比较 baseline 与目标实现。
6. 保存机器可读结果、失败样本和决策说明。
7. 用未知输入或未知故障执行迁移验证。

项目 README 应使用 [`templates/practice-README.md`](../templates/practice-README.md) 中的实验任务书结构，明确哪些机制必须自行实现、允许使用哪些库以及禁止哪些捷径。

## Foundation 推荐主线

| 步骤 | 实践 | 必须亲手完成 | 代表性故障 |
|---|---|---|---|
| 1 | `model-behavior-lab` | 最小 ModelClient、停止原因和能力记录 | 截断、拒答、限流、usage 缺失 |
| 2 | `structured-output` | 抽取、schema 校验和类型化错误 | 非法 JSON、多对象、缺字段 |
| 3 | 基础 Prompt contract 子任务 | 稳定 ID/版本、变量边界、拒答、候选比较和回归 | 指令冲突、变量注入、输出不合规 |
| 4 | `tool-calling` | tool schema、参数校验和执行边界 | 未知工具、非法参数、超大结果 |
| 5 | `agent-loop` | 类型化状态、预算、取消和终止原因 | 重复动作、振荡、连续无进展 |
| 6 | `context-management` | context builder、来源、信任和 token 预算 | 越权、陈旧、重复、冲突 |
| 7 | `error-recovery` | 错误分类、有限重试和降级 | 超时、限流、不可重试错误 |
| 8 | `deterministic-evaluation` | 数据集隔离、baseline、确定性评分 | 评测污染、只保留成功样本 |
| 9 | 小型综合项目 | 组合前述边界并完成盲测 | 未公开输入或故障 |

基础 Prompt contract 子任务不是新增概念项目，应嵌入一个声明 `prompt` profile 的 Foundation 项目，例如 `tool-calling` 或 `agent-loop`；必须保存两个有意义候选或单候选理由、版本记录和回归 evidence。后续 `prompt-engineering` 仍是综合验收，不是第一次接触 Prompt contract。

这是一条推荐路径，不是新增项目依赖。学习者可以用等价项目证明 Foundation Gate，但必须覆盖 [`assessment/rubric.md`](../assessment/rubric.md#foundation-gate) 的全部行为。

## Systems 推荐组合

Foundation 通过后，选择一个有持久状态的系统项目，并组合以下能力：

- `durable-execution` 或 `long-running-agent`：checkpoint、恢复和任务所有权；主要产生 `reliability` evidence。
- `mcp-client`、`mcp-server` 或企业连接器：跨进程契约；主要产生 `interop` evidence。
- `tool-permissions` 和 `prompt-injection-risk-reduction`：权限与不可信输入；主要产生 `security` evidence。
- `tracing-observability` 和 `agent-regression-gate`：可观测与回归；主要产生 `evaluation`、`reliability` evidence。
- `idempotent-side-effects`：重复执行和部分成功；主要产生 `tool-loop`、`reliability` evidence。

通过前必须完成一次跨模块故障定位，并证明故障不会造成越权、失控重试或重复不可逆副作用。

## Specialization 选择

根据岗位选择一条主线，并至少与另一能力域集成：

| 方向 | 推荐组合示例 |
|---|---|
| 企业授权 | token exchange + scope down + tool authorization |
| 检索与记忆 | ingestion + ACL-aware retrieval + deletion propagation |
| 多 Agent | Subagent lifecycle + context isolation + reduced permission |
| 分布式运行 | persistent queue + lease + idempotent side effects |
| 协议 | MCP 或 A2A + conformance + authorization boundary |
| 浏览器 | untrusted page + approval binding + result verification |
| 评测工程 | dataset design + statistics + regression gate |

未选择的方向不要求完成全部项目，但必须能够说明适用场景、风险和不应采用它的情况。

## Capstone 与生产化

Capstone 不是把所有模式堆在一起。必须从明确问题出发，先证明 Agent 相对 baseline 的收益，再选择必要机制。每个 Capstone 需要：

- 架构设计、ADR、威胁模型和数据流图。
- 评测报告、负载测试和成本分析。
- 故障演练、runbook、rollback 和事故复盘。
- 至少一次 schema、prompt、模型、配置或状态迁移。
- 仅在 Capstone 内抽取稳定共享接口，并记录兼容策略。

正式门禁和能力等级以 [`assessment/rubric.md`](../assessment/rubric.md) 为准。

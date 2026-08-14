# 实践概念目录

项目名用于保持仓库范围稳定。它们没有自动能力等级和严格先后依赖；具体范围由项目 `practice.toml` 定义。不得为了增加项目数量拆分没有独立验收价值的概念。

## 模型与单 Agent 基础

- `model-behavior-lab`：随机性、上下文、截断、停止原因、拒答、限流和能力差异。
- `structured-output`：JSON Schema 或等价约束、解析与校验。
- `tool-calling`：工具定义、参数解析、执行和结果回传。
- `agent-loop`：模型、动作、观察和稳定停止。
- `react-agent`：结构化决策、行动和观察，不暴露隐藏思维链。
- `plan-and-execute`：生成计划并逐步执行。
- `dynamic-replanning`：根据结果修订剩余计划。
- `reflection-agent`：失败后生成结构化反思并受限重试。
- `evaluator-optimizer`：用评估反馈驱动迭代。
- `self-consistency`：生成多个候选并聚合或选择。
- `error-recovery`：分类处理模型、工具、网络、超时和替代动作。

## 上下文、知识与记忆

- `context-management`
- `conversation-memory`
- `summary-memory`
- `episodic-memory`
- `semantic-memory`
- `procedural-memory`
- `basic-rag`
- `hybrid-retrieval`
- `reranking`
- `grounded-answering`

这些项目必须区分当前会话上下文、可持久化记忆和外部知识，不得因文本相似而混用权限、保留期或删除语义。

## Workflow 与编排

- `prompt-chaining`
- `request-routing`
- `parallel-fanout`
- `orchestrator-workers`
- `map-reduce-agent`
- `state-machine-agent`
- `graph-workflow`
- `durable-execution`
- `human-in-the-loop`
- `event-driven-agent`

每个项目必须说明为什么采用 Workflow 或 Agent，以及更简单的确定性实现是否足够。

## 多 Agent 与 Subagent

- `supervisor-agents`
- `agent-handoff`
- `role-based-team`
- `debate-and-vote`
- `blackboard-collaboration`
- `peer-to-peer-agents`
- `planner-executor-reviewer`
- `competitive-agents`
- `consensus-agents`
- `subagent-spawn`
- `subagent-lifecycle`
- `subagent-result-return`
- `subagent-context-isolation`
- `subagent-concurrency`

Subagent 项目必须满足：

- 每个 Subagent 使用独立 session ID。
- 父 Agent 只显式传递当前子任务需要的上下文。
- Subagent 不能读取父 Agent 的凭据存储。
- 子任务失败不得破坏父任务状态。
- 预算、并发量、排队长度和递归深度必须有上限。
- 父任务结束时必须取消或接管未完成子任务。

## Skill 与扩展

- `dynamic-skill-plugin`

该项目至少包含 `skill-host` 和独立 `cdylib` 插件，使用版本化 C ABI 和 JSON 字节边界，覆盖初始化、执行、释放、目录扫描、重复名称、allowlist、来源或哈希校验、ABI 校验、审计、超时和结构化错误。禁止跨动态库直接传递 Rust trait object。

每个 Skill 必须声明稳定名称、Skill 版本、描述、ABI 版本和能力列表；host 必须在加载和执行前确定性校验这些元数据、重复名称、兼容 ABI 与 allowlist，不得依赖插件自报成功。

原生动态库不是安全沙箱。即使完成签名、哈希和 allowlist 校验，插件仍可能导致进程崩溃、内存安全问题和任意代码执行，只允许加载可信插件。

## 互操作协议

- `mcp-server`
- `mcp-client`
- `a2a-agent-card`
- `a2a-task-delegation`

MCP 用于 Agent 与工具或资源连接；A2A 用于独立 Agent 之间发现和委托。不得把两者合并为同一个抽象。

协议项目必须锁定实现版本、声明兼容版本和未实现能力，覆盖版本不兼容，并在存在参考实现或 conformance 工具时执行跨实现测试。具体版本只从 [`technology-baseline.md`](technology-baseline.md#协议基线) 读取。

## Agent Gateway

- `agent-gateway`

最小控制平面包含 connection registry、authentication、session registry、message router、agent runtime adapter、approval manager 和 event broadcaster。

最小协议包含 `connect`、`message.send`、`session.create`、`session.list`、`session.close`、`agent.run`、`agent.cancel`、`agent.status`、`approval.request`、`approval.resolve` 和 `event.subscribe`。

每条消息必须包含 request ID；相关操作必须包含 session ID 和 agent ID。错误使用稳定结构化错误码。同一 session 串行执行，不同 session 可以并发。实现必须覆盖认证、心跳、背压、超时、取消、断线和优雅关闭。

教学 Gateway 采用单进程、单节点和内存状态，信任模型为单一可信操作者，不得宣称可以隔离敌对多租户。

## 生产级能力

- `input-output-guardrails`
- `tool-permissions`
- `sandboxed-execution`
- `prompt-injection-risk-reduction`
- `tracing-observability`
- `trajectory-evaluation`
- `budget-control`
- `resilience-patterns`
- `model-routing-fallback`
- `deterministic-replay`
- `long-running-agent`

## 企业系统集成

- `oidc-enterprise-login`
- `enterprise-api-connector`
- `webhook-agent-trigger`
- `event-bus-integration`
- `connector-capability-discovery`
- `enterprise-data-boundary`
- `enterprise-integration-gateway`

企业集成项目必须使用项目自己的 Keycloak、realm、client 和测试用户，执行真实 OAuth/OIDC 网络交互，并使用项目内模拟企业 API。必须显式处理分页、限流、超时、重试、幂等、错误映射、Webhook 签名、时间戳、重放窗口和进入模型前的数据最小化。

最低专项测试必须覆盖适用的多页分页、限流与退避、Webhook 签名错误、过期时间戳、重放窗口、重复消息幂等和跨 tenant 拒绝。只测试单页成功请求不得通过企业集成验收。

## Agent 代表用户执行

- `delegated-user-token`
- `oauth-token-exchange`
- `on-behalf-of-agent`
- `audience-bound-token`
- `scope-down-delegation`
- `tool-level-authorization`
- `delegation-chain`
- `subagent-identity-propagation`
- `step-up-authorization`
- `delegated-token-lifecycle`
- `delegated-access-audit`
- `confused-deputy-defense`

声明 `auth` profile 的身份与授权实践必须使用项目自己的 Keycloak、realm、OAuth client 和测试用户，执行真实 OAuth/OIDC 网络交互。不得用本地自签或伪造 JWT 替代授权服务器的签发、发现、撤销、introspection、token exchange、audience 和 scope 行为；业务资源服务器可以使用项目内 mock。离线单元测试仍应使用固定 fixture，真实授权服务器交互作为独立集成验收执行。

## 评测工程

- `eval-dataset-design`
- `deterministic-evaluation`
- `llm-as-judge`
- `retrieval-evaluation`
- `agent-regression-gate`
- `online-evaluation`
- `evaluation-statistics`

评测工程项目必须至少包含一个结果质量指标、一个安全或契约不变量指标、baseline、数据集版本、p50/p95 延迟和适用的 token/成本统计。随机输出必须报告重复运行和方差或置信区间；LLM-as-judge 必须报告与人工标注的一致性。

## 分布式运行与生产运维

- `persistent-task-queue`
- `idempotent-side-effects`
- `distributed-agent-runtime`
- `multi-tenant-runtime`
- `slo-alerting`
- `deployment-rollout`
- `incident-response`

## 数据、检索与记忆生命周期

- `document-ingestion`
- `index-lifecycle`
- `acl-aware-retrieval`
- `retrieval-freshness`
- `memory-governance`
- `memory-poisoning-defense`

## 模型能力适配

- `model-capability-adapter`
- `streaming-tool-calling`
- `provider-contract-testing`

模型替换必须先通过离线回归，再进入 shadow 或 canary。实现必须保留模型和配置版本，支持回滚，并验证兼容端点在结构化输出、工具调用、流式片段、错误类型、usage 和停止原因上的实际行为。

## 浏览器、计算机操作与多模态

- `browser-agent`
- `computer-use-agent`
- `unsafe-ui-action-approval`
- `multimodal-context`

页面内容属于不可信数据，不得成为高优先级控制指令。执行前后必须验证目标、主体、参数和实际结果。审批后目标或参数发生变化时必须重新审批。

## Prompt、Context 与 Loop Engineering

- `prompt-engineering`
- `context-engineering`
- `loop-engineering`

这三个项目是对前序能力的综合验收，不是第一次接触相应机制。责任边界以 [`handbook/engineering-contracts.md`](handbook/engineering-contracts.md) 为准。

`prompt-engineering` 必须比较至少两个有意义候选，覆盖变量转义、指令冲突、不可信数据和版本回归。`context-engineering` 必须量化相关信息纳入率、无权限内容进入率、陈旧或冲突内容处理、token 预算和质量/成本变化。`loop-engineering` 必须量化任务成功、终止原因、无进展、重复动作、预算和取消行为。

## 人机交互与 Agent 产品

- `agent-interaction-design`

覆盖意图澄清、计划和影响预览、不确定性表达、用户纠错、部分接受、撤销、审批疲劳、通知策略和无障碍交互。未经校准不得把模型自报置信度当作真实概率。

## 综合毕业项目

- `capstone-research-agent`
- `capstone-enterprise-agent`
- `capstone-production-operations`

综合项目必须在自身目录内实现所需能力，不得依赖其他概念目录。每个项目必须包含架构设计、ADR、威胁模型、评测报告、负载测试、成本分析、故障演练、数据流图和已知限制。

Capstone 还必须完成 productionization bridge：识别内部重复机制，建立仅在 Capstone 内共享的稳定接口，执行至少一次 schema、prompt、模型、配置或持久化状态迁移，并记录兼容与 rollback 策略。

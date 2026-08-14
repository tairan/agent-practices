# Agent 工程实现契约

## 推荐最小边界

推荐使用以下概念边界建立清晰的数据流，具体命名可以不同：

```text
User Goal
  -> PromptSpec + ContextBuilder
  -> ModelClient
  -> Proposed Action
  -> Deterministic Validation / Authorization / Approval
  -> ToolExecutor
  -> Normalized Observation
  -> AgentState transition
  -> TerminationReason or next step
```

常见核心类型包括 `ModelClient`、`PromptSpec`、`ContextItem`、`ContextBuilder`、`ToolDefinition`、`ToolInvocation`、`AgentState`、`AgentEvent`、`AgentBudget` 和 `TerminationReason`。这是推荐心智模型，不要求跨项目共享 crate。

## Model contract

声明 `model` profile 的项目必须通过项目内部最小接口接入模型，例如 `ModelClient` trait。默认真实接入和版本规则见 [`../technology-baseline.md`](../technology-baseline.md#模型与-provider-基线)。引入第二家供应商必须在 README 说明动机。

项目 README 必须声明真实模型所依赖的能力等价类、该等价类中的必要能力、同类替换条件和跨等价类升级条件。

接口必须显式表达：

- 流式输出支持情况。
- 工具调用和并行工具调用语义。
- 结构化输出和 JSON Schema 方言。
- 上下文、输出和总 token 限制。
- token 统计和估算方式。
- prompt cache 能力和计费语义。
- 响应模型标识、供应商和可获得的 API 版本。
- 停止原因、拒答和内容过滤结果。
- 可重试、不可重试、限流、超时、协议和能力错误。

不得因 API 宣称兼容而假设行为完全一致。响应未提供模型指纹、API 版本或 usage 时，必须记录 `unknown` 或 `estimated`，不得伪造字段。

同能力等价类模型替换不得修改概念代码，但必须重新运行 provider conformance 和回归评测。跨等价类替换走版本升级流程。

核心测试必须使用固定响应或项目内 mock。真实模型测试必须单独标记、显式启用、限制成本，并记录模型指纹。

### Provider conformance 最小集

真实 provider 验证至少记录：

- 请求与响应模型标识、API 版本和缺失字段行为。
- 结构化输出所用 schema 方言及拒绝不支持关键字的行为。
- 单工具、未知工具、并行工具和流式工具参数片段。
- usage、停止原因、拒答、内容过滤、限流、超时和错误映射。
- 上下文或输出超限行为。

没有凭据时可以延期真实 conformance，但不得声称真实兼容已经验证。

## Prompt contract

声明 `prompt` profile 的项目必须：

- 使用稳定 prompt ID 和版本。
- 明确指令层次、目标、输入变量、输出契约、拒答和降级行为。
- 将可信指令与不可信数据分隔。
- 按目标格式校验和转义变量。
- 不把 secret、授权判断或确定性业务规则交给 prompt。
- 至少比较两个有意义候选，或说明单一 prompt 足够的理由。
- 记录 prompt 变化与回归结果。

不得要求、记录或展示模型隐藏思维链。需要解释时只记录结构化决策摘要、动作、观察、证据和终止原因。

Prompt Engineering 只决定如何表达任务和模型契约，不负责权限判断、secret 注入或确定性业务校验。

## Context contract

声明 `context` profile 的项目必须为每个片段保存：

- source 和可定位 provenance。
- trust level。
- tenant 和访问条件。
- 时间戳、版本和有效期。
- token 数或一致估算值。
- 选择、排序、压缩或丢弃原因。

每次模型调用必须通过显式 context builder 构造上下文。无权限、过期、重复、冲突、低价值或超预算内容不得错误进入上下文，缓存必须定义失效条件。

Context builder 可以消费上游已验证的访问结果，但不得把自行作出的主体/资源授权决策伪装成普通选择逻辑。项目一旦验证身份/token，或判断主体是否可访问受保护资源与租户数据，就同时触发 `auth` profile。仅对公开合成数据执行 context inclusion policy 时不触发 `auth`，但 README 必须明确 `N/A: auth`。

Context Engineering 只决定本次调用给模型哪些信息以及为什么，不得无差别塞入全部可用数据。

## Loop contract

声明 `loop` profile 的项目必须使用显式状态机或等价类型化状态转换，并定义：

- 最大步骤、墙钟超时、token 和成本预算。
- 每轮进度信号和状态不变量。
- 动作前置条件和观察归一化。
- 重复动作、重复观察、状态振荡和连续无进展阈值。
- 工具错误分类和有限重试。
- 取消传播、checkpoint 和适用的恢复语义。
- 稳定枚举形式的终止原因。

不得通过递归 loop 或 Subagent 绕过预算，不得把模型自称完成作为唯一终止条件。成功终止必须由可观察的目标状态或确定性后置条件支持。

Loop Engineering 负责是否继续调用模型、怎样转换状态以及何时停止。

## Tool contract

声明 `tool` profile 的项目必须为每个工具定义：

- 稳定名称、版本和描述。
- 输入、输出和错误 schema。
- 权限、主体、资源和 tenant 边界。
- 风险级别和副作用分类。
- 超时、取消和最大响应大小。
- 幂等键、重复调用和部分成功语义。
- 日志、脱敏和 provenance 规则。

模型只能提出工具调用。确定性代码必须在执行边界完成 schema 校验、授权、审批、资源限制和最终参数绑定。工具结果进入模型前必须归一化、限制大小并标记信任等级。

## 网络与持久化类型

声明 `network` profile 的项目必须为接收和发送的网络消息定义显式类型、版本和大小边界，不得让未校验的通用 JSON 值直接进入业务状态。声明 `stateful` profile 的项目必须为持久化记录定义显式类型、schema 版本和迁移路径；读取旧版、未知版或部分记录时必须产生稳定错误或执行预先定义的迁移。

## 审批与人机交互

声明 `side-effect` profile 的项目必须：

- 执行前显示主体、目标、动作、关键参数和预期影响。
- 对删除、发送、支付、批量写入、权限变更和管理操作默认要求 step-up 或人工审批。
- 将审批绑定到规范化后的最终请求摘要。
- 审批后目标、参数、权限或环境状态改变时重新审批。
- 明确撤销、补偿、部分成功和不可恢复边界。

Agent 必须在目标、主体或影响不明确时请求澄清。不得用频繁、模糊审批把风险判断转嫁给用户。审批界面必须允许拒绝、部分接受和必要时的纠正。

## 评测与实验设计

评测必须覆盖最终结果和适用的中间轨迹，不得只展示少量成功示例。

- 调优前固定开发集、验收集和盲测集边界。
- 保留失败样本和错误分类。
- 使用非 Agent、较简单 Workflow 或人工流程作为适用 baseline。
- 将确定性评分与模型评审分开。
- 对随机输出报告重复运行、方差或置信区间。
- 对变更执行消融，区分 prompt、context、loop、工具、模型和数据影响。
- 防止训练数据、prompt 示例、人工修复或评审说明污染验收集。
- 指标改善不得以非法动作、安全退化或不可接受成本为代价。

按 profile 选择任务成功率、格式合规率、工具选择与参数正确率、非法动作执行率、引用正确率、忠实度、无证据拒答率、p50/p95 延迟、token、成本、恢复率、重复副作用率、终止原因分布、人工接管率和纠错成功率。不适用时说明理由。

使用 LLM-as-judge 时必须处理位置偏差、长度偏差和评分漂移，保留可复核样本，并用人工标注计算一致性。评审模型不得看到不应影响评分的候选身份或实现信息。

以下变化必须触发相关回归：model family 或能力声明、prompt、工具 schema、context policy、loop、数据集、知识库、embedding、索引、权限、协议、依赖、配置和持久化 schema。模型漂移后不得仅凭调用成功判断兼容；离线回归通过后才能进入 shadow 或 canary，guardrail 失败时必须停止发布。

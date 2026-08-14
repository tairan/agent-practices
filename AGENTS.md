# Agent Practices 仓库规范

## 0. 文档体系与权威性

本文件是仓库治理入口，定义规则优先级、适用性、隔离边界和不可降级的安全不变量。详细规则按主题拆分，但仍属于同一规范体系。

### 0.1 规范性文档

| 文档 | 负责内容 | 何时必须阅读 |
|---|---|---|
| 本文件 | 治理、profile、隔离、安全不变量、开发协作 | 所有变更 |
| [`docs/README.md`](docs/README.md) | 文档分层、单一权威来源和模板同步规则 | 修改规范、学习文档或模板时 |
| [`docs/project-contract.md`](docs/project-contract.md) | 项目声明、验收契约、README、Evidence、测试矩阵 | 新增或修改任何实践项目 |
| [`docs/practice-catalog.md`](docs/practice-catalog.md) | 实践题库及 Subagent、Skill、协议、Gateway 等专项强制规则 | 选择、新增或修改对应专项实践时 |
| [`docs/handbook/engineering-contracts.md`](docs/handbook/engineering-contracts.md) | Model、Prompt、Context、Loop、Tool 和评测实现契约 | 声明对应 profile 时 |
| [`docs/handbook/security-and-operations.md`](docs/handbook/security-and-operations.md) | 数据、身份、安全、供应链、可靠性和生产运营 | 涉及数据；或声明高风险 profile 时 |
| [`docs/technology-baseline.md`](docs/technology-baseline.md) | Rust、依赖、工具、镜像、协议和模型版本 | 修改构建、依赖、服务、模型或协议时 |
| [`assessment/rubric.md`](assessment/rubric.md) | 能力等级、阶段门禁和综合项目评分 | 声明能力证据或执行阶段验收时 |
| [`assessment/blind-task-protocol.md`](assessment/blind-task-protocol.md) | 盲测隔离、实施与复核 | 执行任何未知或盲测样本时，包括 Foundation Gate |

以下文档用于导航和学习，不得覆盖规范性规则：

- [`README.md`](README.md)：仓库入口。
- [`docs/learning-path.md`](docs/learning-path.md)：推荐实操主线。
- [`templates/README.md`](templates/README.md)：可复制文档模板。

第 0.1 节表中明确列出的规范性文档，其“必须”或“不得”规则与本文件效力相同。导航、学习和模板文档中的说明不独立产生强制规则。项目不得通过只读取本文件而忽略适用的规范性文档。

### 0.2 规则词与冲突处理

- **必须**：不可省略。违反后项目不得通过验收。
- **条件必须**：仅在声明对应 profile 或触发所述风险时适用。
- **应该**：默认采用；偏离时必须在 README 或 ADR 中记录理由。
- **可以**：可选实现，不影响最低验收。

规则冲突时按以下顺序处理：

1. 本文件和安全运营契约中的凭据、身份、授权、隐私、租户和不可逆副作用不变量。
2. 适用 profile 的规范性“必须”规则。
3. 当前项目 `acceptance.toml` 中预先声明的验收契约。
4. 当前项目 README 和 ADR 中记录的项目决策。
5. 通用实现建议和学习路径。

`acceptance.toml` 可以收紧要求或选择适用指标，但不得降低前两项。项目决策不得把“必须”降为可选。

版本号、镜像、模型和协议基线属于易变信息，只能通过独立升级变更修改。概念定义、能力标准和安全不变量属于稳定规则，不得因升级而顺带改变。文档不得使用易失效的“§数字”跨文件引用，应使用相对文件链接和标题锚点。

## 1. 项目目标与非目标

本仓库用于使用 Rust 独立实践 Agent 开发中的主流机制、协议和生产工程能力。

每个概念项目必须能够单独构建、测试和运行。概念之间不得共享代码、配置、凭据、数据、缓存、数据库、会话、运行状态或构建产物。

本仓库强调理解和自行实现核心机制，不以封装现有 Agent 框架为目标。完成核心机制后，可以通过对照实验评估框架、托管 runtime 或供应商方案，但不得让其替代当前概念的核心实现。

本仓库不是：

- Agent 模式数量竞赛。
- 通过测试即获得资深能力的认证系统。
- 生产系统应复制所有隔离和重复实现方式的架构模板。
- 鼓励在确定性程序足够时使用模型、多 Agent、RAG、长期记忆或动态执行的理由。

能力必须由可复核 evidence 判断，包括未知任务实现、故障诊断、量化评测、安全审查、生产运行或等价演练、架构答辩和决策质量。不得根据项目数量、代码行数、模型调用次数或测试通过数量直接推断能力。

## 2. 核心术语

- **Agent**：模型根据当前目标、显式上下文和观察结果动态选择下一步动作的系统。
- **Workflow**：执行路径主要由代码预先定义，模型只负责局部决策的系统。
- **Tool**：Agent 可调用的外部能力，具有版本化输入输出、权限、风险、副作用和错误边界。
- **Model capability**：模型或兼容 API 明确支持的能力，如流式输出、工具调用、结构化输出、上下文限制和 token 统计。
- **Prompt Engineering**：设计模型交互契约，包括指令层次、变量、输出约束、拒答和降级行为，并用评测选择版本。
- **Context Engineering**：设计每次模型调用可见信息的供应链，包括来源、信任、权限、选择、排序、压缩、时效、预算和 provenance。
- **Loop Engineering**：设计 Agent 闭环的状态、动作、观察、进度、停止、恢复和收敛机制。
- **Subagent**：父 Agent 针对明确子任务创建的独立 Agent 实例，拥有独立 session、上下文、预算、权限和生命周期。
- **Skill**：可被运行时发现和调用，并声明输入、输出、能力和权限边界的扩展。
- **Native Plugin**：Skill 的一种进程内实现载体；不得将 Skill 等同于动态库。
- **Gateway**：连接客户端、渠道、session、Agent、审批和事件流的控制平面。
- **Delegation**：Agent 在用户已授权范围内代表该用户访问下游资源。
- **Impersonation**：系统绕过用户授权直接模拟用户身份；本仓库默认禁止。
- **Evidence**：能够复核能力的代码、测试、评测数据、运行记录、决策文档或答辩结果。
- **Baseline**：不使用目标 Agent 机制的较简单实现，用于判断新增复杂度是否产生收益。
- **Blind task**：学习者事先不知道输入、故障或验收样本的考核任务。

## 3. 适用性 profile

每个项目必须声明 `base`，并根据实际能力声明其他 profile。只有 `base` 和已声明 profile 的条件规则适用。

| Profile | 触发条件 |
|---|---|
| `base` | 所有项目 |
| `model` | 调用真实或模拟模型 |
| `prompt` | 构造或版本化 prompt |
| `context` | 选择、拼装、检索或压缩上下文 |
| `loop` | 存在多轮模型或动作闭环 |
| `tool` | 定义或执行 Agent 工具 |
| `network` | 发起或接收网络请求 |
| `stateful` | 保存任务、会话、记忆、索引或执行状态 |
| `side-effect` | 执行写入、发送、删除、支付、权限变更等外部副作用 |
| `auth` | 项目实现或验证登录、token、主体/资源授权决策或受保护数据的租户隔离 |
| `protocol` | 实现 MCP、A2A、自定义 Gateway 或其他线协议 |
| `distributed` | 涉及多进程、多实例、消息总线、租约或分布式状态 |
| `untrusted-code` | 执行插件、命令、脚本或用户提供代码 |
| `browser` | 操作浏览器、页面或桌面界面 |
| `production` | 以生产运行、容量、发布、SLO 或事故处理为目标 |

**高风险 profile** 是 `side-effect`、`auth`、`protocol`、`distributed`、`untrusted-code`、`browser` 和 `production`。声明任一高风险 profile 的项目必须提交威胁模型，并满足安全运营手册中的专项要求。该集合是仓库内“高风险 profile”的唯一含义。

项目不得为了满足规则而人为添加无关 profile、依赖、网络调用、数据库、loop 或工具。看似相关但实际不适用时，README 必须使用 `N/A: <理由>` 记录，不得创建无意义测试。

`context` 项目保存 tenant、访问条件或消费上游已验证的访问结果，不自动触发 `auth`。如果项目自行判断某主体能否访问受保护资源、验证身份/token，或执行受保护数据的租户隔离，则必须声明 `auth`；不得把授权决策改名为 context filtering 来规避高风险规则。仅处理公开合成数据时必须在 README 明确写 `N/A: auth` 及数据公开边界。

profile 的最低测试矩阵见 [`docs/project-contract.md`](docs/project-contract.md#profile-验收)。专项实践的附加规则见 [`docs/practice-catalog.md`](docs/practice-catalog.md)。

## 4. 仓库结构与隔离规则

所有概念放在 `practices/<concept-name>/`，标准结构和声明格式见 [`docs/project-contract.md`](docs/project-contract.md)。

必须遵守以下隔离规则：

1. 仓库根目录不得创建 Cargo workspace。
2. 每个概念必须拥有自己的 `Cargo.toml`、`Cargo.lock` 和 `rust-toolchain.toml`。
3. 禁止使用指向其他概念的 path dependency。
4. 禁止创建供多个概念运行时使用的共享 crate、公共源码目录或公共测试 fixture。
5. 禁止读取其他概念的 `.env`、配置、数据、数据库、缓存、会话或执行结果。
6. 每个概念必须使用独立端口、容器名、数据库、Keycloak realm、OAuth client 和测试用户。
7. 删除仓库中的其他概念后，当前概念仍必须能够构建、测试和运行。
8. 多 Agent 项目可以在自身目录和进程边界内共享任务状态，但不得跨概念共享。
9. 一次普通变更默认只修改一个概念；修改根规范时不得顺带重构已有概念。
10. 根目录可以包含静态检查、文档生成或仓库治理工具，但任何概念不得依赖它们才能构建、测试或运行。
11. Capstone 可以在自身目录内使用多个 crate 并抽取内部平台能力；这些 crate 不得被其他概念引用。
12. 所有构建、测试和运行必须使用当前概念锁定的工具链和 lockfile。

教学项目完全隔离用于证明机制独立性，不代表生产系统应复制公共安全、协议、身份和可观测性实现。

## 5. 不可降级不变量

以下规则适用于所有相关项目，任何配置、README、ADR 或验收契约均不得降低：

1. 凭据、token 和 secret 不得进入仓库、模型上下文、prompt、工具输出、日志、trace、错误、任务结果或 evidence。
2. 模型只能提出动作；schema 校验、授权、审批、资源限制和最终参数绑定必须由确定性代码在执行边界完成。
3. 模型不能声明权限。资源服务器和工具执行边界必须验证主体、tenant、issuer、audience、scope 和适用的授权条件。
4. 有用户参与的操作默认使用用户委托，不得使用 client credentials 或服务账号模拟用户，不得 direct naked impersonation。
5. 下游权限必须等于或小于用户原有权限，并缩减到当前动作需要；Subagent 不得继承原始用户 token。
6. ACL-aware retrieval 必须在召回前过滤权限，不得先召回敏感内容再遮盖。
7. 删除、发送、支付、批量写入、权限变更和管理操作必须在执行前展示影响，并默认要求 step-up 或人工审批。
8. 审批必须绑定到规范化后的最终请求摘要；目标、参数、权限或环境状态变化后必须重新审批。
9. 外部调用、loop、重试、并发、队列、递归、成本和资源使用必须有明确上限，并支持适用的超时与取消传播。
10. 产生副作用的重试必须具有幂等键或等价保护；不得通过重试产生重复不可逆副作用。
11. 页面、检索内容、工具输出、记忆、Skill 和 Agent Card 都属于可能不可信的数据，不得提升为高优先级控制指令。
12. 不得宣称 prompt injection 可以彻底消除；只能通过隔离、最小权限、确定性校验和影响范围限制降低风险。
13. 核心测试必须使用固定响应或项目内 mock，不得强制访问真实模型或外部服务。
14. 真实生产数据不得直接用于教学 fixture；数据进入模型、记忆、索引、日志或评测集前必须完成来源、租户、最小化和敏感信息处理。
15. 不得通过修改阈值、删除失败样本、污染验收集或缩小测试范围掩盖退化。

详细实现与验证要求见 [`docs/handbook/security-and-operations.md`](docs/handbook/security-and-operations.md)。

## 6. 开发 Agent 协作规则

在本仓库工作的开发 Agent 必须：

1. 修改前阅读本文件、目标项目 `practice.toml`、`acceptance.toml` 和 README，并根据第 0.1 节读取适用文档。
2. 先识别任务范围和适用 profile，不实现无关规则。
3. 检查并保留用户已有且与任务无关的修改。
4. 将普通变更限制在用户指定概念，不创建跨概念共享基础设施。
5. 优先实现可运行、可评测的最小闭环，再按风险补充失败路径。
6. 不因代码相似而抽取跨概念共享模块；概念独立性优先。
7. 不提交真实凭据，也不使用看似真实的 secret 示例。
8. 完成后运行与变更风险和 profile 相称的检查，并明确报告未运行检查。
9. 标准或外部 API 可能变化时，先核对当前官方规范并记录版本和日期。
10. 不使用博客、二手文章或框架行为替代官方协议和安全规范。
11. 不为了展示 Agent 能力而引入不必要的模型调用、多 Agent、长期记忆或动态执行。
12. 发现需求可以由确定性程序更安全、便宜或可靠地完成时，必须明确提出该方案。
13. 不得通过修改验收阈值、删除失败样本或缩小测试范围掩盖退化。
14. 未经明确要求，不顺带升级依赖、协议、模型等价类或技术基线。

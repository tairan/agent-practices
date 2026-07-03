# Agent Practices 项目规范

## 1. 项目目标

本仓库用于使用 Rust 独立实践 Agent 开发中的主流范式、协议和工程能力。

仓库中的每个概念都是一个可单独构建、测试和运行的实验项目。概念之间不得共享代码、配置、凭据、数据、缓存、数据库、会话、运行状态或构建产物。

本仓库强调理解和自行实现核心机制，不以封装现有 Agent 框架为目标。

完成单个概念项目只能证明学习者掌握了对应机制。是否具备资深 Agent 开发能力，必须通过综合项目、量化评测、生产运行、故障处理、安全审查和架构答辩共同判断，不得仅以完成项目数量或测试通过作为结论。

## 2. 核心术语

- **Agent**：模型可根据当前目标、上下文和观察结果动态决定下一步动作。
- **Workflow**：执行路径主要由代码预先定义，模型只负责其中的局部决策。
- **Tool**：Agent 可调用的外部能力，必须具有明确的输入、输出、权限和错误边界。
- **Prompt Engineering**：设计模型交互契约，包括指令、角色、示例、变量、输出约束和失败行为，并用评测验证其效果。
- **Context Engineering**：设计每次模型调用可见信息的完整供应链，包括来源、选择、排序、压缩、权限、时效、预算和来源追踪。
- **Loop Engineering**：设计 Agent 闭环的状态、动作、观察、进度、停止、恢复和收敛机制，防止无效循环和不可控副作用。
- **Subagent**：由父 Agent 针对明确子任务创建，拥有独立上下文、session、预算、权限和生命周期的 Agent 实例；既可以同步运行，也可以后台运行。
- **Skill**：可被运行时发现和调用，并声明输入、输出、能力和权限边界的扩展。
- **Native Plugin**：Skill 的一种进程内实现载体。本仓库使用 Rust 动态库实践该载体，但不得将 Skill 等同于动态库。
- **Gateway**：连接客户端、渠道、session、Agent 和审批流程的控制平面。
- **Delegation**：Agent 在用户已授权的范围内代表该用户访问下游资源。
- **Impersonation**：系统绕过用户授权直接模拟用户身份。本仓库默认禁止此模式。

## 3. 仓库结构与隔离规则

所有概念放在 `practices/<concept-name>/` 下：

```text
practices/<concept-name>/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md
├── .env.example
├── src/
├── tests/
└── fixtures/
```

需要身份系统或外部服务时，可以增加：

```text
├── compose.yaml
└── keycloak/
    └── realm.json
```

必须遵守以下规则：

1. 根目录不得创建 Cargo workspace。
2. 每个概念必须拥有自己的 `Cargo.toml` 和 `Cargo.lock`。
3. 禁止使用指向其他概念的 path dependency。
4. 禁止创建供多个概念使用的共享 crate、公共源码目录或公共测试夹具。
5. 禁止读取其他概念的 `.env`、配置、数据目录、数据库、缓存或执行结果。
6. 每个概念必须使用独立端口、容器名、数据库、Keycloak realm、OAuth client 和测试用户。
7. 删除仓库中的其他概念后，当前概念仍必须能够构建、测试和运行。
8. 多 Agent 项目可以在该项目自身的进程和目录内共享任务状态，但不得跨概念共享。
9. 一次变更默认只修改一个概念；修改根规范时不得顺带重构已有概念。
10. 每个概念必须提交自己的 `rust-toolchain.toml` 和 `Cargo.lock`，构建、测试和运行必须使用锁定版本。

## 4. 技术约束

- 开发语言统一使用 Rust。
- 可以使用 `tokio`、`reqwest`、`serde`、`serde_json`、`axum`、`tracing`、数据库驱动、协议库和测试库等通用 crate。
- 不得使用 Rig、LangChain、LangGraph、AutoGen、ADK 等现成 Agent 框架实现核心流程。
- 可以使用协议级基础库，但必须在 README 中说明哪些行为由本项目实现、哪些由依赖提供。
- 模型接入必须通过项目内部定义的最小接口（如 `ModelClient` trait）完成。本仓库默认接入 Google Gemini 的 OpenAI 兼容端点；模型抽象层的目的是练习接口边界设计与可替换性，不要求同时维护多家供应商适配器，但接口必须满足“更换同等价类模型不需要修改概念代码”。引入第二家供应商必须在对应概念 README 中说明动机。
- 模型接口必须显式表达能力差异，包括流式输出、工具调用、结构化输出、上下文限制、token 统计、停止原因和错误类型；不得假设所有兼容 API 行为完全一致。
- 真实模型连接通过环境变量配置兼容 API；不得提交 API key、token、client secret 或证书私钥。
- 核心测试必须支持固定响应或项目内 mock，不得强制访问外部模型 API。
- 身份与授权实验必须使用真实 Keycloak 和真实 OAuth/OIDC 请求；业务 API 可以由项目内 HTTP 服务模拟。
- 教学项目之间完全隔离是为了验证概念独立性，不代表生产系统应复制公共的安全、协议、身份和可观测性实现。

### 4.1 固定 Rust 工具链

技术栈基线最后核对日期为 2026-06-23。所有概念统一使用：

| 项目 | 固定版本 |
|---|---|
| Rust | `1.96.0` |
| Cargo | `1.96.0` 随 Rust toolchain 提供 |
| Rust edition | `2024` |
| Cargo resolver | `3` |
| rustfmt | `1.96.0` toolchain 组件 |
| Clippy | `1.96.0` toolchain 组件 |
| Fuzz/Miri 专用 nightly | `nightly-2026-06-22` |

每个概念的 `rust-toolchain.toml` 必须为：

```toml
[toolchain]
channel = "1.96.0"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

单 crate 概念的根 `Cargo.toml` 必须包含：

```toml
[package]
edition = "2024"
rust-version = "1.96.0"

[workspace]
resolver = "3"
```

这里的空 `[workspace]` 只用于为当前独立概念设置 resolver，不得在仓库根目录创建 workspace，也不得把其他概念加入成员列表。

需要多个 crate 的概念可以在该概念目录内使用本地 Cargo workspace，根 manifest 必须包含：

```toml
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
edition = "2024"
rust-version = "1.96.0"
```

成员 crate 必须从 `[workspace.package]` 继承 edition 和 rust-version。该 workspace 的 member、default-member、exclude 和 path dependency 都不得越出当前概念目录。

### 4.2 固定 Rust 依赖版本

直接依赖必须使用精确版本 `=x.y.z`，禁止使用 `*`、范围、仅 major/minor 或未固定的 Git branch/tag。传递依赖由当前概念提交的 `Cargo.lock` 锁定。只允许指向当前概念内部 crate 的 path dependency，并且必须同时声明精确 `version`。

基础依赖版本如下；项目只添加实际需要的依赖和 feature：

| 用途 | crate | 固定版本 |
|---|---|---|
| 异步运行时 | `tokio` | `=1.52.3` |
| HTTP 客户端 | `reqwest` | `=0.13.4` |
| HTTP 服务 | `axum` | `=0.8.9` |
| 中间件 | `tower` | `=0.5.3` |
| HTTP 中间件 | `tower-http` | `=0.7.0` |
| 序列化 | `serde` | `=1.0.228` |
| JSON | `serde_json` | `=1.0.150` |
| JSON Schema 校验 | `jsonschema` | `=0.46.5` |
| JSON Schema 生成 | `schemars` | `=1.2.1` |
| 结构化日志 | `tracing` | `=0.1.44` |
| 日志订阅 | `tracing-subscriber` | `=0.3.23` |
| 库错误 | `thiserror` | `=2.0.18` |
| 应用错误 | `anyhow` | `=1.0.102` |
| async trait object | `async-trait` | `=0.1.89` |
| UUID | `uuid` | `=1.23.3` |
| 日期时间 | `chrono` | `=0.4.45` |
| URL | `url` | `=2.5.8` |
| Future/Stream 工具 | `futures` | `=0.3.32` |
| Tokio 扩展 | `tokio-util` | `=0.7.18` |
| PostgreSQL | `sqlx` | `=0.9.0` |
| 动态库加载 | `libloading` | `=0.9.0` |
| JWT | `jsonwebtoken` | `=10.4.0` |
| OAuth 客户端 | `oauth2` | `=5.0.0` |
| OpenTelemetry API | `opentelemetry` | `=0.32.0` |
| tracing 与 OTel 桥接 | `tracing-opentelemetry` | `=0.33.0` |
| NATS | `async-nats` | `=0.49.1` |
| Kafka | `rdkafka` | `=0.39.0` |
| Qdrant | `qdrant-client` | `=1.18.0` |
| gRPC | `tonic` | `=0.14.6` |
| Protocol Buffers | `prost` | `=0.14.4` |
| HTTP mock | `wiremock` | `=0.6.5` |
| 临时文件 | `tempfile` | `=3.27.0` |
| 属性测试 | `proptest` | `=1.11.0` |
| 并发模型测试 | `loom` | `=0.7.2` |
| Benchmark | `criterion` | `=0.8.2` |

示例：

```toml
[dependencies]
serde = { version = "=1.0.228", features = ["derive"] }
serde_json = "=1.0.150"
thiserror = "=2.0.18"
tokio = { version = "=1.52.3", features = ["macros", "rt-multi-thread", "signal", "time"] }
```

未列出的 crate 可以使用，但必须：

1. 选择发布到 crates.io 的稳定非预发布版本。
2. 使用完整精确版本。
3. 在 README 说明用途、选择原因和核对日期。
4. 检查许可证、维护状态、MSRV、已知漏洞和重复依赖影响。
5. 禁止使用已 yanked 版本；未经明确说明不得使用 Git dependency。

不得仅为统一而添加未使用依赖。不得用 `cargo update` 无差别刷新 `Cargo.lock`。

### 4.3 固定开发工具版本

需要相应能力时使用以下版本：

| 工具 | 固定版本 |
|---|---|
| `cargo-audit` | `0.22.2` |
| `cargo-deny` | `0.19.9` |
| `cargo-nextest` | `0.9.138` |
| `cargo-llvm-cov` | `0.8.7` |
| `cargo-fuzz` | `0.13.2` |

CI、开发容器或安装脚本必须指定完整版本，例如：

```bash
cargo install cargo-audit --version 0.22.2 --locked
```

不得使用 `cargo install <tool>` 安装浮动最新版。

普通构建、测试和运行不得使用 nightly。只有 `cargo-fuzz`、Miri 或确实依赖 nightly 的诊断任务可以使用固定的 `nightly-2026-06-22`，调用时必须显式写出：

```bash
cargo +nightly-2026-06-22 fuzz run <target>
cargo +nightly-2026-06-22 miri test
```

禁止使用未带日期的 `nightly`。

使用 Miri 前必须执行 `rustup component add miri --toolchain nightly-2026-06-22`；使用覆盖率前必须执行 `rustup component add llvm-tools-preview --toolchain 1.96.0`。安装命令中的 toolchain 不得省略。

### 4.4 固定服务与容器版本

需要对应服务时使用以下基线：

| 服务 | 镜像与版本 |
|---|---|
| Keycloak | `quay.io/keycloak/keycloak:26.6.3` |
| PostgreSQL | `postgres:18.4` |
| NATS | `nats:2.14.2` |
| Apache Kafka | `apache/kafka:4.3.0` |
| Qdrant | `qdrant/qdrant:v1.18.2` |

`compose.yaml` 中不得使用 `latest`、浮动 major/minor tag 或本地未版本化镜像。提交时必须将镜像写为 `name:tag@sha256:<digest>`；表中的 tag 用于说明版本，digest 用于保证实际拉取内容不漂移。不同 CPU 架构导致 digest 不同时，必须在 README 和启动脚本中明确支持的架构及对应 digest。

操作系统包、数据库扩展、Keycloak provider、Kafka 插件和模型服务也必须使用精确版本。测试 fixture 和协议样例必须记录生成它们的软件版本。

真实模型模式不依赖 snapshot 保证可重复性。模型供应商无法长期承诺特定 snapshot 持续可用，因此 Agent 工程必须把“模型会变”作为正常假设处理。代码与配置文件中引用模型时使用 model family name（如 `gemini-3-flash`、`claude-sonnet-4-6`），`latest`、`default` 等会随供应商主版本跳跃的标识符仍禁止使用，因为它们跨越能力等价类。

真实模型调用必须同时满足以下三条不变量：

1. **模型指纹留痕**：每次真实调用必须记录响应中返回的实际模型标识、API 版本和供应商，写入结构化日志；涉及测试快照时必须将该字段纳入快照，但快照断言只比对输出内容，不比对指纹字段值。
2. **能力等价类声明**：每个使用真实模型的概念必须在 README 中声明所依赖的能力等价类（如 `tier: flash` / `tier: pro` / `tier: opus`）。同等价类内的模型替换不得需要修改概念源码；跨等价类替换走 §4.5 升级流程。
3. **评测门禁吸收变化**：模型替换或漂移后，必须由该概念的评测集判定通过与否，不得仅凭“调通了”判断兼容。无评测集的概念不得依赖真实模型作为验收手段；此类概念的 `cargo test` 必须使用 mock 或固定响应。

### 4.5 版本升级规则

版本升级必须作为独立变更处理，不得在实现普通概念时顺带升级。升级时必须：

1. 核对官方 release notes、安全公告、MSRV、迁移指南和协议兼容性。
2. 明确列出旧版本、新版本、升级原因和受影响概念。
3. 先修改一个代表性概念并通过回归、性能和兼容性测试。
4. 使用 `cargo update -p <crate> --precise <version>` 定向更新。
5. 审查 `Cargo.lock` 的传递依赖变化，禁止无关的大范围刷新。
6. 重新执行格式化、Clippy、测试、漏洞扫描、许可证检查和 smoke test。
7. 更新本节的固定版本、核对日期及相关 README。

安全修复可以突破“普通变更只修改一个概念”的默认规则，但必须保持独立提交、说明修复范围并完成受影响项目回归。

## 5. 实践概念目录

编号表示建议学习顺序，不表示项目之间存在依赖。

### 5.1 单 Agent 基础

1. `structured-output`：使用 JSON Schema 或等价约束解析和校验模型输出。
2. `tool-calling`：定义工具、解析参数、执行调用并将结果返回模型。
3. `agent-loop`：实现模型、动作、观察循环及明确停止条件。
4. `react-agent`：交替执行结构化决策、行动和观察；不得要求模型暴露或持久化隐藏思维链。
5. `plan-and-execute`：先生成计划，再逐步执行计划。
6. `dynamic-replanning`：根据执行结果修订剩余计划。
7. `reflection-agent`：失败后生成结构化反思并重试。
8. `evaluator-optimizer`：由评估器反馈驱动生成器迭代改进。
9. `self-consistency`：生成多个候选并聚合或选择结果。
10. `error-recovery`：分类处理模型错误、工具错误、超时和替代动作。

### 5.2 上下文、知识与记忆

11. `context-management`：选择、排序、裁剪和预算化上下文。
12. `conversation-memory`：维护当前会话的短期对话历史。
13. `summary-memory`：压缩历史并验证关键信息未丢失。
14. `episodic-memory`：保存和检索历史任务经历。
15. `semantic-memory`：按语义检索长期知识。
16. `procedural-memory`：保存可复用步骤、策略和行为规则。
17. `basic-rag`：完成切分、索引、检索和基于证据生成。
18. `hybrid-retrieval`：组合关键词检索和向量检索。
19. `reranking`：对召回结果进行二阶段排序。
20. `grounded-answering`：只根据证据回答，并返回可定位的来源。

### 5.3 工作流与编排

21. `prompt-chaining`：实现固定多步骤流水线。
22. `request-routing`：根据输入分类选择处理分支。
23. `parallel-fanout`：并行执行独立子任务并聚合结果。
24. `orchestrator-workers`：动态拆分任务并调度 worker。
25. `map-reduce-agent`：批量映射处理后归并结果。
26. `state-machine-agent`：使用显式状态和条件转换驱动执行。
27. `graph-workflow`：实现节点、边、条件分支和循环。
28. `durable-execution`：实现检查点、暂停、恢复和安全重放。
29. `human-in-the-loop`：支持人工审批、修改、拒绝和恢复。
30. `event-driven-agent`：由事件异步触发、推进和结束任务。

### 5.4 多 Agent 与 Subagent

31. `supervisor-agents`：监督者选择和管理专业 Agent。
32. `agent-handoff`：当前 Agent 将控制权和必要上下文交给另一 Agent。
33. `role-based-team`：按规划、研究、执行等角色协作。
34. `debate-and-vote`：独立论证、互评和投票。
35. `blackboard-collaboration`：多个 Agent 操作项目内部共享任务板。
36. `peer-to-peer-agents`：实现无中心协调的点对点通信。
37. `planner-executor-reviewer`：分离规划、执行和审查职责。
38. `competitive-agents`：多个方案竞争并由裁判选择。
39. `consensus-agents`：通过多轮协商形成一致结论。
40. `subagent-spawn`：父 Agent 创建具有独立 session、上下文、预算和生命周期的 Agent 实例。
41. `subagent-lifecycle`：管理启动、运行、取消、超时和状态查询。
42. `subagent-result-return`：子任务完成后向父 Agent 返回结构化结果。
43. `subagent-context-isolation`：只传递明确任务输入，不复制父会话全部上下文。
44. `subagent-concurrency`：限制并发量、排队长度、递归深度和资源预算。

Subagent 项目必须体现以下不变量：

- 每个 Subagent 使用独立 session ID。
- 父 Agent 只能显式选择要传递的上下文。
- Subagent 不能读取父 Agent 的凭据存储。
- 子任务失败不得破坏父任务状态。
- 父任务结束时必须取消或接管尚未完成的子任务。

### 5.5 Skill 与扩展

45. `dynamic-skill-plugin`：使用 Rust 原生动态插件作为 Skill 实现载体，实践运行时发现和加载。

该项目至少包含：

- `skill-host` 主程序和一个独立编译的 `cdylib` 示例插件。
- 使用 `libloading` 或等价机制加载动态库。
- 版本化 C ABI；禁止跨动态库直接传递 Rust trait object。
- Skill 元数据：名称、版本、描述、ABI 版本和能力列表。
- 初始化、执行和释放生命周期。
- 使用 JSON 字节作为 ABI 输入输出边界。
- 插件目录扫描、重复名称处理、allowlist 和 ABI 版本校验。
- 加载审计、执行超时和结构化错误。

原生动态库不是安全沙箱。该项目只允许加载可信插件，并必须在 README 中明确进程崩溃、内存安全和任意代码执行风险。

### 5.6 互操作协议

46. `mcp-server`：自行实现最小 MCP Server，暴露工具、资源和提示。
47. `mcp-client`：发现并调用 MCP Server 能力，处理协议错误和授权挑战。
48. `a2a-agent-card`：描述和发现远程 Agent 的身份与能力。
49. `a2a-task-delegation`：提交远程任务、跟踪状态并接收结果。

MCP 用于 Agent 与工具或资源连接；A2A 用于独立 Agent 之间发现和委托。不得将两者合并为同一个抽象。

协议项目必须在 README 中锁定实现版本、列出兼容版本和未实现能力，并提供版本不兼容测试。MCP 项目默认以 `2025-11-25` 规范为目标，至少覆盖生命周期、能力协商、JSON Schema 2020-12、stdio 或 Streamable HTTP 中的一种传输，以及与所选传输一致的授权边界。A2A 项目默认以 v1.0 为目标，至少覆盖版本协商、Agent Card、任务生命周期和一种协议绑定。

上述协议基线最后核对日期为 2026-06-23。实现前必须重新核对 [MCP 官方规范](https://modelcontextprotocol.io/specification/2025-11-25)、[A2A 官方规范](https://a2a-protocol.org/latest/specification/) 和相关 IETF RFC；如果最新版已经变化，应在目标项目 README 中说明选择旧版或新版的原因。

### 5.7 Agent Gateway

50. `agent-gateway`：实现一个受 OpenClaw 架构启发的最小控制平面。

```text
Client / Channel
       │ WebSocket + JSON
       ▼
Agent Gateway
 ├── connection registry
 ├── authentication
 ├── session registry
 ├── message router
 ├── agent runtime adapter
 ├── approval manager
 └── event broadcaster
       │
       ▼
Independent agent sessions
```

最小协议包含：

- `connect`：认证客户端并声明能力。
- `message.send`：向指定 session 发送消息。
- `session.create`、`session.list`、`session.close`。
- `agent.run`、`agent.cancel`、`agent.status`。
- `approval.request`、`approval.resolve`。
- `event.subscribe`：订阅 token、工具调用、状态和完成事件。

每条消息必须包含 request ID；相关操作必须包含 session ID 和 agent ID。错误必须使用稳定的结构化错误码。

Gateway 负责连接、路由、session、审批和事件，不负责具体业务 Agent 逻辑。同一 session 的消息串行执行，不同 session 可以并发。实现必须包含认证、心跳、背压、超时、取消、断线处理和优雅关闭。

教学实现采用单进程、单节点和内存状态，信任模型为单一可信操作者。不得宣称它能隔离敌对多租户。

### 5.8 生产级能力

51. `input-output-guardrails`：校验输入、模型输出和工具参数。
52. `tool-permissions`：实现工具白名单、最小权限和审批策略。
53. `sandboxed-execution`：限制文件、命令、网络和资源使用。
54. `prompt-injection-risk-reduction`：隔离不可信内容、数据和控制指令，限制其影响范围；不得宣称能够彻底消除 prompt injection。
55. `tracing-observability`：记录模型调用、工具调用和执行轨迹。
56. `trajectory-evaluation`：同时评测最终结果和中间决策轨迹。
57. `budget-control`：限制 token、成本、步骤数和执行时间。
58. `resilience-patterns`：实现超时、重试、退避、熔断和幂等。
59. `model-routing-fallback`：按能力、成本和失败情况切换模型。
60. `deterministic-replay`：使用固定响应重放完整执行过程。
61. `long-running-agent`：管理后台任务、取消、续期和状态查询。

生产级安全实验必须基于明确威胁模型，覆盖工具和连接器 SSRF、DNS rebinding、重定向、内网访问、工具输出外传、RAG/记忆/Skill/Agent Card 投毒、依赖与构建供应链、secret 轮换、审批内容绑定和安全降级。动态插件必须校验来源、哈希或签名、版本和 allowlist；但这些校验不能把进程内原生插件变成安全沙箱。

### 5.9 企业系统集成

62. `oidc-enterprise-login`：使用 Keycloak Authorization Code + PKCE 完成企业登录。
63. `enterprise-api-connector`：将企业 REST API 封装为 Agent 工具。
64. `webhook-agent-trigger`：验证 Webhook 后将事件转换为 Agent 任务。
65. `event-bus-integration`：通过 Kafka 或 NATS 接收任务并发布结果。
66. `connector-capability-discovery`：发现连接器的资源、动作、scope 和风险级别。
67. `enterprise-data-boundary`：进入模型前执行字段选择、脱敏和租户校验。
68. `enterprise-integration-gateway`：统一处理连接器发现、凭据获取、调用和审计。

企业集成项目必须：

- 使用项目自己的 Keycloak 容器、realm 配置、客户端和测试用户。
- 使用真实 OAuth/OIDC 网络交互，不得用伪造 JWT 替代授权服务器。
- 使用项目内模拟企业 API，确保实验可重复且不要求云厂商账号。
- 对分页、限流、超时、重试、幂等和错误映射进行显式处理。
- 验证 Webhook 签名、时间戳和重放窗口。
- 在数据进入模型前执行最小字段选择和敏感信息处理。

### 5.10 Agent 代表用户执行

69. `delegated-user-token`：Agent 使用用户 access token 访问受保护资源。
70. `oauth-token-exchange`：按 RFC 8693 使用 Keycloak Standard Token Exchange。
71. `on-behalf-of-agent`：实现从用户登录到下游 API 调用的完整委托链。
72. `audience-bound-token`：保证 token 只能用于指定资源。
73. `scope-down-delegation`：将委托权限缩减到当前动作所需最小 scope。
74. `tool-level-authorization`：在每次工具执行前进行授权判断。
75. `delegation-chain`：记录用户、Agent、Subagent 和下游 API 的委托关系。
76. `subagent-identity-propagation`：为 Subagent 签发任务专用的缩减权限 token。
77. `step-up-authorization`：高风险操作要求重新认证、额外 scope 或人工审批。
78. `delegated-token-lifecycle`：处理 token 过期、刷新、撤销、登出和任务取消。
79. `delegated-access-audit`：记录主体、代理、权限、资源、动作和结果。
80. `confused-deputy-defense`：防止 Agent 被诱导使用错误主体或高权限凭据。

完整代表用户执行流程为：

```text
User
  │ OIDC Authorization Code + PKCE
  ▼
Agent Application
  │ user access token
  ▼
Keycloak Token Exchange
  │ audience-bound, scope-down delegated token
  ▼
Enterprise API
```

### 5.11 评测工程

81. `eval-dataset-design`：构建黄金集、对抗集、失败样本集，隔离开发集和验收集，并检查评测数据污染。
82. `deterministic-evaluation`：对结构、协议、工具选择、参数和业务规则执行确定性评分。
83. `llm-as-judge`：设计评分量表，处理位置偏差、长度偏差、重复评审和人工校准。
84. `retrieval-evaluation`：评测 Recall@K、MRR、nDCG、引用正确率、答案忠实度和无证据拒答。
85. `agent-regression-gate`：在模型、prompt、工具、知识库或代码变化后执行自动回归门禁。触发条件包括 model family 内 snapshot 变更或响应模型指纹漂移。
86. `online-evaluation`：实践 shadow、canary、A/B、用户反馈、线上失败采样和聚类。
87. `evaluation-statistics`：计算置信区间、方差和显著性，避免根据单次运行或过小样本得出结论。

评测项目必须包含非 Agent 基线，并至少报告任务成功率、非法动作率、工具调用正确率、p50/p95 延迟、token 使用量和估算成本。使用模型评审时必须保留可复核样本，并用人工标注检查评审器与人的一致性。

### 5.12 分布式运行与生产运维

88. `persistent-task-queue`：实现任务持久化、租约、可见性超时、确认和 worker 崩溃恢复。
89. `idempotent-side-effects`：实现幂等键、去重、outbox/inbox 和不可逆操作的补偿边界。
90. `distributed-agent-runtime`：实现多实例任务所有权、租约续期、故障转移和孤儿任务回收。
91. `multi-tenant-runtime`：实现租户隔离、配额、公平调度和 noisy-neighbor 防护。
92. `slo-alerting`：定义并监控可用性、延迟、任务成功率、成本和队列积压指标。
93. `deployment-rollout`：实现配置和 prompt 版本、feature flag、canary、兼容迁移和 rollback。
94. `incident-response`：演练检测、止损、审计保全、降级、恢复和无责事故复盘。

### 5.13 数据、检索与记忆生命周期

95. `document-ingestion`：处理文档解析、OCR、表格、附件、元数据、校验和重复文档。
96. `index-lifecycle`：实现增量更新、删除传播、重建、embedding 迁移和索引版本切换。
97. `acl-aware-retrieval`：在召回前执行租户和资源权限过滤，禁止先召回敏感内容再遮盖。
98. `retrieval-freshness`：处理索引延迟、数据时效、过期证据和来源版本。
99. `memory-governance`：实现记忆来源、用户同意、TTL、更正、导出和删除。
100. `memory-poisoning-defense`：验证记忆来源，隔离不可信写入并处理记忆冲突和污染。

### 5.14 模型能力适配

101. `model-capability-adapter`：协商模型能力，处理上下文窗口、JSON Schema 方言、token 计数、prompt cache、限流、能力等价类划分与同等价类内模型替换、模型版本差异。
102. `streaming-tool-calling`：解析增量文本和增量工具参数，处理流中断、重复片段、取消、续传和最终一致性。

模型升级必须先通过离线回归，再进入 shadow 或 canary。实现必须保留模型和配置版本，并支持回滚；不得依赖供应商别名始终指向相同行为。

### 5.15 浏览器、计算机操作与多模态

103. `browser-agent`：基于 DOM 或可访问性树执行导航、读取、表单操作和页面状态验证。
104. `computer-use-agent`：基于截图和界面状态执行动作，处理坐标漂移、页面变化和动作失败。
105. `unsafe-ui-action-approval`：在支付、删除、发送、提交和权限变更前展示准确影响并获得确认。
106. `multimodal-context`：统一处理文本、图像和文档输入，保留来源、内容类型、大小限制和不可信边界。

浏览器和计算机操作项目必须防止页面内容充当控制指令，并在执行前后验证目标、主体、参数和实际结果。审批内容必须与最终执行内容绑定，审批后发生变化必须重新审批。

### 5.16 Prompt、Context 与 Loop Engineering

107. `prompt-engineering`：设计版本化 prompt contract，覆盖指令层次、角色和目标、变量插值、分隔与转义、few-shot 示例、输出约束、拒答和降级行为，并通过评测而不是主观观感选择版本。
108. `context-engineering`：构建运行时上下文供应管线，覆盖来源注册、信任标签、权限过滤、检索、排序、去重、压缩、token 预算、时效、缓存失效和 provenance。
109. `loop-engineering`：构建可观测且可收敛的 Agent loop，覆盖显式状态、进度度量、动作前置条件、观察归一化、重复动作检测、停滞检测、预算、超时、取消、恢复和终止原因。

这三项不是到第 107 项才首次学习，而是对前面练习的综合工程化验收：

- Prompt Engineering 贯穿 `structured-output`、`tool-calling`、`prompt-chaining`、guardrails 和 evaluation。
- Context Engineering 贯穿上下文、记忆、RAG、权限边界、数据生命周期和多模态。
- Loop Engineering 贯穿 `agent-loop`、错误恢复、状态机、durable execution、预算、韧性和长期任务。

三类工程必须明确区分：

- Prompt Engineering 决定“如何向模型表达任务和契约”，不得承担权限判断、secret 注入或确定性业务校验。
- Context Engineering 决定“本次调用给模型哪些信息以及为什么”，不得把所有可用数据无差别塞入上下文。
- Loop Engineering 决定“何时再次调用模型、执行何种状态转换以及何时停止”，不得把无限重试或模型自称完成作为终止策略。

`prompt-engineering` 必须比较 zero-shot、few-shot 或结构化版本中的至少两个候选，使用固定评测集报告成功率、格式合规率、越权指令服从率、token 和延迟。prompt 模板必须将可信指令与不可信数据分开，变量必须按目标格式转义，prompt 变化必须具有版本和回归记录。

`context-engineering` 必须为每个上下文片段保存 source、trust level、tenant、权限、时间戳、版本、token 数和选择原因。测试必须证明无权限、过期、重复、冲突和低价值内容不会错误进入模型上下文，并量化不同上下文策略对质量和成本的影响。

`loop-engineering` 必须使用显式状态机或等价的类型化状态转换，定义每轮进度信号和不变量。至少实现最大步骤、墙钟超时、token/成本预算、连续无进展阈值、重复动作阈值、工具错误分类、取消传播和稳定终止原因；不得用递归无限创建新 loop 或 Subagent 规避预算。

### 5.17 综合毕业项目

110. `capstone-research-agent`：完成检索、引用、工具调用、审批、评测、可观测性和失败恢复的完整研究 Agent。
111. `capstone-enterprise-agent`：完成 Keycloak 登录、token exchange、Subagent、企业 API、权限撤销和审计链。
112. `capstone-production-operations`：完成持久化任务、多实例运行、滚动升级、负载测试、故障注入、SLO 和事故复盘。

综合项目必须在自身目录内独立实现所需能力，不得依赖其他概念目录。每个综合项目必须包含架构设计文档、ADR、威胁模型、评测报告、负载测试、成本分析、故障演练和已知限制。

## 6. 访问控制原则

访问控制围绕“Agent 代表用户执行”设计。

1. 严格区分用户委托权限、Agent 应用自身权限和后台机器身份权限。
2. 有用户参与的操作默认使用用户委托，不能退化为应用级服务账号调用。
3. Agent 不得使用 client credentials token 模拟用户。
4. 禁止 direct naked impersonation；委托必须来源于已验证的用户登录和受控 token exchange。
5. 入口 token 和下游 token 必须具有不同且正确的 audience。
6. 下游 token 的 scope 必须等于或小于用户原有权限，并限制为当前工具调用所需权限。
7. 模型不能自行声明权限；授权决策必须发生在工具执行边界。
8. Token 不得进入模型上下文、prompt、工具输出、日志、trace、错误消息或任务结果。
9. Subagent 不得继承原始用户 token，只能获得任务专用的短期缩减权限 token。
10. 写操作、批量操作、资金相关操作和管理操作默认要求 step-up 或人工审批。
11. 用户权限撤销、登出或任务取消后，不得继续刷新或交换新 token。
12. 每次调用必须保留用户主体、Agent 客户端、Subagent、目标资源、scope、审批和结果信息。
13. 必须按 access token profile 验证 token。JWT access token 按 RFC 9068 验证签名和适用 claims；opaque token 通过 RFC 7662 introspection 或授权服务器定义的等价机制验证。不得假设所有 token 都包含 `azp` 或其他非必需 claim。
14. 所有受保护调用都必须验证 issuer 或授权服务器来源、有效期、目标 audience/resource、scope 和主体绑定；适用时验证 client、authorized party、tenant、delegation chain 和 token type。
15. OAuth 实现遵循 RFC 9700；委托和 token exchange 遵循 RFC 8693；资源指示器使用 RFC 8707；MCP HTTP 授权发现适用时使用 RFC 8414 和 RFC 9728。

## 7. README 要求

每个概念的 README 必须包含：

- 所实践的概念及不解决的问题。
- 独立场景和预期行为。
- 架构与执行流程。
- 核心类型和协议边界。
- 所需环境变量及 `.env.example` 说明。
- 构建、运行和测试命令。
- 成功路径、失败路径和预期执行轨迹。
- 安全边界和已知限制。
- 真实模型模式与离线测试模式的使用方法。
- 使用的标准、RFC 或协议版本。
- 与普通程序、固定 Workflow 或单 Agent 的基线比较，以及为何需要当前复杂度。
- 质量、延迟、成本和安全指标的定义与测量方式。
- 数据来源、保留期限、删除方式和敏感信息处理方式。
- 模型、prompt、工具 schema、数据集和配置的版本记录方式。
- Prompt contract、上下文组成和 loop 状态转换的定义，以及三者之间的责任边界。

不得只提供代码而缺少可验证的运行说明。

## 8. 实现要求

- 为模型输出、工具输入、网络响应和持久化数据定义显式类型。
- 对外部输入使用结构化校验，不得依赖模型“保证格式正确”。
- Agent loop 必须设置最大步骤数、超时、取消和终止原因。
- Prompt 必须作为版本化输入管理，模板变量必须校验和转义；不可信内容不得与系统指令拼接为不可区分的文本。
- 每次模型调用必须通过显式 context builder 组装上下文，记录片段来源、权限、信任级别、选择原因和 token 预算。
- Loop 必须保留类型化状态和进度信息，检测重复动作、重复观察、状态振荡和连续无进展，并以稳定枚举表示终止原因。
- 不得要求、记录或展示模型隐藏思维链。需要解释执行过程时，记录结构化决策摘要、动作、观察、证据和终止原因。
- 所有外部调用必须设置超时。
- 重试必须有上限，只对可重试错误执行，并考虑幂等性。
- 并发任务必须设置上限并处理背压。
- 产生外部副作用的操作必须定义幂等性、重复提交、部分成功和补偿边界。
- 错误必须保留上下文，但不得泄露凭据和敏感数据。
- 使用 `tracing` 或等价机制记录结构化事件。
- 生产相关实验必须提供稳定 request ID、session ID、task ID 或 trace ID。
- trace 默认不得记录完整 prompt、模型输出、工具参数和工具结果；确需记录内容时必须显式启用、脱敏并设置访问控制和保留期限。
- 模型调用必须记录供应商、调用时使用的 model family、响应中返回的实际模型标识、声明的能力等价类、配置版本、token 使用量、延迟和终止原因。
- 数据进入模型、记忆、索引或日志前必须执行来源标记、租户校验和敏感字段处理。
- 测试不得依赖执行顺序、共享端口或其他概念已启动。

## 9. 测试与验收

每个概念必须在自身目录通过：

```bash
rustc --version
cargo --version
cargo metadata --locked --format-version 1
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo run --locked
```

`rustc` 和 `cargo` 输出必须与 `rust-toolchain.toml` 一致。`cargo metadata --locked` 不得修改 `Cargo.lock`；如果失败，必须先修复 manifest 和 lockfile 的不一致。

`cargo run` 只适用于可以在有限时间内完成并自行退出的命令行实验。对于服务或长期任务，验收必须设置总超时，并执行“启动服务、等待健康检查、运行 smoke test、发送关闭信号、验证优雅退出”的完整流程。

通用测试至少覆盖：

- 正常成功路径。
- 无效模型输出或协议消息。
- 工具参数校验失败。
- 外部服务超时和不可用。
- 重试耗尽。
- 取消和预算耗尽。
- 敏感信息不出现在日志和错误中。
- 重复请求、部分失败和幂等行为。
- 固定随机种子或固定响应下的可重复结果。
- 模型、prompt、配置或协议版本不兼容。

专项验收要求：

- **Subagent**：上下文隔离、并发上限、递归限制、取消、超时和结果回传。
- **Skill**：有效插件、ABI 不匹配、缺失符号、重复名称、无效 JSON 和超时。
- **Gateway**：认证失败、非法 session、session 内顺序、跨 session 并发、取消、断线、背压和审批。
- **MCP/A2A**：能力发现、协议错误、版本不兼容、超时和授权失败。
- **MCP**：初始化和能力协商、schema 方言、传输边界、资源 audience、token passthrough 拒绝和适用的授权发现。
- **A2A**：Agent Card 验证、版本协商、任务状态转换、流式或异步结果、取消和签名元数据。
- **企业集成**：分页、限流、Webhook 重放、消息重复消费和租户边界。
- **委托授权**：过期 token、错误 issuer、错误 audience、scope 不足、权限撤销和 step-up。
- **Confused deputy**：用户无权但 Agent 应用有权时仍必须拒绝。
- **审计**：能够还原用户到 Agent、Subagent 和下游 API 的完整执行链。
- **评测**：评测集隔离、评分器一致性、基线比较、置信区间和回归门禁。
- **RAG/记忆**：权限过滤、删除传播、索引陈旧、来源冲突、数据污染和无证据拒答。
- **分布式运行**：worker 崩溃、租约过期、重复投递、网络分区、孤儿任务和滚动升级。
- **浏览器/计算机操作**：页面注入、页面漂移、审批后参数变化、不可逆动作和结果验证。
- **Prompt Engineering**：模板注入、变量转义、指令冲突、few-shot 污染、输出不合规、版本回归和模型迁移。
- **Context Engineering**：权限过滤、来源追踪、token 超限、陈旧内容、重复内容、冲突内容、缓存失效和上下文降级。
- **Loop Engineering**：无进展、重复动作、状态振荡、错误重试风暴、预算耗尽、取消传播、崩溃恢复和终止原因。

涉及容器的项目必须提供健康检查和可重复的启动、清理方式。测试完成后不得依赖遗留容器或数据才能再次通过。

涉及解析器、协议或 FFI 的项目应根据风险使用 `proptest`、`cargo-fuzz`、Miri 或 sanitizer。涉及复杂并发状态的项目应根据风险使用 `loom` 或等价工具。超时、重试和租约测试应使用可控时间，不得依赖真实长时间等待。

涉及生产能力的项目必须执行故障注入，至少覆盖慢响应、连接中断、进程崩溃、部分写入和依赖不可用。只验证正常路径不得通过验收。

## 10. 新增概念检查清单

新增概念前确认：

- [ ] 该概念无法由现有项目通过一个小测试完整表达。
- [ ] 目录名使用小写 kebab-case。
- [ ] 项目拥有独立 Cargo 配置和锁文件。
- [ ] 项目包含固定为 Rust `1.96.0` 的 `rust-toolchain.toml`。
- [ ] 单 crate 或概念内 workspace 按规范使用 edition 2024、resolver 3 和 `rust-version = "1.96.0"`。
- [ ] 所有直接依赖使用完整精确版本，`Cargo.lock` 在 `--locked` 模式下有效。
- [ ] 未引用其他概念的代码、配置或数据。
- [ ] 端口、容器、数据库和身份配置独立。
- [ ] 所有容器镜像同时固定 tag 和 digest，未使用 `latest` 或浮动 tag。
- [ ] README 完整描述目标、边界、运行方法和预期结果。
- [ ] 提供无需真实模型 API 的核心测试。
- [ ] 定义可量化的成功标准，并提供复杂度更低的基线。
- [ ] 所有循环、并发、网络调用和重试均有明确上限。
- [ ] 外部副作用具有明确的幂等、审批和补偿策略。
- [ ] 数据来源、权限、保留、删除和版本边界已定义。
- [ ] 协议、模型、prompt、schema 和配置版本可追踪。
- [ ] 使用真实模型的概念已声明能力等价类、记录响应模型指纹、具备评测门禁；无评测门禁的概念不依赖真实模型作为验收手段。
- [ ] 已明确 Prompt Engineering、Context Engineering 和 Loop Engineering 的责任边界及各自评测指标。
- [ ] Prompt 变量经过校验和转义，上下文片段具有来源与权限，loop 具有进度检测和稳定终止原因。
- [ ] 对高风险项目完成威胁模型和故障注入计划。
- [ ] 凭据不进入仓库、模型上下文、日志或测试快照。
- [ ] 格式化、Clippy、测试和独立运行均通过。

## 11. 学习阶段与能力验收

不得把“完成全部概念”直接写成“具备资深能力”。学习成果按以下阶段判断：

### 11.1 基础实现能力

完成 1 至 30，并能独立解释模型接口、工具边界、Agent loop、上下文、RAG、状态机、取消、预算和失败恢复。该阶段证明具备独立实现基础 Agent 和 Workflow 的能力。

### 11.2 系统工程能力

完成 31 至 68，并通过至少一个跨进程协议项目和一个生产能力项目。学习者必须能够解释多 Agent 是否必要、控制平面与运行时边界、协议兼容性、安全边界和可观测性设计。

### 11.3 企业与授权能力

完成 69 至 80，并通过真实 Keycloak 网络交互、委托链审计、权限撤销和 confused deputy 测试。该阶段证明能够实现受控的代表用户执行，不代表已具备完整生产运营经验。

### 11.4 资深候选人验收

资深候选人必须额外满足：

1. 完成 81 至 106 中与目标岗位相关的大部分项目，并能够说明未选择项目的原因。
2. 完成 107 至 109，并能够分别诊断 prompt、context 和 loop 导致的失败，不能把三类问题混为“模型效果不好”。
3. 完成至少两个综合毕业项目，其中必须包含 `capstone-production-operations`。
4. 为综合项目建立非 Agent 基线，并用统计上可解释的评测证明采用 Agent 的收益。
5. 完成安全审查、负载测试、成本分析、故障演练、SLO、告警和事故复盘。
6. 能够在模型质量、延迟、成本、安全、隐私和可维护性之间做出有依据的取舍。
7. 能够指出不应使用 Agent、多 Agent、RAG、长期记忆或动态插件的场景。
8. 通过架构答辩和代码评审，能够解释关键不变量、失败模式、替代方案和演进路线。

资深能力最终由综合交付质量、决策质量和生产问题处理能力判断，不由代码行数、模式数量或模型调用次数判断。

### 11.5 综合项目评分

每个综合项目按以下维度评分，总分 100：

- 正确性与任务效果：20。
- 架构边界与可维护性：15。
- 评测质量与基线比较：15。
- 安全、身份与数据治理：15。
- 可靠性、幂等与故障恢复：15。
- 可观测性、SLO 与运营能力：10。
- 成本、延迟与容量分析：5。
- 文档、ADR 和答辩质量：5。

单项出现严重凭据泄露、跨租户访问、不可控副作用、伪造授权、评测数据污染或无法恢复的数据破坏时，综合项目直接不通过。资深候选人要求每个综合项目至少 80 分，且任一维度不得低于该维度满分的 60%。

## 12. Agent 协作规则

在本仓库工作的开发 Agent 必须：

1. 修改前先阅读本文件和目标概念的 README。
2. 先检查工作区状态，保留用户已有且与任务无关的修改。
3. 将变更限制在用户指定的概念；除非任务明确要求，不创建共享基础设施。
4. 优先实现可运行的最小闭环，再补充边界测试。
5. 不因代码相似而抽取跨概念共享模块；本仓库以概念独立性优先。
6. 不提交真实凭据，也不在示例中使用看似真实的 secret。
7. 完成后运行与变更风险相称的检查，并明确报告未运行的检查。
8. 如果标准或外部 API 已变化，先核对当前官方规范，再实现对应行为。
9. 不得为了展示 Agent 能力而引入不必要的模型调用、多 Agent、长期记忆或动态执行。
10. 新增协议或安全实现时，必须记录核对的官方规范版本和日期；博客、二手文章和框架行为不得替代规范。

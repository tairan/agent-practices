# 实践项目契约

## 标准目录

```text
practices/<concept-name>/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── practice.toml
├── acceptance.toml
├── README.md
├── .env.example
├── src/
├── tests/
├── fixtures/
└── evidence/
```

不需要环境变量、fixture 或提交型 evidence 的项目可以省略对应路径。身份系统或外部服务可以增加 `compose.yaml` 和项目自己的 `keycloak/realm.json`。

## `practice.toml`

每个项目必须声明范围和适用规则：

```toml
schema-version = 1
id = "tool-calling"
title = "Tool Calling"
stage = "foundation"
profiles = ["base", "model", "prompt", "tool"]
competencies = ["model", "tool-loop", "evaluation"]

[scope]
solves = "解析、校验并执行一次模型工具调用"
does-not-solve = "长期任务、分布式执行和敌对插件隔离"

[acceptance]
spec = "acceptance.toml"
```

`stage` 只能是 `foundation`、`systems`、`specialization` 或 `capstone`。`profiles` 必须满足 [`AGENTS.md`](../AGENTS.md#3-适用性-profile)；`competencies` 必须引用 [`assessment/rubric.md`](../assessment/rubric.md#能力域) 中的能力域。

## `acceptance.toml`

每个项目必须在实现或调优前定义验收契约：

```toml
schema-version = 1
contract-version = 1

[baseline]
name = "deterministic-router"
description = "不调用模型的规则路由"
command = "baseline-evaluation"
dataset = "fixtures/acceptance.jsonl"
scorer = "exact-task-outcome-v1"
artifact = "evidence/baseline.json"

[[metric]]
name = "task_success_rate"
direction = "higher"
threshold = 0.90
sample-size = 100
dataset = "fixtures/acceptance.jsonl"
scorer = "exact-task-outcome-v1"
command = "offline-evaluation"
artifact = "evidence/evaluation.json"

[[metric]]
name = "illegal_action_rate"
direction = "lower"
threshold = 0.00
sample-size = 100
dataset = "fixtures/acceptance.jsonl"
scorer = "policy-violation-v1"
command = "offline-evaluation"
artifact = "evidence/evaluation.json"

[budgets]
p95-latency-ms = 2000
max-estimated-cost-usd = 1.00
measurement-command = "offline-evaluation"
artifact = "evidence/evaluation.json"

[[command]]
name = "baseline-evaluation"
run = "cargo run --locked --bin evaluate-baseline -- --dataset fixtures/acceptance.jsonl --output evidence/baseline.json"
required = true

[[command]]
name = "offline-tests"
run = "cargo test --locked"
required = true

[[command]]
name = "offline-evaluation"
run = "cargo run --locked --bin evaluate -- --dataset fixtures/acceptance.jsonl --output evidence/evaluation.json"
required = true
```

要求：

- 指标定义、方向、样本来源、阈值和预算必须在查看最终结果前确定。
- 修改阈值必须增加 `contract-version` 并解释原因，不得为通过测试而事后降低标准。
- 不适用的 baseline、指标或预算必须在 README 说明理由。
- 随机模型评测必须记录重复次数、方差或置信区间。
- 真实模型成本可以是可选验收，不得成为核心测试的强制依赖。
- command 必须能够从当前概念目录执行，不得依赖其他概念或根目录工具。
- 每个 metric 必须通过 `command`、`scorer` 和 `artifact` 映射到可执行评分器和机器可读结果；预算必须声明测量命令和结果 artifact。
- baseline 必须声明 `command`、`dataset`、`scorer` 和 `artifact`，并与目标实现使用同一输入版本和评分语义。纯确定性测试可以使用 `artifact = "process:exit-code"`，但测试名和断言必须足以复核每个样本的预期结果。
- 每个项目至少保留一个结果质量指标和一个安全或契约不变量指标，不得将 baseline、全部指标和全部预算同时标记 N/A。
- 随机评测必须在契约中预注册 seed 或 seed 生成规则、重复次数、置信区间方法和置信水平。

随机评测使用以下固定字段和类型；确定性项目可以省略整个表：

```toml
[stochastic]
enabled = true
seed = 42
seed-policy = "fixed"
repetitions = 10
confidence-method = "bootstrap"
confidence-level = 0.95
```

已有项目首次补录契约时必须在 README 标记迁移日期，说明该版本不能证明历史实现经过预注册；该契约只对补录后的实现、调优和验收生效。不得据此追溯声称旧结果满足预注册要求。

## 实验任务书

项目 README 必须在编码前明确：

1. 学习目标和前置知识。
2. 独立问题场景和非 Agent baseline。
3. 必须自行实现的核心机制。
4. 可以使用的库和明确禁止的捷径。
5. 成功输入、失败 fixture 和故障注入计划。
6. 验收指标、命令和预期 evidence。
7. 常见错误和不在范围内的扩展挑战。

使用 SDK 时必须说明 SDK 负责的部分，不能让 SDK 的自动执行、自动重试、隐式记忆或托管 runtime 替代目标机制。

## Evidence

`evidence/` 只提交可复核且不包含凭据、敏感数据或完整生产内容的材料。适用时包括：

- 评测摘要和机器可读结果。
- 失败样本分类和修复前后比较。
- 架构图、ADR、威胁模型和数据流图。
- 负载测试、故障注入、恢复和 rollback 记录。
- 人工评审一致性报告。
- 盲测或答辩评分记录。

生成型大文件、原始 trace 和可能含敏感内容的数据不得默认提交。README 必须说明重新生成方式、保留策略、脱敏方式和未执行检查。Evidence 必须标记生成时间、代码或配置版本、数据集版本和执行环境；手工编辑的摘要不能替代原始机器可读结果的可复现方式。

## README 必备内容

每个 README 必须包含：

- 目标、非目标、适用 profile 和能力域。
- 实验任务书、独立场景、预期行为和较简单 baseline。
- 架构、执行流程、核心类型和边界。
- `practice.toml` 与 `acceptance.toml` 的关键决策。
- 环境变量和 `.env.example` 说明。
- 构建、运行、离线测试和可选真实模型命令。
- 成功、失败和恢复路径的预期轨迹。
- Prompt contract、context 组成、loop 状态转换及其责任边界；不适用时写 `N/A: <理由>`。
- 工具风险、副作用、审批和授权边界。
- 数据来源、外部处理方、保留、删除和敏感信息处理。
- 项目实际采用的模型、prompt、工具 schema、数据集和配置版本；共享默认值链接技术基线，项目偏离必须说明理由。
- 质量、延迟、成本、安全和人机交互指标。
- 项目使用的标准、RFC 和协议及其技术基线链接；共享版本与最后核对日期不得在项目内复制维护。
- 安全边界、已知限制、N/A 项和 evidence 复现方式。

声明 [`AGENTS.md` 定义的高风险 profile](../AGENTS.md#3-适用性-profile) 或 `capstone` stage 的项目还必须包含 ADR、威胁模型、数据流图、评测报告、runbook 和事故复盘。不得只提供代码或成功示例。

## Base 验收

每个概念必须在自身目录执行：

```bash
expected_toolchain="$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml)"
test "$(rustc --version | awk '{print $2}')" = "$expected_toolchain"
test "$(cargo --version | awk '{print $2}')" = "$expected_toolchain"
cargo metadata --locked --format-version 1
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
```

可在有限时间退出的可执行项目还必须执行 `cargo run --locked`。服务或长期任务必须使用总超时完成启动、健康检查、smoke test、关闭信号和优雅退出，不得直接无限运行。

Base 测试至少覆盖成功路径、无效外部输入、错误不泄露敏感信息、固定响应下可重复，以及版本或配置不兼容。不存在对应输入或配置时可以标记 N/A。

测试不得依赖执行顺序、共享端口、其他概念已启动或前一次运行遗留状态。并行测试需要独立资源命名；涉及端口时必须使用项目独占配置或运行时分配并避免跨测试共享。

## Profile 验收

| Profile | 最低专项测试 |
|---|---|
| `model` | 无效响应、拒答、限流、超时、能力缺失、模型指纹变化 |
| `prompt` | 变量转义、指令冲突、不可信内容、输出不合规、版本回归 |
| `context` | 上游访问结果或公开数据 inclusion policy、来源、token 超限、陈旧、重复、冲突、缓存失效；自行作出受保护资源授权决策时同时声明 `auth` |
| `loop` | 最大步骤、无进展、重复动作、振荡、预算、取消、稳定终止 |
| `tool` | schema、参数、权限、错误映射、超时、结果大小 |
| `network` | 慢响应、不可用、断连、重试耗尽、重定向策略 |
| `stateful` | 崩溃恢复、重复请求、部分写入、删除传播、版本迁移 |
| `side-effect` | 幂等、审批绑定、参数变化、部分成功、补偿边界 |
| `auth` | issuer、audience、scope、过期、撤销、step-up、confused deputy |
| `protocol` | 生命周期、能力协商、非法消息、版本不兼容、授权失败、跨实现 |
| `distributed` | worker 崩溃、租约、重复投递、网络分区、孤儿任务、滚动升级 |
| `untrusted-code` | allowlist、完整性、资源限制、超时、崩溃和恶意输入 |
| `browser` | 页面注入、漂移、审批后变化、不可逆动作和结果验证 |
| `production` | 负载、故障注入、告警、降级、rollback 和事故复盘 |

专项补充要求：

- Subagent 覆盖上下文隔离、并发、递归、取消、超时和结果回传。
- Skill 覆盖 ABI 不匹配、缺失符号、重复名称、无效 JSON 和插件超时。
- Gateway 覆盖认证失败、非法 session、session 内顺序、跨 session 并发、断线、背压和审批。
- MCP 覆盖初始化、能力协商、schema 方言、传输、resource audience、token passthrough 拒绝和授权发现。
- A2A 覆盖 Agent Card、版本协商、任务状态、异步或流式结果、取消和签名元数据。
- RAG 和记忆覆盖 ACL、删除传播、陈旧索引、来源冲突、投毒和无证据拒答。
- 评测覆盖数据集隔离、评分器一致性、baseline、置信区间和回归门禁。

解析器、协议或 FFI 项目应该根据风险使用 `proptest`、fuzz、Miri 或 sanitizer。复杂并发项目应该使用 `loom` 或等价工具。超时、重试和租约测试必须使用可控时间，不得依赖真实长等待。

容器项目必须提供健康检查和可重复清理方式，测试不得依赖遗留容器或数据。

## 新增概念检查清单

- [ ] 现有项目不能通过一个小测试完整表达该能力。
- [ ] 概念名使用小写 kebab-case。
- [ ] `practice.toml` 已声明 scope、profile 和能力域。
- [ ] `acceptance.toml` 已预先定义 baseline、指标、阈值和预算。
- [ ] 项目拥有独立 Cargo 配置、lockfile 和固定工具链。
- [ ] 未引用其他概念的代码、配置、数据或执行结果。
- [ ] 端口、容器、数据库和身份配置独立。
- [ ] 直接依赖和容器镜像使用完整版本，镜像同时固定 tag 和 digest。
- [ ] 只实现适用 profile 的要求，不为合规添加无关复杂度。
- [ ] 核心测试不依赖真实模型 API。
- [ ] 定义可量化成功标准和较简单 baseline。
- [ ] loop、并发、网络、重试和资源具有明确上限。
- [ ] 副作用具有幂等、审批、部分成功和补偿策略。
- [ ] 数据来源、权限、供应商处理、保留、删除和版本已定义。
- [ ] 模型、prompt、context policy、工具 schema、协议和配置可追踪。
- [ ] 高风险 profile 已完成威胁模型和故障注入计划。
- [ ] 凭据不会进入仓库、模型上下文、日志、trace 或 evidence。
- [ ] README 包含边界、运行方法、N/A 理由和预期 evidence。

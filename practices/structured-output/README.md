# structured-output

`agent-practices` 的基础实践，见[模型与单 Agent 基础目录](../../docs/practice-catalog.md#模型与单-agent-基础)。

> "使用 JSON Schema 或等价约束解析和校验模型输出。"

This concept implements one tightly-scoped capability: take a single model
call, locate a JSON object inside the response, validate it against a
JSON Schema (Draft 2020-12), and return either a typed Rust value or a
structured error. **Nothing else.**

---

## 1. 所实践的概念及不解决的问题

**实践**:从模型自然语言输出中可靠地拿到符合契约的结构化数据,核心三段:

1. **抽取**(`src/extract.rs`):剥 markdown 围栏、跳过字符串字面量内的大括号、找平衡的顶层 `{ ... }` 块。
2. **校验**(`src/schema.rs`):用 `jsonschema` Draft 2020-12 校验,一次性收集所有 issue。
3. **反序列化**:校验通过后 `serde_json::from_value::<T>`;此处再失败属于"schema 与 Rust 类型不一致"的真 bug,单独用 `TypedDeserializeFailed` 标记。

**显式不解决**(由后续 concept 承担):

| 不做的事 | 由哪个概念承担 |
|---|---|
| 失败重试 / 退避 | `resilience-patterns` |
| 反思与自动修复 JSON | `reflection-agent` / `error-recovery` |
| 工具调用编排 | `tool-calling` |
| Agent loop | `agent-loop` |
| 随机模型评测与统计 | `eval-dataset-design` / `evaluation-statistics` |
| Streaming | `streaming-tool-calling` |
| Token / 成本预算 | `budget-control` |
| Prompt A/B、优化和综合验收 | `prompt-engineering` |

特别地,**本概念刻意不使用 Gemini 的原生 `response_format` / structured-output**——练习的目的就是亲手实现 schema 校验。

---

## 2. 独立场景与预期行为

**场景**:把一段自由文本会议纪要(`fixtures/meeting_input.txt`)抽取为
结构化 `MeetingMinutes`(`src/lib.rs`):

```rust
pub struct MeetingMinutes {
    title: String,
    date: String,            // YYYY-MM-DD
    attendees: Vec<String>,
    decisions: Vec<String>,
    action_items: Vec<ActionItem>,
}
```

输入示例片段:

```
QA Standup, 2026-06-24
Attendees: Alice (Lead), Bob (Engineer), Carol (PM).
...
```

预期成功输出:`(MeetingMinutes, ModelCallMetadata)`,模型指纹和调用元数据见[真实模型模式](#9-真实模型模式-vs-离线测试模式)。

---

## 3. 架构与执行流程

```
                ┌──────────────────────────┐
                │ ModelClient (trait)      │
                ├──────────────────────────┤
                │ MockClient (固定响应)    │  ← 所有测试 + 默认 demo
                │ GeminiOpenAiClient (HTTP)│  ← STRUCTURED_OUTPUT_MODE=real / 有 API key
                └────────────┬─────────────┘
                             │ CompletionResponse { content, fingerprint, usage }
                             ▼
                ┌──────────────────────────┐
                │ extract::extract_json    │  ← 剥围栏 / 平衡括号 / 拒歧义
                └────────────┬─────────────┘
                             │ serde_json::Value
                             ▼
                ┌──────────────────────────┐
                │ schema::SchemaValidator  │  ← Draft 2020-12,一次性收集所有 issue
                └────────────┬─────────────┘
                             │ T : DeserializeOwned
                             ▼
                ┌──────────────────────────┐
                │ (T, ModelCallMetadata)   │
                └──────────────────────────┘
```

顶层入口:`extract_structured<T>(client, schema, request)` in `src/lib.rs`。

---

## 4. 核心类型和协议边界

| 类型 | 文件 | 边界 |
|---|---|---|
| `ModelClient` (trait) | `src/model/mod.rs` | 唯一的模型抽象;当前真实适配器仅验证 Gemini OpenAI 兼容端点 |
| `CompletionRequest` / `CompletionResponse` | 同上 | 单跳,无 streaming/tool/response_format 字段 |
| `ModelFingerprint` | 同上 | [Model contract](../../docs/handbook/engineering-contracts.md#model-contract) 中模型标识与能力记录的载体 |
| `MockClient` | `src/model/mock.rs` | 测试与离线 demo;不访问网络 |
| `GeminiOpenAiClient` | `src/model/gemini_openai.rs` | OpenAI 兼容 chat completions;只用 `model` / `messages` / `temperature` / `max_tokens` |
| `extract_json` | `src/extract.rs` | 纯函数,无 IO;失败模式枚举化 |
| `SchemaValidator` | `src/schema.rs` | 两阶段:`iter_errors` 全收集,再 `from_value::<T>` |
| `StructuredOutputError` | `src/error.rs` | 8 个 pipeline 变体覆盖每个失败点;错误不携带原始模型内容 |
| `ContextBuilder` / `ContextItem` | `src/context.rs` | 保存来源、provenance、信任、tenant、版本、时效、token 估算和选择理由 |
| `ModelCallMetadata` | `src/model/mod.rs` | 保留 fingerprint、usage、停止原因、过滤结果和调用延迟 |

协议:**OpenAI Chat Completions**(经 Gemini `v1beta/openai/` 兼容端点) +
**JSON Schema Draft 2020-12**。本项目只实现 OpenAI 协议的最小子集(单轮、
非流式、无工具),Schema 校验依赖 `jsonschema=0.46.5`(已确认支持 2020-12)。

---

## 5. 环境变量

见 `.env.example`。所有变量都**只在 `cargo run` 入口读取**——测试代码绝不读
`STRUCTURED_OUTPUT_MODE` 或 `GEMINI_API_KEY`,以保证 `cargo test --locked`
在任何环境下行为一致。

| 变量 | 必需? | 说明 |
|---|---|---|
| `STRUCTURED_OUTPUT_MODE` | 否 | `mock` / `real` / 未设;详见[模式解析表](#9-真实模型模式-vs-离线测试模式) |
| `GEMINI_API_KEY` | 仅 real | 不提交；仅存在 key 不会触发 real 模式 |
| `GEMINI_BASE_URL` | 禁止 | 凭据绑定到代码内固定的 Google 官方 HTTPS endpoint；设置该变量时 real 模式拒绝启动 |
| `GEMINI_MODEL_FAMILY` | 否 | 固定 `gemini-3.5-flash`；其他值在联网前拒绝 |
| `GEMINI_TIMEOUT_SECS` | 否 | 默认 30；real 模式严格接受 1..=120，非法值拒绝启动 |

---

## 6. 构建、运行、测试命令

**前提**:用户全局 shell 若 export 了 `RUSTUP_TOOLCHAIN`,会覆盖
`rust-toolchain.toml`。本仓库每个 concept 的 `rust-toolchain.toml` 都钉到
`1.96.1`,因此运行前请 unset:

```bash
unset RUSTUP_TOOLCHAIN
```

按[项目 Base 验收契约](../../docs/project-contract.md#base-验收)执行以下命令(均在本目录执行):

```bash
rustc --version                                                    # 期望 1.96.1
cargo --version                                                    # 期望 1.96.1
cargo metadata --locked --format-version 1 > /dev/null
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo run --locked --bin evaluate -- --output evidence/evaluation.json
cargo run --locked                                                 # 干净环境 → mock,退出 0
```

---

## 7. 成功路径、失败路径与预期执行轨迹

每个 `fixtures/mock_responses/*` 都在共享 evaluation manifest 中注册并由单元测试、
baseline 和 evaluator 使用；核心抽取分支另由 `tests/end_to_end_mock.rs` 覆盖:

| Fixture | 预期 |
|---|---|
| `ok.json` | 抽取 → 校验 → 反序列化 → `Ok((MeetingMinutes, fingerprint))` |
| `fenced.txt` | 剥围栏 → 后续同上 → `Ok` |
| `chatty.txt` | 候选扫描 → 唯一候选 → 后续同上 → `Ok` |
| `empty.txt` | `EmptyResponse` |
| `truncated.txt` | `JsonExtractionFailed` |
| `multi_json.txt` | `MultipleJsonCandidates { count: 2 }` |
| `missing_field.json` | `SchemaValidationFailed { issues: [..] }` |
| `type_mismatch.json` | `SchemaValidationFailed { issues: [..] }` |
| `missing_due_date.json` | Prompt 要求字段缺失，`SchemaValidationFailed` |
| `extra_field.json` | Serde 会忽略但 schema 必须拒绝，`SchemaValidationFailed` |

---

## 8. 安全边界和已知限制

- **不是生产代码**:无重试、无退避、无熔断、无审批。
- **错误默认不包含原始内容**:模型输出、provider 错误正文和 API key 均不进入
  `Display`、`Debug` 或入口日志;错误只保留稳定分类、状态码和安全元数据。
- **不调用 LLM 修复 JSON**:抽取失败就失败;自动修复属于 `reflection-agent`。
- **离线成本预算为 0**:`evaluate` 不调用模型;真实模式没有自动供应商花费上限,
  仅允许显式手工运行并应同时监控 Gemini 控制台用量。
- **本地原生 TLS**:`reqwest` 启用 `rustls`,默认载入系统证书。
- **HTTP 超时必须存在**:默认 30s,可通过 `GEMINI_TIMEOUT_SECS` 在 1..=120 秒内调整；
  非数字、0 或超过上限均拒绝启动。响应正文最大 1 MiB,客户端禁止自动重定向。

---

## 9. 真实模型模式 vs 离线测试模式

**模式解析表**(实现见 `src/main.rs::resolve_mode`，默认永不联网):

| `STRUCTURED_OUTPUT_MODE` | `GEMINI_API_KEY` | 实际模式 | 行为 |
|---|---|---|---|
| `mock` | 任意 | mock | 强制 mock |
| `real` | 已设 | real | 调用 Gemini 兼容端点 |
| `real` | 未设 | — | **exit 1**,refusing to call real provider |
| 未设 | 任意 | mock | 安全默认；仅存在 key 不会触发网络或费用 |
| 非法值 | 任意 | — | **exit 1** |

**模型指纹留痕示例**，遵循[模型接口契约](../../docs/handbook/engineering-contracts.md#model-contract)。
真实模式 stdout 末尾:

```
--- fingerprint ---
provider         = gemini-openai-compat
requested_family = gemini-3.5-flash
response_model   = <provider 返回的实际 model 字符串,可能是带日期后缀的 snapshot>
model_id_missing = false
api_version      = Some("v1beta")
capability_tier  = flash
usage            = <reported 或 unknown>
finish_reason    = <provider 值或 unknown>
content_filter   = <过滤枚举或 none>
```

`response_model` 进入 fingerprint 是为了让"模型漂移"在日志中可见,但
**测试不把单个 snapshot 值固化为常量**,但会验证响应 snapshot 的变化可被留痕；
请求 family 仍必须是当前唯一验证的 `gemini-3.5-flash`。

**能力等价类声明**，遵循[模型与 Provider 基线](../../docs/technology-baseline.md#模型与-provider-基线):
本概念当前只声明精确的 `gemini-3.5-flash` 适配类,依赖单轮非流式文本、
固定消息角色、正常停止原因和可选 usage。`ModelClient::capabilities` 分别记录
`provider_declared`、`adapter_implemented` 和 `conformance_status`；供应商声明支持的
stream、tool、structured output 与 cache 不会被误写成本适配器已实现，未知项保持
`unknown`。同一 Gemini 兼容等价类替换仍必须通过
真实 provider conformance;跨供应商或跨 `flash`/`pro` 等能力成本层级必须修改
适配器或走[版本升级流程](../../docs/technology-baseline.md#版本升级)。

`deferred`:本项目尚未执行带真实凭据的 provider conformance,因此不得声称已验证
真实端点兼容性;本地原始 HTTP fixture 只证明适配器的确定性协议边界。

**评测门禁说明**，遵循[评测与实验设计契约](../../docs/handbook/engineering-contracts.md#评测与实验设计):
本概念使用 10 个由 contract v4 manifest 逐项固定的 fixture 做确定性验收，并用
`tests/baseline.rs` 对照直接
`serde_json` 解析。它不包含随机真实模型评测，因此真实模型不构成验收手段；
`cargo test` 全部走 mock。真实模式仅供学习者手动观察响应。模型变化的统计评测
由 `eval-dataset-design`、`evaluation-statistics` 和 `agent-regression-gate` 承担。

---

## 10. 使用的标准

- **JSON Schema Draft 2020-12** —— <https://json-schema.org/draft/2020-12/schema>
- **OpenAI Chat Completions** —— Gemini `v1beta/openai/` 兼容子集
- **仓库共享标准与版本** —— 采用[技术基线](../../docs/technology-baseline.md)记录的版本和核对日期；本项目未声明偏离

---

## 11. 与基线比较:为何不直接 unwrap?

最低复杂度基线 = `serde_json::from_str::<MeetingMinutes>(&content).unwrap()`。
它**至少**在四种情况下崩溃或给出无效结果:

1. 模型把 JSON 包在 markdown ``` 围栏里(常见)。
2. 模型在 JSON 前后加客套话:"Sure, here is the JSON: ..."(更常见)。
3. JSON 类型对但缺字段(serde 报字段缺失,但 schema 还能给出更多信息:
   有几个问题、分别在什么 JSON Pointer 路径)。
4. JSON 字段多了 schema 不允许的内容(`additionalProperties: false`)——
   serde 默认会**忽略**而不是报错;只有 schema 能拦住。

本概念用显式边界代码换回:对 10 种结果与失败模式的明确分类、脱敏 issue
列表和可审计的 fingerprint 留痕。

---

## 12. 质量 / 延迟 / 成本 / 安全指标

| 指标 | 定义 | 测量方式 |
|---|---|---|
| Fixture 契约通过率 | 10 个 fixture 的成功或稳定错误分类符合预期的比例 | evaluator 与 `evaluation::tests` |
| 首次校验失败定位率 | 校验失败时 `issues.len() ≥ 1` 的比例 | 100%(测试中验证) |
| p50/p95 pipeline 延迟 | 10 fixture × 100 次完整抽取、schema 校验和反序列化 | `evaluate` 写入 `evidence/evaluation.json`;p95 ≤ 10ms |
| 真实调用延迟 | `GeminiOpenAiClient::complete` 端到端 | 取决于 Gemini;默认 HTTP 超时 30s |
| 敏感错误回显率 | canary 出现在错误 `Display`/`Debug` 的比例 | 0%;单元测试验证 |

---

## 13. 数据来源 / 保留 / 删除

- `fixtures/meeting_input.txt`、`fixtures/mock_responses/*`:**人工编写**,
  与任何外部数据无关;随仓库提交。
- 真实模式下 Gemini 端点接收 system prompt + 会议纪要原文;**会议纪要本身
  也是 fixture(不含真实人员/事项)**,不存在个人信息处理问题。
- 模型调用路径不写盘、不缓存、不持久化输入或输出。离线 evaluator 只写
  `evidence/evaluation.json`，其中不含 fixture 原文或模型原文。
- Evidence 保留于仓库；清理时删除该 JSON，使用 README 中的 evaluator 命令重建。
  产物记录 UTC epoch、实际命令、源码/配置与数据集的逐文件 SHA-256 清单及组合摘要、
  契约/prompt/context policy/数据集版本、实际 rustc/cargo 输出和 OS/架构；真实
  provider conformance 明确记录为未执行。
- Gemini 侧的保留、训练使用、处理地域和删除能力在本项目中均记录为
  `unknown`,未做供应商政策核验;因此真实模式只允许发送仓库内合成 fixture,
  不得发送真实会议纪要或生产数据。供应商边界状态核对日期为 2026-07-22。

---

## 14. 版本记录方式

- **依赖**:`Cargo.toml` 直接依赖全部 `=x.y.z` 精确版本,符合[Rust 依赖基线](../../docs/technology-baseline.md#rust-依赖);
  `Cargo.lock` 提交,锁定传递依赖。
- **toolchain**:`rust-toolchain.toml` 钉 1.96.1。
- **模型版本**:请求 family 固定为 `gemini-3.5-flash`;其他 family 在联网前拒绝。
  `ModelFingerprint::response_model` 留痕 provider 返回的实际 model 字符串。
- **能力来源**:供应商声明输入上限 1,048,576 token、输出上限 65,536 token，未声明独立的总量字段；当前
  适配器只实现单轮非流式文本与 usage 解析，并在联网前拒绝超出输入、输出上限的请求，同时保守地把
  输入与请求输出之和限制为 1,048,576 token。能力描述中的 `total_token_limit = None` 表示供应商未声明，
  `adapter_implemented.total_token_limit = 1,048,576` 表示本地强制边界。
  共享来源与核对日期见技术基线，真实端点 conformance 仍为 `deferred`。

---

## 15. Prompt / Context / Loop Engineering 责任边界

按[Agent 工程实现契约](../../docs/handbook/engineering-contracts.md)的要求,**显式划分**三类工程在本概念
中的责任分配:

### Prompt Engineering
- 唯一的 prompt contract 是 `src/main.rs::SYSTEM_PROMPT`:要求"只返回单个
  JSON 对象,不要 prose,不要 markdown 围栏";稳定 ID 为
  `structured-output.system`,版本为 `1`。
- 本概念不做 A/B 或 few-shot;本 SYSTEM_PROMPT 故意写得朴素,以暴露
  "模型仍可能违规"的现实(从而让 `extract::extract_json` 不得不存在)。
- 不可信内容(`fixtures/meeting_input.txt`)被显式包在 user message 内,
  与 system 指令分离。

### Context Engineering
- 本概念 context **只有两段**:`system`(可信项目指令)+ `user`(不可信合成
  fixture)。两段都通过 `ContextBuilder` 保存来源、provenance、tenant、版本、
  时间、有效期、token 估算和选择理由。
- Builder 拒绝过期、跨 tenant、重复 role 和超预算片段，并按最终不可变文本重算
  token 估算。本项目无缓存、RAG、
  压缩或动态排序。

### Loop Engineering
- 本概念**无 loop**:单跳即终止。
- 终止原因稳定枚举:只有 `FinishReason::Stop` 可进入解析；过滤、输出上限、未知或
  缺失停止原因均先返回类型化 `ModelError`。最终结果为 `Ok((T, metadata))` 或
  `StructuredOutputError`。
- 这是后续 `agent-loop`、`error-recovery`、`reflection-agent`
  的**起点**——它们将围绕本概念的失败枚举构建重试/反思/loop。

---

## 16. 已知限制

- 抽取器明确拒绝顶层 JSON 数组并返回 `UnexpectedTopLevelType`;本概念只接受对象。
- provider 缺失 `model` 时记录 `response_model = "unknown"` 和
  `response_model_missing = true`;非字符串等协议不合规则返回稳定 `Protocol` 错误。
- Clippy 配置为默认 lints;未为本概念定制更严格规则。

---

## 17. 项目声明与验收契约迁移

- `practice.toml` 声明 `base`、`model`、`prompt`、`context` 和 `network` profile。
- `acceptance.toml` contract v4 通过 manifest 摘要预注册 10 个固定 fixture，并预注册
  prompt ID/version/SHA；evaluator 直接解析该文件取得全部阈值与预算，对比直接 `serde_json`
  baseline 0.60 与当前抽取、schema 校验实现；核心验收不访问网络，估算模型成本为 0。
- Prompt ID 为 `structured-output.system`，版本为 `1`。选择单一 prompt 是为了
  保持本项目变量最少，并刻意让违规输出由确定性解析边界处理；本项目不据此
  声称完成 `prompt-engineering` 综合能力。
- 当前契约于 2026-07-22 为既有项目补录，不能证明此前实现经过预注册；它只约束
  此日期后的实现、调优和验收。共享外部标准版本与核对日期见技术基线。

---

## 18. Profile 验收说明与 N/A

- `model`:HTTP fixture 覆盖拒答、带内容的过滤、输出上限、未知/缺失停止原因、限流、
  超时、协议错误、缺失模型标识、4xx/5xx 重试分类和能力描述；缺失 usage 明确记录为 `unknown`。
- `prompt`:`ContextBuilder` 测试证明可信 instruction 与不可信变量保持分离,
  Prompt ID/版本和内容 SHA-256 绑定;非法输出由 10 个 fixture 做确定性回归。
- `context`:覆盖过期、未来时间、跨 tenant、静态 inclusion 拒绝、trust/role 提权、
  重复 role、空低价值项、伪造估算和 token 超预算；来源与信任元数据通过
  `BuiltContext::items()` 只读访问。
- `network`:覆盖慢响应、不可用、断连、400/401/403/408/429/500/503、禁止重定向
  和 1 MiB 响应上限。
- `N/A: network retry exhaustion`:本项目刻意不实现重试,任何 transport 或
  provider 错误单次失败;重试属于 `resilience-patterns`。
- `N/A: Prompt candidate A/B`:本项目目标是确定性抽取和 schema 校验,固定单一
  prompt 用于暴露模型违规;候选比较属于后续 `prompt-engineering` 综合实践。
- `N/A: context cache invalidation`:本项目没有 context 缓存或持久化。
- `N/A: auth`:全部 context 都是仓库内公开合成数据；`tenant` 仅是 context namespace，
  `AccessDecision` 仅演示静态 inclusion policy，不验证身份/token，也不判断主体对受保护
  资源的权限。引入受保护数据或主体/资源授权决策时必须新增 `auth` profile 和威胁模型。

适配类限定为精确的 Gemini OpenAI-compatible `gemini-3.5-flash`，真实 conformance
当前为 `deferred`;跨供应商或跨能力/成本层级时必须走适配器变更和独立升级流程。

# structured-output

Practice **#1** of `agent-practices` (§5.1 in `AGENTS.md`).

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
| 失败重试 / 退避 | `resilience-patterns` #58 |
| 反思与自动修复 JSON | `reflection-agent` #7 / `error-recovery` #10 |
| 工具调用编排 | `tool-calling` #2 |
| Agent loop | `agent-loop` #3 |
| 评测集与基线 | `eval-dataset-design` #81 |
| Streaming | `streaming-tool-calling` #102 |
| Token / 成本预算 | `budget-control` #57 |
| Prompt 版本化 | `prompt-engineering` #107 |

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

预期成功输出:`(MeetingMinutes, ModelFingerprint)`,模型指纹见 §9。

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
                │ (T, ModelFingerprint)    │
                └──────────────────────────┘
```

顶层入口:`extract_structured<T>(client, schema, request)` in `src/lib.rs`。

---

## 4. 核心类型和协议边界

| 类型 | 文件 | 边界 |
|---|---|---|
| `ModelClient` (trait) | `src/model/mod.rs` | 唯一的模型抽象;任何符合 `flash` tier 的 OpenAI 兼容模型可替换实现 |
| `CompletionRequest` / `CompletionResponse` | 同上 | 单跳,无 streaming/tool/response_format 字段 |
| `ModelFingerprint` | 同上 | §4.4 不变量 #1 的载体 |
| `MockClient` | `src/model/mock.rs` | 测试与离线 demo;不访问网络 |
| `GeminiOpenAiClient` | `src/model/gemini_openai.rs` | OpenAI 兼容 chat completions;只用 `model` / `messages` / `temperature` / `max_tokens` |
| `extract_json` | `src/extract.rs` | 纯函数,无 IO;失败模式枚举化 |
| `SchemaValidator` | `src/schema.rs` | 两阶段:`iter_errors` 全收集,再 `from_value::<T>` |
| `StructuredOutputError` | `src/error.rs` | 8 个变体覆盖每个失败点;原文均经 `truncate_excerpt`(≤512 字符) |

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
| `STRUCTURED_OUTPUT_MODE` | 否 | `mock` / `real` / 未设;详见 §9 模式解析表 |
| `GEMINI_API_KEY` | 仅 real | 不提交;留空且未显式选 real 则 fallback 到 mock |
| `GEMINI_BASE_URL` | 否 | 默认 `https://generativelanguage.googleapis.com/v1beta/openai/` |
| `GEMINI_MODEL_FAMILY` | 否 | 默认 `gemini-3.5-flash`;同 tier 内可替换 |
| `GEMINI_TIMEOUT_SECS` | 否 | 默认 30 |

---

## 6. 构建、运行、测试命令

**前提**:用户全局 shell 若 export 了 `RUSTUP_TOOLCHAIN`,会覆盖
`rust-toolchain.toml`。本仓库每个 concept 的 `rust-toolchain.toml` 都钉到
`1.96.0`,因此运行前请 unset:

```bash
unset RUSTUP_TOOLCHAIN
```

按 AGENTS.md §9 验收 6 条命令(均在本目录执行):

```bash
rustc --version                                                    # 期望 1.96.0
cargo --version                                                    # 期望 1.96.0
cargo metadata --locked --format-version 1 > /dev/null
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo run --locked                                                 # 干净环境 → mock,退出 0
```

---

## 7. 成功路径、失败路径与预期执行轨迹

每个 `fixtures/mock_responses/*` 都映射一种执行路径,均有 E2E 测试覆盖
(`tests/end_to_end_mock.rs`):

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

---

## 8. 安全边界和已知限制

- **不是生产代码**:无重试、无退避、无熔断、无审批。
- **原始模型输出截断**:任何嵌入错误的原始片段都经 `truncate_excerpt`
  截到 ≤ 512 字符(`src/error.rs::MAX_EXCERPT_CHARS`),避免日志被模型输出
  撑爆;符合 AGENTS.md §8 "trace 默认不得记录完整模型输出"。
- **不调用 LLM 修复 JSON**:抽取失败就失败;自动修复属于 `reflection-agent`。
- **不实现 token / 成本预算**:实测时请自行盯 Gemini 控制台用量。
- **本地原生 TLS**:`reqwest` 启用 `rustls`,默认载入系统证书。
- **HTTP 超时强制 30s**:符合 §8 "所有外部调用必须设置超时"。

---

## 9. 真实模型模式 vs 离线测试模式

**模式自动解析表**(实现见 `src/main.rs::resolve_mode`):

| `STRUCTURED_OUTPUT_MODE` | `GEMINI_API_KEY` | 实际模式 | 行为 |
|---|---|---|---|
| `mock` | 任意 | mock | 强制 mock |
| `real` | 已设 | real | 调用 Gemini 兼容端点 |
| `real` | 未设 | — | **exit 1**,refusing to call real provider |
| 未设 | 已设 | real | **自动 real**,stderr 不打 fallback 提示 |
| 未设 | 未设 | mock | fallback 到 mock,stderr 提示 "set GEMINI_API_KEY to run against the real ..." |
| 非法值 | 任意 | — | **exit 1** |

**模型指纹留痕示例**(AGENTS.md §4.4 不变量 #1)。
真实模式 stdout 末尾:

```
--- fingerprint ---
provider         = gemini-openai-compat
requested_family = gemini-3.5-flash
response_model   = <provider 返回的实际 model 字符串,可能是带日期后缀的 snapshot>
api_version      = Some("v1beta")
capability_tier  = flash
```

`response_model` 进入 fingerprint 是为了让"模型漂移"在日志中可见,但
**测试不对其值断言**,以满足"同 tier 内可替换"。

**能力等价类声明**(AGENTS.md §4.4 不变量 #2):
本概念依赖 `tier: flash`。同 tier 内允许替换为 Claude Haiku /
GPT-4o-mini / 其他 Gemini Flash 系列等,只要它们能在 system prompt 引导下
返回包含合法 JSON 的文本——**不需要改本目录的任何源码**。跨 tier
(切到 `pro` / `opus`)需走 AGENTS.md §4.5 升级流程。

**评测门禁说明**(AGENTS.md §4.4 不变量 #3):
本概念**没有评测集**(按设计),因此真实模型不构成验收手段——`cargo test`
全部走 mock。真实模式仅供学习者手动观察响应。要把模型变化纳入回归判定,
是 `eval-dataset-design` #81 与 `agent-regression-gate` #85 的工作。

---

## 10. 使用的标准

- **JSON Schema Draft 2020-12** —— <https://json-schema.org/draft/2020-12/schema>
- **OpenAI Chat Completions** —— Gemini `v1beta/openai/` 兼容子集
- **AGENTS.md** 仓库根 —— 最后核对日期 2026-06-24

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

本概念用 ~600 行代码换回:对 8 种失败模式的明确分类、可逐条修复的 issue
列表、§4.4 fingerprint 留痕。

---

## 12. 质量 / 延迟 / 成本 / 安全指标

| 指标 | 定义 | 测量方式 |
|---|---|---|
| 抽取成功率 | E2E 测试中 `Ok(_)` 的比例 | `cargo test --locked` 输出 |
| 首次校验失败定位率 | 校验失败时 `issues.len() ≥ 1` 的比例 | 100%(测试中验证) |
| p50 抽取延迟 | `extract_json` 在 ok 路径下的耗时 | < 1ms(本机,纯字符串扫描;实测可用 criterion 测) |
| 真实调用延迟 | `GeminiOpenAiClient::complete` 端到端 | 取决于 Gemini;HTTP 超时 30s |
| 错误信息泄露上限 | 嵌入错误的原文不超过 | 512 字符(`MAX_EXCERPT_CHARS`) |

---

## 13. 数据来源 / 保留 / 删除

- `fixtures/meeting_input.txt`、`fixtures/mock_responses/*`:**人工编写**,
  与任何外部数据无关;随仓库提交。
- 真实模式下 Gemini 端点接收 system prompt + 会议纪要原文;**会议纪要本身
  也是 fixture(不含真实人员/事项)**,不存在个人信息处理问题。
- 本概念不写盘、不缓存、不持久化任何东西。

---

## 14. 版本记录方式

- **依赖**:`Cargo.toml` 直接依赖全部 `=x.y.z` 精确版本,符合 §4.2;
  `Cargo.lock` 提交,锁定传递依赖。
- **toolchain**:`rust-toolchain.toml` 钉 1.96.0。
- **模型版本**:**不在配置中固化**;`ModelFingerprint::response_model` 留痕
  响应里 provider 返回的实际 model 字符串(可能含日期后缀的 snapshot)。

---

## 15. Prompt / Context / Loop Engineering 责任边界

按 AGENTS.md §5.16 与 §10 检查清单的硬要求,**显式划分**三类工程在本概念
中的责任分配:

### Prompt Engineering
- 唯一的 prompt contract 是 `src/main.rs::SYSTEM_PROMPT`:要求"只返回单个
  JSON 对象,不要 prose,不要 markdown 围栏"。
- 本概念**不练习 prompt 版本化、不做 A/B、不实现 few-shot**——那是
  `prompt-engineering` #107。本 SYSTEM_PROMPT 故意写得朴素,以暴露
  "模型仍可能违规"的现实(从而让 `extract::extract_json` 不得不存在)。
- 不可信内容(`fixtures/meeting_input.txt`)被显式包在 user message 内,
  与 system 指令分离。

### Context Engineering
- 本概念 context **只有两段**:`system`(指令)+ `user`(会议纪要原文)。
- 没有 RAG、没有记忆、没有动态拼接,因此无来源标签 / 信任级别 / 时效问题
  ——但**留出了 context provenance 的位置**:`MeetingMinutes` 解析结果
  可作为下游 concept 的 context source 输入。

### Loop Engineering
- 本概念**无 loop**:单跳即终止。
- 终止原因稳定枚举:`Ok((T, fingerprint))` 或 `StructuredOutputError`
  的 8 个变体之一。
- 这是后续 `agent-loop` #3、`error-recovery` #10、`reflection-agent` #7
  的**起点**——它们将围绕本概念的失败枚举构建重试/反思/loop。

---

## 16. 已知限制

- 抽取器**不支持顶层 JSON 数组**(本概念 demo 是单对象);加上不难,
  但暂未加入以保持失败面紧凑。
- `gemini_openai.rs` 不解析 `model` 缺失或非字符串的奇怪响应——视为
  `ModelError::Http`(反序列化失败)。生产代码应该更宽容。
- Clippy 配置为默认 lints;未为本概念定制更严格规则。

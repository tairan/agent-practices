# 技术与版本基线

最后核对日期：2026-07-22。实现前必须重新核对易变的协议、供应商 API 和外部标准；发生变化时在项目 README 记录继续采用旧版或迁移新版的理由。

## Rust 工具链

| 项目 | 固定版本 |
|---|---|
| Rust | `1.96.1` |
| Cargo | `1.96.1` 随 Rust toolchain 提供 |
| Rust edition | `2024` |
| Cargo resolver | `3` |
| rustfmt | `1.96.1` toolchain 组件 |
| Clippy | `1.96.1` toolchain 组件 |
| Fuzz/Miri nightly | `nightly-2026-06-22` |

每个概念的 `rust-toolchain.toml` 必须为：

```toml
[toolchain]
channel = "1.96.1"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

单 crate 项目的 `Cargo.toml` 必须包含：

```toml
[package]
edition = "2024"
rust-version = "1.96.1"

[workspace]
resolver = "3"
```

空 `[workspace]` 只用于当前独立概念设置 resolver，不得在仓库根目录创建 workspace。多 crate 概念可以在自身目录使用 workspace；成员、exclude 和 path dependency 均不得越出当前概念目录。

多 crate 概念必须在自身目录声明统一 package 基线：

```toml
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
edition = "2024"
rust-version = "1.96.1"
```

成员 crate 必须使用 `edition.workspace = true` 和 `rust-version.workspace = true` 继承，不能自行回退 edition 或 MSRV。`member`、`default-member`、`exclude` 和 path dependency 均不得越出当前概念目录。

`1.96.1` 替代 `1.96.0`，用于纳入同系列错误编译、Cargo HTTP 和 libssh2 安全修复；升级依据见 [Rust 1.96.1 官方公告](https://blog.rust-lang.org/2026/06/30/Rust-1.96.1/)。

普通构建不得使用 nightly。只有 fuzz、Miri 或确需 nightly 的诊断任务可以使用固定版本：

```bash
cargo +nightly-2026-06-22 fuzz run <target>
cargo +nightly-2026-06-22 miri test
```

禁止未带日期的 `nightly`。Miri 和覆盖率组件必须安装到上述固定工具链。

## Rust 依赖

直接依赖必须使用精确版本 `=x.y.z`。禁止 `*`、范围、仅 major/minor 或未固定 Git branch/tag。传递依赖由项目自己的 `Cargo.lock` 锁定。内部 path dependency 必须同时声明精确 `version`，且路径不得离开当前概念。

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
| Future/Stream | `futures` | `=0.3.32` |
| Tokio 扩展 | `tokio-util` | `=0.7.18` |
| PostgreSQL | `sqlx` | `=0.9.0` |
| 动态库加载 | `libloading` | `=0.9.0` |
| JWT | `jsonwebtoken` | `=10.4.0` |
| OAuth | `oauth2` | `=5.0.0` |
| OpenTelemetry | `opentelemetry` | `=0.32.0` |
| tracing OTel 桥接 | `tracing-opentelemetry` | `=0.33.0` |
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
| SHA-2 内容摘要 | `sha2` | `=0.10.9` |
| TOML 解析 | `toml` | `=0.9.8` |

未列出的 crate 可以使用，但必须选择 crates.io 稳定非预发布版本，固定完整版本，并在 README 记录用途、选择理由、核对日期、许可证、维护状态、MSRV、已知漏洞和重复依赖影响。禁止已 yanked 版本；未经明确说明不得使用 Git dependency。不得为统一而添加未使用依赖，不得使用无差别 `cargo update`。

## 开发工具

| 工具 | 固定版本 |
|---|---|
| `cargo-audit` | `0.22.2` |
| `cargo-deny` | `0.19.9` |
| `cargo-nextest` | `0.9.138` |
| `cargo-llvm-cov` | `0.8.7` |
| `cargo-fuzz` | `0.13.2` |

安装命令必须指定完整版本和 `--locked`。

## 服务和容器

| 服务 | 镜像与版本 |
|---|---|
| Keycloak | `quay.io/keycloak/keycloak:26.6.3` |
| PostgreSQL | `postgres:18.4` |
| NATS | `nats:2.14.2` |
| Apache Kafka | `apache/kafka:4.3.0` |
| Qdrant | `qdrant/qdrant:v1.18.2` |

`compose.yaml` 不得使用 `latest`、浮动 major/minor tag 或本地未版本化镜像。提交时必须使用 `name:tag@sha256:<digest>`。不同架构 digest 不同时，README 和启动脚本必须说明支持架构及对应 digest。

操作系统包、数据库扩展、Keycloak provider、Kafka 插件和模型服务同样必须固定版本。fixture 和协议样例必须记录生成软件版本。

## 模型与 Provider 基线

默认真实接入为 Google Gemini 的 OpenAI 兼容端点。真实模型不能依赖长期 snapshot 可用性保证可重复。配置使用能力等价类内的 model family name，禁止 `latest`、`default` 等跨能力等价类的浮动别名。

当前默认 family `gemini-3.5-flash` 的供应商声明输入上限为 1,048,576 token、输出上限为 65,536 token，并支持 caching、function calling 和 structured outputs；OpenAI 兼容接口声明支持 streaming。项目必须把这些“供应商声明”与“当前适配器已实现能力”分开记录，真实 conformance 未执行时不得把声明写成已验证事实。来源：[Gemini 3.5 Flash 官方模型页](https://ai.google.dev/gemini-api/docs/models/gemini-3.5-flash)、[Gemini OpenAI compatibility](https://ai.google.dev/gemini-api/docs/openai)。

真实调用必须记录供应商、请求 model family、响应实际模型标识、可获得的 API 版本和能力等价类。评测快照可以包含指纹，但输出断言不得要求指纹永久不变。

## 协议基线

- MCP 默认以 `2025-11-25` 规范为目标，至少覆盖生命周期、能力协商、JSON Schema 2020-12、一种标准传输和对应授权边界。
- A2A 默认以 v1.0 为目标，至少覆盖版本协商、Agent Card、任务生命周期和一种协议绑定。

协议项目必须锁定实现版本、声明兼容版本和未实现能力，覆盖版本不兼容，并在存在参考实现或 conformance 工具时执行跨实现测试。不得将 MCP 和 A2A 合并为同一抽象。

## 版本升级

版本升级必须作为独立变更：

1. 核对官方 release notes、安全公告、MSRV、迁移指南和协议兼容性。
2. 列出旧版、新版、原因和受影响概念。
3. 先修改一个代表性概念并执行回归、性能和兼容性验证。
4. 使用 `cargo update -p <crate> --precise <version>` 定向更新。
5. 审查传递依赖变化，禁止无关刷新。
6. 执行格式化、Clippy、测试、漏洞扫描、许可证检查和 smoke test。
7. 更新版本表、核对日期、provider conformance 和相关 README。

安全修复可以突破单概念变更限制，但必须保持独立提交并完成受影响项目回归。

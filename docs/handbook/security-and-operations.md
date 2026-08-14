# 安全、数据与生产运营契约

## 数据、隐私与记忆

数据进入模型、记忆、索引、日志或评测集前必须完成来源标记、租户校验、最小字段选择和敏感信息处理。

声明 `stateful`、`context` 或 `model` profile 的项目必须记录：

- 数据来源和合法用途。
- 数据分类、tenant 和访问控制。
- 是否发送到模型供应商或其他外部处理方。
- 供应商保留、训练使用、地域和删除能力的已知边界。
- 保存位置、加密方式、TTL、导出、更正和删除方法。
- 数据、embedding、索引和 schema 版本。
- 删除传播、缓存失效和备份残留边界。

ACL-aware retrieval 必须在召回前过滤权限。记忆写入必须保留来源和信任等级，并隔离不可信或冲突内容。真实生产数据不得直接用于教学 fixture，评测样本必须检查隐私、授权和数据污染风险。

## 威胁模型

声明 [`AGENTS.md` 定义的任一高风险 profile](../../AGENTS.md#3-适用性-profile)，即 `side-effect`、`auth`、`protocol`、`distributed`、`untrusted-code`、`browser` 或 `production` 的项目，必须提交威胁模型。其他项目根据风险决定。

威胁模型至少识别：

- 主体、资产、信任边界和攻击者能力。
- prompt、工具输出、RAG、记忆、Skill、Agent Card 和页面投毒。
- SSRF、DNS rebinding、重定向、内网访问和工具输出外传。
- 越权、confused deputy、审批替换和 token 泄露。
- 依赖、构建、镜像、插件和 fixture 供应链。
- 凭据和 secret 的过期、轮换、撤销、双版本窗口、旧凭据失效与失败回滚。
- 控制失效时的降级、止损和恢复路径。

使用 [`../../templates/threat-model.md`](../../templates/threat-model.md) 记录数据流、攻击路径、控制、验证和残留风险。不得宣称 prompt injection 可以彻底消除。

## 访问控制与委托

1. 严格区分用户委托权限、Agent 应用自身权限和后台机器身份权限。
2. 有用户参与的操作默认使用用户委托，不得退化为应用级服务账号模拟用户。
3. Agent 不得使用 client credentials token 模拟用户。
4. 禁止 direct naked impersonation；委托必须来源于已验证登录和受控 token exchange。
5. 入口 token 和下游 token 必须具有适用于各自资源的 audience；委托链练习默认使用不同 audience。
6. 下游 scope 必须等于或小于用户原权限，并缩减到当前动作所需。
7. 模型不能声明权限；授权决策必须发生在工具边界和资源服务器。
8. Token 不得进入模型上下文、prompt、工具输出、日志、trace、错误或任务结果。
9. Subagent 不得继承原始用户 token，只能获得任务专用、短期、缩减权限 token。
10. 用户撤权、登出或任务取消后，不得继续刷新或交换新 token。
11. 每次调用必须保留用户主体、Agent client、Subagent、资源、scope、审批和结果的审计关联。
12. JWT access token 按 RFC 9068 验证签名和适用 claims；opaque token 使用 RFC 7662 introspection 或授权服务器等价机制。
13. 受保护资源必须验证 issuer 或授权服务器来源、有效期、audience/resource、scope 和主体绑定；适用时验证 client、authorized party、tenant、delegation chain 和 token type。
14. OAuth 实现遵循 RFC 9700；token exchange 遵循 RFC 8693；资源指示器使用 RFC 8707；MCP HTTP 授权发现适用时使用 RFC 8414 和 RFC 9728。
15. 调用方不得仅通过解码 access token 代替资源服务器授权，也不得假设所有 token 都包含 `azp` 或其他非必需 claim。

## 攻击实验最小集

高风险项目必须将适用攻击变成可复现 fixture 或故障注入：

- Prompt 或检索内容要求读取凭据、改变权限或忽略可信指令。
- 工具输出、Agent Card、记忆或页面包含伪造控制指令。
- URL 重定向、DNS rebinding、内网地址和工具结果外传。
- 跨 tenant 访问、scope 扩大、错误 audience 和 confused deputy。
- token 出现在日志、trace、错误或模型请求。
- 审批对象与最终执行对象不一致。
- Subagent 获得超出任务需要的上下文或权限。
- 生产凭据轮换失败、旧凭据仍可使用或新旧版本切换导致服务中断。

每个案例必须记录攻击目标、信任边界、不变量、检测方式、缓解结果和残留风险。

凭据轮换演练不得在 fixture、命令输出或 evidence 中记录真实 secret；只能记录不可逆标识、版本和验证结果。

## 供应链

依赖、容器和工具必须锁定版本并检查许可证、维护状态、MSRV、已知漏洞和重复依赖影响。生产项目应该生成 SBOM 或等价清单并记录构建来源。

动态插件必须验证来源、版本、allowlist 以及适用的哈希或签名。该验证不能把进程内插件变成安全沙箱；只允许加载可信插件。

## 运行时可靠性

按 profile 适用：

- 所有外部调用必须设置超时和取消。
- 重试必须有上限、退避和抖动，只对明确可重试错误执行。
- 副作用重试必须使用幂等键或等价保护。
- 并发、队列、递归和资源使用必须有上限并处理背压。
- 持久任务必须定义所有权、租约、可见性超时、确认、恢复和孤儿任务处理。
- checkpoint 和重放必须区分纯计算、可重复读取和不可逆副作用。
- 优雅关闭必须停止接收新任务、传播取消并处理在途状态。

## 可观测性

使用 `tracing` 或等价机制记录结构化事件。生产项目必须提供稳定 request ID、session ID、task ID 和 trace ID，并在跨进程调用中传播。

trace 默认不得记录完整 prompt、模型输出、工具参数或工具结果。确需记录内容时必须显式启用、脱敏、限制访问并设置保留期限。

模型调用必须记录：

- 供应商和请求 model family。
- 响应实际模型标识，缺失时为 `unknown`。
- 可获得的 API 版本和能力等价类。
- prompt、context policy、工具 schema 和配置版本。
- token 使用量或 `estimated` 标记。
- 延迟、重试、停止原因和错误分类。

## 生产实操

声明 `production` profile 的项目必须执行故障注入，至少覆盖慢响应、连接中断、进程崩溃、部分写入和依赖不可用，并按设计增加模型漂移、队列积压、限流、权限撤销或错误发布。

使用长期凭据或 secret 的生产项目还必须演练过期、轮换、撤销、双版本窗口、失败回滚和旧凭据失效验证。

必须定义 SLI、SLO、错误预算、告警和 runbook。事故演练必须记录检测、止损、审计保全、降级、恢复、rollback、MTTD、MTTR、残留风险和无责复盘。

资深运营 evidence 必须来自真实部署运行，或包含未提前公开故障的受控演练。仅重复自己编写的成功路径不构成运营能力证明。

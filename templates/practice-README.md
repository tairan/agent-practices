# <concept-name>

## 1. 学习目标与前置知识

TODO: 描述要形成的可观察能力，以及开始前应掌握的 Rust、模型或协议知识。

## 2. 目标、非目标、Profile 与能力域

- 目标：TODO
- 非目标：TODO
- Profiles：`base`, TODO
- Competencies：TODO

## 3. 独立场景与 Baseline

- 问题场景：TODO
- 预期行为：TODO
- 非 Agent baseline：TODO
- 引入目标机制的假设收益：TODO

## 4. 实现边界

- 必须自行实现：TODO
- 可以使用的库：TODO
- 禁止的捷径：TODO
- SDK 或外部服务负责的部分：TODO

## 5. 架构、流程与核心类型

TODO: 描述确定性边界、状态转换、外部依赖和数据流。

## 6. Prompt、Context 与 Loop

- Prompt contract：TODO 或 `N/A: <理由>`
- Context builder：TODO 或 `N/A: <理由>`
- Loop 状态、预算和终止：TODO 或 `N/A: <理由>`

## 7. Tool、权限与审批

TODO 或 `N/A: <理由>`。

## 8. 数据、安全与隐私

TODO: 来源、tenant、外部处理方、敏感信息、保留、删除和已知边界。

## 9. Fixture 与故障注入

| 案例 | 注入方式 | 预期不变量 | 预期结果 |
|---|---|---|---|
| 成功路径 | TODO | TODO | TODO |
| 非法输入 | TODO | TODO | TODO |
| 超时或取消 | TODO | TODO | TODO |

按项目 profile 补充拒答、权限、重复动作、部分写入、崩溃恢复、审批替换或协议不兼容；不适用时写 `N/A: <理由>`。

## 10. 验收契约与指标

TODO: 解释 `acceptance.toml` 中 baseline、阈值、数据集、评分器、机器可读 artifact 和预算的依据。

## 11. 环境变量与外部依赖

TODO: 说明 `.env.example`、mock/real 模式、凭据边界、外部服务版本和项目独立资源；无环境变量时写 `N/A: <理由>`。

## 12. 构建、运行与测试

```bash
cargo metadata --locked --format-version 1
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
```

TODO: 添加可执行项目、服务和可选真实模型命令。

## 13. 成功、失败与恢复轨迹

- 成功轨迹：TODO
- 失败轨迹和稳定错误：TODO
- 取消、补偿或恢复轨迹：TODO 或 `N/A: <理由>`

## 14. Evidence 与复现

- 机器可读结果位置：TODO
- 重新生成命令：TODO
- 保留期限和清理方式：TODO
- 脱敏和访问边界：TODO
- 失败分类、ADR、威胁模型和未执行检查：TODO

## 15. 高风险与 Capstone 材料

声明高风险 profile 或 `capstone` stage 时，链接 ADR、威胁模型、数据流图、评测报告、runbook、事故复盘、[`fault-injection-report.md`](fault-injection-report.md)、[`load-test-report.md`](load-test-report.md) 和成本 artifact；否则逐项写 `N/A: <理由>`。

## 16. 常见错误

TODO: 列出容易让测试看似通过但没有实现目标机制的错误，以及诊断入口。

## 17. 项目版本与共享技术基线

TODO: 记录项目实际采用的模型、prompt、工具 schema、数据集和配置版本；链接共享技术基线并说明偏离，不复制维护共享版本核对日期。

## 18. 已知限制、N/A 与扩展挑战

- 已知限制：TODO
- N/A：TODO
- 扩展挑战：TODO；不得混入当前最低验收。

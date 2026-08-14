# Threat Model: <project>

- 版本：1
- 日期：YYYY-MM-DD
- Profiles：TODO

## 主体、资产与攻击者能力

TODO

## 信任边界与数据流

TODO: 给出数据流图，并标记模型、工具、存储、外部服务和审批边界。

## 威胁与控制

| ID | 来源到目标 | 攻击方式 | 影响 | 预防控制 | 检测 | 恢复 | Fixture/注入参数 | 执行命令 | Artifact | 残留风险 |
|---|---|---|---|---|---|---|---|---|---|---|
| T-001 | TODO | TODO | TODO | TODO | TODO | TODO | TODO | TODO | TODO | TODO |

## 安全不变量

- 凭据、token 和 secret 不进入仓库、prompt、模型上下文、工具输出、日志、trace、错误、任务结果或 evidence。
- 模型不能声明权限或绕过确定性执行边界。
- TODO: 添加项目特定不变量。

## 控制失效、止损与残留风险

TODO

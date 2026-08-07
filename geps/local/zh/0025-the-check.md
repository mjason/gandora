---
gep: 25
title: 检查
description: 一个判定——编译器诊断加上 Advisor 建议——由 `gan check` 输出，由 `gan lsc check` 以 JSON 返回，并对 `gan build` 进行门控。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-04
updated: 2026-08-07
revision: 4
requires: [12, 13, 22]
replaces: [23]
superseded-by: null
resolution: null
language: zh-CN
source: ../../0025-the-check.md
source-revision: 4
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0025-the-check.md](../../0025-the-check.md)。

# GEP-0025: 检查

## 摘要

`gan check` 是编译器对整个项目的总体判定：每一条诊断（错误和 GEP-0022 lint）**以及**每一条 Advisor 建议——实践差距、跨语言迁移提示，以及基于编辑距离、对照真实符号的“你是不是想要”提示。`gan lsc check` 以单个 JSON 对象 `{diagnostics, suggestions}` 返回同样的判定。**`gan build` 会先运行 check**：错误会在写入任何构建产物之前停止构建；警告和建议则被打印出来，并让构建继续——这就是重量级编译器的契约，正如 Rust 的行为方式。GEP-0023 沙盒（`gan try`）已退役：它的教学引擎现在存在于这里，在项目范围内，由现有命令承载。

## 动机

每个代码片段的判定（`try`）重复了 `check` 本应提供的功能。一种拼写——check——即可服务于终端前的开发者、通过 JSON 通信的智能体、借助相同诊断信息工作的编辑器，以及构建门禁，无需额外学习任何东西。

## 范围

检查判定与构建门禁。执行仍由
`gan run`/`gan test`/`gan repl` 负责。

## 规范

**GEP-0025-R001（判定）：** `gan check` 按文件打印：编译器诊断（带源码位置；错误和警告），随后是 Advisor 建议，每条都带有 `practice` | `migration` | `did_you_mean` 标记。只要存在任何错误，退出码即为 1；警告和建议永远不会让检查失败。`gan lsc check` 以 `{"diagnostics": [...], "suggestions": [...]}` 的形式发出相同的判定，每条记录都带路径，并采用相同的退出码契约。

**GEP-0025-R002（Advisor）：** 建议引擎是 gandora-tool 中的 `Advisor`——为 runner 和 `lsc` 提供同一个实现。它会在任何文本检查之前屏蔽字符串/heredoc/sigil/注释内容（普通文本永远不会触发模式），按消息去重，并查询真实候选：成员拼写错误查询模块符号（带有 gandora-std 内省回退，因此没有 venv 的项目仍能获得 `Enum.mpa → Enum.map`）；未定义变量查询文件自身的标识符；构造拼写错误查询关键字列表。

**GEP-0025-R003（构建门禁）：** `gan build` 先执行 R001 判定。任何错误都会以 `build aborted: check failed` 中止并以退出码 1 结束；否则构建继续，此时警告和建议已经打印完毕。`ganc build`（底层管道命令）保持不受门禁约束。`gan run` 以同一判定为门禁，但抑制建议——错误会阻止运行，教学性建议环节属于 `check`/`build`。

**GEP-0025-R004（红绿灯）：** `lsc check` 的判定以两个布尔值开头：`"ok"`（无错误——程序编译通过）和 `"clean"`（ok，且没有警告，且没有建议）。智能体的循环是：红灯（`ok: false`）→ 修复错误；黄灯（`ok` 但不是 `clean`）→ 阅读建议；绿灯（`clean: true`）→ 提交。

**GEP-0025-R005（信任线）：** 一个惯用项目 MUST 得到零建议的判定——噪音会让智能体学会忽略 Advisor。参考基准：语言自身的标准库、工具链、tour 和 playground 在各自的检查下保持零建议。规则只有通过该基准的检验，才有资格被采纳：`$builtins` 豁免于 pyimport 重复提示（它是环境命名空间）；`rescue _e ->` 是刻意吞掉异常，不构成裸 rescue（bare-rescue）违规；在 `@spec`/`@type` 行中，`Mod.t()` 是类型拼写，绝不是成员拼写错误；在 `@spec` 行中，`$mod.Type()` 是规范语法本身规定的宿主类型拼写（GEP-0017-R002），绝不计入 pyimport 重复；单次调用的 `fn` 包装仅在被调用方可捕获时才会被标记（`&f/1`——绝不会针对 `g.(x)` 点调用）；测试模块（`use Test` 或 `TestX` 命名）是消费者，而非库接口表面——注解覆盖率和 doctest 提示不适用于它们。

**GEP-0025-R006（项目覆盖范围）：** 判定覆盖已配置的源码根目录以及顶层 `tests/*.gan`——正好是 `gan test` 运行的范围；`tests/` 下更深层的目录是 fixture，不提供建议。测试文件只接受 Advisor 检查；它们的编译诊断属于 `gan test`。

**GEP-0025-R007（合并与锚点）：** 来自多个文件的相同建议消息会合并为一条，并标注散布范围（`(also in N other file(s))`）；每条建议都带有其首个证据的从 1 开始计数的行号，因此智能体可以直接跳转到该处。字面量屏蔽会保留分隔符，并平衡 sigil 主体中的嵌套括号——`~python(next((...), None))` 永远不会把它的 `None` 泄漏到迁移提示中。

**GEP-0025-R008（产物验证）：** 判定包含对编译产物的一遍解析检查：生成的 Python 使用 ty 进行检查，并限制为只执行解析规则——未定义名称、无法解析的导入，或任何模块都不提供的成员，都是运行时致命事实，因此会作为**错误**报告，并映射回 `.gan` 源码（还原后的名称、源码行、did-you-mean）。类型流相关意见（运算符支持、参数类型）不进入门禁；它们永远不会阻止构建。当 ty 不可用时，该层静默降级。原生扩展通过随附的 `.pyi` 存根暴露其接口（gandora-core 随附了一个）。

**GEP-0025-R009（构建即判定）：** 不存在单独的 check 命令——`gan build` 运行完整判定（诊断、建议、产物验证），并在出现任何错误时、在写入产物之前停止。`gan run` 以同一判定为门禁，并抑制建议。`gan lsc check` 仍然是同一判定面向工具和智能体的 JSON 接口。已退役的 `gan check` 形式会打印一条迁移说明，并委托给 build。

## 理由

将沙盒并入 check 遵循了工具自身的经验：代理的循环是 编写 → 判定 → 修复 → 构建，而判定本就归属于每个开发者已在运行的命令。将 Advisor 移入 gandora-tool 后，教学引擎便置于所有消费者（runner、lsc、未来的编辑器界面）之下，无需新增包。

## 向后兼容性

**破坏性变更**：`gan try` 和 `gan lsc try`/`review` 被移除；`gan lsc check` 的形态从列表变为 `{diagnostics, suggestions}`。GEP-0023 被本 GEP 取代。

## 安全性与确定性

Check 不执行任何操作；构建门禁仅对现有步骤重新排序。

## 工具与 AI 使用

智能体循环：编写 → `gan check`（或用于 JSON 的 `lsc check`）→ 修复每条诊断信息，应用每条建议 → `gan test` → `gan build`。
概念查找继续使用 `gan lsc doc <construct>`。

## 已拒绝的备选方案

### 在 check 旁边保留 `try`

一个裁决两个名称；片段执行部分就是 `gan run` 多走几步。

### 也对 `ganc build` 进行门控

底层命令对脚本保持可预测；高层命令（`gan build`）承载策略。

## Conformance

一个基于 `gan lsc check` 的 BDD 测试套件 MUST 覆盖：干净模块的静默保证；错误/警告退出码契约；每一种建议类型；带有已习得修正的 lint 透传；恶意输入考验（绝不崩溃，始终 JSON）；以及构建门禁的出错即中止。
对于修订版 2，它 MUST 还覆盖：`ok`/`clean` 红绿灯；每项 R005 豁免（测试模块、`rescue _e`、`Mod.t()` 规范、点调用的 `fn` 包装，以及 `$builtins` 重复各自判定为 clean）；嵌套括号 sigil 掩蔽；以及带行锚点的跨文件合并。

## 变更历史

- 修订版 4，2026-08-07：实践检查（practice pass）引入了管道风格，并已写定：三个相邻的嵌套调用（`f(g(h(x)))`）提示使用 `|>` 管道（断言行和受宏保护的表达式（如 `safe/2`）除外；`@spec`/`@type` 行排除在外——类型从设计上讲就是调用）；当项目能够解析标准库时，单变量的裸 `for x <- xs, do: f(x)` 提示使用 `Enum.map`（`for` 仍适用于带 filter/pattern-skip/`into:`/await 的主体）；现有的 map+filter 链式规则修正为仅在相邻的 filter-then-map 上触发（map-then-filter 没有对应的 `for` 写法）；GEP-0010-R011 所包装的主机互操作将获得一条按文件合并的提示，指向 `Path`/`File`/`System`（包装模块本身除外）。规则的完整形式见 docs/practices.md；`gan lsc doc practices` 提供摘要，`gan agent` 简报和 gan-mcp composer 提示词也承载该内容。

- 修订版 1，2026-08-04：初始版本——取代 GEP-0023。
- 修订版 3，2026-08-04：R008 工件验证（ty 对编译后的 Python 的解析规则，映射回源码；类型流永不作为门禁）；R009 构建涵盖检查——一个裁决，一条命令。
- 修订版 2，2026-08-04：R004 红绿灯（`ok`/`clean`）；R005 零噪音信任线，以及幸存规则的细化；R006 项目表面包括顶层测试；R007 跨文件整合、行锚点、平衡的 sigil 掩码。

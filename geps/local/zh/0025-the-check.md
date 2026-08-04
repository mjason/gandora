---
gep: 25
title: 检查
description: 一个裁决——编译器诊断加上Advisor建议——由`gan check`打印，由`gan lsc check`以JSON形式返回，并门控`gan build`。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-04
updated: 2026-08-04
revision: 2
requires: [12, 13, 22]
replaces: [23]
superseded-by: null
resolution: null
language: zh-CN
source: ../../0025-the-check.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0025-the-check.md](../../0025-the-check.md)。

# GEP-0025: 检查

## 摘要

`gan check` 是编译器对项目的整体裁决：所有诊断（错误和 GEP-0022 lints）**以及**所有 Advisor 建议——实践差距、跨语言迁移提示以及针对真实符号的编辑距离“是不是你想找”功能。`gan lsc check` 返回相同的裁决，作为一个 JSON 对象 `{diagnostics, suggestions}`。**`gan build` 首先运行 check**：错误会在写入任何工件之前阻止构建；警告和建议会打印出来并允许构建继续——这是重型编译器的契约，就像 Rust 的行为一样。GEP-0023 沙箱（`gan try`）被淘汰：其教学引擎现在存在于此处，在项目范围内，属于已经存在的命令之下。

## 动机

每个片段（`try`）的判定结果重复了 `check` 本应实现的功能。统一的拼写 —— check —— 服务于终端前的人类用户、通过 JSON 通信的智能体、使用相同诊断信息的编辑器，以及构建门禁，无需额外学习成本。

## 范围

检查结果与构建门禁。执行仍由 `gan run`/`gan test`/`gan repl` 负责。

## 规范

**GEP-0025-R001（裁定）：** `gan check` 针对每个文件打印：编译器诊断（带位置；错误和警告），随后是 Advisor 建议，每条标记为 `practice` | `migration` | `did_you_mean`。当存在任何错误时退出码为 1；警告和建议永远不会导致检查失败。`gan lsc check` 发出相同的裁定，格式为 `{"diagnostics": [...], "suggestions": [...]}`，每个条目包含路径，且遵循相同的退出码约定。

**GEP-0025-R002（Advisor）：** 建议引擎是 `gandora-tool` 中的 `Advisor`——运行器和 `lsc` 共用一个实现。它屏蔽字符串/heredoc/sigil/注释内容（在任何文本检查之前，散文永远不会触发模式），按消息去重，并查询真实候选项：模块符号用于成员拼写错误（带有 `gandora-std` 自省回退，因此无 venv 的项目仍能获得 `Enum.mpa → Enum.map`），文件自身的标识符用于未定义变量，以及关键字列表用于构造器拼写错误。

**GEP-0025-R003（构建门控）：** `gan build` 首先运行 R001 裁定。任何错误都会中止并输出 `build aborted: check failed`，退出码为 1；否则构建继续进行，警告和建议已打印。`ganc build`（管道）保持未门控。`gan run` 基于相同的裁定进行门控，但抑制建议——错误会停止运行，教学环节属于 `check`/`build`。

**GEP-0025-R004（交通灯）：** `lsc check` 裁定以两个布尔值开头：`"ok"`（无错误——程序可编译）和 `"clean"`（ok，且无警告，且无建议）。代理的循环是：红色（`ok: false`）→ 修复错误；黄色（`ok` 但不 `clean`）→ 阅读建议；绿色（`clean: true`）→ 提交。

**GEP-0025-R005（信任线）：** 一个符合习惯的项目必须裁定零建议——噪音会让代理学会忽略 Advisor。参考标准：语言自身的标准库、工具链、教程和操场在其自身检查下保持无建议。规则通过通过该标准来赢得其地位：`$builtins` 豁免于 pyimport 重复提示（它是环境命名空间），`rescue _e ->` 是有意的吞并而非裸 rescue 违规，`@spec`/`@type` 行中的 `Mod.t()` 是类型拼写而非成员拼写错误，`@spec` 行上的 `$mod.Type()` 是规范语法本身强制的主机类型拼写（GEP-0017-R002）且不计入 pyimport 重复，单次调用 `fn` 包装仅在被调用者可捕获时（`&f/1`——从不用于 `g.(x)` 点调用）才被标记，测试模块（使用 `use Test` 或 `TestX` 命名）是消费者而非库表面——注释覆盖率和 doctest 提示不适用于它们。

**GEP-0025-R006（项目表面）：** 裁定覆盖配置的源根目录以及顶层 `tests/*.gan`——正是 `gan test` 所运行的内容；`tests/` 下的更深层目录是测试夹具，不被建议。测试文件仅接收 Advisor 传递；它们的编译诊断属于 `gan test`。

**GEP-0025-R007（合并与锚点）：** 来自多个文件的相同建议消息合并为一个条目，并标注扩散范围（`(also in N other file(s))`），且每个建议携带其首个证据的从 1 开始的行号，以便代理可以直接跳转到该位置。字面量屏蔽保留分隔符并平衡 sigil 体中的嵌套括号——`~python(next((...), None))` 永远不会将其 `None` 泄露到迁移提示中。

## 理由

将沙箱折叠到 check 中遵循了工具自身的经验教训：一个代理的循环是 write → verdict → fix → build，而 verdict 属于每个开发者已经运行的命令。将 Advisor 移入 gandora-tool 将教学引擎置于每个消费者（runner、lsc、未来的编辑器界面）之下，而无需新的包。

## 向后兼容性

**破坏性变更**：`gan try` 和 `gan lsc try`/`review` 已被移除；`gan lsc check` 的形式从列表变为 `{diagnostics, suggestions}`。GEP-0023 被本 GEP 取代。

## 安全性与确定性

Check 不执行任何操作；构建门仅重新排序现有步骤。

## 工具与AI使用

智能体循环：编写 → `gan check`（或 JSON 文件使用 `lsc check`）→ 修复所有诊断结果，应用所有建议 → `gan test` → `gan build`。概念查询通过 `gan lsc doc <construct>` 进行。

## 被拒绝的替代方案

### 保留 `try` 与 `check` 并存

同一个裁决的两个名称；片段执行部分曾是带有额外步骤的 `gan run`。

### 也对 `ganc build` 进行门控

管道（plumbing）对脚本保持可预测性；瓷器（`gan build`）承载策略。

## 符合性

一个针对 `gan lsc check` 的 BDD 测试套件 MUST 覆盖：干净模块的静默保障；错误/警告的退出码契约；每种建议类型；带有已教导修正的 lint 直通；敌意输入考验（绝不崩溃，始终 JSON）；以及构建门的出错中止。对于修订版 2，它 MUST 还覆盖：`ok`/`clean` 信号灯；每个 R005 豁免项（测试模块、`rescue _e`、`Mod.t()` 规格、点调用 `fn` 包装，以及每次判定为干净的 `$builtins` 重复）；嵌套括号的 sigil 掩蔽；以及带有行锚点的跨文件合并。

## 修订历史

- 修订版 1，2026-08-04：初始版本 —— 取代 GEP-0023。
- 修订版 2，2026-08-04：R004 交通灯（`ok`/`clean`）；R005 零噪声信任线及幸存规则改进；R006 项目表面包含顶层测试；R007 跨文件合并、行锚点、平衡符号掩码。

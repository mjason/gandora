---
gep: 25
title: 检查
description: 一个裁决——编译器诊断结果加上 Advisor 建议——由 `gan check` 打印，由 `gan lsc check` 返回为 JSON，并控制 `gan build` 的通过。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-04
updated: 2026-08-04
revision: 3
requires: [12, 13, 22]
replaces: [23]
superseded-by: null
resolution: null
language: zh-CN
source: ../../0025-the-check.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0025-the-check.md](../../0025-the-check.md)。

# GEP-0025: 检查

## 摘要

`gan check` 是编译器对一个项目的完整裁决：所有诊断信息（错误和 GEP-0022 代码检查）**以及**每条 Advisor 建议——实践差距、跨语言迁移提示、以及针对真实符号的编辑距离“您是否在找”。`gan lsc check` 返回相同的裁决，格式为单个 JSON 对象 `{diagnostics, suggestions}`。**`gan build` 先运行 check**：错误会在写入任何工件之前停止构建；警告和建议会打印出来，并允许构建继续进行——这是重型编译器合约，正如 Rust 的行为方式。GEP-0023 沙盒（`gan try`）被废弃：其教学引擎现在存在于项目范围内，由已有的命令所承载。

## 动机

每个片段（`try`）的裁决重复了`check`本应是的功能。一种拼写——check——服务于终端前的人类、通过 JSON 的代理、通过相同诊断的编辑器，以及构建门禁，无需额外学习。

## 范围

检查结果和构建门控。执行仍由`gan run`/`gan test`/`gan repl`负责。

## 规范

**GEP-0025-R001（裁决）：** `gan check` 按文件输出：编译器诊断信息（附带范围信息；错误和警告），随后是 Advisor 建议，每条建议标记为 `practice` | `migration` | `did_you_mean`。存在任何错误时退出码为 1；警告和建议永远不会导致检查失败。`gan lsc check` 以 `{"diagnostics": [...], "suggestions": [...]}` 的形式输出相同的裁决，每个条目附带路径，并遵循相同的退出码约定。

**GEP-0025-R002（顾问）：** 建议引擎是 gandora-tool 中的 `Advisor`——为运行器和 `lsc` 提供统一实现。它在任何文本检查之前屏蔽字符串/heredoc/sigil/注释内容（普通文本永远不会触发模式），按消息去重，并查询真正的候选对象：针对成员拼写错误的模块符号（借助 gandora-std 内省回退，使得无虚拟环境项目仍能获得 `Enum.mpa → Enum.map`）、针对未定义变量的文件自身标识符，以及针对构造器拼写错误的关键字列表。

**GEP-0025-R003（构建门禁）：** `gan build` 首先执行 R001 裁决。任何错误都会以 `build aborted: check failed` 中止构建并退出码为 1；否则构建继续，警告和建议已打印完毕。`ganc build`（底层命令）保持无门禁。`gan run` 基于同一裁决进行门禁，但建议被抑制——错误会阻止运行，教学环节属于 `check`/`build`。

**GEP-0025-R004（交通信号灯）：** `lsc check` 的裁决以两个布尔值开头：`"ok"`（无错误——程序可编译）和 `"clean"`（ok，且无警告，且无建议）。代理的循环为：红色（`ok: false`）→ 修复错误；黄色（`ok` 但非 `clean`）→ 阅读建议；绿色（`clean: true`）→ 提交。

**GEP-0025-R005（信任线）：** 一个符合语言习惯的项目 MUST 裁决零建议——噪音会教会代理忽略 Advisor。参考标准：语言自身的标准库、工具链、教程和游乐场在自身检查下保持无建议。规则通过经受该基准线考验而获得其地位：`$builtins` 免于 pyimport 重复提示（它是环境命名空间），`rescue _e ->` 是有意吞并而非裸 rescue 违规，`Mod.t()` 在 `@spec`/`@type` 行中是类型拼写而非成员拼写错误，`$mod.Type()` 在 `@spec` 行中是规范语法本身要求的宿主类型拼写（GEP-0017-R002）且不计入 pyimport 重复，单次调用的 `fn` 包装仅在可捕获的调用方（`&f/1`——从不用于 `g.(x)` 点调用）时被标记，测试模块（使用 `use Test` 或 `TestX` 命名）是消费者而非库表面——注释覆盖率和 doctest 提示不适用于它们。

**GEP-0025-R006（项目表面）：** 裁决覆盖配置的源码根目录以及顶层 `tests/*.gan`——恰好是 `gan test` 运行的内容；`tests/` 下的更深层目录是测试夹具，不建议检查。测试文件仅接收 Advisor 扫描；其编译诊断属于 `gan test`。

**GEP-0025-R007（合并与锚点）：** 来自多个文件的相同建议消息合并为一条条目，并附带分布范围注释（`(also in N other file(s))`），每条建议携带其第一条证据的基于 1 的行号，以便代理可以直接跳转到该处。字面量屏蔽保留分隔符并在 sigil 主体中平衡嵌套括号——`~python(next((...), None))` 永远不会将其 `None` 泄漏到迁移提示中。

**GEP-0025-R008（工件验证）：** 裁决包含对编译工件的解析扫描：生成的 Python 使用 `ty` 进行检查，仅限于解析规则——未定义名称、无法解析的导入或模块未提供的成员是运行时致命事实，因此作为 **错误** 报告，并映射回 `.gan` 源码（去混淆后的名称、源码行、did-you-mean）。类型流意见（运算符支持、参数类型）不参与门禁；它们永远不会阻止构建。当 `ty` 不可用时，该层静默降级为无输出。原生扩展通过随附的 `.pyi` 存根暴露其表面（gandora-core 随附一个）。

**GEP-0025-R009（构建即裁决）：** 不存在独立的检查命令——`gan build` 运行完整的裁决（诊断、建议、工件验证），并在任何错误时在写入工件之前停止。`gan run` 基于同一裁决进行门禁，建议被抑制。`gan lsc check` 仍然是同一裁决的 JSON 表面，供工具和代理使用。已退役的 `gan check` 拼写会打印一条迁移说明并委托给构建。

## 原理

将沙箱折叠到检查（check）中遵循了工具自身的教训：一个智能体的循环是 编写 → 判决 → 修复 → 构建，而判决属于每个开发者已经运行的命令。将 Advisor 移入 gandora-tool 将教学引擎置于每个消费者（runner、lsc、未来的编辑器界面）之下，而无需新增一个包。

## 向后兼容性

**重大变更**：移除了 `gan try` 和 `gan lsc try`/`review`；
`gan lsc check` 从列表形式更改为 `{diagnostics, suggestions}`。
GEP-0023 被本 GEP 取代。

## 安全性与确定性

Check 不执行任何操作；构建门仅重新排序现有的步骤。

## 工具与 AI 使用

代理循环：编写 → `gan check`（或对于 JSON 使用 `lsc check`）→ 修复每个诊断，应用每个建议 → `gan test` → `gan build`。概念查询使用 `gan lsc doc <construct>`。

## 被拒绝的备选方案

### 保留 `try` 与 `check` 并存

两个名称对应同一个判决；代码片段执行部分实际上是 `gan run` 加上额外步骤。

### 对 `ganc build` 也进行门控

Plumbing 对脚本保持可预测性；porcelain（`gan build`）承载策略。

## 符合性

在`gan lsc check`上的BDD套件必须涵盖：干净模块的静默保证；错误/警告退出码契约；每种建议类型；使用已教纠正的lint直通；恶意输入考验（从不崩溃，始终JSON）；以及构建门的出错中止。对于修订版2，还必须涵盖：`ok`/`clean`红绿灯；每个R005豁免（一个测试模块，一个`rescue _e`，一个`Mod.t()`规范，一个点调用`fn`包装，以及`$builtins`重复，每个判定干净）；嵌套括号符号掩码；以及带有行锚点的跨文件合并。

## 变更历史

- 修订版 1，2026-08-04：初始版本 —— 取代 GEP-0023。
- 修订版 3，2026-08-04：R008 工件验证（基于已编译 Python 的类型解析规则，映射回源代码；类型流永不阻塞）；R009 构建包含检查 —— 一个判决，一个命令。
- 修订版 2，2026-08-04：R004 交通灯（`ok`/`clean`）；R005 零噪声信任线及存留规则细化；R006 项目表面包含顶层测试；R007 跨文件合并、行锚点、平衡符号掩码。

---
gep: 25
title: 检查
description: 一个裁决——编译器诊断加上顾问建议——由`gan check`打印，由`gan lsc check`以JSON格式返回，并门控`gan build`。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-04
updated: 2026-08-04
revision: 1
requires: [12, 13, 22]
replaces: [23]
superseded-by: null
resolution: null
language: zh-CN
source: ../../0025-the-check.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0025-the-check.md](../../0025-the-check.md)。

# GEP-0025: The Check

技术

## Abstract

`gan check` 是编译器对项目的整体裁决：所有诊断（错误和 GEP-0022 检查项）**以及**所有 Advisor 建议——实践差距、跨语言迁移提示、以及针对真实符号的编辑距离“你是不是想找”。`gan lsc check` 返回相同的裁决，作为一个 JSON 对象 `{diagnostics, suggestions}`。**`gan build` 首先运行检查**：在写入任何产物之前，错误会阻止构建；警告和建议会打印出来并让构建继续——这是重型编译器的契约，Rust 的方式。GEP-0023 沙箱 (`gan try`) 已退役：其教学引擎现在存在于此处，在项目范围内，位于已有的命令之下。

## 动机

每个片段（`try`）的判定重复了 `check` 本应做的事情。一个拼写——check——服务于终端的人类、通过 JSON 的代理、通过相同诊断的编辑器，以及构建门，无需额外学习。

## 范围

检查判定和构建门禁。执行仍然由 `gan run`/`gan test`/`gan repl` 负责。

## 规范

**GEP-0025-R001（裁决）：** `gan check` 按文件输出：编译器诊断（带跨度；错误和警告），随后是 Advisor 建议，每项建议标记为 `practice` | `migration` | `did_you_mean`。当存在任何错误时退出码为 1；警告和建议永远不会导致检查失败。`gan lsc check` 发出相同的裁决，格式为 `{"diagnostics": [...], "suggestions": [...]}`，每个条目包含路径，并遵循相同的退出码约定。

**GEP-0025-R002（Advisor）：** 建议引擎是 `gandora-tool` 中的 `Advisor` —— 为运行器和 `lsc` 提供统一实现。它在任何文本检查之前屏蔽字符串/heredoc/标识符/注释内容（散文永远不会触发模式），按消息去重，并查询真实候选项：用于成员拼写错误的模块符号（带有 gandora-std 内省回退，因此没有虚拟环境的项目仍然可以获得 `Enum.mpa → Enum.map`），用于未定义变量的文件自身标识符，以及用于构造器拼写错误的关键字列表。

**GEP-0025-R003（构建门禁）：** `gan build` 首先执行 R001 的裁决。任何错误都会导致构建中止，输出 `build aborted: check failed` 并以退出码 1 退出；否则构建继续，此时警告和建议已打印完毕。`ganc build`（管道命令）保持无门禁状态。

## 理由

将沙箱折叠到 check 中，遵循了该工具自身的经验教训：智能体的循环是 编写 → 裁决 → 修复 → 构建，而裁决属于每个开发者已在运行的命令。将 Advisor 移入 gandora-tool 后，教学引擎便置于所有使用者（runner、lsc、未来的编辑器界面）之下，无需新增包。

## 向后兼容性

**破坏性变更**：`gan try` 和 `gan lsc try`/`review` 被移除；
`gan lsc check` 从列表形式变更为
`{diagnostics, suggestions}`。GEP-0023 被本 GEP 取代。

## Security and Determinism

Check 不执行任何操作；构建门仅重新排序现有步骤。

## 工具与 AI 使用

智能体循环：编写 → `gan check`（对于 JSON 使用 `lsc check`）→ 修复每条诊断，应用每条建议 → `gan test` → `gan build`。  
概念查询使用 `gan lsc doc <construct>`。

## 被拒绝的替代方案

### 将 `try` 与 `check` 一同保留

一个裁决对应两个名称；代码片段执行的那一半是 `gan run` 加上额外的步骤。

### 也对 `ganc build` 进行门控

底层管道对脚本保持可预测性；上层命令（`gan build`）承载策略。

## 符合性

一个针对 `gan lsc check` 的 BDD 测试套件 MUST 覆盖：干净模块的静默保证；错误/警告退出码契约；每种建议类型；经过教导修正的 lint 传递；恶意输入测试（绝不崩溃，始终输出 JSON）；以及构建门的错误中止。

## 变更历史

- 修订版 1，2026-08-04：初始版本 — 取代 GEP-0023。

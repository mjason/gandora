---
gep: 2
title: 宏系统
description: 编译时 defmacro 元编程，支持 quote/unquote、默认卫生性，以及确定性的沙盒化展开阶段。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Macros
created: 2026-08-01
updated: 2026-08-01
revision: 1
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0002-macro-system.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0002-macro-system.md](../../0002-macro-system.md)。

# GEP-0002: 宏系统

## 摘要

Gandora 宏是编译时从语法到语法的函数，使用 Elixir 写法的 `defmacro`、`quote`、`unquote` 和 `unquote_splicing` 编写。宏展开默认是卫生的，在编译器内部的确定性沙箱中运行，并在 Python 代码生成之前完成。这遵循了 Osiris 项目的宏模型（阶段分离、卫生、数据驱动），同时保留了 Elixir 的编写界面。

## 动机

宏系统使得标准库和用户代码能够扩展语言（控制流形式、DSL、消除样板代码），而无需为每个特性增大编译器。需要卫生和阶段分离，以确保展开可以安全地组合，并且构建保持可重现。

## Scope

`defmacro` 定义和调用，引用形式，卫生规则，扩展沙箱，以及 `gan expand`。跨包（超出项目本地模块）的宏导出/导入被推迟。

## Terminology

- **AST value**：宏接收和返回的 Gandora 语法的带引号表示。
- **Expansion phase**：宏体在编译时的求值过程。
- **Hygiene**：宏模板引入的名称不会捕获或与调用点处的名称冲突的属性。

## 规范

**GEP-0002-R001:** `defmacro name(params) do body end` 定义一个宏。宏 MUST 在模块顶层定义。宏调用在编译时展开；其结果 MUST 是一个 AST 值，该值会重新进入展开过程，直到没有宏调用剩余，且具有有界展开深度。

**GEP-0002-R002:** 宏参数以未求值的 AST 值形式到达。`quote do ... end` 构建一个 AST 值；`unquote(expr)` 将一个已求值的 AST 值插入到 quote 中；`unquote_splicing(list)` 将一个 AST 值列表拼接到封闭序列中。引用的表示 MUST 遵循 Elixir 的形状：调用是 `{name, meta, args}`，字面量表示自身。

**GEP-0002-R003:** 宏体 MUST 由确定性的编译时解释器在 Gandora 的表达子集上求值（绑定、算术和比较、布尔形式、`if`/`case`/`cond`、列表、元组和映射构造、模式匹配、对其他编译时可用的函数和宏的调用，以及引用形式）。宏体 MUST NOT 执行 I/O、导入 Python、读取时钟或环境，或观察其参数之外的任何内容。相同的输入 MUST 产生相同的展开。

**GEP-0002-R004:** 展开 MUST 默认是卫生的：在 `quote` 模板内部绑定的变量会被重命名为一个新的、展开唯一的名称，因此它不能捕获或被子调用点绑定捕获。通过 `unquote` 插入的变量保持调用点身份。`var!(name)` 在 quote 内部故意逃避卫生并使用调用点名称。

**GEP-0002-R005:** 在展开宏时引发的编译错误 MUST 报告宏调用点和宏定义内部的位置，作为有序的起源链。

**GEP-0002-R006:** 模块使用的宏 MUST 来自同一模块或来自通过 `require` 或 `import` 命名的模块（GEP-0001-R006）。宏解析 MUST NOT 依赖于文件编译顺序；编译器首先解析项目依赖图，宏依赖循环是编译错误。

**GEP-0002-R007:** `defmodule`、`def`、`defp`、`defmacro`、`alias`、`import`、`require` 和 `pyimport` MUST NOT 在 v0 中由宏展开生成；宏在体内部产生表达式和语句。此限制 MAY 由未来的 GEP 解除。

**GEP-0002-R008:** `gan expand <file>` MUST 打印经过完整宏展开后的模块，以表面语法呈现，不写入任何产物。

## 理由

在编译器内部解释宏体（而不是将其编译为 Python 并导入）使得编译过程不受用户代码执行的影响，并保持构建的可确定性——这与 Osiris 在其第一阶段求值器中的决定相同。Elixir 的 `{name, meta, args}` 引用形式被保留，以便文档和直觉直接传递。

默认卫生（hygiene）并带有显式 `var!` 转义的方式与 Elixir 和 Osiris 一致；默认非卫生的系统会使组合宏以用户无法调试的方式失败。

## 向后兼容性

宏表面的基础提案；引用的AST形状（R002）是未来工具所依赖的兼容性契约。

## 安全性与确定性

扩张沙箱（R003）是安全边界：宏展开不能窃取数据、访问文件系统或使构建不可重现。展开深度和步骤限制可以防止编译永不终止。

## 工具与 AI 使用

代理应使用 `gan expand` 验证宏行为，而不是在脑海中推理宏展开，并应将 R007 的限制视为在 v0 中宏可生成内容的边界。

## 被拒绝的替代方案

### 将宏编译为 Python 并在构建时导入

对于繁重的宏来说更快，但编译会执行任意用户代码，并依赖于本地 Python 环境，破坏了确定性和 GEP-0001 的无导入保证。

### 不卫生的宏与手动生成符号

实现起来更简单，但将正确性责任推给了每个宏作者；组合失败会在用户调用点暴露。

## 开放问题

v0 无。

## 一致性

测试套件必须覆盖：卫生式重命名、`var!` 捕获、头部和尾部位置的 `unquote` 和 `unquote_splicing`、嵌套宏调用、递归深度失败、R007 限制诊断以及 `gan expand` 输出稳定性。

## 变更历史

- 修订版 1，2026-08-01：初始版本。

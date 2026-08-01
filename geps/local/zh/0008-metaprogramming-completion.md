---
gep: 8
title: 元编程补全
description: 定义生成宏，使用/__using__，在定义头部使用unquote，以及带有定义钩子的用户可扩展属性系统。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Macros
created: 2026-08-01
updated: 2026-08-01
revision: 1
requires: [2, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0008-metaprogramming-completion.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0008-metaprogramming-completion.md](../../0008-metaprogramming-completion.md)。

# GEP-0008: 元编程补全

## 摘要

本提案完善了 Elixir 的元编程体系（引用与反引用、宏、领域特定语言指南）。宏现在可生成定义；`def unquote(name)(...)` 在模板中生效；`use Mod` 调用 `Mod.__using__`。内置属性不再特殊：`defattr` 注册用户属性（可累积也可不累积），`@on_definition` 命名一个钩子宏，该宏接收每个定义及其收集的属性，并可重写该定义——这正是 `@doc`/`@example` 类特性所基于的机制，现开放给用户使用。

## Motivation

v0 宏系统（GEP-0002）有意禁止生成定义，这阻碍了定义 Elixir 模式：`deftest` 风格的 DSL、`use` 注入的 API，以及属性驱动的代码生成（路由、模式）。与此同时，Gandora 自身的 `@doc`/`@example` 是由编译器硬编码的；Elixir 展示了更好的形态——`Module.register_attribute/3` 加上 `@on_definition`——语言由此在用户空间中生长出自己的注解系统。

## 范围

定义生成、`use`、`unquote` 在头部中、属性注册和定义钩子。`@before_compile`/`@after_compile` 钩子和跨包钩子分发被推迟。

## 术语

- **声明宏**：在模块顶层调用的宏，其展开产生定义。
- **已注册属性**：由 `defattr` 声明的属性。
- **定义钩子**：由 `@on_definition` 命名的宏，对每个后续定义运行。

## 规范

**GEP-0008-R001:** 模块顶层宏展开可以产生 `def`、`defp`、`defstruct`、文档属性、`@decorate` 以及这些内容的 `__block__` 序列，编译器会将其展平到模块主体中。头部形式（`defmodule`、`alias`、`import`、`require`、`pyimport`、`use`）MUST NOT 由宏生成。此项修正了 GEP-0002-R007。

**GEP-0008-R002:** 在引号模板中，`def unquote(expr)(params)`（以及 `defp`）定义了一个函数，其名称是 `expr` 在展开时的原子或字符串值，从而允许名称计算的定义。

**GEP-0008-R003:** 模块顶层使用 `use Mod` 和 `use Mod, opts` 等价于先 `require Mod`，然后原地展开 `Mod.__using__(opts)`（参数数量为 0 或 1）。如果 `use` 的目标没有 `__using__`，则产生编译错误，并指出该模块名称。

**GEP-0008-R004:** `defattr :name` 为当前模块注册一个模块属性；`defattr :name, accumulate: true` 使得重复的 `@name value` 按源码顺序累积，而不是报错。值为引号项。`@name` 读取遵循 GEP-0004-R011；累积属性读取为列表。未注册的内置属性仍然保持为 GEP-0004 的模块属性绑定。

**GEP-0008-R005:** `@on_definition Mod.hook`（在 `require Mod` 之后）注册一个定义钩子。对于后续的每个 `def`/`defp`，编译器展开 `Mod.hook(kind, head, attrs, body)`，其中 `kind` 为 `:def`/`:defp`，`head` 和 `body` 是定义的引号项，`attrs` 是自前一个定义以来收集的已注册属性值的关键字列表（这些值随后被重置，类似于 `@doc`）。钩子返回替换后的顶层语法（通常是重构后的定义及其注册），受限于 R001。钩子在 GEP-0002-R003 的沙箱中运行，并遵循其确定性和限制。

**GEP-0008-R006:** 内置属性（`@doc` 家族、`@example`、`@decorate`、`@moduledoc` 家族）保持其 GEP-0007/0003 语义，且对钩子不可见；`defattr` 的名称与内置属性冲突时会引发编译错误。

## 理由

定义钩子接收 (kind, head, attrs, body) 正是 Elixir 库构建注解系统的方式；将收集到的属性传递给重写宏，涵盖了装饰器注册表、路由表和类似文档的通道，而无需为每个用例添加编译器特性。`use` 加上声明宏则完整覆盖了 DSL 指南中的 `deftest` 模式。头部形式保持不可生成，从而模块标识和依赖图保持静态可知（GEP-0002-R006 的保证）。

## 向后兼容性

修订GEP-0002-R007（仅放宽）。现有属性语义（GEP-0004）对于未注册名称保持不变。

## 安全性与确定性

所有内容都在现有的扩展沙箱中运行；钩子不添加新功能，只添加新输入。扩展保持确定性和有界性。

## 工具与 AI 使用

Agent 应选用 `use` 加声明宏（declaration macros）来处理 DSL，选用 `defattr` 加 `@on_definition` 来处理注解系统，并使用 `gan expand` 验证生成的定义。

## 被否决的替代方案

### 针对每个特性的编译器专用装饰器

每个新的注解（路由、缓存、追踪）都需要编译器的工作；钩子机制将这项工作转移到了库中，正如 Elixir 所做的那样。

### 允许宏生成的导入/defmodule

这将使模块图依赖于扩展结果，从而破坏静态宏解析和包发现。

## 开放问题

此版本无。

## 符合性

测试必须涵盖：一个生成多个定义的声明宏（包括扁平化）；`def unquote(name)(...)`；带和不带选项的 `use` 及其缺少 `__using__` 的诊断；累加和非累加的 `defattr`（具有按定义重置语义）；一个装饰并注册定义的 `@on_definition` 钩子；R006 冲突诊断；以及上述所有内容的 `gan expand` 输出。

## 变更历史

- 修订版 1, 2026-08-01: 初始版本。

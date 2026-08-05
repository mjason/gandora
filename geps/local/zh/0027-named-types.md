---
gep: 27
title: 具名类型
description: @type — 命名和泛型类型别名，带有声明点、元数检查和编译时结构展开；零运行时开销。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-05
updated: 2026-08-05
revision: 1
requires: [17]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0027-named-types.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0027-named-types.md](../../0027-named-types.md)。

# GEP-0027: 命名类型

## 摘要

类型语言曾经拥有内置类型、类型变量和类——但缺少*命名*类型的方式。`@type` 添加了具名别名和泛型别名，并带有真正的声明点：参数在头部声明，引用会进行参数数量检查，所有内容在编译时按结构展开为 PEP 484 注解。如同往常，零运行时开销。

```elixir
@type age() :: integer()
@type result(t) :: tuple(atom(), t)
@type scores() :: map(string(), age())

@spec parse(string()) :: result(integer())   # -> tuple[str, int]
@spec load(string()) :: Mod.result(string()) # cross-module
```

## Specification

**GEP-0027-R001 (声明):** `@type name(params) :: type` 在模块顶层声明一个命名类型。名称必须是纯小写单词；MUST NOT 与内置类型冲突，MUST NOT 为 `t`（已被 GEP-0017 修订版 5 废弃），且 MUST NOT 被声明两次。前置的 `@doc` 为该类型提供文档；`@spec`/`@param` 不适用于该类型。

**GEP-0027-R002 (泛型具有声明点):** 参数是列在头部的一个或两个小写字母的类型变量。主体中若出现未声明的裸变量，则视为编译错误。（在 `@spec` 内部，变量仍保持隐式作用域限定于该 spec —— 轻量级层级保持不变。）

**GEP-0027-R003 (引用与展开):** 命名类型以调用方式引用 —— 在其自身模块中为 `age()`、`result(integer())`，从任意项目模块中为 `Mod.result(string())`。引用需检查参数数量。展开是在编译时对最终注解进行结构替换；递归定义视为编译错误（带有深度上限的循环检测）。引用一个模块未声明的类型视为编译错误；未知小写类型会获得针对内置类型及该模块自身 `@type` 名称的“您是否在找”提示。

**GEP-0027-R004 (建议):** 返回类型完全由未出现在任何参数中的类型变量构成的 `@spec` 会得到一个实践提示 —— 仅使用一次的变量不约束任何东西。

## 已推迟

约束泛型（`a when a: number()`）、泛型结构体、`@opaque` 以及跨*包*命名类型（需要 wheel 中的类型元数据）。宏生成的 `@type` 声明不在此版本中收集。

## 合规性

测试 MUST 涵盖：别名和泛型展开（同模块和跨模块、嵌套别名）；参数个数、未声明参数、重复、遮蔽、`t` 和循环错误；针对近似名称的“您是否指的是”功能；以及孤立返回变量提示。

## 变更历史

- 修订版 1, 2026-08-05: 初始版本。

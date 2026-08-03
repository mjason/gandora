---
gep: 20
title: 推导式
description: Elixir的for推导式——生成器、过滤器和into——被编译成Python推导式。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-03
updated: 2026-08-03
revision: 1
requires: [1, 19]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0020-comprehensions.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0020-comprehensions.md](../../0020-comprehensions.md)。

# GEP-0020: 推导式

## 摘要

Elixir 的 `for`:

```elixir
for x <- xs, x > 0, y <- ys, do: {x, y}
```

编译为 Python 推导式 `[(x, y) for x in xs if x > 0 for y in ys]`。生成器绑定模式，子句间的裸表达式是过滤器，`into: %{}` 产生字典推导式。通过 GEP-0019 的递归和 Enum，这完成了迭代故事，使得 `loop` 退役（GEP-0014 修订版 3）。

## 动机

`for` 是 Elixir 的核心语法结构（GEP-0001-R005 要求在支持的地方提供该语法表面），并且一对一地映射到 Python 最受喜爱的构造——编译后的输出正是一个 Python 审查者会手动编写的代码。

## 范围

列表和字典推导式，包含生成器、过滤器和 `into:`。`reduce:`、`uniq:` 和二元生成器是未来修订。

## 规范

**GEP-0020-R001：** `for gen_or_filter, ..., do: body` 是一个表达式。每个 `pattern <- enumerable` 是一个生成器；任何其他子句表达式都是一个过滤器，在已有的绑定下求值。子句从左到右嵌套。`do:` 主体（简写形式或块）为每个通过组合生成一个元素。

**GEP-0020-R002：** 生成器模式如果是普通变量，则直接绑定（`for x in xs`）。任何其他模式编译为一次匹配，该匹配会 **跳过不匹配的元素**（Elixir 语义），通过匹配守卫上的过滤器实现——绝不会引发 `GanMatchError`。

**GEP-0020-R003：** 如果没有 `into:`，结果为列表，编译为列表推导式。`into: %{}` 要求主体必须是二元组 `{k, v}`，并编译为字典推导式。其他 `into:` 目标均为编译错误，并引用此规则。

**GEP-0020-R004：** 推导式变量作用域限于推导式内部（Python 3 推导式作用域规则）；绑定不会泄漏。推导式主体不是尾位置（GEP-0019-R001）。

## 理由

编译为原生推导式同时保持了零运行时承诺和最佳可读性。模式跳过生成器需要谨慎处理：元组模式成为结构守卫，匹配 Elixir 的过滤非崩溃语义。

## 向后兼容性

`for` 之前是一个不支持结构的诊断。新增。

## 安全性与确定性

仅本地控制流；确定性输出。

## 工具与 AI 使用

Agents SHOULD 优先使用 `for` 而非 `Enum.map`/`filter` 链，当单个列表推导式的可读性更好时，并且 MUST NOT 使用 `for` 进行副作用操作（应使用 `Enum.each`）。

## 被拒绝的替代方案

### 脱糖为 Enum 调用

`for x <- xs, do: f(x)` 作为 `Enum.map` 会在输出中引入标准库依赖，而原生的推导式更清晰且更快。

## 符合性

测试 MUST 涵盖：单生成器和多生成器、生成器之间和之后的过滤器、跳过的模式生成器、`into: %{}`、嵌套推导式、非泄漏作用域，以及非元组体 `into: %{}` 错误。

## 变更历史

- 修订版 1，2026-08-03：初始版本。

---
gep: 17
title: Typespecs 和 Typed Boundary
description: Elixir风格的@spec注解，编译为Python类型提示——类型是用于工具和互操作边界的声明，而非运行时。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0017-typespecs.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0017-typespecs.md](../../0017-typespecs.md)。

# GEP-0017: 类型规范与类型化边界

## 摘要

`@spec` 将 Elixir 的 typespec 表面带到了 Gandora，并将其编译为生成 Python 中的 PEP 484 类型提示：

```elixir
@spec mean(list(number()), integer()) :: float()
def mean(xs, precision \\ 2), do: ...
```

变为 `def mean(xs: list[int | float], precision: int = ...) -> float:`。
收益是双重的：生成的模块为下游 Python 调用者提供 *类型化的 Python API*，并且整个 Python 类型检查生态（pyright、mypy）无需修改即可应用于编译后的输出。
`$module.Type` 引用是有效的类型，因此互操作边界可以用宿主自身的类型进行注解。
Specs 是声明——它们在运行时不添加任何东西，而是通过文档和 LSP 呈现出来。

## 动机

Elixir 已承诺引入类型系统，而 Python 已经拥有一个；Gandora 介于两者之间，如果没有类型体系，将比两者都逊色。最省事且诚实的做法是效仿 Elixir：类型化*声明*具有工具价值，无运行时检查——在此以 Python 注解实现，因为这是每个 Python 工具已经理解的唯一目标。

## 范围

`@spec` 属性、类型表达式语言、其到 Python hints 的映射，以及文档/LSP 呈现。Gandora 侧的类型*检查*、`@type` 别名和结构体字段类型是未来修订的内容。

## 术语

- **Spec**：一个 `@spec name(arg_type, ...) :: return_type` 属性。
- **类型表达式**：R002 的术语语言。

## 规范

**GEP-0017-R001:** `@spec fun(type, ...) :: type` 紧跟在定义（或它的 `@doc` 块）之前，附着到下一个具有该名称的 `def`/`defp`/`defmacro` 组。`::` 和联合运算符 `|` 在表达式中被识别，但仅在 `@spec` 内部有效；在其他地方，它们是引用此 GEP 的编译错误。一个类型规范，其参数数量与后续定义的任何子句都不匹配，则是一个编译错误。

**GEP-0017-R002:** 类型表达式及其 Python 映射：

| Gandora 类型 | Python 类型提示 |
| --- | --- |
| `integer()` | `int` |
| `float()` | `float` |
| `number()` | `int \| float` |
| `boolean()` | `bool` |
| `string()` | `str` |
| `atom()` | `str` |
| `nil` | `None` |
| `any()` | `object` |
| `list()` / `list(t)` | `list` / `list[t]` |
| `map()` / `map(k, v)` | `dict` / `dict[k, v]` |
| `tuple()` / `tuple(a, b, ...)` | `tuple` / `tuple[a, b, ...]` |
| `fun()` | `collections.abc.Callable` |
| `a \| b` | `a \| b` |
| `$mod.Type` | `mod.Type`（模块导入已记录） |
| `Mod.t()` | 为 `Mod` 生成的结构体类（GEP-0004） |

任何其他表达式都是一个引用此规则的编译错误。

**GEP-0017-R003:** 编译会对生成的 Python 函数进行注解：一个单一的普通参数子句按顺序接收参数类型提示加上返回类型提示；任何通过 `*args` 进行分派的组（多子句、模式参数或 `\\` 默认值）会在分派器上接收返回注解。输出是确定性的，并且不会添加除类型提示所命名的导入之外的任何导入。类型规范 MUST NOT 产生运行时检查、包装器或模块属性。

**GEP-0017-R004:** 类型规范加入文档渠道：`gan doc` 将它们打印在正文之上，`gandora_core.doc` 将它们作为 `specs` 返回，并且 LSP 在悬停和签名帮助中显示它们（GEP-0015）。

## Rationale

编译为提示而非发明检查器，意味着每个消费者——CI中的pyright、对生成代码的IDE、导入Gandora wheel的Python调用者——从一开始就能获得价值，而未来Gandora侧的检查器也可以基于相同的声明进行分层。规范中的`$mod.Type`使得互操作边界成为语言中类型化*最*强的部分，而非*最*弱的。`atom()`到`str`的映射诚实地遵循了GEP-0001-R009的值映射。

## 向后兼容性

新增：`::` 和 `|` 先前在表达式中是解析错误。

## 安全性与确定性

提示是生成源代码中的文本；不执行任何操作。

## 工具与 AI 使用

代理 SHOULD 在公共函数上编写 `@spec`，并 MAY 对 `outDir` 运行 pyright 或 mypy 作为 Gandora 程序的类型检查。

## 被否决的替代方案

### 先实现 Gandora 侧的类型检查器

没有声明的检查器什么都检查不了；没有检查器的声明仍然是提示、文档和类型化的 API。声明优先。

### 运行时类型断言

违反 GEP-0001-R002 零运行时以及 Elixir 自身的立场：规范用于文档化，工具用于验证；代码在调用时不付出代价。

## 符合性

测试 MUST 涵盖：每个 R002 映射，包括联合类型、参数化类型、`$mod.Type` 和 `Mod.t()`；参数数量不匹配和 `::` 位置错误；多子句和默认参数的发射形状；文档和 LSP 呈现；以及对着带类型注释的输出运行 pyright 以接受一个类型正确的模块。

## 变更历史

- 修订版 1, 2026-08-02: 初始版本。

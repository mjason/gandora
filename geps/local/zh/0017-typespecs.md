---
gep: 17
title: 类型规范与类型边界
description: Elixir风格的@spec注解，编译成Python类型提示——类型是给工具和互操作边界的声明，永远不是运行时。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-02
updated: 2026-08-02
revision: 3
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0017-typespecs.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0017-typespecs.md](../../0017-typespecs.md)。

# GEP-0017：类型规范与类型化边界

## 摘要

`@spec` 将 Elixir 的 typespec 表面引入 Gandora，并将其编译为生成 Python 中的 PEP 484 类型提示：

```elixir
@spec mean(list(number()), integer()) :: float()
def mean(xs, precision \\ 2), do: ...
```

变为 `def mean(xs: list[int | float], precision: int = ...) -> float:`。
收益是双重的：生成的模块成为下游 Python 调用者的 *类型化 Python API*，并且整个 Python 类型检查生态系统（pyright、mypy）无需修改即可应用于编译输出。
`$module.Type` 引用是有效的类型，因此互操作边界可以用宿主自身的类型进行注解。
Specs 是声明——它们在运行时不添加任何内容，并通过文档和 LSP 呈现。

## 动机

Elixir 已经承诺采用类型系统，而 Python 已经拥有一个；Gandora 介于两者之间，如果没有类型化方案，将比两者都更逊色。最诚实的低成本步骤是 Elixir 的做法：类型化的*声明*，具有工具价值，无运行时检查——在此处实现为 Python 注解，因为这是每个 Python 工具已经理解的唯一目标。

## 范围

`@spec` 属性、类型表达式语言、其到 Python 类型提示的映射，以及文档/LSP 的展现。Gandora-side type *checking*、`@type` 别名和结构体字段类型属于未来修订。

## 术语

- **Spec**：一个 `@spec name(arg_type, ...) :: return_type` 属性。
- **类型表达式**：R002 的术语语言。

## 规范

**GEP-0017-R001：** 在定义（或其 `@doc` 块）之前紧邻的 `@spec fun(type, ...) :: type` 会附加到下一个同名的 `def`/`defp`/`defmacro` 组上。`::` 和联合运算符 `|` 在表达式中被识别，但仅在 `@spec` 内部有效；其他地方则会引发编译错误并引用本 GEP。如果某个规范的类型变量数量与后续定义的任何子句不匹配，则引发编译错误。

**GEP-0017-R002：** 类型表达式及其 Python 映射。

标量类型：

| Gandora 类型 | Python 类型提示 |
| --- | --- |
| `integer()` | `int` |
| `float()` | `float` |
| `number()` | `int \| float` |
| `boolean()` | `bool` |
| `string()` | `str` |
| `atom()` | `str` |
| `nil` | `None` |
| `any()` / `term()` | `object` |

具体容器：

| Gandora 类型 | Python 类型提示 |
| --- | --- |
| `list()` / `list(t)` | `list` / `list[t]` |
| `map()` / `map(k, v)` | `dict` / `dict[k, v]` |
| `tuple()` / `tuple(a, b, ...)` | `tuple` / `tuple[a, b, ...]` |
| `keyword()` | `list[tuple[str, object]]` |

抽象容器——一等公民，无需宿主引用；在参数位置优先使用它们（当具体容器不变时，它们是协变的）：

| Gandora 类型 | Python 类型提示 |
| --- | --- |
| `iterable()` / `iterable(t)` | `collections.abc.Iterable[t]` |
| `sequence()` / `sequence(t)` | `collections.abc.Sequence[t]` |
| `mapping()` / `mapping(k, v)` | `collections.abc.Mapping[k, v]` |
| `fun()` | `collections.abc.Callable` |

组合与边界：

| Gandora 类型 | Python 类型提示 |
| --- | --- |
| `a \| b` | `a \| b` |
| `$mod.Type` | `mod.Type`（记录模块导入） |
| `$mod.Type(t, ...)` | `mod.Type[t, ...]`——参数化宿主类型 |
| `Mod.t()` | 为 `Mod` 生成的 struct 类（GEP-0004） |

`$mod.Type` 用于宿主*特定*类型（`$np.ndarray`、`$re.Pattern`、`$decimal.Decimal`）；上述标准抽象是内建的，因此惯用的规范永远不需要引用带引号的模块。任何其他表达式都会引发编译错误并引用本规则。

**GEP-0017-R003：** 编译会对生成的 Python 函数进行注解：单个普通参数子句会按顺序获得参数类型提示外加返回类型提示；任何通过 `*args` 进行分发的组（多子句、模式参数或 `\\` 默认值）会在分发函数上获得返回类型注解。生成是确定性的，并且除了提示中命名的导入外，不会添加任何导入。规范 MUST NOT 产生运行时检查、包装器或模块属性。

**GEP-0017-R005：** 在类型位置上，最多两个字符的小写裸名称是一个**类型变量**：`@spec map(list(a), fun()) :: list(b)`。每个变量编译为一个模块级别的 `typing.TypeVar` 声明（`_T_a = typing.TypeVar("_T_a")`），确定性地生成，重复使用会统一类型。较长的裸名称会引发编译错误，建议使用 `name()` 拼写——这可以防止忘记括号的拼写错误无声地变成变量。Elixir 的 `when a: var` 子句被接受为未来语法，目前尚未要求。

**GEP-0017-R004：** 规范加入文档通道：`gan doc` 会将它们打印在正文之上，`gandora_core.doc` 以 `specs` 返回它们，LSP 在悬停和签名帮助中显示它们（GEP-0015）。

## Rationale

编译为类型提示而不是发明一个检查器，意味着每个消费者——CI 中的 pyright、处理生成代码的 IDE、导入 Gandora wheel 的 Python 调用者——从第一天起就能获得价值，并且未来的 Gandora 端检查器仍然可以基于相同的声明进行分层。规范中的 `$mod.Type` 使得互操作边界成为语言中 *最* 有类型化的部分，而不是最没有。`atom()` 映射到 `str` 忠实地遵循了 GEP-0001-R009 的值映射。

## 向后兼容性

新增的：`::`和`|`以前在表达式中是解析错误。

## 安全性与确定性

提示是生成的源代码中的文本；不执行任何操作。

## 工具与 AI 使用

智能体 SHOULD 在公共函数上编写 `@spec`，并 MAY 对 `outDir` 运行 pyright 或 mypy，作为 Gandora 程序的类型检查。

## 被拒绝的替代方案

### 先实现 Gandora 侧的类型检查器

没有声明的检查器检查不了任何东西；而没有检查器的声明仍然是提示、文档和类型化的 API。声明优先。

### 运行时类型断言

违反了 GEP-0001-R002 的零运行时（zero-runtime）要求以及 Elixir 自身的立场：规范文档和工具验证；代码在调用时不应付出代价。

## 符合性

测试必须涵盖：每个 R002 映射，包括联合类型、参数化类型、`$mod.Type` 和 `Mod.t()`；参数数量不匹配和 `::` 位置错误；多子句和默认参数发射形状；文档和 LSP 呈现；以及 pyright 对带注释的输出运行，接受一个类型良好的模块。

## 变更历史

- 修订3，2026-08-02：抽象容器（`iterable`, `sequence`, `mapping`）和 `keyword()`/`term()` 成为内置类型；添加了类型变量（R005，编译为 `typing.TypeVar`）；重新组织了 R002，指导原则是 `$mod.Type` 仅用于特定于宿主机的类型，而 `$mod.sub.Type` 链根据 GEP-0003-R010 规则解析，无需引用。

- 修订2，2026-08-02：R002 — 参数化的宿主类型 `$mod.Type(t, ...)` 映射到下标的类型提示，例如 `$"collections.abc".Sequence(number())` 变成 `collections.abc.Sequence[int | float]`。

- 修订1，2026-08-02：初始版本。

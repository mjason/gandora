---
gep: 17
title: 类型规范与类型边界
description: Elixir风格的@spec注解，编译为Python类型提示——类型是用于工具和互操作边界的声明，而非运行时。
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

# GEP-0017: 类型规范与类型化的边界

## 摘要

`@spec` 将 Elixir 的 typespec 表面引入 Gandora，并将其编译为生成的 Python 中的 PEP 484 类型提示：

```elixir
@spec mean(list(number()), integer()) :: float()
def mean(xs, precision \\ 2), do: ...
```

变为 `def mean(xs: list[int | float], precision: int = ...) -> float:`。
收益是双重的：生成的模块成为 *带类型的 Python API*，供下游 Python 调用者使用，并且整个 Python 类型检查生态系统（pyright, mypy）无需修改即可应用于编译后的输出。
`$module.Type` 引用是有效的类型，因此互操作边界可以用宿主自身的类型进行注解。Spec 是声明——它们在运行时不会添加任何内容，并通过文档和 LSP 呈现。

## 动机

Elixir 已承诺采用类型系统，而 Python 已经拥有一个；Gandora 介于两者之间，如果没有类型系统，将比两者都更逊色。最经济且诚实的步骤是借鉴 Elixir 的做法：带有工具价值的类型化*声明*，不进行运行时检查——在这里以 Python 注解的形式实现，因为这是每个 Python 工具都已理解的目标。

## 范围

`@spec` 属性、类型表达式语言、类型表达式语言到 Python hints 的映射，以及文档/LSP 展示。Gandora 侧的类型 *检查*、`@type` 别名和结构体字段类型属于未来版本。

## 术语

- **Spec**：一个 `@spec name(arg_type, ...) :: return_type` 属性。
- **类型表达式**：R002 的项语言。

## 规范

**GEP-0017-R001：** 在定义（或其 `@doc` 块）之前立即出现的 `@spec fun(type, ...) :: type` 会附加到紧随其后的同名 `def`/`defp`/`defmacro` 组。`::` 和联合运算符 `|` 在表达式中被识别，但仅在 `@spec` 内部有效；在其他地方会导致命名此 GEP 的编译错误。一个规范，其元数与后续定义的任何子句都不匹配，则产生编译错误。

**GEP-0017-R002：** 类型表达式及其 Python 映射。

标量类型：

| Gandora 类型 | Python 提示 |
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

| Gandora 类型 | Python 提示 |
| --- | --- |
| `list()` / `list(t)` | `list` / `list[t]` |
| `map()` / `map(k, v)` | `dict` / `dict[k, v]` |
| `tuple()` / `tuple(a, b, ...)` | `tuple` / `tuple[a, b, ...]` |
| `keyword()` | `list[tuple[str, object]]` |

抽象容器——一等公民，无需宿主引用；在参数位置优先使用它们（它们是协变的，而具体容器是不变的）：

| Gandora 类型 | Python 提示 |
| --- | --- |
| `iterable()` / `iterable(t)` | `collections.abc.Iterable[t]` |
| `sequence()` / `sequence(t)` | `collections.abc.Sequence[t]` |
| `mapping()` / `mapping(k, v)` | `collections.abc.Mapping[k, v]` |
| `fun()` | `collections.abc.Callable` |

组合类型与边界：

| Gandora 类型 | Python 提示 |
| --- | --- |
| `a \| b` | `a \| b` |
| `$mod.Type` | `mod.Type`（记录模块导入） |
| `$mod.Type(t, ...)` | `mod.Type[t, ...]` —— 参数化的宿主类型 |
| `Mod.t()` | 为 `Mod` 生成的 struct 类（GEP-0004） |

`$mod.Type` 用于宿主*特定*类型（`$np.ndarray`、`$re.Pattern`、`$decimal.Decimal`）；上述标准抽象是内置的，因此惯用的规范永远不需要显式的模块边界。任何其他表达式都会导致命名此规则的编译错误。

**GEP-0017-R003：** 编译会注释生成的 Python 函数：单个普通参数子句会按顺序接收参数提示以及返回提示；任何通过 `*args` 进行分派的组（多子句、模式参数或 `\\` 默认值）会在分派器上接收返回注解。输出是确定性的，且不会添加超出提示所命名的导入。规范 MUST NOT 产生运行时检查、包装器或模块属性。

**GEP-0017-R005：** 在类型位置中，一个最多两个字符的裸小写名称是一个**类型变量**：`@spec map(list(a), fun()) :: list(b)`。每个变量编译为一个模块级的 `typing.TypeVar` 声明（`_T_a = typing.TypeVar("_T_a")`），确定性输出，且重复使用会统一。更长的裸名称是一个编译错误，建议使用 `name()` 拼写——这是为了防止忘记括号的拼写错误悄悄变成变量。Elixir 的 `when a: var` 子句是接受的未来语法，目前尚未要求。

**GEP-0017-R004：** 规范加入文档通道：`gan doc` 在散文上方打印它们，`gandora_core.doc` 将它们作为 `specs` 返回，并且 LSP 在悬停和签名帮助中显示它们（GEP-0015）。

## 理由

编译为提示而非发明一个检查器，意味着每个消费者——CI 中的 pyright、针对生成代码的 IDE、导入 Gandora wheel 的 Python 调用者——从第一天起就获得价值，并且未来的 Gandora 端检查器仍可基于相同的声明进行分层。规范中的 `$mod.Type` 使得互操作边界成为语言中类型化程度最高的部分，而非最低。`atom()` 映射到 `str` 如实遵循 GEP-0001-R009 的值映射。

## 向后兼容性

新增：`::`和`|`在之前的表达式中是解析错误。

## 安全性与确定性

提示是生成源中的文本；不执行任何操作。

## 工具与AI使用

代理 SHOULD 在公共函数上编写 `@spec`，并 MAY 对 `outDir` 运行 pyright 或 mypy，作为 Gandora 程序的类型检查。

## 被拒绝的备选方案

### 先实现Gandora侧的类型检查器

没有声明的检查器检查不了任何东西；没有检查器的声明仍然是提示、文档和类型化API。声明优先。

### 运行时类型断言

违反了GEP-0001-R002 zero-runtime和Elixir自身的立场：规范由文档和工具验证；代码在调用时不应付出代价。

## Conformance

Tests MUST cover: 每一个 R002 映射，包括联合类型、参数化类型、`$mod.Type` 和 `Mod.t()`；参数数量不匹配及 `::` 错位的错误；多子句和默认参数发射形态；文档和 LSP 呈现；以及在带类型标注的输出上运行一次 pyright，接受一个类型正确的模块。

## 变更历史

- 修订版 3，2026-08-02：抽象容器（`iterable`、`sequence`、`mapping`）以及 `keyword()`/`term()` 成为内置类型；新增类型变量（R005，编译为 `typing.TypeVar`）；R002 根据指导原则重新组织：`$mod.Type` 仅用于宿主特定类型，而 `$mod.sub.Type` 链的解析遵循 GEP-0003-R010 规则，无需引号。

- 修订版 2，2026-08-02：R002 — 参数化宿主类型 `$mod.Type(t, ...)` 映射为下标提示，例如 `$"collections.abc".Sequence(number())` 变为 `collections.abc.Sequence[int | float]`。

- 修订版 1，2026-08-02：初始版本。

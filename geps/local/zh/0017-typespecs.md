---
gep: 17
title: Typespecs和Typed Boundary
description: Elixir风格的@spec注解，编译为Python类型提示——类型是用于工具和互操作边界的声明，而非运行时。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-02
updated: 2026-08-02
revision: 5
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0017-typespecs.md
source-revision: 5
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0017-typespecs.md](../../0017-typespecs.md)。

# GEP-0017: 类型规范与类型化边界

## 摘要

`@spec` 将 Elixir 的类型规范接口（typespec surface）引入 Gandora，并编译为生成的 Python 代码中的 PEP 484 类型提示：

```elixir
@spec mean(list(number()), integer()) :: float()
def mean(xs, precision \\ 2), do: ...
```

变为 `def mean(xs: list[int | float], precision: int = ...) -> float:`。
收益是双重的：生成的模块成为*带类型注解的 Python API*，供下游 Python 调用者使用；并且整个 Python 类型检查生态系统（pyright、mypy）无需修改即可直接应用于编译后的输出。
`$module.Type` 引用是合法的类型，因此互操作边界可以用宿主自身的类型进行注解。
Spec 是声明——它们在运行时不增加任何东西，并通过文档和 LSP 呈现。

## 动机

Elixir 已经承诺要使用类型系统，而 Python 已经拥有一个；Gandora 处于两者之间，如果没有类型系统的故事，会比两者都逊色。最经济实惠且诚实的做法是 Elixir 的：带有工具价值的类型化 *声明*，没有运行时检查——这里通过 Python 注解实现，因为这是每个 Python 工具都已理解的唯一目标。

## 范围

`@spec` 属性、类型表达式语言、其到 Python 类型提示的映射，以及文档/LSP 展示。Gandora 端的类型 *检查*、`@type` 别名和结构体字段类型将是未来修订的内容。

## 术语

- **Spec**：一个 `@spec name(arg_type, ...) :: return_type` 属性。
- **类型表达式**：R002 的项语言。

## 规范

**GEP-0017-R001:** `@spec fun(type, ...) :: type` 紧接在定义（或其 `@doc` 块）之前，将附加到下一个具有该名称的 `def`/`defp`/`defmacro` 组。`::` 和联合运算符 `|` 在表达式中被识别，但仅在 `@spec` 内有效；其他地方将导致编译错误，并引用本 GEP。如果某个规范的参数数量与后续定义的任何子句都不匹配，则是一个编译错误。

**GEP-0017-R002:** 类型表达式及其 Python 映射。

标量：

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

抽象容器——一等公民，无需宿主引用；在参数位置优先使用它们（它们在具体容器为不变时是协变的）：

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
| `$mod.Type(t, ...)` | `mod.Type[t, ...]`——参数化的宿主类型 |
| `Mod.t()` | 为 `Mod` 生成的结构体类（GEP-0004） |

`$mod.Type` 用于宿主**特定**类型（`$np.ndarray`、`$re.Pattern`、`$decimal.Decimal`）；上述标准抽象已内建，因此惯用的规范永远不需要显式模块边界。任何其他表达式都会导致编译错误，并引用本规则。

**GEP-0017-R003:** 编译过程会注解生成的 Python 函数：单个纯参数子句按顺序接收参数提示以及返回提示；任何通过 `*args` 分发（多子句、模式参数或 `\\` 默认值）的组将接收调度器上的返回注解。输出是确定性的，且不会添加提示名称之外的导入。规范 MUST NOT 产生运行时检查、包装器或模块属性。

**GEP-0017-R005:** 类型位置上的裸小写名称（最多两个字符）是一个**类型变量**：`@spec map(list(a), fun()) :: list(b)`。每个变量编译为一个模块级别的 `typing.TypeVar` 声明（`_T_a = typing.TypeVar("_T_a")`），确定性地发出，且重复使用会统一。更长的裸名称是一个编译错误，建议使用 `name()` 拼写——这是为了防止忘记括号的拼写错误无声地变成变量。Elixir 的 `when a: var` 子句被视为未来语法，目前不要求。

**GEP-0017-R004:** 规范加入文档通道：`gan doc` 在散文上方打印它们，`gandora_core.doc` 以 `specs` 形式返回它们，LSP 在悬停和签名帮助中显示它们（GEP-0015）。

**GEP-0017-R00X（一条拼写规则，修订版 4）：** 一个类型是一个调用。`integer()`、`list(t)`、`fun()`、`$mod.Type()` 和 `Mod()` 都需要括号；唯一的裸拼写是**类型变量**（一或两个小写字母）和字面量 `nil`。编译器在所有地方强制执行此规则：裸 `Mod` 或 `$mod.Type` 会报错，并给出带括号的修正；`Mod(...)` 带参数也会报错（结构体类不接受参数）；裸小写单词会得到现有的相近纠正建议。

**模块即类型（修订版 5）：** 一种结构体类型通过调用模块来拼写——`App.Shop()`——与宿主类的 `$mod.Type()` 类似：大写调用 = 类。之前的 `Mod.t()` 拼写已弃用；它会报错，并给出 `Mod()` 的修正。类型语言中不会出现 `t`。

## 理由

将代码编译成类型提示而不是发明一个检查器，意味着每个消费者——CI 中的 pyright、基于生成代码的 IDE、导入 Gandora wheel 的 Python 调用者——都能从一开始就获得价值，而未来 Gandora 侧的检查器仍可基于相同的声明进行分层。规范中的 `$mod.Type` 使互操作边界成为语言中类型化程度 *最* 高的部分，而非最低的部分。`atom()` 映射到 `str` 忠实地遵循了 GEP-0001-R009 的值映射。

## 向后兼容性

新增特性：`::` 和 `|` 以前在表达式中是解析错误。

## Security and Determinism

Hints are text in the generated source; nothing executes.

## Tooling and AI Usage

Agents SHOULD 在公共函数上编写 `@spec`，并且 MAY 在 `outDir` 上运行 pyright 或 mypy 作为 Gandora 程序的类型检查（typed-lint）。

## Rejected Alternatives

### A Gandora-side type checker first

A checker without declarations checks nothing; declarations without a
checker are still hints, docs, and typed APIs. Declarations first.

### Runtime type assertions

Violates GEP-0001-R002 zero-runtime and Elixir's own stance: specs
document and tools verify; code does not pay at call time.

## 符合性

测试 MUST 覆盖：每个R002映射，包括联合（unions）、参数化（parametrics）、`$mod.Type`和`Mod.t()`；参数数量不匹配（arity-mismatch）和`::`位置错误（misplaced-`::`）错误；多子句（multi-clause）和默认参数（default-argument）的发射形状（emission shapes）；文档和LSP展示（doc and LSP surfacing）；以及在带注释的输出上运行pyright，接受一个类型良好的模块。

## 变更历史

- 修订版 5，2026-08-05：模块即类型——`App.Shop()` 取代 `Mod.t()`（修正了配方中的拼写错误）；大写调用 = 类，统一使用 `$mod.Type()`。

- 修订版 4，2026-08-05：一条拼写规则——类型即调用；裸类型 = 仅类型变量和 `nil`；不带括号的 `Mod.t`/`$mod.Type` 以及参数化的 `Mod.t(...)` 在修复后现在为编译错误。

- 修订版 3，2026-08-02：抽象容器（`iterable`、`sequence`、`mapping`）以及 `keyword()`/`term()` 成为内置类型；添加了类型变量（R005，编译为 `typing.TypeVar`）；R002 按照指南重新组织，即 `$mod.Type` 仅用于宿主特定类型，且 `$mod.sub.Type` 链按照 GEP-0003-R010 规则解析，无需引号。

- 修订版 2，2026-08-02：R002——参数化宿主类型 `$mod.Type(t, ...)` 映射到下标提示，例如 `$"collections.abc".Sequence(number())` 变为 `collections.abc.Sequence[int | float]`。

- 修订版 1，2026-08-02：初始版本。

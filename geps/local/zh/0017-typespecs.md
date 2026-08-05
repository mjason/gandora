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
revision: 4
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0017-typespecs.md
source-revision: 4
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0017-typespecs.md](../../0017-typespecs.md)。

# GEP-0017: Typespecs and the Typed Boundary

## 摘要

`@spec` 将 Elixir 的 typespec 表面引入 Gandora，并将其编译为生成 Python 代码中的 PEP 484 类型提示：

```elixir
@spec mean(list(number()), integer()) :: float()
def mean(xs, precision \\ 2), do: ...
```

变为 `def mean(xs: list[int | float], precision: int = ...) -> float:`  。
好处是双重的：生成的模块成为面向下游 Python 调用者的 *类型化 Python API*，并且整个 Python 类型检查生态系统（pyright、mypy）无需修改即可应用于编译后的输出。`$module.Type` 引用是有效的类型，因此互操作边界可以用宿主语言自身的类型进行注解。Spec 是声明——它们在运行时不添加任何东西，并通过文档和 LSP 呈现。

## 动机

Elixir 已承诺引入类型系统，Python 已拥有之一；Gandora 位于两者之间，若没有类型化故事，将比两者都更逊色。最廉价的诚实步骤是借鉴 Elixir 的做法：类型化 *声明* 附带工具价值，无运行时检查——此处以 Python 注解实现，因为这是每个 Python 工具都已理解的唯一目标。

## 范围

`@spec` 属性、类型表达式语言、其到 Python 类型提示的映射，以及文档/LSP 展示。Gandora 侧的类型*检查*、`@type` 别名和结构体字段类型属于未来修订。

## 术语

- **Spec**：一个 `@spec name(arg_type, ...) :: return_type` 属性。
- **Type expression**：R002 的术语语言。

## 规范

**GEP-0017-R001：** `@spec fun(type, ...) :: type` 紧接在定义（或其 `@doc` 块）之前，将附加到具有该名称的后续 `def`/`defp`/`defmacro` 组。`::` 和联合运算符 `|` 在表达式中被识别，但仅在 `@spec` 内部有效；在其他地方，它们会引发编译错误，并命名本 GEP。形参数量与后续定义的任何子句都不匹配的 spec 是编译错误。

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

抽象容器——一等公民，无需宿主引用；在参数位置上优先使用它们（它们是协变的，而具体容器是不变的）：

| Gandora 类型 | Python 提示 |
| --- | --- |
| `iterable()` / `iterable(t)` | `collections.abc.Iterable[t]` |
| `sequence()` / `sequence(t)` | `collections.abc.Sequence[t]` |
| `mapping()` / `mapping(k, v)` | `collections.abc.Mapping[k, v]` |
| `fun()` | `collections.abc.Callable` |

组合与边界：

| Gandora 类型 | Python 提示 |
| --- | --- |
| `a \| b` | `a \| b` |
| `$mod.Type` | `mod.Type`（记录模块导入） |
| `$mod.Type(t, ...)` | `mod.Type[t, ...]`——参数化宿主类型 |
| `Mod.t()` | 为 `Mod` 生成的 struct 类（GEP-0004） |

`$mod.Type` 用于宿主*特定*类型（`$np.ndarray`、`$re.Pattern`、`$decimal.Decimal`）；上述标准抽象已内建，使得惯用的 spec 无需显式模块边界。任何其他表达式都是编译错误，并命名本规则。

**GEP-0017-R003：** 编译会注释生成的 Python 函数：单个纯参数子句按顺序接收参数提示加上返回提示；任何通过 `*args` 进行分发的组（多子句、模式参数或 `\\` 默认值）都会在调度器上接收返回注解。输出是确定性的，并且不会添加超出提示命名的导入。Specs  MUST NOT 产生运行时检查、包装器或模块属性。

**GEP-0017-R005：** 在类型位置上的、至多两个字符的裸小写名称是一个**类型变量**：`@spec map(list(a), fun()) :: list(b)`。每个变量编译为一个模块级别的 `typing.TypeVar` 声明（`_T_a = typing.TypeVar("_T_a")`），确定性输出，并且重复使用会统一。更长的裸名称是编译错误，提示使用 `name()` 拼写——这是对忘记括号的笔误静默变成变量的防范。Elixir 的 `when a: var` 子句被接受为未来语法，目前尚未要求。

**GEP-0017-R004：** Specs 加入文档通道：`gan doc` 将其打印在散文之上，`gandora_core.doc` 将其作为 `specs` 返回，LSP 在悬停和签名帮助中显示它们（GEP-0015）。

**GEP-0017-R00X（一条拼写规则，修订版 4）：** 类型是一个调用。`integer()`、`list(t)`、`fun()`、`$mod.Type()` 和 `Mod.t()` 都需要括号；唯一的裸拼写是**类型变量**（一或两个小写字母）和字面量 **`nil`**。编译器在任何地方都强制执行此规则：裸的 `Mod.t` 或 `$mod.Type` 会报错并给出加括号的修正，`Mod.t(...)` 带参数会报错（struct 类不接受参数），裸小写单词会得到现有的“您是否指的是”修正。

## Rationale

编译成类型提示（hints）而非发明一个检查器，意味着每个使用者——CI 中的 pyright、处理生成代码的 IDE、导入 Gandora wheel 的 Python 调用者——都可以在第一天就获得价值，而未来 Gandora 侧的检查器仍然可以基于同一组声明进行叠加。规范中的 `$mod.Type` 使得互操作边界成为语言中*类型化最强*的部分，而非最弱的部分。`atom()` 映射到 `str` 则忠实地遵循了 GEP-0001-R009 的值映射规则。

## 向后兼容性

新增的：`::` 和 `|` 在之前是表达式中的解析错误。

## 安全性与确定性

提示是生成源代码中的文本；不会执行任何操作。

## 工具与AI使用

智能体 SHOULD 在公共函数上编写 `@spec`，并且 MAY 对 `outDir` 运行 pyright 或 mypy 作为 Gandora 程序的类型检查。

## 被拒绝的备选方案

### 先实现 Gandora 端的类型检查器

没有声明的检查器什么也检查不了；没有检查器的声明仍然是提示、文档和类型化 API。声明优先。

### 运行时类型断言

违反 GEP-0001-R002 零运行时原则以及 Elixir 自身的立场：规范用于文档化，工具用于验证；代码在调用时不应付出代价。

## 符合性

测试 MUST 覆盖：每个 R002 映射，包括联合类型、参数化类型、`$mod.Type` 和 `Mod.t()`；参数数量不匹配与错位 `::` 错误；多子句与默认参数生成形式；文档与 LSP 呈现；以及对带注释输出运行 pyright 并接受一个类型良好的模块。

## 变更历史

- 修订版 4，2026-08-05：一条拼写规则——类型即调用；裸类型仅限类型变量和 `nil`；`Mod.t`/`$mod.Type` 不带括号以及参数化 `Mod.t(...)` 现在成为编译错误。

- 修订版 3，2026-08-02：抽象容器（`iterable`、`sequence`、`mapping`）以及 `keyword()`/`term()` 成为内置类型；添加了类型变量（R005，编译为 `typing.TypeVar`）；R002 重新组织，指导原则是 `$mod.Type` 仅用于宿主特定类型，且 `$mod.sub.Type` 链通过 GEP-0003-R010 规则解析，无需引号。

- 修订版 2，2026-08-02：R002——参数化宿主类型 `$mod.Type(t, ...)` 映射为下标类型提示，例如，`$"collections.abc".Sequence(number())` 变为 `collections.abc.Sequence[int | float]`。

- 修订版 1，2026-08-02：初始版本。

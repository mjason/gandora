---
gep: 17
title: 类型规范与类型化边界
description: Elixir风格的@spec注解，编译为Python类型提示——类型是用于工具和互操作边界的声明，而非运行时。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-02
updated: 2026-08-02
revision: 2
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0017-typespecs.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0017-typespecs.md](../../0017-typespecs.md)。

# GEP-0017: Typespecs 和 Typed Boundary

## 摘要

`@spec` 将 Elixir 的 typespec 表面引入 Gandora，并将其编译为生成 Python 中的 PEP 484 类型提示：

```elixir
@spec mean(list(number()), integer()) :: float()
def mean(xs, precision \\ 2), do: ...
```

变为 `def mean(xs: list[int | float], precision: int = ...) -> float:`。
好处是双重的：生成的模块成为下游 Python 调用者的*类型化 Python API*，并且整个 Python 类型检查生态系统（pyright, mypy）无需修改即可应用于编译后的输出。
`$module.Type` 引用是有效类型，因此互操作边界可以用宿主自身的类型进行注解。Spec 是声明——它们在运行时不添加任何东西，并且通过文档和 LSP 暴露出来。

## 动机

Elixir 已经承诺采用类型系统，而 Python 已经拥有一个；Gandora 处于两者之间，如果没有类型化的故事，它将比两者都更贫乏。最便宜且诚实的步骤是 Elixir 的做法：类型化的 *声明*，具有工具价值，没有运行时检查——在此处通过 Python 注解实现，因为这是每个 Python 工具都已经理解的唯一目标。

## 范围

`@spec` 属性、类型表达式语言、到 Python 类型提示的映射以及文档/LSP 展示。Gandora 侧的类型*检查*、`@type` 别名和结构体字段类型将在未来版本中处理。

## 术语

- **Spec**: 一个 `@spec name(arg_type, ...) :: return_type` 属性。
- **类型表达式**: R002 的术语语言。

## 规范

**GEP-0017-R001：** `@spec fun(type, ...) :: type` 紧跟在定义（或其 `@doc` 代码块）之前时，会附加到下一个同名的 `def`/`defp`/`defmacro` 分组上。`::` 和联合运算符 `|` 在表达式中可识别，但仅在 `@spec` 内部有效；在其他位置使用会引发编译错误，并引用本 GEP。若某个规范的元数与后续定义的所有子句都不匹配，则引发编译错误。

**GEP-0017-R002：** 类型表达式及其 Python 映射：

| Gandora 类型 | Python 提示 |
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
| `$mod.Type` | `mod.Type`（记录模块导入） |
| `$mod.Type(t, ...)` | `mod.Type[t, ...]` —— 参数化的宿主类型 |
| `Mod.t()` | 为 `Mod` 生成的结构体类（GEP-0004） |

任何其他表达式都会引发编译错误，并引用本规则。

**GEP-0017-R003：** 编译过程会为生成的 Python 函数添加注解：只有一个普通参数子句时，会依次添加参数类型提示及返回类型提示；任何通过 `*args` 分发的分组（多子句、模式参数或 `\\` 默认值）都只在分发函数上添加返回注解。输出是确定性的，且不会添加除类型提示所引用的类型之外的任何导入。规约 MUST NOT 产生运行时检查、包装器或模块属性。

**GEP-0017-R004：** 规约会加入文档通道：`gan doc` 会将其打印在说明文字之前，`gandora_core.doc` 会将其作为 `specs` 返回，LSP 会在悬停和签名帮助中显示它们（GEP-0015）。

## 理由

将类型信息编译为提示（hints）而非发明一个检查器，意味着每个使用者——CI 中的 pyright、在生成代码上工作的 IDE、导入 Gandora wheel 的 Python 调用者——都能在第一天就获得价值，而未来 Gandora 侧的检查器仍然可以基于相同的声明进行分层。规范中的 `$mod.Type` 使得互操作边界成为语言中类型化*最强*的部分，而非最弱。`atom()` 映射到 `str` 则诚实地遵循了 GEP-0001-R009 的值映射。

## 向后兼容性

新增：`::` 和 `|` 在表达式中先前是解析错误。

## 安全性与确定性

提示是生成的源代码中的文本；不执行任何操作。

## Tooling and AI Usage

Agents SHOULD write `@spec` on public functions and MAY run pyright or mypy over `outDir` as a typed-lint of Gandora programs.

## 被拒绝的替代方案

### 先实现 Gandora 侧的类型检查器

没有声明的检查器什么都检查不了；没有检查器的声明仍然是提示、文档和类型化的 API。声明优先。

### 运行时类型断言

违反了 GEP-0001-R002 零运行时和 Elixir 自身的立场：规范文档和工具负责验证；代码在调用时不承担开销。

## Conformance

Tests MUST cover: every R002 mapping including unions, parametrics,
`$mod.Type`, and `Mod.t()`; the arity-mismatch and misplaced-`::`
errors; multi-clause and default-argument emission shapes; doc and
LSP surfacing; and a pyright run over annotated output accepting a
well-typed module.

## 变更历史

- 修订版 2, 2026-08-02: R002 — 参数化宿主类型
  `$mod.Type(t, ...)` 映射到下标提示，例如
  `$"collections.abc".Sequence(number())` 变为
  `collections.abc.Sequence[int | float]`。

- 修订版 1, 2026-08-02: 初始版本。

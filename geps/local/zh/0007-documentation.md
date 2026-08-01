---
gep: 7
title: 文档
description: 带有本地化变体的Markdown @doc、隐藏文档、编译为原生Python doctests的doctests，以及gan doc / gan test命令。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Tooling
created: 2026-08-01
updated: 2026-08-01
revision: 3
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0007-documentation.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0007-documentation.md](../../0007-documentation.md)。

# GEP-0007: 文档

## 摘要

`@doc` 和 `@moduledoc` 是 Markdown 格式，与 Elixir 中一样。关键字形式添加了本地化变体（`@doc default: "...", zh_CN: "..."`），遵循 Osiris 模型，其中默认文本是运行时回退，而区域设置是工具元数据。`@doc false` 将函数从文档中隐藏。使用 `gan>` 提示符编写的示例会话会编译为生成的文档字符串内的原生 Python doctests，因此 `gan test`（以及任何标准 Python doctest 运行器）将执行它们。`gan doc` 以请求的区域设置打印模块或函数的文档。

## 动机

Elixir 将文档视为一等、可测试的制品（hexdocs 的 writing-documentation 指南）；Osiris 添加了语言环境映射，以便一个代码库可以用审阅者的语言呈现文档。Gandora 应提供两者，而无需发明运行时机制——其编译到 Python 的特性提供了一个捷径：Python 标准库中已经有一个 doctest 运行器，因此编译后的示例就变成了普通的 Python doctest。

## 范围

`@doc`/`@moduledoc` 值形式、Markdown 语义、区域设置查找、文档测试语法和编译，以及 `gan doc` 和 `gan test` 命令。HTML 渲染、`@spec`/`@typedoc`、`since:` 元数据以及文档覆盖率工具特性被推迟。

## 术语

- **Doc map**：由单个 `@doc`/`@moduledoc` 附加的一组带语言标签的文本。
- **Default text**：`default:` 条目（当给定裸字符串时，即整个值）。
- **Doctest line**：在文档文本中以提示符 `gan> ` 开头的一行，后跟一行预期输出。

## 规范

### 值形式与 Markdown

**GEP-0007-R001:** `@doc` 和 `@moduledoc` 接受一个字符串（默认文本）或 `false`。文档文本是 Markdown；编译器 MUST 原样保存，MUST NOT 重排或重新格式化。

**GEP-0007-R001A:** 本地化变体通过 `@doc_trans`（紧随其翻译的 `@doc` 之后）与 `@moduledoc_trans`（紧随 `@moduledoc` 之后）附加，携带一个或多个 `<locale>: "text"` 对，键以 `_` 拼写 BCP 47 标签中的 `-`（`zh_CN` ≡ `zh-CN`）。该属性 MAY 重复出现以添加更多语言；重复的 locale、`default:` 键、或缺少前置文档属性的 `*_trans` 均为编译错误。

**GEP-0007-R001B:** `@example "..."`（可重复，置于其记录的 `def` 之前，可与 `@doc` 搭配或单独使用）声明一个共享的、语言无关的示例块。示例块附加在生成 docstring 的默认散文之后，其 `gan>` 行会被编译（R006），并由 `gan doc` 在所有语言中展示。翻译只含散文：`@doc_trans` / `@moduledoc_trans` 内出现 `gan>` 行是编译错误，并指引作者使用 `@example`——示例只写一次、只测一次。

**GEP-0007-R002:** 默认文本（在 doctest 编译后，R006）成为生成的 Python 文档字符串。本地化文本是从源代码（或包的发行源代码）读取的工具元数据；它们 MUST NOT 出现在生成的代码中。

**GEP-0007-R003:** `@doc false` 抑制文档字符串并将函数标记为隐藏；`gan doc` MUST 说明这一点，而不是什么都不打印。如果存在其他区域但缺少 `default:` 条目，则是一个编译错误。

### 区域查找与 gan doc

**GEP-0007-R004:** `gan doc <Module>[.<function>] [--locale <tag>]` 使用 RFC 4647 查找打印请求区域的文档文本：精确标签匹配（不区分大小写），然后逐步缩短前缀，最后为默认文本。不带 `--locale` 时打印默认文本。

**GEP-0007-R005:** `gan doc` 从项目源和已安装的包标记（GEP-0006-R006）解析模块，静态读取，并且从不导入 Python。

### 文档测试

**GEP-0007-R006:** 在文档文本中，一个 doctest 行 `gan> <expr>` 包含一个 Gandora 表达式；接下来的非提示行是期望输出。编译器 MUST 将表达式编译为 Python，并将这对内容作为标准 Python doctest 发出到生成的文档字符串中（`>>> <compiled expr>` 后跟期望行，缩进保留）。非 doctest 行原样通过。

**GEP-0007-R007:** 期望输出由 Python 的 doctest 运行器比较，因此它是结果的 `repr`——与 `inspect/1` 打印的内容相同（`{:ok, 1}` 显示为 `('ok', 1)`）。Doctest 表达式 MUST 是单行，并且在 v0 中 MUST NOT 使用宏；编译失败的 doctest 表达式是一个编译错误，携带函数的位置。

**GEP-0007-R008:** `gan test` 将项目编译到构建缓存中，并使用项目解释器（GEP-0001-R021 选择，`-P`，缓存位于 `PYTHONPATH`）通过 Python 的标准 doctest 运行器运行每个生成的非纯宏模块。它报告每个模块的结果，全部通过时退出 0，否则退出 1。由于发出的 doctest 是标准的，`python -m doctest` 和 pytest 的 `--doctest-modules` MUST 在没有 `gan` 的情况下在生成的文件上工作。

## 理由

将 `gan>` 示例编译为 Python doctest 可保持无运行时属性（文档字符串即产物），并复用成熟的测试运行器而非自行构建。预期输出即 repr 的规则忠实于 GEP-0001-R009 的数据映射：文档精确展示 Python 消费者将看到的值。

将区域设置作为工具元数据借鉴了 Osiris 的做法：产物中保留一个运行时回退，通过工具从源码提供更丰富的语言视图，从而保证生成的 Python 保持精简且确定性。

## 向后兼容性

增量式。现有的裸字符串文档是 `default:` 情况。文本中在行首包含 `gan> ` 先前会原样传递；现在它会被编译——这是一个破坏性边界，仅对于意外使用了该提示符的文档。

## 安全性与确定性

文档测试编译即为普通表达式编译；编译时不会执行任何操作。`gan test` 执行用户自己生成的代码，与 `gan run` 完全相同。

## 工具与AI使用

Agent 应使用 Markdown `@doc` 为公开函数编写文档，对非显而易见的行为包含 `gan>` 示例，在编辑后运行 `gan test`，并使用 `gan doc Mod.fun --locale <tag>` 阅读本地化文档，而非猜测。

## 被拒绝的替代方案

### 一个 Gandora 端的 doctest 运行器

重新实现了 Python 内置的功能，在每个环境中都需要一个运行器，并且与 pytest 用户在 CI 中已经运行的测试不同。

### 在文档字符串中嵌入所有语言环境

用大多数消费者无法阅读的文本膨胀每个工件，并使输出依赖翻译编辑；工具改为从源代码读取语言环境。

### 与 iex> 兼容的提示符

重用 `iex>` 会提示 Elixir 语义（检查格式、字符列表），这是 Gandora 故意不提供的；`gan>` 标记了边界。

## 开放问题

本次修订中无。

## 符合性

测试 MUST 覆盖：裸字符串、关键字形式和 `false` 值；区域设置查找（包括前缀回退和默认值）；R003 缺失默认诊断；保持缩进对表达式（管道、互操作调用）进行doctest编译；通过 `gan test` 检测到的失败doctest；以及直接运行 `python -m doctest` 对生成的模块进行测试。

## 变更历史

- 修订版 3，2026-08-01：新增共享 @example 块（R001B）；翻译只含散文，示例不会脱离测试而腐烂。
- 修订版 2，2026-08-01：以独立的 @doc_trans / @moduledoc_trans 属性取代 @doc 的 locale 关键字形式（R001A）。
- 修订版 1，2026-08-01：初始版本。

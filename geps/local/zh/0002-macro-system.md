---
gep: 2
title: 宏系统
description: 编译时 defmacro 元编程，支持 quote/unquote、默认卫生、以及确定性的沙盒化展开阶段。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Macros
created: 2026-08-01
updated: 2026-08-01
revision: 3
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0002-macro-system.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0002-macro-system.md](../../0002-macro-system.md)。

# GEP-0002：宏系统

## 摘要

Gandora 宏是编译期从语法到语法的函数，使用 Elixir 拼写中的 `defmacro`、`quote`、`unquote` 和 `unquote_splicing` 编写。默认情况下展开是卫生的，在编译器内部的确定性沙箱中运行，并在 Python 代码生成之前完成。该模型是阶段分离的、卫生的且数据驱动的，使用 Elixir 的编写表面。

## 动机

宏系统允许标准库和用户代码扩展语言（控制流形式、DSL、消除样板代码），而无需为每个功能扩展编译器。需要卫生和阶段分离，以便扩展组合安全且构建保持可重复。

## 范围

`defmacro` 定义和调用，引用形式，卫生规则，扩展沙盒，以及 `gan expand`。跨包（超出项目本地模块）的宏导出/导入被推迟。

## 术语

- **AST值**：宏接收和返回的Gandora语法的引号表示。
- **展开阶段**：宏体的编译时求值。
- **卫生性**：宏模板引入的名称不会捕获或与调用点的名称冲突的属性。

## Specification

**GEP-0002-R001：** `defmacro name(params) do body end` 定义了一个宏。宏 MUST 在模块顶层定义。宏调用在编译时展开；其结果 MUST 是一个 AST 值，该值会重新进入展开过程，直到没有宏调用剩余，且展开深度有界。

**GEP-0002-R002：** 宏参数以未求值的 AST 值形式传入。`quote do ... end` 构建一个 AST 值；`unquote(expr)` 将一个求值后的 AST 值插入到 quote 中；`unquote_splicing(list)` 将一个 AST 值列表拼接到外层序列中。引号表示 MUST 遵循 Elixir 的形状：一个调用是 `{name, meta, args}`，字面量表示自身。

**GEP-0002-R003：** 宏体 MUST 由一个确定性的编译时解释器对 Gandora 的表达式子集（绑定、算术与比较、布尔形式、`if`/`case`/`cond`、列表、元组和映射构造、模式匹配、对其他编译时可用的函数和宏的调用，以及引用形式）进行求值。宏体 MUST NOT 执行 I/O、导入 Python、读取时钟或环境，或观察其参数之外的任何内容。相同的输入 MUST 产生相同的展开结果。

**GEP-0002-R004：** 展开 MUST 默认是卫生的：在 `quote` 模板中绑定的变量会被重命名为一个新鲜的、展开唯一的名称，因此它无法捕获或被子调用点绑定捕获。通过 `unquote` 插入的变量保持调用点身份。`var!(name)` 在 quote 内部有意转义卫生性，使用调用点的名称。

**GEP-0002-R005：** 展开宏时引发的编译错误 MUST 同时报告宏调用点和宏定义内部的位置，以有序的来源链形式呈现。

**GEP-0002-R006：** 模块使用的宏 MUST 来自同一模块或由 `require` 或 `import` 引用的模块（GEP-0001-R006）。宏解析 MUST NOT 依赖于文件编译顺序；编译器首先解析项目依赖图，宏依赖循环构成编译错误。

**GEP-0002-R007：** `defmodule`、`def`、`defp`、`defmacro`、`alias`、`import`、`require` 和 `pyimport` MUST NOT 在 v0 中通过宏展开生成；宏在主体内产生表达式和语句。此限制 MAY 被未来的 GEP 解除。

**GEP-0002-R008：** `gan expand <file>` MUST 打印经过完整宏展开后的模块，以表面语法形式呈现，且不写入任何工件。

**GEP-0002-R009：** 宏仅存在于编译时。如果一个模块的主体未声明任何运行时代码（没有函数、结构体、模块属性、没有 `import` 再导出），则 MUST 不生成任何 Python 文件，并且构建过程 MUST 移除为其先前生成的文件。运行此类模块 MUST 失败，并显示一条标识该模块的诊断信息。

**GEP-0002-R010（宏工具包，修订版 2）：** 宏体额外拥有：字符串内置函数 `downcase/1`、`replace/3`、`slug/1`（小写，非字母数字字符折叠为 `_`）和 `to_atom/1`——足以将 `test "reads Well!"` 转换为 `def test_reads_well`；**对引用代码的模式匹配**——一个引用的调用按其 GEP-0012 编码解构为 `{name, meta, args}`（例如 `{:"==", _m, [l, r]}`），关键字对解构为 `{key, value}`（do 块）；从宏体调用另一个本地宏，会将其作为编译时函数求值，参数已求值（包括递归，受步骤限制）。

**GEP-0002-R011（编译时反馈）：** 宏体中的 `compile_warn(message)` 会在宏调用点引发一个带跨度（span）的编译器警告，通过与 GEP-0022 检查相同的渠道——`gan check`、`gan build`、编辑器波浪线、沙箱。库宏以与内核相同的声音进行教学：AI 反馈故事的扩展端。

## 理由

在编译器内部解释宏体（而不是将其编译为 Python 并导入）可以保持编译过程不执行用户代码，并确保构建具有确定性。保持了 Elixir 的 `{name, meta, args}` 引用形式，以便文档和直觉直接迁移。

默认卫生性（hygiene）配合显式的 `var!` 转义机制与 Elixir 一致；而默认不卫生的系统会导致组合宏以用户无法调试的方式失败。

## 向后兼容性

宏表面的奠基提案；引用的 AST 形状（R002）是未来工具所依赖的兼容性合约。

## 安全性与确定性

展开沙箱（R003）是安全边界：宏展开不得泄露数据、接触文件系统或导致构建不可重现。展开深度和步骤限制防止了不会终止的编译。

## 工具与 AI 使用

代理 SHOULD 使用 `gan expand` 来验证宏行为，而不是在头脑中推理展开，并 SHOULD 将 R007 的限制视为宏在 v0 中可能生成的范围。

## 被拒绝的备选方案

### 在构建时编译宏为 Python 并导入它们

对于重型宏更快，但编译会执行任意用户代码，并依赖于本地 Python 环境，破坏了确定性和 GEP-0001 的无导入保证。

### 使用手动 gensym 的不卫生宏

实现更简单，但将正确性责任推给每个宏作者；组合失败会在用户调用点暴露。

## 未决问题

v0 阶段无未决问题。

## 一致性

测试套件 MUST 涵盖：卫生重命名、`var!` 捕获、`unquote` 和 `unquote_splicing` 在头部和尾部位置、嵌套宏调用、递归深度失败、R007 限制诊断，以及 `gan expand` 输出的稳定性。

## 变更历史

- 修订版 3，2026-08-04：R010 宏工具包（字符串内建函数、引用代码解构、宏作为函数调用）；R011 compile_warn——来自库宏的反馈。
- 修订版 2，2026-08-01：新增 R009——仅宏模块不生成运行时 Python 文件。
- 修订版 1，2026-08-01：初始版本。

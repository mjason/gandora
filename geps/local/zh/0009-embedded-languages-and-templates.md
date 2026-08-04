---
gep: 9
title: 嵌入式语言和模板
description: ~<lang> 符号家族扩展到了 ~python 之外，使用 EEx 风格的 <%= %> 插值来处理值和代码。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 3
requires: [5]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0009-embedded-languages-and-templates.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0009-embedded-languages-and-templates.md](../../0009-embedded-languages-and-templates.md)。

# GEP-0009: 嵌入式语言和模板

## 摘要

`~python` 是一种范式的一个实例，而非特例：任何 `~<lang>` 标记（`~markdown`、`~sql`、`~html`……）都会嵌入一个以其语言标记的原始主体。每个嵌入的主体都支持 EEx 风格的拼接：`<%= expr %>` 插入一个 Gandora 表达式。对于 `~python`，拼接是代码（编译后的表达式文本，与之前一样，现在可参数化）；对于其他所有语言，该标记会求值为一个包含运行时拼接值的字符串。一个机制涵盖了嵌入语言家族和 EEx 的模板标记。

## 动机

原始嵌入体（GEP-0005）故意抑制 `#{}` 插值，因为 `#` 和 `{` 在大多数目标语言中有特定含义。但真正的嵌入内容需要参数——SQL 查询的表名、基于 Gandora 绑定的 Python 表达式（带有计算片段）、带有值的 Markdown 报告。EEx 的 `<%= %>` 标记在实践中无冲突、熟悉，并且在所有语言中工作方式相同。

## 范围

嵌入式标记家族、`<%= %>` 拼接和语言标签。
EEx 语句/循环标签（`<% %>`）、`.eex` 模板文件、布局和按语言验证均被推迟。

## 术语

- **嵌入式签名**：其名称不是 GEP-0005 内建签名 (`w`、`s`、`r`) 之一的签名；其名称是语言标签。
- **拼接点**：包含一个 Gandora 表达式的 `<%= expr %>` 标记。

## 规范

**GEP-0009-R001:** 任何名称非内置符号的符号均为嵌入式符号。其主体是原始文本（遵循GEP-0005-R002反斜杠规则；无`#{}`插值）。GEP-0005-R009中关于未知符号的诊断被废除。语言标签是供工具（高亮、未来验证）使用的编译时元数据；编译器不解释主体语言。

**GEP-0009-R002:** 在每个嵌入式主体内部，`<%= expr %>` 包含一个 Gandora 表达式，该表达式会使用符号所在作用域中的绑定进行解析和编译。标记内的空白字符被忽略；`<%%=` 转义字面量 `<%=`。拼接（Splices）MUST 为单个表达式；解析失败则会生成一个指出该符号的编译错误。

**GEP-0009-R003:** `~python` 保持其 GEP-0005-R007 语义——主体按原样拼接（spliced）到生成的代码中，作为一个表达式——每个 `<%= expr %>` 被替换为编译后的表达式文本（括号括起）。它仍然是唯一一个主体作为代码进入程序的符号。

**GEP-0009-R004:** 所有其他嵌入式符号求值为字符串：主体文本中每个拼接被替换为其表达式的运行时值，格式化方式与`#{}`插值相同（当存在拼接时编译为 f-string，否则编译为普通字面量）。

**GEP-0009-R006:** `~prompt` 是 R004 用于面向 AI 模型的散文的首选拼写：`~prompt(...)` 用于单行，`~prompt"""..."""` 用于多行块。主体是原始的——引号、花括号、反斜杠和 JSON 无需转义——而 `<%= expr %>` 仍然拼接值。工具（文档卡片、手册）教授此名称；任何其他 R004 符号名称的行为完全相同。

**GEP-0009-R005:** 多行嵌入式主体使用 GEP-0005 的分隔符，包括 `"""`；拼接工作方式相同。`"""` 主体遵循 GEP-0001-R026 的 heredoc 缩进语义——开头的换行符被丢弃，并且每行中与结束分隔符缩进相同的部分被移除——因此模板可以在模块内自然缩进，同时生成左对齐的值。在该词法修剪之外，生成的输出 MUST 在拼接之外逐字节保持主体文本。

## Rationale

为所有语言使用一个标记优于每种语言各自的插值规则，而 `<%= %>` 是 Elixir 中已确立的写法（EEx）。`~python` 使用代码拼接，而其他所有内容使用值拼接，这与各主体的*本质*相匹配：`~python` 主体会执行，其他主体是数据。语言标签不进行验证，可以在保持编译器小巧的同时，为工具链提供一个稳定的钩子。

## Backwards Compatibility

取消未知标签错误（新增：之前被拒绝的程序变为有效）。包含字面文本 `<%=` 的 `~python` 主体 MUST 切换为 `<%%=`；没有其他现有表面更改。

## Security and Determinism

Splices are ordinary compiled expressions; embedded bodies remain
inert text at compile time. `~python` retains exactly the audit
surface it had: author-written code visible in the output.

## 工具与AI使用

智能体应使用 `~sql`/`~markdown`/等来表示嵌入式内容，而非拼接字符串；使用 `<%= %>` 进行参数化；并且永远不要从未受信任的字符串构建 `~python` 代码——拼接将*值*插入其他语言，但将*代码*插入Python。

## Rejected Alternatives

### Enabling #{} in embedded bodies

与目标语言语法冲突（`#` 注释、`{}` 块、Python 字典/集合显示）—— 这正是原始主体的存在理由。

### A full EEx engine now

语句标签、布局和文件模板属于库级别的范围；值/代码拼接满足了语言层面的需求，未来的 GEP 可以在不更改此契约的前提下，在现有基础上添加 `EEx` 类工具。

## 开放性问题

本修订版中无待解决的问题。

## 符合性

测试MUST涵盖：任意语言标签求值得到其确切正文；值拼接（包括含有管道和互操作调用的表达式）编译为f-strings；`~python` 代码拼接；`<%%=` 转义；多行 `"""` 正文；单表达式诊断；以及拼接外的字节保留。

## 变更历史

- 修订版 3，2026-08-04：R006 — `~prompt` 被指定为 AI 提示的散文标记（原始正文，无转义，`<%= %>` 拼接）。

- 修订版 2，2026-08-02：R005 — `"""` 标记体遵循 GEP-0001-R026 的 heredoc 缩进。

- 修订版 1，2026-08-01：初始版本。

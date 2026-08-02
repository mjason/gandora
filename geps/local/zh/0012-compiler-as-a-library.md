---
gep: 12
title: 作为库的编译器
description: gandora-core — 编译器以Rust库和Python扩展模块的形式存在，采用固定的带引号术语编码，因此工具（LSP、REPL、任务运行器）均使用Gandora本身编写。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
  - Interop
created: 2026-08-02
updated: 2026-08-02
revision: 2
requires: [1, 2, 6]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0012-compiler-as-a-library.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0012-compiler-as-a-library.md](../../0012-compiler-as-a-library.md)。

# GEP-0012: 编译器作为库

## 摘要

编译器变为从同一个 Rust 代码库构建的三个产物：`gandora-core` 库 crate（语言规则的单一来源）、`gan` 二进制文件（一个薄 CLI 封装）以及 `gandora-core` Python 扩展轮（PyO3/abi3），任何 Python 进程——因此通过互操作，任何 Gandora 程序——都可以导入。`$gandora_core.parse(src)` 返回引用的项作为原生的 Gandora 数据，采用 Elixir 编码，因此模式匹配可以在进程内对语言自身的语法进行匹配。这是任务运行器、REPL、LSC 和 LSP 用 Gandora 自身编写（GEP-0013+）的基础，就像 `mix` 用 Elixir 编写、`cargo` 用 Rust 编写一样。

## 动机

语言智能必须具有唯一的实现方式。将其仅通过二进制文件访问，会迫使工具链要么进入 Rust（封闭生态系统），要么进入子进程和 JSON 管道（缓慢、模式繁重）。Elixir 的解决方案——将编译器作为库公开（`Code.string_to_quoted/1`）并用该语言构建工具链——非常适合 Gandora：引用的 AST 形状已经是一个公共契约（GEP-0002-R002），并且已经映射到 Python 数据（GEP-0001-R009）。导出它是在履行已有的承诺，而不是发明一种格式。

## Scope

三个工件、Python API 表面、quoted-term 编码和版本规范。任务运行器、插件协议、REPL 和 LSP 属于 GEP-0013+；它们需要的控制流添加项属于 GEP-0014。

## 术语

- **核心**：指 `gandora-core` Rust 库 crate。
- **扩展**：指 `gandora_core` Python 模块（cdylib wheel）。
- **引用术语**：指形状为 GEP-0002-R002 的 Gandora 语法树，按下方 R005 编码。

## 规范

### 产物

**GEP-0012-R001:** 仓库是一个Cargo工作区：`crates/core`（编译器作为库，无第三方依赖）、`crates/gan`（CLI二进制文件，是对core的薄封装）以及`crates/core-py`（PyO3绑定）。所有语言规则——词法分析、解析、展开、解析、代码生成、诊断——仅存在于core中。

**GEP-0012-R002:** PyPI分发`gandora-core`（abi3 cdylib wheel，可导入为`gandora_core`）以及`gandora-lang`和`gandora-std`，所有分发版本均以相同版本号同步发布。

### Python API

**GEP-0012-R003:** 扩展的第1版精确暴露以下接口：

- `version() -> str` — 核心版本。
- `parse(source, path="nofile")` — 源文件的引用项。
- `expand(source, path="nofile", root=None)` — 宏展开后的引用项；若提供`root`，则项目宏和已安装宏将像在构建中一样解析。
- `diagnostics(source, path="nofile", root=None)` — 来自完整流水线（解析、展开、生成）的`{message, line, col, severity}`字典列表，其中`severity`为`"error"`或`"warning"`；空列表表示源文件编译通过。
- `compile_string(source, path="nofile", root=None)` — 单个模块生成的Python源代码。
- `compile_snippet(source, root=None)` — 编译一个语句序列以供交互使用：返回执行该片段并将最终表达式的值留在变量`_`中的Python代码；REPL/`exec`原语。
- `resolve(root, module_name)` — 模块引用如何解析：返回`{kind: "project"|"installed"|"mechanical", python, source}`，遵循GEP-0006-R005A优先级。

错误会引发`gandora_core.CompileError`，携带`message`、`path`、`line`和`col`。对此接口的补充将作为本GEP的修订版本出现。

### 引用项编码

**GEP-0012-R004:** 引用项以Elixir自身的引用约定进行编码，在GEP-0001-R009数据映射中实现：

| 项 | Python编码 |
| --- | --- |
| 整数、浮点数、布尔值、nil | 自身 |
| 原子、普通字符串 | `str`（运行时映射；参见R006） |
| 插值字符串 | `("__interp__", meta, [part, ...])` |
| 变量 | `(name, meta, context)` — 上下文为`None`或整数 |
| 模块别名 | `("__aliases__", meta, [segment, ...])` |
| 模块引用 `$name` | `("__pyref__", meta, [name])` |
| 列表 | `list` |
| 2元组 | Python 2-`tuple` |
| n元组 (n ≠ 2) | `("{}", meta, [item, ...])` |
| 映射 | `("%{}", meta, [(key, value), ...])` |
| 关键字对 | `(key, value)` 2元组 |
| 本地调用 | `(name, meta, [arg, ...])` |
| 远程/方法调用 | `((".", meta, [base, name]), meta, [arg, ...])` |
| 匿名调用 `f.(x)` | `((".", meta, [f]), meta, [arg, ...])` |
| 块 | `("__block__", meta, [stmt, ...])` |

`meta`是一个字典，至少包含`line`和`col`（0表示未知）。3元组保留给节点；2元组始终是数据——与Elixir使用的消歧义方式相同。

**GEP-0012-R005:** 该编码是与GEP-0002-R002同级别的公共兼容性契约；变更需要在此处修订，并遵循R007的版本策略。

**GEP-0012-R006:** 已知限制：由于原子和普通字符串共享一个运行时表示，版本1在引用项中不区分`:abc`和`"abc"`。需要这种区分的工具等待修订；该限制MUST在扩展的文档中说明。

### 版本策略

**GEP-0012-R007:** 基于该扩展构建的工具MUST检查`gandora_core.version()`是否与其构建时的版本匹配，并在不匹配时发出警告。同步发布（三个分发版本使用同一版本）仍然是策略；扩展本身在主要版本内不会破坏R003接口。

## 理由

一个代码库通过三种方式编译，从构造上消除了源级漂移；剩下的只有分发级偏差，而锁步版本加上强制运行时检查可将这种偏差降为警告，而非静默分歧。选择 Elixir 精确的引号惯例（`__aliases__`、`{}`/`%{}` 包裹、三元组即节点），而非自定义模式，意味着每一份 Elixir 元编程直觉——以及我们自身宏系统的每一份现有文档——都原封不动地适用于导出的数据。

`compile_snippet` 将其结果留在 `_` 中，这镜像了所有主流语言的 REPL 惯例，并使该原语免受 I/O 策略的干扰：调用者自行决定如何显示 `_`。

## 向后兼容性

`gan` CLI 接口保持不变；二进制文件变成核心之上的薄层。
没有现有的 GEP 合约发生变化；本 GEP 增加了工件。

## 安全性与确定性

扩展将文本编译为数据和代码（作为文本）；它不执行任何内容（`compile_snippet` 返回代码 —— 运行代码是调用者的明确行为）。GEP-0001-R024 的确定性保证原封不动地沿用，因为它是相同的代码。

## 工具与AI使用

Gandora工具应当 `pyimport gandora_core` 并直接对带引号的术语进行模式匹配。AI代理通过 `gan exec` 片段中的 `:gandora_core` 互操作，或通过 GEP-0013 的 `lsc` 界面获得相同的功能。没有人应该再用正则表达式解析Gandora文本了。

## 被拒绝的备选方案

### 子进程 + JSON AST 导出

这种方式可行，但每次查询都需要支付序列化和进程生成的开销，并且强制在引号编码之外再维护第二种（JSON）模式。扩展方案在进程内提供了相同的单一真相源；CLI 视图则作为 GEP-0013 中的轻量级消费者保留给 shell 使用。

### 在 Gandora 中基于文本重新实现分析

两套解析器必然导致分歧——在本 GEP 系列中已被拒绝。

### 定制的 AST 模式

Elixir 的约定是经过验证、有文档记录且通过继承（GEP-0002-R002）已经属于我们的；另起炉灶会让每个用户付出双重代价。

## 开放问题

此修订版无开放问题。

## 一致性

测试必须覆盖：从纯 Python 和 Gandora 互操作中导入扩展；对已知源进行 parse/expand/diagnostics/compile_string 的往返测试；逐个条目的编码表（包括节点与数据元组的消歧和插值字符串）；`compile_snippet` 保留 `_`；`resolve` 跨三种类型；`CompileError` 字段；以及 `version()` 与 crate 版本的相等性。

## 变更历史

- 修订版 2，2026-08-02：为 `$` 模块引用添加了 `__pyref__` 引用项编码；示例更新至 GEP-0003 修订版 2。

- 修订版 1，2026-08-02：初始版本。

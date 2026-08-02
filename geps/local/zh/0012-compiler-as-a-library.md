---
gep: 12
title: 编译器作为库
description: gandora-core — 作为Rust库和Python扩展模块的编译器，具有固定的引用术语编码，因此工具（LSP、REPL、任务运行器）本身是用Gandora编写的。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
  - Interop
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [1, 2, 6]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0012-compiler-as-a-library.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0012-compiler-as-a-library.md](../../0012-compiler-as-a-library.md)。

# GEP-0012: 编译器作为库

## 摘要

编译器由同一个 Rust 代码库构建为三个构件：`gandora-core` 库 crate（语言规则的唯一来源）、`gan` 二进制文件（一个轻量级 CLI 封装）以及 `gandora-core` Python 扩展 wheel（PyO3/abi3），任何 Python 进程——因此也是任何 Gandora 程序（通过互操作）——都可以导入它。`:gandora_core.parse(src)` 以 Elixir 编码将引用的项作为原生 Gandora 数据返回，因此模式匹配可以在进程内对语言自身的语法进行操作。这构成了任务运行器、REPL、LSC 和 LSP 用 Gandora 自身编写（GEP-0013+）的基础，就像 `mix` 用 Elixir 编写、`cargo` 用 Rust 编写一样。

## 动机

语言智能必须有且仅有一种实现。仅通过二进制可执行文件使其可访问，会迫使工具要么进入 Rust（封闭生态系统），要么采用子进程加 JSON 的管道方式（缓慢、模式繁重）。Elixir 的答案——将编译器作为库暴露（`Code.string_to_quoted/1`）并使用该语言构建工具链——异常适合 Gandora：quoted AST 形状早已是一个公开约定（GEP-0002-R002），并且已经映射到 Python 数据（GEP-0001-R009）。将其导出是在兑现已有的承诺，而非发明一种新格式。

## Scope

三个制品：Python API 表面、引用术语编码以及版本规范。任务运行器、插件协议、REPL 和 LSP 属于 GEP-0013+；它们所需的控制流扩展属于 GEP-0014。

## 术语

- **Core**：`gandora-core` Rust 库 crate。
- **Extension**：`gandora_core` Python 模块（cdylib wheel）。
- **Quoted term**：符合 GEP-0002-R002 形状的 Gandora 语法树，按照下面的 R005 编码。

## 规范

### 制品

**GEP-0012-R001：** 仓库为一个 Cargo 工作区：`crates/core`（编译器作为库，无第三方依赖）、`crates/gan`（CLI 二进制文件，作为 core 的薄封装）和 `crates/core-py`（PyO3 绑定）。所有语言规则——词法分析、解析、展开、解析、代码生成、诊断——均仅位于 core 中。

**GEP-0012-R002：** PyPI 分发 `gandora-core`（abi3 cdylib 轮子，可导入为 `gandora_core`）以及 `gandora-lang` 和 `gandora-std`，三者以相同版本号同步发布。

### Python 接口

**GEP-0012-R003：** 扩展的版本 1 精确暴露以下接口：

- `version() -> str` — 核心版本号。
- `parse(source, path="nofile")` — 源文件的引号项。
- `expand(source, path="nofile", root=None)` — 宏展开后的引号项；若指定 `root`，则项目宏和已安装宏的解析方式与构建时相同。
- `diagnostics(source, path="nofile", root=None)` — 来自完整流水线（解析、展开、生成）的 `{message, line, col, severity}` 字典列表，`severity` 取值为 `"error"` 或 `"warning"`；空列表表示源文件编译通过。
- `compile_string(source, path="nofile", root=None)` — 单个模块生成的 Python 源代码。
- `compile_snippet(source, root=None)` — 编译用于交互式使用的语句序列：返回可执行该代码片段并将最终表达式值存入变量 `_` 的 Python 代码；REPL/`exec` 原语。
- `resolve(root, module_name)` — 模块引用的解析结果：按 GEP-0006-R005A 优先级返回 `{kind: "project"|"installed"|"mechanical", python, source}`。

错误时抛出 `gandora_core.CompileError`，携带 `message`、`path`、`line`、`col`。对此接口的扩展将以本 GEP 的修订版形式发布。

### 引号项编码

**GEP-0012-R004：** 引号项采用 Elixir 自身的引号约定编码，通过 GEP-0001-R009 数据映射实现：

| 项 | Python 编码 |
| --- | --- |
| 整数、浮点数、布尔值、nil | 自身 |
| 原子、普通字符串 | `str`（运行时映射，见 R006） |
| 插值字符串 | `("__interp__", meta, [part, ...])` |
| 变量 | `(name, meta, context)` — context 为 `None` 或整数 |
| 模块别名 | `("__aliases__", meta, [segment, ...])` |
| 列表 | `list` |
| 2 元组 | Python 2-`tuple` |
| n 元组（n ≠ 2） | `("{}", meta, [item, ...])` |
| 映射 | `("%{}", meta, [(key, value), ...])` |
| 关键字对 | `(key, value)` 2 元组 |
| 本地调用 | `(name, meta, [arg, ...])` |
| 远程/方法调用 | `((".", meta, [base, name]), meta, [arg, ...])` |
| 匿名调用 `f.(x)` | `((".", meta, [f]), meta, [arg, ...])` |
| 块 | `("__block__", meta, [stmt, ...])` |

`meta` 是一个字典，至少包含 `line` 和 `col`（0 表示未知）。3 元组保留给节点；2 元组始终是数据——与 Elixir 使用的消歧义方式相同。

**GEP-0012-R005：** 该编码是与 GEP-0002-R002 同等级别的公开兼容性契约；变更需要在此处进行修订，并遵循 R007 的版本规范。

**GEP-0012-R006：** 已记录的限制：由于原子和普通字符串在运行时共享同一表示，版本 1 在引号项中无法区分 `:abc` 和 `"abc"`。需要区分二者的工具等待修订版；该限制 MUST 在扩展的文档中说明。

### 版本规范

**GEP-0012-R007：** 基于扩展构建的工具 MUST 检查 `gandora_core.version()` 是否与其构建时的版本匹配，并在不匹配时发出警告。同步发布（三个发行版使用同一版本号）仍是政策；扩展本身在同一个主版本内不会破坏 R003 接口。

## 理由

一个代码库以三种方式编译，从构造上消除了源码层面的漂移；剩下的只有分发层面的偏差，而锁定版本加上强制性运行时检查将其降为警告，而非静默分歧。选择 Elixir 精确的引号约定（`__aliases__`、`{}`/`%{}` 包裹器、3-tuples-are-nodes）而非自定义模式，意味着每一项 Elixir 元编程直觉——以及关于我们自身宏系统的每一份现有文档——都原封不动地适用于导出的数据。

`compile_snippet` 将其结果留在 `_` 中，这模仿了每种主流语言的 REPL 约定，并使该原语不受 I/O 策略的约束：调用者自行决定如何显示 `_`。

## 向后兼容性

`gan` CLI 界面保持不变；二进制文件变为核心之上的薄层。
现有 GEP 合约不变；本 GEP 新增构件。

## 安全性与确定性

该扩展将文本编译为数据和代码（以文本形式）；它不执行任何内容（`compile_snippet` 返回代码 —— 运行代码是调用方的显式行为）。GEP-0001-R024 的确定性保证原样沿用，因为此处的代码是相同的。

## 工具与AI使用

Gandora 工具化工具 SHOULD `pyimport gandora_core` 并直接模式匹配引用的术语。AI 代理通过 `gan exec` 片段中的 `:gandora_core` 互操作，或通过 GEP-0013 的 `lsc` 界面获得相同的能力。任何人都不 SHOULD 再使用正则表达式解析 Gandora 文本。

## 被拒绝的替代方案

### 子进程 + JSON AST 导出

可行，但每次查询都需要付出序列化和进程生成的代价，并且在引号编码之外还强制引入第二个（JSON）模式。该扩展在进程内提供了相同的单一事实来源；GEP-0013中保留了一个CLI视图，作为shell的轻量级消费者。

### 在Gandora中基于文本重新实现分析

两个解析器，必然导致漂移——在整个GEP系列中已被拒绝。

### 特制的AST模式

Elixir的惯例是经过验证、有文档记录，并且已经通过继承（GEP-0002-R002）为我们所用；另行发明方案会让每个用户承受双倍负担。

## 开放问题

此修订版无。

## 符合性

测试 MUST 涵盖：从纯 Python 和从 Gandora 互操作中导入扩展；对已知来源进行 parse/expand/diagnostics/compile_string 往返测试；逐条编码表（包括节点与数据元组消歧和插值字符串）；`compile_snippet` 保留 `_`；对三种类型的 `resolve`；CompileError 字段；以及 `version()` 与 crate 版本的相等性。

## 变更历史

- 修订版 1，2026-08-02：初始版本。

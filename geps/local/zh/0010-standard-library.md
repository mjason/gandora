---
gep: 10
title: 标准库
description: 标准库作为一个普通的Gandora包——gandora-std——用Gandora编写，独立于编译器进行版本管理，通过包标记解析。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Standard Library
created: 2026-08-02
updated: 2026-08-02
revision: 2
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0010-standard-library.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0010-standard-library.md](../../0010-standard-library.md)。

# GEP-0010: 标准库

## 摘要

标准库——`Enum`、`String`、`Map`、`List`、`Keyword`——是一个普通的 Gandora 包 `gandora-std`，用 Gandora 编写并通过 GEP-0006 通道发布。它不内嵌在编译器中：项目显式地安装它（`uv add gandora-std`），在 `pyproject.toml` 中像任何依赖项一样进行版本管理，并且无需接触编译器即可升级。两个简单、通用的机制使得这些简洁的名称能够正常工作：一个包可以在 Python 包前缀（`pyPackage`）下编译其输出，并且标记模块名称在运行时解析中具有权威性——因此 `Enum.map(xs, f)` 被编译为 `gandora_std.enum.map(xs, f)`，同时不会遮蔽任何 Python 模块。该库是双语的，并通过 GEP-0007 进行文档测试；一旦包安装完成，`gan doc Enum.map --locale zh` 即可工作。

## 动机

Interop 使每个 Python API 都可触及，但 Elixir 开发者与 AI 代理本能地会使用 `xs |> Enum.map(f) |> Enum.filter(p)`，而 Python 的函数优先 API（`map(f, xs)`）在管道中读起来是反向的。数据优先的标准库恢复了这一惯用语法。用 Gandora 编写它证明了该语言能够承载自己的库层；以包形式发布它使编译器免于嵌入内容，赋予该库自己的发布节奏，并使其与任何用户包一样可检查、可文档化、可测试——其函数即为地道 Gandora 的活例。

## 范围

包、`pyPackage` 输出前缀、基于标记的运行时解析以及初始模块集。延迟流、基于协议的 Enumerable、`Kernel` 中对 GEP-0001 内置函数的迁移以及标准库宏被推迟。

## 术语

- **Stdlib 模块**：由 `gandora-std` 包提供的单段式驼峰命名模块（`Enum`、`String`、……）。
- **输出前缀**：通过 `pyPackage` 配置的项目编译模块所放置的 Python 包。

## 规范

**GEP-0010-R001：** 标准库是 Gandora 包 `gandora-std`，以 Gandora 编写，通过 GEP-0006 发布和消费，编译器不做特殊处理。它位于编译器仓库的 `std/` 目录下，与每个编译器版本同步发布到 PyPI，并且 `gan init` 在包发布后 SHOULD 将其添加到新项目的依赖中——但它仍然是一个普通的、显式声明的、可独立升级的依赖。

**GEP-0010-R002：** `gandora.jsonc` 接受一个可选的 `pyPackage` 字符串（修订 GEP-0001-R019）：编译后的输出位于该 Python 包下（当 `"pyPackage": "gandora_std"` 时，`enum.gan` → `gandora_std/enum.py`），同级引用在其内部解析。`gandora-std` 使用该前缀，因此其模块永远不会遮蔽 Python 的模块（如 `enum`、`string`）。

**GEP-0010-R003：** 标记模块名称对于运行时解析具有权威性（GEP-0006-R005A）：对项目中未定义的模块的引用，通过已安装的标记解析到其记录的 Python 路径，然后回退到机械的 GEP-0001-R014 映射。项目模块始终优先于已安装的名称。因此，在 `uv add gandora-std` 之后，`Enum.map(xs, f)` 编译为 `gandora_std.enum.map(xs, f)`。

**GEP-0010-R004：** 标准库函数采用数据优先：主体是第一个参数，因此每个函数可以直接用于 `|>`。语义遵循 Elixir（在 GEP-0001-R009 数据映射允许的情况下），否则遵循 Python；每个差异 MUST 记录在函数的 `@doc` 中。

**GEP-0010-R005：** 集合包含：`Enum`（map, filter, reject, reduce, sum, count, sort, sort_by, reverse, join, at, take, drop, zip, with_index, member?, all?, any?, empty?, uniq, flat_map, each, min, max, find, find_index, frequencies, group_by, max_by, min_by, product, take_while, drop_while, chunk_every, concat, intersperse, slice, dedup），`String`（upcase, downcase, capitalize, split, split_on, trim, replace, contains?, starts_with?, ends_with?, length, slice, pad_leading, pad_trailing, to_integer, to_float, at, reverse, duplicate, trim_leading, trim_trailing, codepoints, match?），`Map`（get, put, delete, keys, values, merge, has_key?, to_list, new, update, fetch!, put_new, take, drop, filter），`List`（first, last, flatten, wrap, duplicate, insert_at, delete_at, to_tuple, starts_with?, replace_at, update_at），`Keyword`（get, put, keys, values, has_key?, delete, merge）。新增内容通过本 GEP 的修订版本累积。

**GEP-0010-R006：** 每个标准库函数 MUST 携带默认的 `@doc`、一个 `zh-CN` 的 `@doc_trans`，以及对于名称不能明显体现其行为的函数，一个 `@example` 文档测试。`gan doc` 解析已安装包所附带的源代码（GEP-0007-R009）；包自身的 `gan test` 运行所有文档测试。

**GEP-0010-R007：** GEP-0001-R006 中 Kernel 风格的内置函数（`length`、`hd`、`div` 等）仍由编译器内联；标准库不包装它们。

### 纪律

标准库会因积累而退化——混合层次（宏、函数、运行时辅助）、部分移植在覆盖率矩阵中跟踪、范围因功能请求而增长。这些规则旨在防止这种失败模式，并且只能通过修订本 GEP 来更改：

**GEP-0010-R008：** 标准库只有一个层次：普通的、急切的、数据优先的函数。没有标准库宏、没有运行时模板、没有惰性变体、没有状态。需要这些中任何一种的能力都是单独的 GEP，而不是标准库的补充。

**GEP-0010-R009：** 每个标准库函数 MUST 是一个薄包装——对 Python 内置函数/方法的几行代码。需要真正算法代码的函数不属于标准库；它属于一个包（GEP-0006）。

**GEP-0010-R009A：** 标准库内部的依赖 MUST 保持无环且分层：`Enum` 是基础；`List` 和 `Keyword` 可以调用 `Enum`；`String` 和 `Map` 独立。对当前模块的限定引用编译为本地调用（无自身导入）。

**GEP-0010-R010：** R005 列表是完整的库。没有针对 Elixir 的部分对等性跟踪：一个函数要么完全存在（有文档、翻译、文档测试、Elixir 命名、主体优先），要么不存在。新增内容以扩展 R005 的 GEP 修订版本形式出现，绝不以未列出的代码形式出现。

## 理由

嵌入编译器的库将每个库的修复与编译器发布耦合在一起，模糊了使两者保持可审查性的边界——包边界即是纪律。作为普通依赖，标准库像其他所有东西一样被 `uv` 锁定、差异比较和升级，并且“无隐藏运行时”属性以最强形式成立：部署中包含的仅有的与 Gandora 相关的东西就是 `pyproject.toml` 所声明的。

两个支持机制刻意设计为通用而非标准库专用：任何包都可以声明一个输出前缀，并且任何包的标记名称已经必须对宏具有权威性——将其扩展到运行时解析对称地完成了 GEP-0006。

## 向后兼容性

新增性：`pyPackage` 是一个新的可选字段；基于标记的运行时解析仅影响以前在运行时失败的引用。单段 stdlib 名称按约定主张，而非编译器保留——名为 `Enum` 的项目模块仅会遮蔽该包（项目模块优先）。

## 安全性与确定性

基于标记的解析只读取静态文件（GEP-0006-R006的规则）。标准库像任何包一样编译；编译时不执行任何操作。

## 工具和AI使用

智能体应优先使用标准库调用而非原始互操作来进行列表/字符串/映射操作（`Enum.map(xs, f)` 优先于 `:builtins.map`），通过 `gan doc Enum.sort_by` 阅读语义，并将标准库源代码视为惯用表达的权威参考。

## Rejected Alternatives

### Embedding the stdlib in the compiler binary

Couples library releases to compiler releases, grows the trusted
artifact with content, and blurs the compiler/library boundary — the
accretion path. Rejected in review of revision 1's draft.

### Writing the stdlib in Rust codegen

Every function would grow the compiler and bypass the language's own
doc/test machinery; self-hosting keeps the core small and the library
honest.

### Top-level output modules (enum.py, string.py)

Would shadow Python's stdlib for sibling imports — the exact failure
the `-P` work fixed; a package namespace avoids the class of bug.

## 开放问题

此版本暂无。

## 符合性

测试MUST涵盖：`pyPackage` 输出的放置和同级解析；基于标记的运行时解析，带有项目模块优先顺序；每个模块针对已安装的 `gandora-std` 的管道使用；在两个语言环境中对stdlib目标执行 `gan doc`；以及包自身的 `gan test` 通过所有doctest。

## 变更历史

- 修订版 2，2026-08-02：R005 扩展 —— Enum +14、String +7、Map +5、List +4、Keyword +2 个函数。
- 修订版 1，2026-08-02：初始版本。

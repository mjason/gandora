---
gep: 10
title: 标准库
description: 标准库作为一个普通的Gandora包——gandora-std——用Gandora编写，版本独立于编译器，通过包标记解析。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Standard Library
created: 2026-08-02
updated: 2026-08-02
revision: 3
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0010-standard-library.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0010-standard-library.md](../../0010-standard-library.md)。

# GEP-0010: 标准库

## 摘要

标准库——`Enum`、`String`、`Map`、`List`、`Keyword`——是一个普通的 Gandora 包 `gandora-std`，使用 Gandora 编写并通过 GEP-0006 通道发布。它并未嵌入编译器：项目显式安装（`uv add gandora-std`），在 `pyproject.toml` 中像任何依赖一样进行版本管理，且升级时无需触碰编译器。两个小巧的通用机制使得这些简洁名称得以工作：一个包可以在 Python 包前缀（`pyPackage`）下编译其输出，并且标记模块名称在运行时解析中具有权威性——因此 `Enum.map(xs, f)` 编译为 `gandora_std.enum.map(xs, f)`，同时不会遮蔽任何 Python 模块。该库是双语的，并通过 GEP-0007 进行文档测试；一旦包安装完毕，`gan doc Enum.map --locale zh` 即可工作。

## Motivation

Interop 使得每个 Python API 都可访问，但 Elixir 开发者和 AI 代理本能地使用 `xs |> Enum.map(f) |> Enum.filter(p)`，而 Python 的函数优先 API（`map(f, xs)`）在管道中读起来是反的。数据优先的标准库恢复了这种惯用风格。用 Gandora 编写它证明了该语言能够承载自己的库层；作为包发布使编译器保持无嵌入式内容，赋予库自己的发布节奏，并使其与任何用户包一样可检查、可文档化和可测试——其函数是地道 Gandora 的活范例。

## 范围

包、`pyPackage` 输出前缀、基于标记的运行时解析以及初始模块集。惰性流、基于协议的 Enumerable、`Kernel` 对 GEP-0001 内置函数的迁移以及标准库宏被推迟。

## 术语

- **Stdlib模块**：由 `gandora-std` 包提供的单段驼峰式模块（`Enum`、`String` 等）。
- **输出前缀**：通过 `pyPackage` 配置的项目编译模块所在的 Python 包。

## 规范

**GEP-0010-R001：** 标准库是 Gandora 包 `gandora-std`，使用 Gandora 编写，通过 GEP-0006 发布和消费，编译器不做任何特例化处理。它位于编译器仓库的 `std/` 目录下，并在每次编译器发布时同步发布到 PyPI。`gan init` SHOULD 在包发布到 PyPI 后将其添加为新项目的依赖项——但它仍然是一个普通的、显式声明的、可独立升级的依赖项。

**GEP-0010-R002：** `gandora.jsonc` 接受一个可选的 `pyPackage` 字符串（修订 GEP-0001-R019）：编译输出将位于该 Python 包下（当 `"pyPackage": "gandora_std"` 时，`enum.gan` → `gandora_std/enum.py`），并且同级引用在其内部解析。`gandora-std` 使用此前缀，因此其模块永远不会遮蔽 Python 的模块（`enum`、`string`）。

**GEP-0010-R003：** 标记模块名称对于运行时解析具有权威性（GEP-0006-R005A）：对项目中未定义的模块的引用，会通过已安装的标记解析到其记录的 Python 路径，然后回退到机械的 GEP-0001-R014 映射。项目模块始终优先于已安装的名称。因此，在 `uv add gandora-std` 之后，`Enum.map(xs, f)` 会被编译为 `gandora_std.enum.map(xs, f)`。

**GEP-0010-R004：** 标准库函数采用数据优先原则：主体是第一个参数，因此每个函数都准备好使用 `|>` 管道操作符。语义在 GEP-0001-R009 数据映射允许的情况下遵循 Elixir，否则遵循 Python；每种差异 MUST 在函数的 `@doc` 中记录。

**GEP-0010-R005：** 集合为：`Enum`（map, filter, reject, reduce, sum, count, sort, sort_by, reverse, join, at, take, drop, zip, with_index, member?, all?, any?, empty?, uniq, flat_map, each, min, max, find, find_index, frequencies, group_by, max_by, min_by, product, take_while, drop_while, chunk_every, concat, intersperse, slice, dedup）、`String`（upcase, downcase, capitalize, split, split_on, trim, replace, contains?, starts_with?, ends_with?, length, slice, pad_leading, pad_trailing, to_integer, to_float, at, reverse, duplicate, trim_leading, trim_trailing, codepoints, match?）、`Map`（get, put, delete, keys, values, merge, has_key?, to_list, new, update, fetch!, put_new, take, drop, filter）、`List`（first, last, flatten, wrap, duplicate, insert_at, delete_at, to_tuple, starts_with?, replace_at, update_at）、`Keyword`（get, put, keys, values, has_key?, delete, merge）。添加内容将通过修订累积到本 GEP 下。

**GEP-0010-R006：** 每个标准库函数 MUST 携带一个默认的 `@doc`、一个 `zh-CN` 的 `@doc_trans`，以及——对于行为不显而易见的函数——一个 `@example` 文档测试。`gan doc` 解析已安装包的源码（GEP-0007-R009）；包自身的 `gan test` 运行所有文档测试。

**GEP-0010-R007：** GEP-0001-R006 中 Kernel 风格的内建函数（`length`、`hd`、`div` 等）仍由编译器内联；标准库不对其进行包装。

### 规范

标准库会因不断累积而退化——混合层级（宏、函数、运行时辅助函数）、在覆盖率矩阵中部分跟踪移植情况、范围随功能请求而增长。这些规则旨在防止这种失败模式，并且只能通过修订本 GEP 来更改：

**GEP-0010-R008：** 标准库只有一个层级：普通的、急切的、数据优先的函数。没有标准库宏、没有运行时模板、没有惰性变体、没有状态。需要任何这些功能的能力属于单独的 GEP，而不是标准库的添加内容。

**GEP-0010-R009：** 每个标准库函数 MUST 是一个薄包装——几行代码，包装在 Python 的内建函数/方法之上。需要真正算法代码的函数不属于标准库；它属于一个包（GEP-0006）。

**GEP-0010-R009A：** 标准库内部依赖 MUST 保持无环且分层：`Enum` 是基础；`List` 和 `Keyword` 可以调用 `Enum`；`String` 和 `Map` 独立存在。对当前模块的限定引用会编译为本地调用（无自导入）。

**GEP-0010-R010：** R005 列表就是完整的库。没有针对 Elixir 的部分对等跟踪：一个函数要么完全存在（有文档、有翻译、有文档测试、使用 Elixir 名称、数据优先），要么不存在。添加内容应作为修订本 GEP 的 R005 扩展，绝不能作为未列出的代码出现。

## 理由

嵌入编译器中的库将每个库的修复与编译器发布耦合在一起，并模糊了使两者都可审查的边界——包边界就是纪律。作为普通依赖项，stdlib 像其他所有东西一样被 `uv` 固定、比较和升级，而 no-hidden-runtime 属性以其最强形式成立：部署中包含的仅与 Gandora 相关的内容就是 `pyproject.toml` 所声明的。

这两个支持机制有意地通用而非特定于 stdlib：任何包都可以声明一个输出前缀，并且任何包的标记名称对于宏来说已经是权威的——将其扩展到运行时解析则对称地完成了 GEP-0006。

## 向后兼容性

增量式：`pyPackage` 是一个新的可选字段；基于标记的运行时解析仅影响之前运行时失败的引用。单字段标准库名称通过惯例声明，而非编译器保留——名为 `Enum` 的项目模块简单地遮蔽了该包（项目模块优先）。

## 安全性与确定性

基于标记的解析仅读取静态文件（GEP-0006-R006 的规定）。标准库的编译方式与任何包相同；编译时不会执行任何操作。

## 工具与AI使用

代理应优先使用标准库调用而非原始互操作来处理列表/字符串/映射工作（优先使用 `Enum.map(xs, f)` 而非 `$builtins.map`），使用 `gan doc Enum.sort_by` 查阅语义，并将标准库源码视为惯用法的规范参考。

## 被拒绝的替代方案

### 将标准库嵌入编译器二进制文件

将库版本与编译器版本耦合，增加受信任产物的大小，并模糊编译器与库的边界——这是一种积累路径。已在修订版1的审查中被拒绝。

### 在Rust codegen中编写标准库

每个函数都会增加编译器体积，并绕过语言自身的文档/测试机制；自托管方式保持核心小巧且库诚实。

### 顶层输出模块（enum.py, string.py）

会遮蔽Python的标准库，导致同级导入问题——这正是`-P`修复所解决的确切故障；包命名空间避免了此类错误。

## 开放问题

本修订版暂无。

## 符合性

测试MUST涵盖：`pyPackage` 输出放置与同级
解析；基于标记的运行时解析（项目模块
优先）；每个模块针对已安装的 `gandora-std`
的管道使用；在两个语言环境中对 stdlib 目标执行 `gan doc`；以及该包自身的 `gan test` 通过所有 doctest。

## 变更历史

- 修订版 3, 2026-08-02: 示例更新为使用 GEP-0003 修订版 2 的 `$` 互操作语法。

- 修订版 2, 2026-08-02: R005 扩展 — Enum +14, String +7, Map +5, List +4, Keyword +2 函数。
- 修订版 1, 2026-08-02: 初始版本。

---
gep: 10
title: 标准库
description: 标准库作为普通的Gandora包 — gandora-std — 使用Gandora编写，版本独立于编译器，通过包标记解析。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Standard Library
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0010-standard-library.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0010-standard-library.md](../../0010-standard-library.md)。

# GEP-0010: 标准库

## 摘要

标准库——`Enum`、`String`、`Map`、`List`、`Keyword`——是一个普通的 Gandora 包，即 `gandora-std`，使用 Gandora 编写并通过 GEP-0006 通道发布。它并未嵌入编译器：项目需显式安装（`uv add gandora-std`），像任何其他依赖一样在 `pyproject.toml` 中指定版本，并且升级时无需改动编译器。两个小型通用机制使得简洁名称得以工作：一个包可以在 Python 包前缀（`pyPackage`）下编译其输出，同时标记模块名称在运行时解析中具有权威性——因此 `Enum.map(xs, f)` 编译为 `gandora_std.enum.map(xs, f)`，且不会遮蔽任何 Python 模块。该库是双语的，并通过 GEP-0007 进行文档测试；在包安装后，`gan doc Enum.map --locale zh` 即可工作。

## 动机

互操作性使每个 Python API 都可触及，但 Elixir 开发者与 AI 代理会本能地使用 `xs |> Enum.map(f) |> Enum.filter(p)`，而 Python 的函数优先 API（`map(f, xs)`）在管道中读起来是反的。数据优先的标准库恢复了这一惯用风格。在 Gandora 中编写该库证明该语言能够承载自己的库层；以包的形式分发使编译器免于嵌入内容，赋予库独立的发布节奏，并使其与任何用户包一样可检查、可文档化、可测试——其函数是惯用 Gandora 的活示例。

## 范围

包、`pyPackage` 输出前缀、基于标记的运行时解析以及初始模块集。延迟处理的内容包括：惰性流、基于协议的 `Enumerable`、`Kernel` 对 GEP-0001 内置函数的迁移以及标准库宏。

## 术语

- **标准库模块（Stdlib module）**：由 `gandora-std` 包提供的单段驼峰式模块（`Enum`、`String` 等）。
- **输出前缀（Output prefix）**：项目编译后的模块所在的 Python 包，由 `pyPackage` 配置。

## 规范

**GEP-0010-R001：** 标准库是 Gandora 包 `gandora-std`，以 Gandora 编写，通过 GEP-0006 发布和消费，编译器不做特殊处理。它与每个编译器版本同步发布到 PyPI，`gan init` SHOULD 在发布后将其添加到新项目的依赖中——但它仍然是一个普通的、显式声明的、可独立升级的依赖。

**GEP-0010-R002：**`gandora.jsonc` 接受一个可选的 `pyPackage` 字符串（修订 GEP-0001-R019）：编译输出位于该 Python 包下（当 `"pyPackage": "gandora_std"` 时，`enum.gan` → `gandora_std/enum.py`），同级引用在其内部解析。`gandora-std` 使用此前缀，使其模块永远不会遮蔽 Python 的模块（`enum`、`string`）。

**GEP-0010-R003：** 标记模块名称对于运行时解析具有权威性（GEP-0006-R005A）：对项目中未定义的模块的引用，通过已安装的标记解析到其记录的 Python 路径，然后回退到机械的 GEP-0001-R014 映射。项目模块始终优先于已安装的名称。因此，`Enum.map(xs, f)` 在 `uv add gandora-std` 后编译为 `gandora_std.enum.map(xs, f)`。

**GEP-0010-R004：** 标准库函数采用数据优先：主体是第一个参数，因此每个函数都准备好使用 `|>`。语义遵循 Elixir（在 GEP-0001-R009 数据映射允许的情况下），否则遵循 Python；每个差异 MUST 记录在函数的 `@doc` 中。

**GEP-0010-R005：** 初始集合：`Enum`（map、filter、reject、reduce、sum、count、sort、sort_by、reverse、join、at、take、drop、zip、with_index、member?、all?、any?、empty?、uniq、flat_map、each、min、max），`String`（upcase、downcase、capitalize、split、split_on、trim、replace、contains?、starts_with?、ends_with?、length、slice、pad_leading、pad_trailing、to_integer、to_float），`Map`（get、put、delete、keys、values、merge、has_key?、to_list、new、update），`List`（first、last、flatten、wrap、duplicate、insert_at、delete_at），`Keyword`（get、put、keys、values、has_key?）。后续添加通过此 GEP 的修订累积。

**GEP-0010-R006：** 每个标准库函数 MUST 携带一个默认的 `@doc`、一个 `zh-CN` 的 `@doc_trans`，以及——对于行为从名称中不明显看出的函数——一个 `@example` 文档测试。`gan doc` 解析已安装包的源代码（GEP-0007-R009）；包自身的 `gan test` 运行每个文档测试。

**GEP-0010-R007：** GEP-0001-R006 中内核风格的内建函数（`length`、`hd`、`div`、...）保持编译器内联；标准库不包装它们。

### 纪律

标准库会因积压而退化——混合层级（宏、函数、运行时助手）、部分移植通过覆盖矩阵跟踪、范围随功能请求增长。这些规则旨在防止这种失败模式，只有通过修订本 GEP 才能更改：

**GEP-0010-R008：** 标准库是一个层级：普通的、急切的、数据优先的函数。没有标准库宏、没有运行时模板、没有惰性变体、没有状态。需要其中任何一项的能力是一个单独的 GEP，而不是标准库的添加。

**GEP-0010-R009：** 每个标准库函数 MUST 是一个薄包装——对 Python 内置/方法的几行包装。需要真正算法代码的函数不属于标准库；它属于一个包（GEP-0006）。

**GEP-0010-R010：** R005 列表是完整的库。没有针对 Elixir 的部分一致性跟踪：一个函数要么完全存在（已文档化、已翻译、已文档测试、以 Elixir 命名、主体优先），要么不存在。添加内容作为扩展 R005 的 GEP 修订出现，而不是作为未列出的代码。

## 理由

嵌入编译器中的库将每个库的修复与编译器发布耦合在一起，模糊了保持两者可审查的边界——包边界就是纪律。作为普通依赖项，标准库由 `uv` 像其他所有东西一样被固定、比较和升级，并且无隐藏运行时属性以其最强形式成立：部署中包含的与 Gandora 相关的唯一事物就是 `pyproject.toml` 所声明的。

这两种支持机制有意保持通用而非标准库专用：任何包都可以声明输出前缀，并且任何包的标记名称已经必须对宏具有权威性——将其扩展到运行时解析对称地完成了 GEP-0006。

## 向后兼容性

增量式：`pyPackage`是一个新的可选字段；基于标记的运行时解析仅影响之前运行时失败的引用。单段标准库名称通过约定被声明，而非编译器保留——名为`Enum`的项目模块简单地遮蔽了该包（项目模块优先）。

## 安全性与确定性

基于标记的解析仅读取静态文件（GEP-0006-R006 的规则）。标准库像任何包一样编译；编译时不执行任何操作。

## 工具与AI使用

代理应优先使用标准库调用，而非直接采用互操作方式处理列表/字符串/映射操作（使用 `Enum.map(xs, f)` 而非 `:builtins.map`），通过 `gan doc Enum.sort_by` 阅读语义，并将标准库源码视为规范惯用语法参考。

## 被拒绝的替代方案

### 将标准库嵌入编译器二进制文件

将库的发布与编译器的发布耦合，增加了受信任工件的内容，并模糊了编译器/库的边界——这是渐进式增长路径。在修订版1的草案审查中被拒绝。

### 在Rust代码生成中编写标准库

每个函数都会增加编译器的体积，并绕过语言自身的文档/测试机制；自举方式保持了核心小巧和库的诚实性。

### 顶级输出模块（enum.py, string.py）

会遮蔽Python标准库的同级导入——这正是`-P`工作所修复的失败类型；包命名空间避免了这类错误。

## 开放问题

本修订版无。

## 符合性

测试MUST涵盖：`pyPackage` 输出放置和同级解析；基于标记的运行时解析，项目模块优先；每个模块针对已安装的 `gandora-std` 的管道使用；在两个区域中对 stdlib 目标执行 `gan doc`；以及包自身的 `gan test` 通过所有 doctest。

## 变更历史

- 修订版 1，2026-08-02：初始版本。

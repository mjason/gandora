---
gep: 10
title: 标准库
description: 标准库作为普通的Gandora包——gandora-std——使用Gandora编写，独立于编译器进行版本管理，通过包标记解析。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Standard Library
created: 2026-08-02
updated: 2026-08-07
revision: 4
requires: [1, 3, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0010-standard-library.md
source-revision: 4
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0010-standard-library.md](../../0010-standard-library.md)。

# GEP-0010: 标准库

## 摘要

标准库 —— `Enum`、`String`、`Map`、`List`、`Keyword` —— 是一个普通的 Gandora 包，名为 `gandora-std`，使用 Gandora 编写，并通过 GEP-0006 通道发布。它并未内嵌于编译器：项目显式安装它（`uv add gandora-std`），像任何依赖一样在 `pyproject.toml` 中管理其版本，并且升级它无需改动编译器。两个小而通用的机制让这些简洁名称得以生效：一个包可以将其编译输出置于某个 Python 包前缀之下（`pyPackage`），并且标记模块名称对运行时解析具有权威性 —— 因此 `Enum.map(xs, f)` 编译为 `gandora_std.enum.map(xs, f)`，同时不遮蔽任何 Python 模块。该库是双语的，并通过 GEP-0007 进行 doctest；一旦安装了该包，`gan doc Enum.map --locale zh` 即可工作。

## 动机

互操作使每个 Python API 都可触达，但 Elixir 开发者和 AI 代理会本能地使用 `xs |> Enum.map(f) |> Enum.filter(p)`，而 Python 函数优先的 API（`map(f, xs)`）在管道中读起来是反的。数据优先的标准库恢复了这种惯用风格。用 Gandora 编写它证明了该语言能够承载自身的库层；以包的形式发布它，使编译器不必包含嵌入式内容，赋予库自己的发布节奏，并使其与任何用户包一样可检查、可文档化、可测试——其函数就是惯用 Gandora 的鲜活示例。

## 范围

范围包括：该包、`pyPackage` 输出前缀、基于标记的运行时解析，以及初始模块集。惰性流、基于协议的 Enumerable、GEP-0001 内建函数的 `Kernel` 迁移，以及标准库宏均被推迟。

## 术语

- **Stdlib 模块**：由 `gandora-std` 包提供的单段 CamelCase 模块（`Enum`、`String` 等）。
- **输出前缀**：由 `pyPackage` 配置的 Python 包，项目编译后的模块放置于该包之下。

## 规格说明

**GEP-0010-R001：** 标准库就是 Gandora 包 `gandora-std`，用 Gandora 编写，通过 GEP-0006 发布和消费，编译器不做任何特殊处理。它位于编译器仓库的 `std/` 目录下，并随每次编译器发布同步发布到 PyPI；一旦发布到那里，`gan init` SHOULD 将其加入新项目的依赖中——但它仍然是一个普通的、显式声明的、可独立升级的依赖。

**GEP-0010-R002：** `gandora.jsonc` 接受一个可选的 `pyPackage` 字符串（修订 GEP-0001-R019）：编译输出落在该 Python 包之下（当 `"pyPackage": "gandora_std"` 时，`enum.gan` → `gandora_std/enum.py`），并且兄弟引用在该包内解析。`gandora-std` 使用此前缀，因此其模块永远不会遮蔽 Python 的模块（`enum`、`string`）。

**GEP-0010-R003：** 标记模块名称对运行时解析具有权威性（GEP-0006-R005A）：对项目内未定义的模块的引用，先通过已安装的标记解析到其记录的 Python 路径，然后回退到机械式的 GEP-0001-R014 映射。项目模块始终优先于已安装名称。因此，在 `uv add gandora-std` 之后，`Enum.map(xs, f)` 编译为 `gandora_std.enum.map(xs, f)`。

**GEP-0010-R004：** 标准库函数是数据优先的：主语是第一个参数，因此每个函数都可直接用于 `|>`。在 GEP-0001-R009 数据映射允许的情况下，语义遵循 Elixir；否则遵循 Python；每个差异都 MUST 记录在该函数的 `@doc` 中。

**GEP-0010-R005：** 函数集合：`Enum`（map、filter、reject、reduce、sum、count、sort、sort_by、reverse、join、at、take、drop、zip、with_index、member?、all?、any?、empty?、uniq、flat_map、each、min、max、find、find_index、frequencies、group_by、max_by、min_by、product、take_while、drop_while、chunk_every、concat、intersperse、slice、dedup）、`String`（upcase、downcase、capitalize、split、split_on、trim、replace、contains?、starts_with?、ends_with?、length、slice、pad_leading、pad_trailing、to_integer、to_float、at、reverse、duplicate、trim_leading、trim_trailing、codepoints、match?）、`Map`（get、put、delete、keys、values、merge、has_key?、to_list、new、update、fetch!、put_new、take、drop、filter）、`List`（first、last、flatten、wrap、duplicate、insert_at、delete_at、to_tuple、starts_with?、replace_at、update_at）、`Keyword`（get、put、keys、values、has_key?、delete、merge）、`Path`（join——作用于列表的 /1 和 /2、dirname、basename、extname、expand、absolute?、wildcard）、`File`（cwd!、read、read!、write!、exists?、dir?、ls!、mkdir_p!、rm_rf!）、`System`（cmd——/2 和 /3、get_env——/1 和 /2、find_executable、argv、halt）。新增内容通过修订版本累积在本 GEP 之下。（`Test` 和 `Task` 是由各自 GEP——0024 和 0029——管理的标准库模块，列在那些 GEP 中，而不是这里。）

**GEP-0010-R006：** 每个标准库函数都 MUST 带有一个默认的 `@doc`、一个 `zh-CN` 的 `@doc_trans`，并且——对于行为从名称不能一目了然的函数——还要带一个 `@example` doctest。`gan doc` 解析已安装包随附的源码（GEP-0007-R009）；包自身的 `gan test` 运行所有 doctest。

**GEP-0010-R007：** GEP-0001-R006 中 Kernel 风格的内建函数（`length`、`hd`、`div`……）仍由编译器内联实现；标准库不包装它们。

### 纪律

标准库会因累积而退化——混合的层级（宏、函数、运行时辅助）、记录在覆盖率矩阵中的部分移植、随每个功能请求而增长的范围。以下规则就是为了防止这种失败模式而存在，并且只能通过修订本 GEP 来更改：

**GEP-0010-R008：** 标准库只有一个层级：普通、急切求值、数据优先的函数。没有标准库宏、没有运行时模板、没有惰性变体、没有状态。需要上述任何一种特性的能力属于单独的 GEP，而不是标准库新增项。

**GEP-0010-R009：** 每个标准库函数都 MUST 是一个薄包装——对 Python 内建函数/方法的几行包装。需要真正算法代码的函数不属于标准库；它属于某个包（GEP-0006）。

**GEP-0010-R009A：** 标准库内部的依赖 MUST 保持无环且分层：`Enum` 是基础；`List`、`Keyword`、`Path`、`File` 和 `System` 可以调用 `Enum`（并且 `System` 可以调用 `Keyword` 来处理其选项）；`String` 和 `Map` 独立存在。对当前模块的限定引用编译为局部调用（不产生自导入）。

**GEP-0010-R010：** R005 中的列表就是完整的库。不存在对照 Elixir 的部分对等追踪：一个函数要么完整存在（有文档、有翻译、有 doctest、采用 Elixir 名称、主语在前），要么不存在。新增内容以扩展 R005 的 GEP 修订版本形式落地，绝不作为未列出的代码出现。

### 面向宿主的模块

`Path`、`File` 和 `System` 包装宿主机的文件系统和进程表面，正如 `Task` 包装其协程表面：薄、以 Elixir 命名、不做模拟。它们之所以存在，是因为互操作使这些调用*可能*，但并不使其*可审查*——`os.path.join(os.path.dirname(x), y)` 是穿着 Gandora 外衣的 Python，而 `x |> Path.dirname() |> Path.join(y)` 才是该语言在同一条宿主调用之上的自身惯用法。

**GEP-0010-R011：** 它们的契约，每个差异都记录在 `@doc` 中（R004）：

- `Path` 函数是建立在 `os.path` 之上的纯字符串运算；`Path.wildcard/1` 是 Python 的递归 `glob`，并经过排序以保证确定性。`Path.absolute?/1` 将 Elixir 的 `Path.type(p) == :absolute` 表述为谓词。
- `File` 的 bang 函数（`read!`、`write!`、`ls!`、`mkdir_p!`、`rm_rf!`、`cwd!`）让宿主异常原样抛出——一个会翻译异常的包装就成了运行时。`File.read/1` 返回判定结果 `{:ok, text}` / `{:error, message}`，其中宿主的消息是字符串，而不是 Elixir 的 posix 原子。`File.ls!/1` 会排序。`write!`、`mkdir_p!`（缺失的父目录会被创建，已存在则没问题）和 `rm_rf!`（目标缺失也没问题）返回 `:ok`。
- `System.cmd(bin, args, opts \\ [])` 捕获文本输出并返回 `{stdout, exit_status}`，与 Elixir 完全一致；选项为关键字 `cd:`、`stderr_to_stdout:`（Elixir 的），另外还有以毫秒计的 `timeout:`（这是 Gandora 的扩展——宿主在超时时抛出异常，调用方像对待其他抛出异常一样处理）。`System.argv/0` 不包含程序名，与 Elixir 一致。`System.halt/1` 退出虚拟机。

这三个模块补全了工具链自身的自用闭环：只要包装后的形态够用，`gan`/`gan-lsc`/`gan-mcp` 的源码 MUST 优先使用它们，而不是直接使用 `$os`/`$pathlib`/`$subprocess` 互操作。

## Rationale

嵌入编译器的库会将每个库修复与编译器版本绑定，并模糊了使两者都可审查的边界——包边界就是纪律。作为普通依赖，标准库与其他一切一样由 `uv` 固定版本、比较差异并升级，而“无隐藏运行时”属性以其最强形式成立：部署中所包含的与 Gandora 相关的唯一内容，就是 `pyproject.toml` 所声明的部分。

这两种支持机制刻意保持通用性，而非仅针对标准库：任何包都可以声明输出前缀，且任何包的标记名称原本就必须对宏具有权威性——将其扩展到运行时解析，恰好对称地补全了 GEP-0006。

## 向后兼容性

新增特性：`pyPackage` 是一个新的可选字段；基于标记的运行时解析仅影响先前在运行时失败的引用。单段标准库名称按约定声明，而非编译器保留——名为 `Enum` 的项目模块只是遮蔽了该包（项目模块优先级更高）。

## 安全性与确定性

基于标记的解析仅读取静态文件（GEP-0006-R006 的规则）。标准库像任何包一样编译；在编译时不执行任何内容。`File` 和 `System` 在*运行时*操作文件系统和生成进程，完全如同它们包装的互操作调用一直以来能够做到的那样——它们增加的是可审查性（`System.cmd` 调用可以被 grep 到，而 `$subprocess.run` 则隐藏在导入之中），而非能力。

## 工具与 AI 使用

智能体应优先使用标准库调用，而非针对列表/字符串/映射工作的原始互操作（用 `Enum.map(xs, f)` 而非 `$builtins.map`），通过 `gan doc Enum.sort_by` 了解语义，并将标准库源代码视为规范惯用写法的参考。

## 被否决的备选方案

### 将标准库嵌入编译器二进制文件

将库的发布与编译器的发布耦合在一起，使受信任的构件因附加内容而膨胀，并模糊了编译器与库之间的界限——这是累积式路径。在修订版1草稿的审查中被否决。

### 在 Rust 代码生成器中编写标准库

每个函数都会使编译器膨胀，并绕过语言自身的文档/测试机制；自举使核心保持精简，并让库保持诚实。

### 顶层输出模块（enum.py, string.py）

对于同级导入，会遮蔽 Python 的标准库——恰恰是 `-P` 工作修复的那个缺陷；包命名空间避免了这类缺陷。

## 未决问题

本修订版无。

## 一致性

测试 MUST 涵盖：
`pyPackage` 输出位置及同级解析；基于标记的运行时解析及项目模块优先级；每个模块对已安装 `gandora-std` 的管道用法；在两种区域设置下对 stdlib 目标执行 `gan doc`；以及包自身的 `gan test` 通过每个 doctest。

## 变更历史

- 修订版 4，2026-08-07：R005 扩展了面向宿主的模块 `Path`（7 个函数）、`File`（9 个）、`System`（5 个），置于新的 R011 契约之下；R009A 分层已针对它们更新。其动机来自工具链审计：`gan`/`gan-lsc`/`gan-mcp` 源码中每个文件花费五个 `pyimport` 来重复编写 `os.path` 的绕行逻辑，而这些在本语言自身惯用法中本是一条管道。

- 修订版 3，2026-08-02：示例已更新为 GEP-0003 修订版 2 的 `$` 互操作语法。

- 修订版 2，2026-08-02：R005 扩展——Enum +14、String +7、Map +5、List +4、Keyword +2 个函数。
- 修订版 1，2026-08-02：初始版本。

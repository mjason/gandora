---
gep: 3
title: Python 互操作性
description: 通过一等$module引用、pyimport、后缀访问和装饰器声明实现对Python的无包装访问。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 5
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0003-python-interop.md
source-revision: 5
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0003-python-interop.md](../../0003-python-interop.md)。

# GEP-0003: Python 互操作性

## 摘要

Gandora 通过一个符记与其宿主交互：`$` 表示某个外部 Python 模块，其后的内容直接访问该模块。`$math.sqrt(2.0)` 会编译为 `import math` 加上 `math.sqrt(2.0)`，无需任何包装器、注册表或运行时垫片；而单独的 `$math` 就是模块对象本身——一个一等值。原子（`:ok`）是纯数据，绝不命名模块。`pyimport` 声明别名导入，后缀 `.name` 访问可用于任意值，`@decorate` 将 Python 装饰器附加到生成的函数上。

## 动机

编译为Python的价值在于其生态系统。因此，互操作必须是该语言中最廉价的构造：任何已安装的模块可立即调用，编译器在编译时除了记录导入外不做任何操作——绝不导入或执行Python本身（GEP-0001-R002，GEP-0001的安全章节）。

## 范围

远程原子调用（Remote atom calls）、`pyimport`、后缀属性与方法访问（postfix attribute and method access）、装饰器声明（decorator declarations）以及调用约定（positional and keyword arguments）。带类型的FFI声明和外部签名的静态验证推迟到未来的GEP中。

## 术语

- **远程引用**：`$module`、`$module.function(args)` 或属性读取 `$module.attribute`。
- **外部模块**：由 `$` 引用或 `pyimport` 命名的 Python 模块。

## 规范

**GEP-0003-R001:** `$name` 是对 Python 模块 `name` 的远程引用 —— 任何标识符，包括大写字母（`$PIL`）。通过它的调用会编译为模块级导入加上直接表达式：`$math.sqrt(x)` 变成 `import math` 和 `math.sqrt(x)`。显式的模块边界使用带括号的形式：`$(os.path).join(a, b)` 编译为 `import os.path` 和 `os.path.join(a, b)` —— 使用 sigil 的分隔符家族，而非字符串。

**GEP-0003-R002:** 没有调用的 `$module.name` 是属性读取（`$sys.argv` 对应 `sys.argv`）。单独的 `$module` 求值为模块对象，是一等值：可以绑定、传递、存储在集合中，并用于 Python 接受模块的任何地方 —— `m = $math; m.sqrt(4.0)`，`rescue e in $builtins.ValueError`。

**GEP-0003-R010:** 多段引用 `$a.b.c...` 无需引号即可解析：前导的小写段序列 —— 在最后一段之前停止 —— 是模块路径，整体导入（`import a.b`），其余段是属性访问。`$collections.abc.Sequence` 导入 `collections.abc`；`$importlib.metadata.version(x)` 导入 `importlib.metadata`；`$os.path.join(a, b)` 导入 `os.path`；`$math.pi` 导入 `math`。带点的导入总是安全的，因此该规则是确定性的，且不依赖于父包是否重新导出其子模块。带括号的形式 `$(a.b)` 是显式覆盖，用于启发式无法识别的情况：最后一个小写段本身是子模块的裸引用（作为值的 `$(importlib.metadata)`）、大写命名的子模块（`$(PIL.Image).open(f)`）、以及继续超过模块的小写属性（`$(os.path).sep`），包括单段情况（`$(sys).stderr.write(s)` 导入 `sys`，绝不导入 `sys.stderr`）。边界是锁定的：`)` 之后的段始终是属性访问，被引用的 AST 将边界记录为 `{:__pyref__, [], [text, true]}`。之前的带引号拼写 `$"a.b"` 是编译错误，错误信息会引用此规则。

**GEP-0003-R009:** 原子（Atoms）是纯数据（GEP-0001-R010），绝不会命名模块。原子后跟 `.` 和标识符是编译错误，其错误消息会引用此规则并显示 `$` 拼写 —— 这是针对修订版 1 源码的迁移诊断。

**GEP-0003-R003:** `pyimport module` 和 `pyimport module, as: alias` 是顶层声明，编译为 `import module` / `import module as alias`。然后别名可用作普通标识符：`np.array([1, 2])`。`pyimport` 必须出现在模块体中的任何定义之前。编译 `pyimport` 不得在编译时导入模块。

**GEP-0003-R004:** 后缀访问适用于任何表达式：`expr.name` 编译为属性访问，`expr.name(args)` 编译为方法调用。链式调用从左到右组合：`df.rolling(5).mean()`。当 `expr` 是以大写字母开头的 Gandora 模块路径时，该引用是 Gandora 跨模块调用（GEP-0001-R017），而非互操作；否则就是普通的 Python 属性访问。

**GEP-0003-R005:** 调用（远程、方法、局部和捕获）必须支持在最后一个位置使用 Elixir 关键字参数语法，编译为 Python 关键字参数：`$json.dumps(data, indent: 2)` 编译为 `json.dumps(data, indent=2)`。关键字键在经过 GEP-0001-R015 映射后必须是有效的 Python 标识符。

**GEP-0003-R006:** 紧接在 `def` 之前的 `@decorate expr` 将表达式作为 Python 装饰器附加到生成的函数上，当重复使用时按源码顺序（最接近的装饰器最内层，匹配 Python 的 `@` 堆叠）。该表达式会被编译，但不在编译时求值。

**GEP-0003-R007:** 值通过 GEP-0001-R009 映射跨越边界，无需转换层；外来值是动态类型的，编译器在 v0 版本中不对外来调用执行静态检查。

**GEP-0003-R008:** 外来依赖是 `pyproject.toml` 中的普通条目，由 `uv` 解析；编译器不得维护自己的外来模块包元数据，且不得在编译时验证外来模块是否已安装。缺失的模块会在运行时因 Python 的标准 `ModuleNotFoundError` 而失败。

## Rationale

修订版1借用了Elixir的`:erlang`约定，但这个双关语仅在Elixir中语义上成立，因为在Elixir中模块本质上就是原子。在这里，`:math`这个值是一个字符串，而`$math.sqrt`是一个模块引用——同一种拼写，两种含义，仅能通过尾随的点来区分，并且模块引用永远不能成为一等公民。`$`分开了角色：`:`始终是数据，`$`始终是宿主环境（shell变量的直觉），两者高亮方式不同，并且`$module`获得了诚实的模块对象语义。对于一次性调用，它无需声明，编译后正是审查者期待的Python代码。`pyimport`保留用于别名化、重复使用的场景（`np`、`pd`），在这些场景中，直接引用会显得杂乱。

不在编译时检查外部模块，保留了编译过程绝不导入Python的保证——这一特性使得构建具有确定性和安全性（编译器仅读取静态元数据）。

## Backwards Compatibility

Revision 2 是一个破坏性语法变更：来自 revision 1 的 `$module.name` 引用不再能编译。R009 诊断指出每个这样的位置都使用了 `$` 拼写，迁移是机械的重写；编译输出的形状保持不变。

## 安全性与确定性

编译器将导入记录为文本；它从不执行或导入外部代码，因此恶意包无法在编译期间运行。程序在运行时所能做的一切，与等效的手写 Python 所能做的完全相同。

## Tooling and AI Usage

Agent 应优先使用远程 `$` 引用来处理一次性标准库调用，而对于重复使用的库则应使用 `pyimport ... as:`，且不得生成围绕 Python API 的包装模块——无包装正是此设计本身。

## 被拒绝的替代方案

### 声明式FFI与类型签名（extern块）

按函数声明的 `extern` 可以提供静态检查，但每个函数都需要一次声明，这与 Python 使用应几乎零仪式感的目标相悖。类型声明仍可作为未来的附加 GEP 开放。

### 编译时导入验证

在编译时导入模块可以更早发现拼写错误，但会破坏确定性、减慢构建速度、执行任意代码，并将编译过程与虚拟环境的状态耦合。

## 开放问题

v0 版本暂无开放问题。

## 一致性

测试 MUST 覆盖：带普通模块和带引号（点式）模块的 `$` 调用、属性读取、一级模块对象引用（绑定和传递）、R009 atom-dot 诊断、带和不带 `as:` 的 `pyimport`、表达式上的后缀链、每种调用方式的关键字参数、装饰器堆叠顺序，以及缺少编译时导入（编译引用不存在模块的文件成功）。

## 变更历史

- Revision 5, 2026-08-03: R010 — `$(...)` 边界锁现在覆盖单段引用（`$(sys)` 之前会静默回退到启发式方法）；有界性在引号编码中得以保留。
- Revision 4, 2026-08-03: 显式模块边界是带括号的 `$(a.b)`（标记-分隔符语义）；引号拼写已弃用，并给出定向错误。
- Revision 3, 2026-08-02: 新增 R010 — 点号分隔的 `$` 链通过小写前缀导入规则解析；引号成为罕见的显式覆盖。
- Revision 2, 2026-08-02: 互操作从原子调用移至 `$` 标记（重写 R001/R002，新增 R009）：`:` 现在为纯数据，`$module` 成为一等模块引用。
- Revision 1, 2026-08-01: 初始版本。

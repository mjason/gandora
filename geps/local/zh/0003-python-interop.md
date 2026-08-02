---
gep: 3
title: Python 互操作
description: 通过一流的$module引用、pyimport、后缀访问和装饰器声明，实现对Python的无包装访问。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 4
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0003-python-interop.md
source-revision: 4
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0003-python-interop.md](../../0003-python-interop.md)。

# GEP-0003: Python 互操作

## 摘要

Gandora 通过一个符号进入宿主语言：`$` 表示一个外部 Python 模块，其后的一切都是直接访问。  
`$math.sqrt(2.0)` 编译为 `import math` 加上 `math.sqrt(2.0)`，没有包装器、注册表或运行时垫片，而单独的 `$math` 就是模块对象本身——一个一等值。  
原子（`:ok`）是纯数据，从不命名模块。  
`pyimport` 声明带别名的导入，后缀 `.name` 访问适用于任何值，`@decorate` 将 Python 装饰器附加到生成的函数上。

## 动机

编译到 Python 的价值在于其生态系统。因此，互操作 MUST 是该语言中最廉价的构造：任何已安装的模块立即可调用，编译器在编译时除了记录导入之外不做任何操作——绝不导入或执行 Python 本身（GEP-0001-R002，GEP-0001 的安全章节）。

## 范围

远程原子调用、`pyimport`、后缀属性与方法访问、装饰器声明以及调用约定（位置参数和关键字参数）。类型化 FFI 声明和外部签名的静态验证推迟到未来的 GEP 中。

## 术语

- **远程引用（Remote reference）**：`$module`、`$module.function(args)` 或属性读取 `$module.attribute`。
- **外部模块（Foreign module）**：由 `$` 引用或 `pyimport` 命名的 Python 模块。

## 规范

**GEP-0003-R001：** `$name` 是对 Python 模块 `name` 的远程引用——任何标识符均可，包括大写字母（`$PIL`）。通过它进行的调用将编译为模块级导入加上直接表达式：`$math.sqrt(x)` 编译为 `import math` 和 `math.sqrt(x)`。显式模块边界使用括号形式：`$(os.path).join(a, b)` 编译为 `import os.path` 和 `os.path.join(a, b)`——此处使用的是符号的定界符族，而非字符串。

**GEP-0003-R002：** 不带调用的 `$module.name` 是属性读取（`$sys.argv` 变为 `sys.argv`）。单独的 `$module` 求值得到模块对象，属于一等公民：它可以被绑定、传递、存储在集合中，并用于 Python 接受模块的任何位置——`m = $math; m.sqrt(4.0)`、`rescue e in $builtins.ValueError`。

**GEP-0003-R010：** 多段引用 `$a.b.c...` 无需引号即可解析：前导的小写段序列——在最后一段之前停止——构成模块路径，整体导入（`import a.b`），后续段为属性访问。`$collections.abc.Sequence` 导入 `collections.abc`；`$importlib.metadata.version(x)` 导入 `importlib.metadata`；`$os.path.join(a, b)` 导入 `os.path`；`$math.pi` 导入 `math`。点分导入始终安全，因此规则是确定性的，且不依赖于父包是否重新导出其子模块。括号形式 `$(a.b)` 是显式覆盖，用于启发式规则无法判断的情况：末尾小写段本身是子模块的裸引用（`$(importlib.metadata)` 作为值）、大写命名的子模块（`$(PIL.Image).open(f)`）、以及跨越模块后继续的小写属性（`$(os.path).sep`）。边界是锁定的：`）` 之后的段始终是属性访问。原引号拼写 `$"a.b"` 是编译错误，错误信息中会命名本规则。

**GEP-0003-R009：** 原子是纯数据（GEP-0001-R010），从不命名模块。原子后跟 `.` 和标识符是编译错误，其错误信息会命名本规则并显示 `$` 拼写——这是针对修订版1源码的迁移诊断。

**GEP-0003-R003：** `pyimport module` 和 `pyimport module, as: alias` 是顶层声明，编译为 `import module` / `import module as alias`。别名随后可作为普通标识符使用：`np.array([1, 2])`。`pyimport` 必须出现在模块体中的任何定义之前。编译 `pyimport` 时不得在编译时导入该模块。

**GEP-0003-R004：** 后缀访问适用于任何表达式：`expr.name` 编译为属性访问，`expr.name(args)` 编译为方法调用。链式调用从左到右组合：`df.rolling(5).mean()`。当 `expr` 是以大写字母开头的 Gandora 模块路径时，该引用是 Gandora 跨模块调用（GEP-0001-R017），而非互操作；否则是普通的 Python 属性访问。

**GEP-0003-R005：** 调用（远程、方法、局部和捕获调用）必须支持在末尾位置使用 Elixir 关键字参数语法，编译为 Python 关键字参数：`$json.dumps(data, indent: 2)` 编译为 `json.dumps(data, indent=2)`。关键字键在经过 GEP-0001-R015 映射后必须是有效的 Python 标识符。

**GEP-0003-R006：** 紧接在 `def` 之前的 `@decorate expr` 将表达式作为 Python 装饰器附加到生成的函数上，重复时按源码顺序（最近的装饰器最内层，与 Python 的 `@` 堆叠一致）。该表达式会被编译，但不在编译时求值。

**GEP-0003-R007：** 值通过 GEP-0001-R009 映射跨边界传递，无转换层；外来值是动态类型的，且编译器在 v0 版本中不对外来调用进行静态检查。

**GEP-0003-R008：** 外来依赖是 `pyproject.toml` 中的普通条目，由 `uv` 解析；编译器不得维护自己的外来模块包元数据，且不得在编译时验证外来模块是否已安装。缺失模块将在运行时引发 Python 的标准 `ModuleNotFoundError`。

## Rationale

修订版1借用了Elixir的`:erlang`约定，但这种双关仅在Elixir中语义成立——在那里模块本质上就是原子。这里`:math`的值是字符串，而`$math.sqrt`是模块引用——同一拼写，两种含义，仅能通过尾随的句点区分，且模块引用永远无法成为一等公民。`$`划分了角色：`:`始终是数据，`$`始终是宿主环境（shell变量的直觉），两者高亮方式不同，且`$module`获得了真正的模块对象语义。它无需声明即可用于一次性调用，并编译成审查者期望的Python代码。`pyimport`仍保留给带别名的重复使用场景（如`np`、`pd`），其中引用会显得冗余。

不在编译时检查外部模块，保留了编译从不导入Python的保证——这一特性保持了构建的确定性和安全性（编译器只读取静态元数据）。

## 向后兼容性

修订版2是一项破坏性语法变更：来自修订版1的 `$module.name` 引用不再能编译。R009诊断将每个此类位置指向 `$` 拼写，迁移是机械重写；编译输出的形状保持不变。

## 安全性与确定性

编译器将导入记录为文本；它从不执行或导入外部代码，因此恶意包无法在编译期间运行。程序在运行时所能做的一切，完全等同于等效的手写 Python 所能做的。

## 工具与AI使用

代理应优先使用远程 `$` 引用进行一次性标准库使用，使用 `pyimport ... as:` 进行重复使用的库，并且不应生成围绕 Python API 的包装模块——没有包装即是设计。

## 被拒绝的替代方案

### 带类型签名的声明式 FFI（extern 块）

按函数声明 `extern` 可提供静态检查，但每个函数都需要一次声明，这与“使用 Python 应几乎无需仪式”的目标相悖。类型化声明仍可作为未来增量 GEP 的开放选项。

### 编译时导入验证

在编译期间导入模块可以更早发现拼写错误，但会破坏确定性、拖慢构建速度、执行任意代码，并使编译与虚拟环境的状态耦合。

## 开放问题

v0 版本无开放问题。

## 符合性

测试 MUST 涵盖：带普通和带引号（点号）模块的 `$` 调用、属性读取、一等模块对象引用（绑定和传递）、R009 atom-dot 诊断、带和不带 `as:` 的 `pyimport`、表达式上的后缀链、每种调用类型的关键字参数、装饰器堆叠顺序，以及编译时导入的缺失（编译引用不存在模块的文件成功）。

## 变更历史

- 修订版 4，2026-08-03：显式模块边界为带括号的 `$(a.b)`（记号-分隔符语义）；引用的拼写已弃用，并附带定向错误。

- 修订版 3，2026-08-02：新增 R010 —— 带点的 `$` 链通过小写前缀导入规则解析；引用成为罕见的显式覆盖。

- 修订版 2，2026-08-02：互操作从原子调用转移到 `$` 记号（R001/R002 重写，R009 新增）：`:` 现在纯粹是数据，而 `$module` 是一等模块引用。

- 修订版 1，2026-08-01：初始版本。

---
gep: 3
title: Python 互操作性
description: 通过一等$module引用、pyimport、后缀访问和装饰器声明，实现对Python的无包装访问。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 6
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0003-python-interop.md
source-revision: 6
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0003-python-interop.md](../../0003-python-interop.md)。

# GEP-0003: Python 互操作性

## 摘要

Gandora 通过一个符号与宿主交互：`$` 命名一个外部的 Python 模块，其后内容皆为直接访问。`$math.sqrt(2.0)` 编译为 `import math` 加上 `math.sqrt(2.0)`，无需包装器、注册表或运行时垫片，而单独的 `$math` 就是模块对象本身——一个一等值。原子（`:ok`）是纯数据，从不命名模块。`pyimport` 声明带别名的导入，后缀 `.name` 访问适用于任何值，而 `@decorate` 将 Python 装饰器附加到生成的函数上。

## 动机

编译到 Python 的价值在于其生态系统。因此，互操作必须是该语言中最廉价的构造：任何已安装的模块都可以立即调用，编译器在编译时除了记录导入外不做任何操作——绝不导入或执行 Python 本身（GEP-0001-R002，GEP-0001 的安全性章节）。

## 范围

远程原子调用、`pyimport`、后缀属性与方法访问、装饰器声明以及调用约定（位置参数和关键字参数）。类型化 FFI 声明与外部签名的静态验证推迟到未来的 GEP 中处理。

## 术语

- **远程引用**：`$module`、`$module.function(args)` 或属性读取 `$module.attribute`。
- **外部模块**：由 `$` 引用或 `pyimport` 命名的 Python 模块。

## 规范

**GEP-0003-R001：** `$name`是对Python模块`name`的远程引用——任何标识符，包括大写（`$PIL`）。通过它进行的调用会编译成模块级导入加上直接表达式：`$math.sqrt(x)`变成`import math`和`math.sqrt(x)`。显式模块边界使用括号形式：`$(os.path).join(a, b)`编译成`import os.path`和`os.path.join(a, b)`——使用的是符号的定界符系列，而非字符串。

**GEP-0003-R002：** 不带调用的`$module.name`是属性读取（`sys.argv`对应`$sys.argv`）。单独的`$module`求值为模块对象，是头等值：它可以被绑定、传递、存储在集合中，并在Python接受模块的任何地方使用——`m = $math; m.sqrt(4.0)`，`rescue e in $builtins.ValueError`。

**GEP-0003-R010：** 多段引用`$a.b.c...`无需引号即可解析：前导的小写段序列——在最后一段之前停止——是模块路径，整体导入（`import a.b`），剩余段是属性访问。`$collections.abc.Sequence`导入`collections.abc`；`$importlib.metadata.version(x)`导入`importlib.metadata`；`$os.path.join(a, b)`导入`os.path`；`$math.pi`导入`math`。带点导入总是安全的，因此该规则是确定性的，且不依赖于父包是否重新导出其子模块。括号形式`$(a.b)`是显式覆盖，用于启发式无法看清的情况：其最后一个小写段本身就是子模块的裸引用（`$(importlib.metadata)`作为值）、大写命名的子模块（`$(PIL.Image).open(f)`）、以及继续越过模块的小写属性（`$(os.path).sep`），包括单段情况（`$(sys).stderr.write(s)`导入`sys`，绝不导入`sys.stderr`）。边界是锁定的：`)`之后的段总是属性访问，引用的AST将此边界记录为`{:__pyref__, [], [text, true]}`。原来的引号拼写`$"a.b"`是编译错误，错误信息提及本规则。

**GEP-0003-R009：** 原子是纯数据（GEP-0001-R010），从不命名模块。原子后跟`.`和标识符是编译错误，其错误信息提及本规则并显示`$`拼写——这是针对revision-1源的迁移诊断。

**GEP-0003-R003：** `pyimport module`和`pyimport module, as: alias`是顶层声明，编译成`import module` / `import module as alias`。别名随后可作为普通标识符使用：`np.array([1, 2])`。`pyimport` MUST 出现在模块体中的任何定义之前。编译`pyimport` MUST NOT 在编译时导入模块。

**GEP-0003-R004：** 后缀访问适用于任何表达式：`expr.name`编译成属性访问，`expr.name(args)`编译成方法调用。链式从左向右组合：`df.rolling(5).mean()`。当`expr`是大写字母开头的Gandora模块路径时，该引用是Gandora跨模块调用（GEP-0001-R017），而非互操作；否则是普通的Python属性访问。

**GEP-0003-R005：** 调用（远程、方法、本地和捕获）MUST 支持Elixir关键字参数语法在最后位置，编译成Python关键字参数：`$json.dumps(data, indent: 2)`编译成`json.dumps(data, indent=2)`。关键字键MUST 在GEP-0001-R015映射后是有效的Python标识符。

**GEP-0003-R006：** `@decorate expr`紧接在`def`之前，将表达式作为Python装饰器附加到生成的函数上，重复时按源顺序（最近装饰器最内层，匹配Python的`@`堆叠）。表达式会被编译，但不会在编译时求值。

**GEP-0003-R007：** 值通过GEP-0001-R009映射跨越边界，无需转换层；外来值是动态类型的，编译器在v0中不对外来调用进行静态检查。

**GEP-0003-R008：** 外来依赖是`pyproject.toml`中的普通条目，由`uv`解析；编译器 MUST NOT 维护自己的外来模块包元数据，MUST NOT 在编译时验证外来模块是否已安装。缺失的模块在运行时以Python标准的`ModuleNotFoundError`失败。

## 理由

修订版1借用了Elixir的`:erlang`约定，但这个双关在语义上仅在Elixir中成立，因为在Elixir中模块实际上就是原子。这里`:math`这个值是一个字符串，而`$math.sqrt`是一个模块引用——相同的拼写，两种含义，仅能通过尾随的点来区分，并且模块引用永远不能成为一等公民。`$`分开了角色：`:`始终是数据，`$`始终是宿主环境（shell变量的直觉），两者高亮方式不同，并且`$module`获得了真正的模块对象语义。对于一次性调用，它不需要声明，并能编译成审查者期望的Python代码。`pyimport`保留用于别名、重复使用的情况（`np`、`pd`），在这些情况下引用会显得嘈杂。

不在编译时检查外部模块，保留了编译从不导入Python的保证——这个属性保持了构建的确定性和安全性（编译器只读取静态元数据）。

## 向后兼容性

修订版 2 是一次破坏性语法变更：来自修订版 1 的 `$module.name` 引用不再编译。R009 诊断会指出每个使用 `$` 拼写的位置，迁移是机械式的重写；编译输出形状保持不变。

## 安全性与确定性

编译器将导入记录为文本；它从不执行或导入外部代码，因此恶意包无法在编译期间运行。程序在运行时所能做的一切，与等效的手写 Python 所能做的完全相同。

## 工具与 AI 使用

代理应优先使用远程 `$` 引用来进行一次性标准库调用，而对于重复使用的模块，则应使用 `pyimport`（裸形式或 `as:` 形式）——裸 `pyimport sys` 将 `sys` 绑定为一个普通名称，其属性链（如 `sys.stderr.write(...)`）不会引入导入歧义，这使得重复使用 `$(...)` 边界拼写成为坏味道。对于经常使用的深层属性链，可以一次性绑定一个属性：`@environ $(os).environ`（GEP-0004）。代理不应围绕 Python API 生成包装模块——没有包装就是设计。

## 被拒绝的替代方案

### 带类型签名的声明式FFI（extern块）

每个函数的 `extern` 声明可以提供静态检查，但需要为每个函数增加一个声明，这违背了Python使用应接近零开销的目标。类型化声明仍可作为未来GEP的补充保留。

### 编译时导入验证

在编译时导入模块可以更早地捕获拼写错误，但会破坏确定性、减慢构建速度、执行任意代码，并将编译与虚拟环境的状态耦合。

## 开放问题

v0 无开放问题。

## 一致性

Tests MUST cover: `$` calls with plain and quoted (dotted) modules,
attribute reads, first-class module-object references (bound and
passed), the R009 atom-dot diagnostic, `pyimport` with and without
`as:`, postfix chains on expressions, keyword arguments on every call kind,
decorator stacking order, and the absence of compile-time imports (compiling
a file referencing a nonexistent module succeeds).

## 变更历史

- 修订版 6，2026-08-03：工具指南——重复模块使用 SHOULD 是裸的或别名的 `pyimport`，而不是重复的 `$(...)` 边界拼写。
- 修订版 5，2026-08-03：R010 —— `$(...)` 边界锁现在覆盖单段引用（`$(sys)` 之前会静默回退到启发式方法）；边界性在引用编码中得以保留。
- 修订版 4，2026-08-03：显式模块边界是括号形式的 `$(a.b)`（符号-分隔符语义）；引用拼写已被弃用，并带有定向错误。

- 修订版 3，2026-08-02：新增 R010 —— 带点的 `$` 链按小写前缀导入规则解析；引用成为罕见的显式覆盖。

- 修订版 2，2026-08-02：互操作从原子调用迁移到 `$` 符号（R001/R002 重写，R009 新增）：`:` 现在纯数据，`$module` 是一等模块引用。

- 修订版 1，2026-08-01：初始版本。

---
gep: 3
title: Python 互操作
description: 通过对一等$module引用、pyimport、后缀访问和装饰器声明实现无包装的Python访问。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 2
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0003-python-interop.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0003-python-interop.md](../../0003-python-interop.md)。

# GEP-0003: Python Interop

## 摘要

Gandora 通过一个标识符触及宿主：`$` 表示一个外部的 Python 模块，其后的所有内容都是直接访问。
`$math.sqrt(2.0)` 编译为 `import math` 加上 `math.sqrt(2.0)`，无需包装器、注册表或运行时填充，而单独的 `$math` 就是模块对象本身——一个一等值。原子（`:ok`）是纯数据，从不命名模块。`pyimport` 声明别名导入，后缀 `.name` 访问可作用于任何值，`@decorate` 将 Python 装饰器附加到生成的函数上。

## 动机

编译到 Python 的价值在于其生态系统。因此，互操作（Interop）必须是语言中最廉价的结构：任何已安装的模块都可以立即调用，编译器在编译时除了记录导入之外什么都不做——从不导入或执行 Python 本身（GEP-0001-R002，GEP-0001 的安全章节）。

## 范围

远程原子调用、`pyimport`、后缀属性与方法访问、装饰器声明，以及调用约定（位置参数与关键字参数）。类型化 FFI 声明和外部签名的静态验证将推迟到未来的 GEP 中处理。

## 术语

- **远程引用**：`$module`、`$module.function(args)` 或属性读取 `$module.attribute`。
- **外部模块**：由 `$` 引用或 `pyimport` 命名的 Python 模块。

## 规范

**GEP-0003-R001：** `$name` 是对 Python 模块 `name` 的远程引用——任何标识符皆可，包括大写形式（如 `$PIL`）。通过该引用的调用会编译为模块级导入加上直接表达式：`$math.sqrt(x)` 编译为 `import math` 及 `math.sqrt(x)`。带点的模块采用引号形式：`$"os.path".join(a, b)` 编译为 `import os.path` 及 `os.path.join(a, b)`。

**GEP-0003-R002：** 未伴随调用的 `$module.name` 是属性读取（如 `$sys.argv` 对应 `sys.argv`）。单独的 `$module` 求值为模块对象，且属于一等值：可被绑定、传递、存入集合，并用于任何 Python 接受模块的地方——`m = $math; m.sqrt(4.0)`、`rescue e in $builtins.ValueError`。

**GEP-0003-R009：** 原子（Atom）是纯数据（GEP-0001-R010），绝不命名模块。原子后跟 `.` 及标识符属于编译错误，其错误消息应引用本规则并展示 `$` 拼写形式——这是针对修订版 1 源代码的迁移诊断信息。

**GEP-0003-R003：** `pyimport module` 和 `pyimport module, as: alias` 是顶层声明，编译为 `import module` / `import module as alias`。之后别名可作为普通标识符使用：`np.array([1, 2])`。`pyimport` MUST 出现在模块体内任何定义之前。编译 `pyimport` 时 MUST NOT 在编译期导入该模块。

**GEP-0003-R004：** 后缀访问可应用于任意表达式：`expr.name` 编译为属性访问，`expr.name(args)` 编译为方法调用。链式调用从左向右组合：`df.rolling(5).mean()`。当 `expr` 是首字母大写的 Gandora 模块路径时，该引用属于 Gandora 跨模块调用（GEP-0001-R017），而非互操作；否则为普通的 Python 属性访问。

**GEP-0003-R005：** 调用（远程、方法、本地及捕获）MUST 支持在最后位置使用 Elixir 关键字参数语法，编译为 Python 关键字参数：`$json.dumps(data, indent: 2)` 编译为 `json.dumps(data, indent=2)`。关键字键 MUST 在经 GEP-0001-R015 映射后是有效的 Python 标识符。

**GEP-0003-R006：** 紧接在 `def` 之前的 `@decorate expr` 会将表达式作为 Python 装饰器附加到生成的函数上；若重复出现，则按源代码顺序（最近的装饰器最内层，与 Python 的 `@` 堆叠行为一致）。该表达式会被编译，但不会在编译期求值。

**GEP-0003-R007：** 值通过 GEP-0001-R009 映射跨越边界，无需转换层；外来值是动态类型的，编译器在 v0 版本中不会对外来调用执行任何静态检查。

**GEP-0003-R008：** 外来依赖是 `pyproject.toml` 中的普通条目，由 `uv` 解析；编译器 MUST NOT 维护自身的外来模块包元数据，且 MUST NOT 在编译期验证外来模块是否已安装。缺失的模块会在运行时以 Python 标准的 `ModuleNotFoundError` 失败。

## 理由

修订版 1 借用了 Elixir 的 `:erlang` 惯例，但这一双关语仅在 Elixir 中语义成立——在那里模块本质上就是原子。此处 `:math` 这个值是一个字符串，而 `$math.sqrt` 是一个模块引用——同一个拼写，两个含义，只能通过尾随的点来区分，并且模块引用永远不能成为一等公民。`$` 将角色分开：`:` 始终是数据，`$` 始终是宿主环境（外壳变量直觉），两者高亮显示方式不同，且 `$module` 获得了诚实的模块对象语义。对于一次性调用无需声明，且会编译成审查者所期望的 Python 代码。`pyimport` 保留给别名化、重复使用的场景（`np`、`pd`），在这些场景中引用会显得啰嗦。

编译时不检查外部模块，保留了编译从不导入 Python 的保证——这一特性使构建具有确定性和安全性（编译器只读取静态元数据）。

## 向后兼容性

修订版2是一项语法破坏性变更：来自修订版1的 `$module.name` 引用不再能编译。R009诊断会将这些引用中的每一处都指向 `$` 拼写，而迁移过程是机械式重写；编译输出形状保持不变。

## 安全性与确定性

编译器将导入记录为文本；它从不执行或导入外部代码，因此恶意包无法在编译期间运行。程序在运行时所能做的一切，恰好等同于等效的手写 Python 所能做到的。

## 工具与人工智能使用

对于一次性标准库的使用，代理应优先使用远程 `$` 引用；对于重复使用的库，则应使用 `pyimport ... as:` 方式。代理不应生成围绕 Python API 的包装模块——不包装正是设计所在。

## 被拒绝的替代方案

### 带有类型签名的声明式 FFI（extern 块）

每个函数的 `extern` 声明可以提供静态检查，但每个函数需要一次声明，这与 Python 使用应几乎零仪式感的目标相悖。类型化声明作为未来可添加的 GEP 保持开放。

### 编译时导入验证

在编译期间导入模块可以更早地捕捉拼写错误，但会破坏确定性、减慢构建、执行任意代码，并将编译与虚拟环境的状态耦合。

## 开放问题

v0 无。

## 符合性

测试 MUST 覆盖：使用普通模块和引用（点号）模块的 `$` 调用、属性读取、一等模块对象引用（绑定和传递）、R009 原子-点诊断、带和不带 `as:` 的 `pyimport`、表达式上的后缀链、每种调用类型的关键字参数、装饰器堆叠顺序，以及编译时导入的缺失（编译引用不存在模块的文件成功）。

## 变更历史

- 修订版 2，2026-08-02：互操作从原子调用移至 `$` 符号（重写了 R001/R002，新增 R009）：`:` 现在纯数据，`$module` 是一等模块引用。

- 修订版 1，2026-08-01：初始版本。

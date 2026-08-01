---
gep: 3
title: Python 互操作
description: 通过远程原子调用、pyimport、后缀访问和装饰器声明实现对Python的无包装访问。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-01
revision: 1
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0003-python-interop.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0003-python-interop.md](../../0003-python-interop.md)。

# GEP-0003: Python 互操作

## Abstract

Gandora 接入 Python 的方式，如同 Elixir 对接 Erlang 那样：一个原子（atom）命名外部模块，对该原子的调用就是直接调用该模块。`:math.sqrt(2.0)` 编译为 `import math` 加上 `math.sqrt(2.0)`，无需包装器、注册表或运行时垫片。`pyimport` 声明别名导入，后缀 `.name` 访问适用于任何值，`@decorate` 将 Python 装饰器附加到生成的函数上。

## Motivation

编译到 Python 的价值在于其生态系统。因此，互操作（Interop）MUST 成为该语言中最廉价的构造：任何已安装的模块立即可调用，编译器在编译时除了记录导入外什么也不做 —— 从不导入或执行 Python 本身（GEP-0001-R002，Security section of GEP-0001）。

## 范围

远程原子调用、`pyimport`、后缀属性与方法访问、装饰器声明以及调用约定（位置参数与关键字参数）。类型化FFI声明和外部签名的静态验证推迟到未来的GEP中。

## 术语

- **远程原子调用**：`:module.function(args)` 或属性读取 `:module.attribute`。
- **外部模块**：由原子或 `pyimport` 命名的 Python 模块。

## 规范

**GEP-0003-R001：** 位于调用位置的原子命名一个外部 Python 模块：`:math.sqrt(x)` 编译为模块级的 `import math` 以及表达式 `math.sqrt(x)`。带点号的模块使用带引号的原子：`:"os.path".join(a, b)` 编译为 `import os.path` 和 `os.path.join(a, b)`。

**GEP-0003-R002：** 不带调用的 `:module.name` 是属性读取（`:sys.argv` 对应 `sys.argv`）。单独的 `:module` 求值为模块对象。

**GEP-0003-R003：** `pyimport module` 和 `pyimport module, as: alias` 是顶层声明，编译为 `import module` / `import module as alias`。此后别名可作为普通标识符使用：`np.array([1, 2])`。`pyimport` MUST 出现在模块体中任何定义之前。编译 `pyimport` 时 MUST NOT 在编译期导入该模块。

**GEP-0003-R004：** 后缀访问适用于任何表达式：`expr.name` 编译为属性访问，`expr.name(args)` 编译为方法调用。链式调用从左到右组合：`df.rolling(5).mean()`。当 `expr` 为首字母大写的 Gandora 模块路径时，该引用是 Gandora 跨模块调用（GEP-0001-R017），而非互操作；否则为普通的 Python 属性访问。

**GEP-0003-R005：** 调用（远程、方法、本地和捕获的）MUST 支持在最后位置使用 Elixir 关键字参数语法，编译为 Python 关键字参数：`:json.dumps(data, indent: 2)` 编译为 `json.dumps(data, indent=2)`。关键字键 MUST 在经过 GEP-0001-R015 映射后是有效的 Python 标识符。

**GEP-0003-R006：** 紧接在 `def` 之前的 `@decorate expr` 将该表达式作为 Python 装饰器附加到生成的函数上，重复时按源代码顺序（最近的装饰器最内层，与 Python 的 `@` 堆叠一致）。该表达式会被编译，但不在编译期求值。

**GEP-0003-R007：** 值通过 GEP-0001-R009 映射跨越边界，不带转换层；外部值是动态类型的，编译器在 v0 中不对外部调用执行静态检查。

**GEP-0003-R008：** 外部依赖是 `pyproject.toml` 中的普通条目，由 `uv` 解析；编译器 MUST NOT 为外部模块维护自己的包元数据，并且 MUST NOT 在编译期验证外部模块是否已安装。缺失的模块会在运行时以 Python 标准的 `ModuleNotFoundError` 失败。

## 理由

`:erlang` 风格的原子调用是最小的互操作表面：对于一次性调用无需声明，读取时无歧义（前导的 `:` 标记了外部边界），并且编译成审阅者预期的精确 Python 代码。`pyimport` 用于需要别名、重复使用的场景（如 `np`、`pd`），在这些场景中使用原子会显得冗余。

不在编译时检查外部模块保证了编译过程永远不会导入 Python——这一特性保证了构建的确定性和安全性（这一教训直接来自 Osiris，其编译器只读取静态元数据）。

## 向后兼容性

基础互操作提案。R001–R005 语法和编译输出形态是兼容性契约。

## 安全性与确定性

编译器将导入的模块记录为文本；它从不执行或导入外部代码，因此恶意包无法在编译期间运行。程序在运行时所能做的一切，与等效的手写Python代码所能做的完全一致。

## 工具与 AI 使用

代理应优先选择一次性标准库使用的远程原子调用，以及重复使用的库的 `pyimport ... as:`，不应生成围绕 Python API 的包装模块——无包装即为设计。

## 已否决的替代方案

### 带类型签名的声明式FFI（extern块）

Osiris 风格的 `extern` 声明提供了静态检查，但每个函数都需要一个声明，这与 Python 使用应几乎无需仪式感的目标相矛盾。类型化声明仍可作为未来的增量 GEP 保留。

### 编译时导入验证

在编译期间导入模块可以更早地捕获拼写错误，但会破坏确定性、减慢构建速度、执行任意代码，并将编译与虚拟环境的状态耦合。

## 开放问题

v0 版本无待定问题。

## Conformance

Tests MUST cover: atom calls with plain and quoted (dotted) modules,
attribute reads, module-object references, `pyimport` with and without
`as:`, postfix chains on expressions, keyword arguments on every call kind,
decorator stacking order, and the absence of compile-time imports (compiling
a file referencing a nonexistent module succeeds).

## 变更历史

- 修订版 1，2026-08-01：初始版本。

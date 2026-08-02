---
gep: 3
title: Python 互操作
description: 通过一等公民的$module引用、pyimport、后缀访问和装饰器声明，实现对Python的无包装访问。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 3
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0003-python-interop.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0003-python-interop.md](../../0003-python-interop.md)。

# GEP-0003: Python 互操作

## 摘要

Gandora 通过一个符号与宿主交互：`$` 表示一个外部 Python 模块，其后的一切都是直接访问。`$math.sqrt(2.0)` 编译为 `import math` 加上 `math.sqrt(2.0)`，无需包装、注册表或运行时的垫片；而单独的 `$math` 就是模块对象本身——一个一等公民。原子（`:ok`）是纯数据，从不命名模块。`pyimport` 声明别名导入，后缀 `.name` 访问适用于任何值，而 `@decorate` 将 Python 装饰器附加到生成的函数上。

## Motivation

编译到 Python 的价值在于其生态系统。因此，互操作必须是语言中最便宜的结构：任何已安装的模块立即可调用，编译器在编译时除了记录导入外不做任何操作——绝不导入或执行 Python 本身（GEP-0001-R002，GEP-0001 的安全章节）。

## 范围

远程原子调用、`pyimport`、后缀属性和方法访问、装饰器声明以及调用约定（位置参数和关键字参数）。类型化FFI声明和外部签名的静态验证推迟到未来的GEP。

## 术语

- **远程引用**：`$module`、`$module.function(args)` 或
  属性读取 `$module.attribute`。
- **外部模块**：由 `$` 引用或 `pyimport` 命名的 Python 模块。

## 规范

**GEP-0003-R001:** `$name` 是对 Python 模块 `name` 的远程引用——任何标识符均可，包括大写形式（如 `$PIL`）。通过它进行的调用会编译为模块级导入加上直接表达式：`$math.sqrt(x)` 编译为 `import math` 和 `math.sqrt(x)`。带点的模块使用引号形式：`$"os.path".join(a, b)` 编译为 `import os.path` 和 `os.path.join(a, b)`。

**GEP-0003-R002:** 不带调用的 `$module.name` 是属性读取（`$sys.argv` 对应 `sys.argv`）。单独的 `$module` 求值为模块对象，是一等值：可以绑定、传递、存储在集合中，并用于任何 Python 接受模块的地方——`m = $math; m.sqrt(4.0)`，`rescue e in $builtins.ValueError`。

**GEP-0003-R010:** 多段引用 `$a.b.c...` 无需引号即可解析：开头的小写段序列——在最后一段之前停止——是模块路径，整体导入（`import a.b`），其余段是属性访问。`$collections.abc.Sequence` 导入 `collections.abc`；`$importlib.metadata.version(x)` 导入 `importlib.metadata`；`$os.path.join(a, b)` 导入 `os.path`；`$math.pi` 导入 `math`。带点的导入总是安全的，因此该规则是确定性的，且与父包是否重新导出其子模块无关。引号形式 `$"a.b"` 保留作为对唯一歧义残留的显式覆盖：一个裸引用，其最后的小写段本身是一个需要显式导入的子模块。

**GEP-0003-R009:** 原子是纯数据（GEP-0001-R010），从不命名模块。原子后跟 `.` 和标识符是编译错误，其错误信息指出本规则并显示 `$` 拼写——这是针对修订版 1 源代码的迁移诊断。

**GEP-0003-R003:** `pyimport module` 和 `pyimport module, as: alias` 是顶层声明，编译为 `import module` / `import module as alias`。别名随后可作为普通标识符使用：`np.array([1, 2])`。`pyimport` MUST 出现在模块体中的任何定义之前。编译 `pyimport` MUST NOT 在编译时导入模块。

**GEP-0003-R004:** 后缀访问适用于任何表达式：`expr.name` 编译为属性访问，`expr.name(args)` 编译为方法调用。链式调用从左到右组合：`df.rolling(5).mean()`。当 `expr` 是以大写字母开头的 Gandora 模块路径时，该引用是 Gandora 跨模块调用（GEP-0001-R017），而不是互操作；否则是普通的 Python 属性访问。

**GEP-0003-R005:** 调用（远程、方法、本地和捕获）MUST 支持在最后位置使用 Elixir 关键字参数语法，编译为 Python 关键字参数：`$json.dumps(data, indent: 2)` 编译为 `json.dumps(data, indent=2)`。关键字键 MUST 在 GEP-0001-R015 映射后是有效的 Python 标识符。

**GEP-0003-R006:** 紧接在 `def` 之前的 `@decorate expr` 将表达式作为 Python 装饰器附加到生成的函数上，重复时按源代码顺序（最近装饰器最内层，与 Python 的 `@` 堆叠匹配）。该表达式会被编译但不会在编译时求值。

**GEP-0003-R007:** 值通过 GEP-0001-R009 映射跨边界，无转换层；外来值是动态类型的，编译器在 v0 中不对外来调用执行静态检查。

**GEP-0003-R008:** 外来依赖是 `pyproject.toml` 中的普通条目，由 `uv` 解析；编译器 MUST NOT 为外来模块维护自己的包元数据，并且 MUST NOT 在编译时验证外来模块是否已安装。缺少模块会在运行时失败，并出现 Python 的标准 `ModuleNotFoundError`。

## 理由

版本1借用了Elixir的`:erlang`约定，但这种双关仅在Elixir中语义上成立，因为Elixir中的模块本质上就是原子。这里`:math`是一个字符串值，而`$math.sqrt`是一个模块引用——同一个拼写，两种含义，仅能通过末尾的点号区分，并且模块引用永远不能成为一等公民。`$`分割了角色：`:`始终是数据，`$`始终是宿主环境（类比shell变量），两者高亮不同，并且`$module`获得了真正的模块对象语义。对于一次性调用，无需声明，并且编译出的Python代码正是审查者所期望的。`pyimport`仍保留用于别名化和重复使用的情况（如`np`、`pd`），此时直接引用会显得杂乱。

不在编译时检查外部模块，确保了编译过程永远不会导入Python——这一特性保证了构建的确定性和安全性（编译器仅读取静态元数据）。

## 向后兼容性

修订版2是一个破坏性语法更改：来自修订版1的`$module.name`引用不再编译。R009诊断将每个这样的位置指向`$`拼写，迁移是机械性的重写；编译输出的形状不变。

## 安全性与确定性

编译器将导入记录为文本；它从不执行或导入外部代码，因此恶意包在编译期间无法运行。程序在运行时所能做的所有事情，恰好是等效的手写Python所能做的。

## 工具与 AI 使用

代理应优先使用远程 `$` 引用进行一次性标准库使用，对重复使用的库使用 `pyimport ... as:`，并且不应生成围绕 Python API 的包装模块——不包装是设计上的要求。

## 被拒绝的备选方案

### 带类型签名的声明式 FFI（外部块）

每个函数的 `extern` 声明能提供静态检查，但每个函数都需要一个声明，这与 Python 使用应几乎无需繁琐声明的目标相矛盾。类型化声明作为可选的未来 GEP 仍保持开放。

### 编译时导入验证

在编译时导入模块可以更早地捕获拼写错误，但会破坏确定性、减慢构建速度、执行任意代码，并将编译与虚拟环境的状态耦合。

## Open Questions

v0 版本暂无公开问题。

## 符合性

测试 MUST 覆盖：使用普通模块和带引号（点式）模块的 `$` 调用、属性读取、一等模块对象引用（绑定和传递）、R009 atom-dot diagnostic、带和不带 `as:` 的 `pyimport`、表达式上的后缀链、每种调用形式的关键字参数、装饰器堆叠顺序，以及编译时导入的缺失（编译引用不存在模块的文件成功）。

## 变更历史

- 修订版 3，2026-08-02：新增 R010——带点号的 `$` 链依据小写前缀导入规则进行解析；引号操作变为罕见的显式覆盖。

- 修订版 2，2026-08-02：互操作从原子调用迁移至 `$` 符号（R001/R002 重写，R009 新增）：`:` 现为纯数据，`$module` 成为一等模块引用。

- 修订版 1，2026-08-01：初始版本。

---
gep: 14
title: 控制流完成
description: 在 Elixir 语法中的 try/rescue/after，以及 loop/recur/break 作为尾递归替代，用于在无 TCO 的虚拟机上运行长时间程序。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0014-control-flow-completion.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0014-control-flow-completion.md](../../0014-control-flow-completion.md)。

# GEP-0014: 控制流完成

## 摘要

两个结构体完成了真实程序所必需的控制流故事——GEP-0013+ 的任务运行器、REPL 和 LSP 服务器。`try/rescue/after` 采用了 Elixir 的语法而非 Python 的异常机制。`loop` 绑定一个状态模式，并在 `recur(new_state)` 上重复其主体，直到 `break(value)`（或主体完成）结束它——这是在没有 TCO 的虚拟机上进行尾递归的诚实替代，沿袭了 Clojure 的 `loop/recur` 传统。

## 动机

GEP-0001 延迟了 `try/rescue`；没有它，服务器无法在遇到错误请求时存活下来。并且 Gandora 没有无界迭代：Elixir 服务器使用递归，但 Python 的递归限制（约 1000 帧）使得那成为一次崩溃，而非一种模式。用 Gandora 编写工具链（GEP-0012 的目的）现在迫使这两个缺口被关闭。

## 范围

这两个构造及其编译。带有异常类型的`raise/2`、`defexception`、`catch`/`throw`以及列表推导式`for`仍然推迟。

## 术语

- **救赎子句**：`rescue` 内部的 `pattern -> body`，其中 pattern 是一个变量或 `var in ExceptionRef`。
- **循环状态**：由 `loop` 绑定并由 `recur` 重新绑定的模式。

## 规范

### try/rescue/after

**GEP-0014-R001:** 该形式为 Elixir 的：

```elixir
try do
  body
rescue
  e in :builtins.ValueError -> handled(e)
  e -> fallback(e)
after
  cleanup()
end
```

`rescue` 包含一个或多个 `->` 子句，按从上到下的顺序尝试；`after` 是可选的，且始终执行。整个形式是一个表达式，求值结果为所采用分支的值（`after` 不贡献值，与 Elixir 中一致）。

**GEP-0014-R002:** 救援模式 `e in Ref` 在抛出的 Python 异常是 `Ref` 的实例时匹配（`Ref` 可以是任何求值结果为异常类型的表达式——典型情况是互操作引用，如 `:builtins.ValueError` 或 `:json.JSONDecodeError`）；裸 `e` 匹配所有 `Exception`。绑定的变量是 Python 异常对象，其字段可通过普通互操作访问（`e.args`、`to_string(e)`）。

**GEP-0014-R003:** 编译目标为 Python 的原生机制：`try`/`except ... as e`/`finally`，每个子句对应一个 `except` 分支，按顺序排列。`raise`（GEP-0001）保持不变。`try` 若不带 `rescue` 或 `after` 则为编译错误，并指明该形式。

### loop/recur/break

**GEP-0014-R004:** 该形式为：

```elixir
loop state = initial do
  body
end
```

`state` 是任意模式，进入时绑定到 `initial`。在循环体内，`recur(expr)` 将模式重新绑定到 `expr` 并重新开始循环体；`break(expr)` 以 `expr` 作为值结束循环。若循环体正常结束（未使用 `recur` 或 `break`），则以循环体的值结束循环。`loop` 是一个表达式。

**GEP-0014-R005:** `recur` 和 `break` 仅在 `loop` 循环体内有效（否则为编译错误，并指明该构造）；它们恰好接受一个参数，且永不返回。嵌套循环会将它们绑定到最近的外层 `loop`。重新绑定使用完整的模式匹配：当 `recur` 的值不匹配状态模式时，会引发 GEP-0001-R012 匹配错误。

**GEP-0014-R006:** 编译为 Python 的 `while True:`，状态保存在编译器命名的变量中，每次迭代开始时进行模式匹配，`recur` 实现为赋值加 `continue`，`break(v)` 实现为结果赋值加 `break`——由此保证恒定的栈深度。

## Rationale

将`rescue`映射到Python异常类型，而不是创造Gandora异常层次结构，保持了互操作性的承诺：程序遇到的错误*就是*Python的错误，通过互操作引用来命名它们无需新的命名空间。选择`loop/recur`而非裸`while`，因为它保持了Elixir的绑定与重新绑定数据规范（状态是一种模式，而非变异）；而非TCO风格的自递归，因为Python无法支持它。Clojure在JVM上正好规范了这种折中方案。

## 向后兼容性

新增的。`try`、`loop`、`recur` 和 `break` 先前被诊断为不受支持或未知的调用。

## 安全性与确定性

两种构造均编译为本地 Python 控制流；不新增运行时表面。确定性（GEP-0001-R024）保持不变。

## Tooling and AI Usage

智能体应当使用 `loop` 进行无限迭代（服务器、REPL），使用递归处理有界结构；在裸变量捕获所有之前`rescue`特定异常类型；并且仅将 `after` 用于清理。

## 被拒绝的替代方案

### 裸 `while cond do` 循环

对 Python 更诚实，但放弃了使 Elixir 状态显式化的模式重新绑定规约；`loop` 多花一行代码，但保持了这种规约。

### 蹦床式自递归

将机制隐藏在每个函数调用内部；在生成的输出中比单个 `while True` 更慢且更难阅读。

### Gandora 异常层次结构

会将每个 Python 错误包装两次；目标语言（Python）的异常才是真正的异常。

## 开放问题

本修订版无开放问题。

## 符合性

测试 MUST 涵盖：rescue 子句顺序和类型匹配（包括互操作类型）、catch-all、`after` 在两个路径上运行、`try` 作为表达式；`loop` 配合元组模式状态、`recur` 重启、`break` 值、主体完成值、嵌套循环绑定、循环外诊断；以及一个百万次迭代的循环，证明恒定栈。

## 变更历史

- 修订版 1，2026-08-02：初始版本。

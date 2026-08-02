---
gep: 14
title: 控制流补全
description: 在Elixir语法中的try/rescue/after，以及loop/recur/break作为在无TCO的虚拟机上长时间运行程序所需的尾递归替代方案。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-02
updated: 2026-08-02
revision: 2
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0014-control-flow-completion.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0014-control-flow-completion.md](../../0014-control-flow-completion.md)。

# GEP-0014: 控制流完成

## 摘要

两种构造完善了真实程序——GEP-0013+ 的任务运行器、REPL 和 LSP 服务器——所需的控制流故事。  
`try/rescue/after` 采用 Elixir 的语法，而非 Python 的异常机制。  
`loop` 绑定一个状态模式，并在 `recur(new_state)` 上重复其主体，直到 `break(value)`（或主体完成）结束它——在无 TCO 的 VM 上，这是尾递归的诚实替代，遵循 Clojure 的 `loop/recur` 传统。

## Motivation

GEP-0001 将 `try/rescue` 推迟；没有它，服务器无法在错误请求中存活。并且 Gandora 没有无界迭代：Elixir 服务器使用递归，但 Python 的递归限制（约1000帧）导致那是崩溃，而非模式。用 Gandora 编写工具链（GEP-0012 的目的）迫使现在同时填补这两个空白。

## 范围

这两个构造及其编译。`raise/2` 与异常类型、`defexception`、`catch`/`throw` 以及 `for` 推导式仍被推迟。

## 术语

- **Rescue 子句**：`rescue` 内部的 `pattern -> body`，其中 pattern 是一个变量或 `var in ExceptionRef`。
- **循环状态**：由 `loop` 绑定并由 `recur` 重新绑定的 pattern。

## 规范

### try/rescue/after

**GEP-0014-R001：** 其形式为 Elixir 的：

```elixir
try do
  body
rescue
  e in $builtins.ValueError -> handled(e)
  e -> fallback(e)
after
  cleanup()
end
```

`rescue` 包含一个或多个从上到下依次尝试的 `->` 子句；`after` 是可选的，并且始终执行。整个形式是一个表达式，其值为所采用分支的值（`after` 不贡献值，与 Elixir 中一样）。

**GEP-0014-R002：** 救援模式 `e in Ref` 在抛出的 Python 异常是 `Ref` 的实例时匹配（`Ref` 是任何求值为异常类型的表达式——通常是互操作引用，如 `$builtins.ValueError` 或 `$json.JSONDecodeError`）；裸 `e` 匹配所有 `Exception`。绑定的变量是 Python 异常对象，其字段可通过普通互操作访问（`e.args`、`to_string(e)`）。

**GEP-0014-R003：** 编译目标为 Python 的原生机制：`try`/`except ... as e`/`finally`，每个子句对应一个 `except` 分支，顺序排列。`raise`（GEP-0001）保持不变。没有 `rescue` 或 `after` 的 `try` 是编译错误，错误信息中会指明该形式。

### loop/recur/break

**GEP-0014-R004：** 其形式为：

```elixir
loop state = initial do
  body
end
```

`state` 是任意模式，在进入时绑定到 `initial`。在体内部，`recur(expr)` 将模式重新绑定到 `expr` 并重启体；`break(expr)` 以 `expr` 作为值结束循环。如果体在没有这两种操作的情况下结束，则以体的值结束循环。`loop` 是一个表达式。

**GEP-0014-R005：** `recur` 和 `break` 仅在 `loop` 体内部有效（否则为编译错误，错误信息中会指明该构造）；它们恰好接受一个参数，并且永不返回。嵌套循环将它们绑定到最近的外层 `loop`。重新绑定使用完整的模式匹配：与状态模式不匹配的 `recur` 值会引发 GEP-0001-R012 匹配错误。

**GEP-0014-R006：** 编译为 Python 的 `while True:`：状态保存在编译器命名的变量中，每次迭代顶部进行模式匹配，`recur` 实现为赋值加 `continue`，`break(v)` 实现为结果赋值加 `break`——通过构造保证栈深度恒定。

## 理由

将 `rescue` 映射到 Python 异常类型而非发明一套 Gandora 异常层次结构，保持了互操作承诺：程序遇到的错误*就是* Python 的，通过互操作引用命名它们无需新命名空间。选择 `loop/recur` 而非裸 `while`，是因为它保留了 Elixir 的绑定与重新绑定的数据规范（状态是模式，而非突变），并且避免了尾调用优化式的自递归——因为 Python 无法支持该优化；Clojure 在 JVM 上恰恰规范了这种折衷方案。

## 向后兼容性

添加性的。`try`、`loop`、`recur`和`break`之前被诊断为不支持或未知的调用。

## 安全性与确定性

两种构造都被编译为本地 Python 控制流；不引入新的运行时表面。确定性（GEP-0001-R024）保持不变。

## 工具与AI使用

Agent 应使用 `loop` 进行无界迭代（服务器、REPL），使用递归进行有界结构；在裸变量捕获所有异常之前，`rescue` 特定异常类型；并保持 `after` 仅用于清理。

## 已拒绝的替代方案

### 一个裸的 `while cond do` 循环

对 Python 而言是诚实的，但放弃了使 Elixir 状态显式的模式重绑定规范；`loop` 多花一行代码，但保持了这一规范。

### 蹦床式自递归

将机制隐藏在每个函数调用内部；比一个 `while True` 更慢，且在生成的输出中更难阅读。

### Gandora 异常层次结构

会将每个 Python 错误包装两次；目标语言（Python）的异常才是真正的异常。

## 开放问题

本修订版无开放问题。

## Conformance

测试必须覆盖：rescue子句顺序和类型匹配（包括互操作类型）、catch-all子句、`after`在两条路径上运行、`try`作为表达式、带有元组模式状态的`loop`、`recur`重启、`break`的值、主体完成值、嵌套循环绑定、循环外诊断，以及一个百万次迭代的循环证明恒定栈。

## 变更历史

- 修订版 2，2026-08-02：示例已更新至 GEP-0003 修订版 2 的 `$` 互操作语法。

- 修订版 1，2026-08-02：初始版本。

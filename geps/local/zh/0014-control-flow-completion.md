---
gep: 14
title: 控制流完成
description: Elixir语法中的try/rescue/after；loop/break结构一直使用到GEP-0019递归将其淘汰，而recur作为函数级跳转幸存下来。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-02
updated: 2026-08-02
revision: 3
requires: [1]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0014-control-flow-completion.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0014-control-flow-completion.md](../../0014-control-flow-completion.md)。

# GEP-0014: 控制流完成

## Abstract

两个构造补全了真实程序——任务运行器、REPL 以及 GEP-0013+ 的 LSP 服务器——所需的控制流故事。  
`try/rescue/after` 采用 Elixir 的语法来操作 Python 的异常机制。  
`loop` 绑定一个状态模式，并通过 `recur(new_state)` 重复其主体，直到 `break(value)`（或主体执行完毕）结束它——这是在没有 TCO 的虚拟机上对尾递归的诚实替代，沿袭了 Clojure 的 `loop/recur` 传统。

## 动机

GEP-0001 推迟了 `try/rescue`；没有它，服务器无法从错误请求中幸存。而 Gandora 没有无界迭代：Elixir 服务器使用递归，但 Python 的递归限制（约1000帧）使其成为崩溃，而非一种模式。用 Gandora 编写工具链（GEP-0012 的目的）迫使现在同时填补这两个缺陷。

## 范围

这两个构造及其编译。`raise/2`（带异常类型）、`defexception`、`catch`/`throw`以及推导式`for`仍被推迟。

## 术语

- **救援子句**（Rescue clause）：`rescue` 内的 `pattern -> body`，其中 pattern 是一个变量或 `var in ExceptionRef`。
- **循环状态**（Loop state）：由 `loop` 绑定并由 `recur` 重新绑定的模式。

## 规范

### try/rescue/after

**GEP-0014-R001：** 该形式采用 Elixir 的语法：

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

`rescue` 包含一个或多个从上到下尝试的 `->` 子句；`after` 是可选的，并且始终执行。整个形式是一个表达式，其值为所采用分支的值（`after` 不贡献值，与 Elixir 中相同）。

**GEP-0014-R002：** 当所引发的 Python 异常是 `Ref`（任何求值为异常类型的表达式——通常是互操作引用，如 `$builtins.ValueError` 或 `$json.JSONDecodeError`）的实例时，匹配模式 `e in Ref`；裸 `e` 匹配所有 `Exception`。绑定的变量是 Python 异常对象，其字段可通过普通互操作访问（`e.args`、`to_string(e)`）。

**GEP-0014-R003：** 编译目标为 Python 的原生机制：`try`/`except ... as e`/`finally`，每个子句对应一个 `except` 分支，按顺序排列。`raise`（GEP-0001）保持不变，无变化。没有 `rescue` 或 `after` 的 `try` 是编译错误，错误信息中会提及该形式。

### loop/recur/break

**GEP-0014-R004：** 该形式为：

```elixir
loop state = initial do
  body
end
```

`state` 是任意模式，在入口处绑定到 `initial`。在主体内部，`recur(expr)` 将模式重新绑定到 `expr` 并重新启动主体；`break(expr)` 以 `expr` 作为值结束循环。如果主体执行完毕而没有遇到上述两者，则以主体的值结束循环。`loop` 是一个表达式。

**GEP-0014-R005：** `recur` 和 `break` 仅在 `loop` 主体内部有效（否则为编译错误，错误信息中会提及该构造）；它们只接受一个参数，并且永不返回。嵌套循环会将它们绑定到最近的封闭 `loop`。重新绑定使用完整的模式匹配：`recur` 的值如果与状态模式不匹配，则会引发 GEP-0001-R012 匹配错误。

**GEP-0014-R007：** `loop` 和 `break` 已废弃。GEP-0019 提供了恒定栈的尾递归以及 `recur` 在函数级别的编译器检查含义，GEP-0020 则提供了迭代的 `for` 推导式——该构造的存在理由（一个没有 TCO 的虚拟机）已不复存在，且 Elixir 没有 `loop`。迁移方式很简单：

```elixir
loop state = init do body end
# 转换为
defp step(state) do body end   # recur(x) 不变；break(v) -> v
step(init)
```

`loop` 和 `break` 会产生编译错误，并附带此迁移方案。以下 R004–R006 作为历史规范保留。

**GEP-0014-R006：** 编译为 Python 的 `while True:`，状态保存在编译器命名的变量中，每次迭代开始时匹配模式，`recur` 对应赋值加 `continue`，`break(v)` 对应结果赋值加 `break`——由此保证常量栈深度。

## 理由

将 `rescue` 映射到 Python 异常类型，而不是发明一套 Gandora 异常层次结构，可以保持互操作承诺：程序遇到的错误*就是* Python 的，通过互操作引用命名它们不需要新的命名空间。选择 `loop/recur` 而非裸 `while`，是因为它保留了 Elixir 的绑定-重绑定数据纪律（状态是一种模式，而非突变），并且避免了 TCO 风格的自我递归——因为 Python 无法支持它；Clojure 在 JVM 上正是将这种折中方案标准化了。

## 向后兼容性

增量式。`try`、`loop`、`recur` 和 `break` 先前被诊断为不受支持或未知的调用。

## Security and Determinism

两种结构都编译为本地 Python 控制流；不增加新的运行时层面。确定性 (GEP-0001-R024) 保持不变。

## 工具与 AI 使用

代理应针对无界迭代（如服务器、REPL）使用 `loop`，针对有界结构使用递归；在裸变量捕获所有异常之前，应 `rescue` 特定的异常类型；并仅将 `after` 用于清理操作。

## 被拒绝的替代方案

### 一个裸的 `while cond do` 循环

直接采用 Python 风格，但放弃了使 Elixir 状态显式的模式重绑定规范；`loop` 多花一行代码却能保持规范。

### 蹦床式自递归

将机制隐藏在每次函数调用中；在生成的输出中比一个 `while True` 更慢且更难阅读。

### Gandora 异常层次结构

会将每个 Python 错误包装两次；目标语言（Python）的异常才是真正的异常。

## 开放问题

本修订版无。

## 符合性

测试必须涵盖：异常处理子句的顺序和类型匹配（包括互操作类型）、通配子句、`after` 在两路径上的运行、`try` 作为表达式；带有元组模式状态的 `loop`、`recur` 重启、`break` 值、体完成值、嵌套循环绑定、循环外诊断；以及一个百万次迭代的循环，证明栈空间恒定。

## 变更历史

- 修订版 3，2026-08-03：R007 — `loop`/`break` 已废弃，改用 GEP-0019 递归和 GEP-0020 推导式；`recur` 保留在函数级别。

- 修订版 2，2026-08-02：示例更新为 GEP-0003 修订版 2 的 `$` 互操作语法。

- 修订版 1，2026-08-02：初始版本。

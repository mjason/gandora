---
gep: 30
title: 异步函数
description: 原生 `async def` 和 `await`，与 Python 的协程语法一一对应——协程世界用自己的话语表述，无需注解，也无需上下文重新编译。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
created: 2026-08-06
updated: 2026-08-07
revision: 4
requires: []
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0030-async-functions.md
source-revision: 4
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0030-async-functions.md](../../0030-async-functions.md)。

# GEP-0030: 异步函数

## 摘要

`async def` 和 `await` 是语言语法，一一对应地编译为 Python 自身的语法：`async def fetch(url)` 生成 `async def fetch(url)`，`await expr` 生成 `await expr` —— 裸露的，没有截止时间，也没有包装。没有任何东西会根据上下文被重新编译，也没有任何东西隐藏在注解之后：协程世界中可读的 Python *就是*其语法，因此 Gandora 书写该语法。截止时间、扇出和取消属于库的关注点，位于 `Task`（GEP-0029）中；该语法使 `Task` 可以作为普通标准模块表达。

## 动机

修订版 3 曾在 Python 的两个执行世界之间保持同一套不区分色彩的词汇：`@async true` 注解把 `def` 变成 `async def`，并且在其中 `Task.await(t)` 被重新编译为 `await asyncio.wait_for(t, 5.0)`。该设计在被替换前曾经过度量，但它在最重要的维度上失败了——读者认为代码在说什么：

- 同一行源代码会因别处的注解而表示两种不同的含义。`Task.await(t)` 在一个函数体中是线程 join，在另一个函数体中则是 `wait_for`。
- 重新编译把 5 秒截止时间偷偷塞进了一个看似普通 join 的操作。在 MCP 界面的实际测试中，基于 Gandora 自身已校验语料库的模型支持组合器写下了 `Task.await(task)`，并在文字中断言它“不带任何截止时间地等待”。生成的产物却写着 `asyncio.wait_for(task, 5.0)`。当语言自己的工具都无法理解语言的意图时，这种构造就是错误的。
- 生成的 Python——一种评审级产物（GEP-0001-R002）——比任何 Python 作者会写出的代码都要更嘈杂：在自然产物是裸 `await` 的地方，却包上了 `wait_for`。
- `@async true` 违反了 v0.18.8 中确立的注解契约：注解是描述代码的数据。而这个注解却改变了函数的调用约定。

Gleam 解决了这个设计问题。它的语言核心完全没有并发构造；每个编译目标都有一个库，以薄层方式暴露目标自身的原生模型——`gleam/javascript/promise` 包装 JavaScript 本就有的 Promise，`gleam/erlang/process` 直接暴露 BEAM——并且任何目标都不会模拟另一个目标的模型（gleam_otp v1 删除了自己的 BEAM `task` 模块，而不是保留一个遮蔽了原生模型的抽象）。Gandora 只有一个目标，却包含两个世界；协程世界的原生模型*就是语法*：`async def` 和 `await`。因此该语言原样提供这套语法，其之上的一切都是库。

## Scope

涵盖内容：`async def` / `async defp` 定义形式、`await` 前缀表达式、优先级、闭包和推导式边界、doctests、入口点规则，以及 `@async` 注解的移除。不在范围内：`async for` / `async with` 表面语法（递归会编译为循环，并且被等待的 `__aenter__`/`__anext__`/`__aexit__` 模式涵盖了重要的生命周期——这一点已在修订版 3 之前通过实时流式 API 得到验证）、异步生成器，以及 `Task` 模块本身（GEP-0029）。

## 术语

**异步函数体** — 用 `async def` 或 `async defp` 定义的函数体，不包括在其中编写的任何 `fn` 闭包。

**协程** — 在两个世界中调用异步函数所返回的内容；它仅在被等待或调度时运行。

## Specification

**GEP-0030-R001（async def）：** `async def` 和 `async defp` 定义编译为 Python `async def` 的函数；私有（`defp`）函数保留其名称改写（name mangling）。调用这种函数是一次普通调用，在两个世界中都返回协程对象。多子句函数的所有子句 MUST 携带相同的修饰符——混用 `def` 和 `async def` 子句是编译错误，正如混用 `def` 和 `defp` 已经是编译错误一样。`async def main` 是编译错误：入口点保持同步，并通过 `Task.run`（GEP-0029）进入协程世界。

**GEP-0030-R002（await）：** `await expr` 是前缀表达式，编译为 Python 的 `await`——裸用，没有截止时间，没有包装器。它的绑定比所有二元运算符（包括 `|>`）更紧，比调用、属性和索引链更松——与 Python 自身的优先级一致——因此 `await fetch(u) |> parse()` 会先等待 fetch，再将值送入管道，而 `await a + b` 会在加法前先等待 `a`。`await` 仅在 async 函数体内合法；在其他任何地方都是编译错误，并指明本条规则。

**GEP-0030-R003（闭包边界）：** `fn` 函数体是同步函数，即使写在 async 函数体内也是如此——Python lambda 不能 await，Gandora 也不会佯装可以。`fn` 内的 `await` 是编译错误。推导式主体会编译为外层函数中的原生 Python 推导式，因此 `await` 在那里合法：`for t <- tasks, do: await t` 是顺序连接，生成 `[await t for t in tasks]`。

**GEP-0030-R004（上下文关键字）：** `async` 和 `await` 只在各自位置才作为关键字——`async` 紧接在 `def`/`defp` 之前，`await` 紧接在表达式之前。其他位置二者仍然是普通标识符，因此不会破坏任何既有名称。将它们用作函数名或变量名合法但不建议；Advisor MAY 如此提示。

**GEP-0030-R005（doctest 运行）：** async 函数上的 `@example` 是普通 doctest——通过同步边界书写，它就能执行：`gan> Task.run(M.fetch("x"))` 是可运行的一行。修订版 3 中“只显示但不运行”的例外被撤回；无法运行的示例再次成为缺陷，而不是一种类别。

**GEP-0030-R006（不进行推断）：** 没有修饰符的函数永远不会编译为 `async def`，无论其函数体做什么。编译产物的签名是一个声明，在 diff 中可见，并且在函数体重构下保持稳定。

**GEP-0030-R007（注解已撤回）：** `@async` 再次成为普通模块属性，携带数据而不改变任何行为——修订版 3 的重新解释被撤回，随之撤回的还有整个上下文编译表：`Task.await`、`Task.async`、`Task.await_many`、`Task.try_await` 在任何地方都是普通的标准库调用（GEP-0029 定义它们的行为），编译器不会生成任何隐藏的辅助函数。

## 理由

**语法，而非注解。** 异步性是一个签名层面的事实——它改变调用返回什么——而签名位于函数头部，而非其上方的一个属性。注解路线还违背了 v0.18.8 的定案：注解值是数据；`@async true` 曾是唯一一个重连语义的注解，而它已不复存在。

**无默认期限。** 修订版 3 曾给裸 join 一个隐藏的 5 秒 `wait_for`，以在不同世界之间保持“一行源代码，一种含义”。随着 `Task.await` 消失，这一同一性自然成立——`await` 只出现在异步体中，且含义与 Python 的 `await` 完全一致。隐藏期限的实际代价——语言自身 AI 表面产生的错误陈述，以及审阅者必须略过的产物噪音——并未换来任何显式 `Task.try_await(t, ms)` 不能一目了然提供的东西。`:infinity` 也随之消亡：裸 `await` *就是* 无期限的写法。

**两个世界，直说。** 同步 Gandora 与异步 Gandora 都是真实存在的，就像 Gleam 的两个目标平台都是真实存在的一样。边界是显式的——`Task.run` 进入协程世界，`Task.blocking` 回到阻塞世界（GEP-0029）——而不是被一套在两侧编译方式不同的词汇所模糊。函数着色是 Python 的设计；隐藏它会让产物撒谎，因此 Gandora 把它展示出来。

**上下文关键字，而非保留字。** 若 `async`/`await` 在处处都被占用，会破坏 `Task.async` 本身。只占用这两种并置组合，解析器只需一次前瞻，且不会破坏任何现有代码。

## 向后兼容性

`@async true` 的重新解释仅存在于未发布的工作树中；撤回该解释会使 `@async` 恢复为与其他所有属性相同的状态。在此 GEP 之前，`async def` 的并置和 `await` 前缀均为解析错误，因此没有任何现有模块的含义发生变化。

## 安全性与确定性

不引入新的边界：异步函数运行在等待它的那个事件循环上，与其调用者拥有相同的信任级别。编译器不增加截止时间、调度或状态；所有与时间相关的内容在源代码和产物中都是同样显式的。

## 工具与 AI 使用

构造索引包含 `async def` 和 `await` 卡片；MCP 语料原子展示了由同步边缘连接的异步内部，其 doctest 运行（R005）。格式化器按原样打印这些形式 — `async def f(x) do` 和 `await expr` — 通过打印机的块形式和前缀拼写。智能体被告知：协程世界是原生语法；截止时间和扇出是 `Task` 库调用，在产物中可见。

## 符合性

测试 MUST 覆盖：针对 `async def` 和 `async defp`（有类型和无类型）的一对一代码生成；不带包装的裸 `await` 生成；优先级 — `await f(x) |> g()` 将等待后的值通过管道传递给 `g()`，`await a + b` 先等待再相加；推导式主体中的 `await` 生成原生推导式；编译错误 — `await` 在异步体外、`await` 在 `fn` 内部、`def`/`async def` 子句混合、`async def main`；通过 `Task.run` 执行的 doctest；以及一个端到端模块 — 从同步边缘扇出并汇合的异步内部 — 通过 `gan test`。

## 变更历史

- 修订版 4，2026-08-07：原生语法。`async def`/`async defp` 和
  `await` 前缀表达式取代了 `@async true`；上下文相关的 Task 编译表、
  生成的 `_gan_try_await` 辅助函数、显示但不运行的 doctest 例外，
  以及 `:infinity` 均被撤销。
  截止时间和扇出全部移入 GEP-0029 的 Task。
- 修订版 3，2026-08-07：`:infinity` 编译为裸 `await`
  （对于 `await_many` 则为裸 `gather`）——没有截止时间意味着
  产物中不会出现 `wait_for`。
- 修订版 2，2026-08-06：R002 遵循 GEP-0029 rev 3 的术语——
  `try_await` 通过生成的异步辅助函数原生编译；组合器和 `try_await_many`
  在 `@async` 函数体内是编译错误。
- 修订版 1，2026-08-06：初始版本——`@async true` 注解
  及上下文相关的 Task 编译。

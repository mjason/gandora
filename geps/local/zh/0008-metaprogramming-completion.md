---
gep: 8
title: 元编程完成
description: 生成定义的宏、use/__using__、定义头部中的 unquote，以及带有定义钩子的用户可扩展属性系统。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Macros
created: 2026-08-01
updated: 2026-08-08
revision: 2
requires: [2, 7]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0008-metaprogramming-completion.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0008-metaprogramming-completion.md](../../0008-metaprogramming-completion.md)。

# GEP-0008：元编程补全

## 摘要

本提案为 Gandora 补充元编程原语，使其与 Rust 的过程宏和 Elixir 的宏系统在能力上对齐，同时保持 Gandora 现有的 `quote` 与 `unquote` API 的简洁性。具体而言，本提案引入一个针对域特定语言的编译期求值机、一个作用于语法树节点的优化通道，以及一套受控的反射 API。这些能力共同实现了现有系统无法支持的、依赖于外部输入的代码生成用例（例如：从 JSON Schema 或 OpenAPI 规范生成类型安全的 HTTP 客户端）。

## 动机

Gandora 目前支持通过 `quote`/`unquote` 进行准引用（quasiquotation），但缺乏以下能力：

1. 在编译期间无条件地执行用户定义代码（除 `const` 函数外）。
2. 检查抽象语法树节点，以执行安全分析或结构重写。
3. 生成跨模块边界的、项目级的命名一致性检查。

因此，程序员被迫使用外部代码生成器，这将生成的代码与源注释脱离开来，并使调试变成对中间产物的检查。我们观察到一个反复出现的需求模式：用户希望编写自定义派生（derives）、自定义校验器，以及在 Rust 中通过过程宏或 Elixir 中通过 `use` 宏所实现的类似机制。

## 指南说明

本提案中的关键词“MUST”、“MUST NOT”、“SHOULD”、“SHOULD NOT”以及“MAY”按照 RFC 2119 中的描述进行解释。

## 目标

- 提供一套可组合的、类型安全的编译期代码转换 API。
- 在 Gandora 编译器内维护一个可复现的、确定性的评估模型。
- 保持包在多个目标平台上的可移植性，而不绑定到宿主运行时。

## 非目标

- 我们不打算引入依赖反射的运行时序列化框架。
- 我们不打算让宏执行任意的 IO 或网络请求，除非在显式选择的沙箱模式中。
- 我们不打算重新定义一个全新的宏语法，替代现有的 `quote`/`unquote`。

## 规格说明

### 编译期评估模型

GEP-0008-R001：本提案定义了一个新的内置过程，称为 `eval_compile_time`，它在编译单元的任何语义分析发生之前接受一个 Gandora AST 片段。

GEP-0008-R002：传递给 `eval_compile_time` 的代码片段 MUST 是一个无副作用的纯函数表达式；禁止任何形式的状态变更或 IO。

GEP-0008-R003：编译器 MUST 以深度优先、从左到右的顺序对函数参数求值，并将该顺序视为规范顺序。

GEP-0008-R004：`eval_compile_time` 的结果 MUST 是一个新的有效 Gandora AST 片段，该片段将在当前 AST 节点处进行拼接，并与当前节点遵循相同的范围解析规则。

GEP-0008-R005：评估器 MUST 使用一个大步语义，且不执行部分求值。所有发生在 `quote` 块内部的代码在拼接之前 MUST 被完整求值。

### 语法树遍历与安全检查

GEP-0008-R006：本提案引入一个标准化的 AST 访问器接口，名为 `AstVisitor`，它定义了每个 Gandora 语法节点种类的 `visit_*` 方法。

GEP-0008-R007：一个 AST 访问器 MAY 使用 `spine` 方法返回一个建议的 AST 重写；如果返回 `None`，编译器将保留原始节点不变。

GEP-0008-R008：任何检查代码中未处理的 panic 或求值错误 MUST 被报告为编译错误，并包含原始节点的源码位置。

GEP-0008-R009：单个编译单元中的所有安全检查 MUST 按照声明顺序执行。

### 反射 API

GEP-0008-R010：本提案为在编译期查询模块元数据定义了一个 `compiler_reflect` 模块。

GEP-0008-R011：`compiler_reflect` 暴露了以下函数：`module_exports/1`、`type_arity/1` 和 `source_path/0`。

GEP-0008-R012：`module_exports/1` MUST 返回一个经过排序的、由该模块公开导出的所有名称的列表，`type_arity/1` MUST 返回一个给定的类型构造器所接受的位置参数的数量。

GEP-0008-R013：通过 `compiler_reflect` 检索到的信息 MUST 在编译期被固化（快照）下来，不得引用运行时状态。

### 沙箱与 IO 策略

GEP-0008-R014：除非启用了 `--allow-macro-io` 标志，否则编译期代码 MUST NOT 访问文件系统、环境变量或网络套接字。

GEP-0008-R015：当启用了 `--allow-macro-io` 时，编译器 MAY 缓存重复的编译期计算，但 ONLY IF 其输入哈希值与先前运行中的输入一致。

### 与现有 `quote` 的集成

GEP-0008-R016：在 `quote` 块内，一个被显式标记为 `{compile_time_call, Mod, Fun, Args}` 的节点 MAY 作为拼接前的编译期求值请求来执行。

GEP-0008-R017：在 `quote` 块中出现的所有明确标记的编译期调用 MUST 在拼接发生之前按照从左到右的顺序执行。

GEP-0008-R018：`unquote` 仍然只接受已求值的 Gandora AST 片段；它不得接受任意的编译期函数引用。

## 示例

```gandora
import compiler_reflect.{module_exports, type_arity}

schema @derive(JsonCodec)
defmodule User do
  defstruct [:name, :email]
end

# 编译期派生
quote do
  def encode(%User{name: name, email: email}) do
    %{"name" => name, "email" => email}
  end
end
|> eval_compile_time()

exports = compiler_reflect.module_exports(User)
# exports == [:__struct__, :name, :email, :encode]
```

## 兼容性

GEP-0008-R019：本提案中描述的所有新 API 均作为可选的编译器扩展提供；不引入新 API 的程序 MUST 保持与未打补丁的编译器完全一致的可观察行为。

GEP-0008-R020：现有的 `quote`/`unquote` 用法 MUST 保持完全向后兼容。

GEP-0008-R021：`eval_compile_time` 的结果 MUST 能够在适用的情况下被进一步宏展开，但必须在同一个编译器遍历中进行。

## 安全性与隐私

GEP-0008-R022：除非显式启用，否则编译期执行上下文 MUST 不得访问进程环境变量或用户主目录。

GEP-0008-R023：当编译不受信任的代码时，未使用 `--allow-macro-io` 标志的编译器 MUST 在评估任何编译期代码之前执行一个静态禁令检查。

## 测试与验证

GEP-0008-R024：针对本规范实现的编译测试套件 MUST 包含至少五个正向用例、三个负向用例以及两个沙箱拒绝用例。

GEP-0008-R025：负向用例 MUST 验证来自编译期求值的 panic 被转换为带源码位置的编译错误。

GEP-0008-R026：本规范要求使用 golden 文件测试来锁定所有 `eval_compile_time` 的输出，只要编译器核心 AST 可以稳定地序列化。

## 开放问题

- 在增量编译场景下，编译期求值的缓存失效策略是否应与其他编译工件统一处理？
- 我们是否应该支持 `eval_compile_time` 与包管理器（例如：uv 风格的锁文件）之间的显式交互，以固定编译期依赖项？
- 当涉及编译器内部函数（intrinsics）时，`compiler_reflect` 对用户模块的限制是否也适用于标准库自身？

## 摘要

本提案完善了 Elixir 的元编程体系（quote 与 unquote、宏以及领域特定语言指南）。宏现在可以生成定义；`def unquote(name)(...)` 可在模板中工作；`use Mod` 会调用 `Mod.__using__`。内置属性不再特殊：`defattr` 注册用户属性（可累积或不可累积），而 `@on_definition` 指定一个钩子宏，它接收每个定义及其收集到的属性，并可以重写该定义——这正是构建 `@doc`/`@example` 这一类特性所依据的机制，现在已对用户开放。

## 动机

v0 宏系统（GEP-0002）刻意禁止生成定义，这阻碍了 Elixir 的标志性模式：`deftest` 风格的 DSL、`use` 注入的 API，以及属性驱动的代码生成（路由、数据模式）。与此同时，Gandora 自身的 `@doc`/`@example` 是编译器硬编码的；Elixir 展示了更好的形态——`Module.register_attribute/3` 加上 `@on_definition`——在这种形态下，语言在用户空间内生长出自己的注解系统。

## 范围

定义生成、`use`、头部中的 unquote、属性注册以及定义钩子。`@before_compile`/`@after_compile` 钩子暂缓处理。跨包钩子分发在修订版 1 中被推迟，结果发现除了 GEP-0006 的宏发送之外不需要任何额外机制——钩子就是宏——并且自修订版 2 起已得到实际使用（`gan-lsc` 从 gandora-tool 消费 `Cli.on_command`）。

## 术语

- **声明宏**：在模块顶层调用的宏，其展开产生定义。
- **注册属性**：由 `defattr` 声明的属性。
- **定义钩子**：名为 `@on_definition` 的宏，对每个后续定义运行。

## 规范

**GEP-0008-R001:** 模块顶层的宏展开 MAY 产生 `def`、`defp`、`defstruct`、文档属性、`@decorate`，以及这些内容的 `__block__` 序列，编译器会将它们扁平化到模块体中。展开 MAY 另外产生对已注册属性的写入（rev 2）：这些写入被吸收为注册项——按展开顺序加入 R004 值表，并遵循相同的累积规则——且绝不会作为普通模块属性到达代码生成阶段。这是 R005“定义加注册”的后半部分：没有它，钩子可以重写定义，但永远无法构建使重写有价值的那张表。头部形式（`defmodule`、`alias`、`import`、`require`、`pyimport`、`use`）MUST NOT 由宏生成。本条修订 GEP-0002-R007。

**GEP-0008-R002:** 在 quote 模板中，`def unquote(expr)(params)`（以及 `defp`）定义了一个函数，其名称在展开时是 `expr` 的原子或字符串值，从而支持名称由计算得出的定义。相同的强制转换适用于捕获位置（rev 2）：`&unquote(expr)/n` 捕获该命名局部函数，包括私有名称修饰——收到了定义头部的钩子可以为其注册一个可调用项。

**GEP-0008-R003:** 在模块顶层，`use Mod` 和 `use Mod, opts` 等价于先 `require Mod`，再就地展开 `Mod.__using__(opts)`（元数为 0 或 1）。没有 `__using__` 的 `use` 目标会引发一个指明该模块的编译错误。

**GEP-0008-R004:** `defattr :name` 为当前模块注册一个模块属性；`defattr :name, accumulate: true` 使重复出现的 `@name value` 按源代码顺序收集，而不是报错。值是引用项。`@name` 读取遵循 GEP-0004-R011；累积属性的读取结果是一个列表。未注册的非内置属性仍保持为 GEP-0004 的模块属性绑定。

**GEP-0008-R005:** `@on_definition Mod.hook`（在 `require Mod` 之后）注册一个定义钩子。对于随后的每个 `def`/`defp`，编译器展开 `Mod.hook(kind, head, attrs, body)`，其中 `kind` 是 `:def`/`:defp`，`head` 和 `body` 是该定义的引用项，`attrs` 是一个关键字列表，包含自上一次定义以来收集的已注册属性值（这些值随后被重置，就像 `@doc` 一样）。钩子返回替换用的顶层语法（通常是重建的定义加上注册项），并受 R001 约束。钩子在 GEP-0002-R003 的沙箱中运行，遵循其确定性和限制。

**GEP-0008-R006:** 内置属性（`@doc` 系列、`@example`、`@decorate`、`@moduledoc` 系列）保持其 GEP-0007/0003 语义，并且对钩子不可见；`defattr` 名称与内置属性冲突是一个编译错误。

## 理由

定义钩子接收 (kind, head, attrs, body) 正是 Elixir 库构建注解系统的方式；将收集到的属性传入重写宏，即可涵盖装饰器注册表、路由表和类文档通道，而无需按用例逐一添加编译器特性。`use` 与声明宏相结合，便能端到端地覆盖 DSL 指南中的 `deftest` 模式。头部形式保持不可生成，从而模块标识与依赖图始终保持静态可知（GEP-0002-R006 的保证）。

## 向后兼容性

修订 GEP-0002-R007（仅放宽）。现有属性语义（GEP-0004）对于未注册名称保持不变。

## 安全与确定性

所有内容都在现有的扩展沙盒中运行；钩子不增添任何新能力，只增添新的输入。扩展保持确定性和有界性。

## 工具与 AI 使用

Agent 应该使用 `use` + 声明宏来构建 DSL，使用 `defattr` + `@on_definition` 来构建注解系统，并使用 `gan expand` 验证生成的定义。

## 已否决的替代方案

### 每个特性的编译器专用装饰器

每个新注解（路由、缓存、追踪）都需要编译器工作；而钩子机制则将其移至库中，正如 Elixir 所做的那样。

### 允许宏生成的导入/defmodule

这会使模块图依赖于展开结果，从而破坏静态宏解析和包发现。

## 开放问题

本修订版无。

## 符合性

测试 MUST 覆盖：生成多个 defs（包括扁平化）的声明宏；`def unquote(name)(...)`；带和不带 opts 的 `use` 及其缺少 `__using__` 的诊断；累积和非累积的 `defattr`，具有每次定义重置的语义；一个 `@on_definition` 钩子，用于装饰和注册定义；R006 冲突诊断；以及上述所有内容的 `gan expand` 输出。

## 变更历史

- 修订版 2，2026-08-08：R001 — 展开可以写入已注册属性，这些属性作为注册被吸收（即 R005 的“定义加注册”承诺，此前实现将其当作重复的普通属性而拒绝）；R002 — 原子/字符串强制转换扩展到捕获位置（`&unquote(name)/n`）。其动机是工具链转向声明式：`gan` 的命令表和 `gan-mcp` 的工具表由 `@on_definition` 钩子根据定义自身的注解构建，因此数据与执行不会漂移。

- 修订版 1，2026-08-01：初始版本。

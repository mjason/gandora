---
gep: 4
title: 结构体和模块属性
description: defstruct 编译为冻结数据类、结构体字面量、模式与更新，以及作为导入时绑定的模块属性，支持有状态的 Python 装饰器。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-02
revision: 2
requires: [1, 3]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0004-structs-and-module-attributes.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0004-structs-and-module-attributes.md](../../0004-structs-and-module-attributes.md)。

# GEP-0004: 结构体和模块属性

## 摘要

本提案在 v0 表面层新增两个模块级数据声明。`defstruct` 声明其模块的结构体，并编译为冻结的 Python `dataclass`，为 Gandora 提供类型化记录的操作便利性——构造 `%User{name: "MJ"}`、模式匹配 `%User{name: n}` 以及函数式更新 `%User{u | age: 2}`——同时生成一个任何 Python 库都能理解的类。模块属性 `@name expr` 编译为在导入时求值的模块级赋值，这使得有状态的 Python 装饰器（例如 `@decorate @app.route("/")` 跟在 `@app $flask.Flask(...)` 之后）无需包装模块即可表达。

## 动机

GEP-0001 有意排除了 `defstruct`，而 GEP-0003 的 `@decorate` 仅覆盖那些可作为纯表达式访问的装饰器。现实中的 Python 复用很快触及这两个限制：结构化数据需要具有名称和默认值的字段，而非原始映射；Python 生态中最常见的装饰器习惯用法——Flask、FastAPI、Celery、atexit 风格注册表——使用模块级对象的方法装饰函数。这两个缺口可通过与惯用 Python 一一对应的声明来填补：数据类（dataclasses）和模块级常量。

## 范围

`defstruct` 声明；结构体字面量、模式匹配和更新语法；普通映射更新形式 `%{m | k: v}`；模块属性和属性读取。跨模块的编译期字段验证、结构体类型化、协议和派生不在范围内。

## 术语

- **结构体**：模块可以用 `defstruct` 声明的单一具名记录类型。
- **结构体类**：表示结构体的生成 Python 数据类。
- **模块属性**：用 `@name expr` 声明的具名模块级绑定，与保留属性 `@doc`、`@moduledoc` 和 `@decorate` 不同。

## 规范

### 结构体声明

**GEP-0004-R001:** `defstruct` 必须仅出现在模块顶层，且每个模块至多出现一次。其参数可以是 `field: default` 对组成的关键字列表、原子列表（字段默认值为 `nil`），或两者混合的列表。字段顺序为声明顺序。

**GEP-0004-R002:** 模块 `App.User` 的结构体必须编译为 Python 的 `dataclasses.dataclass`，并设置 `frozen=True`，以模块最后一段命名（`User`），定义在该模块生成的文件中，位于模块属性和函数之前。字段默认值按声明顺序编译；若默认值为列表、元组或映射字面量，则必须编译为 `default_factory`，以确保可变默认值是每个实例独立的。

**GEP-0004-R003:** 结构体实例是不可变的：对字段赋值将引发标准的 `dataclasses.FrozenInstanceError`。函数式更新（R006）是得到变更值的受支持方式。

### 结构体表达式与模式

**GEP-0004-R004:** `%Mod{field: value, ...}` 构造一个结构体。`Mod` 的解析方式与任何模块引用相同（别名适用）；当解析到当前模块时，生成的代码使用局部类名，否则导入目标模块。构造编译为关键字参数构造函数调用；省略的字段取默认值。

**GEP-0004-R005:** 在 v0 版本中，编译器不执行跨模块字段验证；引用不存在的字段将在运行时失败，并引发 Python 标准的 `TypeError`。未来的修订版本 MAY 添加静态验证。

**GEP-0004-R006:** `%Mod{expr | field: value, ...}` 是函数式更新：它求值为一个新结构体，与 `expr` 相等，但替换了指定字段，编译为 `dataclasses.replace`。字段访问仍采用 GEP-0003-R004 的后缀形式（`user.name`）。

**GEP-0004-R007:** 在模式位置出现的 `%Mod{field: pattern, ...}` 必须精确匹配该结构体类的实例（包括类型检查），然后将每个命名字段的值与其子模式进行匹配。省略的字段不受约束。

**GEP-0004-R008:** `%{expr | key: value, ...}`（无模块）是普通映射更新，编译为字典合并（`{**expr, "key": value}`）。与 Elixir 不同，v0 在键不存在时不会引发异常；此差异必须被记录，并且未来的 GEP MAY 收紧此行为。

### 模块属性

**GEP-0004-R009:** 在模块顶层，`@name expr` 声明了模块属性 `name`，其中 `name` 不是保留属性（`doc`、`moduledoc`、`decorate`）之一。它必须编译为模块级别的 Python 赋值，在导入时求值，并按源代码顺序出现在导入语句和结构体类之后、函数定义之前。

**GEP-0004-R010:** 属性初始化器 MAY 引用导入、`pyimport` 别名、模块的结构体以及模块中先前声明的属性。每个属性必须恰好声明一次；重复声明将导致编译错误。

**GEP-0004-R011:** 在表达式位置出现的 `@name`（包括在 `@decorate` 表达式和后缀链如 `@decorate @app.route("/")` 内部）读取该属性并编译为模块级别的名称。读取未声明的属性是编译错误。

**GEP-0004-R012:** 与 Elixir 不同，Gandora 模块属性是导入时的运行时绑定，而非内联的编译时常量。此差异是故意的（Rationale），并且必须在面向用户的材料中记录。

## Rationale

一个冻结的 dataclass 是 Python 中最接近 Elixir 结构体的对象：带有默认值的命名字段、结构相等性、支持关键字模式的 `match`、以及规范的 repr —— 所有这些都由标准库生成，输出可读，且任何 Python 代码均可使用。`frozen=True` 保留了 Elixir 的不可变性契约，并使 `dataclasses.replace` 成为自然的更新形式。

模块属性与 Elixir 的编译时语义不同，因为主要用例——如 Flask 应用中的装饰器状态——需要一个共享的运行时对象，而不是在每个使用点内联的值。内联 `@app $flask.Flask(...)` 会为每个引用创建一个应用实例，这绝不是作者的本意。导入时赋值与 Python 作者手动编写的方式一致。

结构体更新的写法 `%Mod{expr | ...}`（而不是扩展 `%{expr | ...}`）是必需的，因为结构体编译为类，而非字典，因此两种更新需要不同的生成代码，而编译器无法在零运行时开销下区分它们；Elixir 接受相同的限定写法。

## 向后兼容性

纯属对 GEP-0001/0003 的补充。现有程序编译不变。保留属性集不变；仅有先前被拒绝的语法变得有意义。

## Security and Determinism

属性初始化器和结构体默认值是由 Python 在导入时执行的普通编译表达式，与手动编写的模块级代码完全相同；编译器仍然不会在编译时导入或执行任何内容。生成的输出保持确定性。

## 工具与 AI 使用

代理应优先使用结构体（struct）而非临时映射（ad-hoc map），一旦形状有名称；使用 `%Mod{expr | ...}` 而不要逐字段重建结构体；并将装饰器状态（`Flask`、注册表、会话）声明为模块属性，而非生成包装器 Python 模块。

## Rejected Alternatives

### Compile structs to plain dicts with a type tag

更接近 Elixir 基于映射的结构体，并支持 `%{... | ...}` 更新，但每个 Python 使用者会看到字典而不是类型化对象，从而失去属性访问、`isinstance` 和库互操作——而这正是编译到 Python 的根本原因。

### Elixir-faithful compile-time attributes with inlining

会保持 `@name` 语义与 Elixir 完全相同，但使得标志性的装饰器用例（一个共享的应用对象）无法表达，而常量内联是 Python 不需要我们做的优化。

### Mutable dataclasses (`frozen=False`)

对可变 Python 库更友好，但悄然放弃了 Elixir 的数据模型；需要变动的库可以通过互操作接收字典或专用的 Python 端类来代替。

## 开放问题

此修订版无。

## 一致性

测试必须涵盖：声明形式（关键字列表、原子列表、混合）；可变默认值的默认工厂；带省略字段和不带省略字段的构造；同模块和跨模块的构造与模式；冻结性；函数式更新；映射更新；属性声明顺序、重复和未声明的诊断；函数体中和`@decorate`链中的属性读取；以及一个端到端程序，执行保存在模块属性中的装饰器。

## 变更历史

- 修订版 2，2026-08-02：更新示例，以适配 GEP-0003 修订版 2 的 `$` 互操作语法。

- 修订版 1，2026-08-01：初始版本。

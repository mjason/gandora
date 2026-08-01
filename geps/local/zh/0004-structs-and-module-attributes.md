---
gep: 4
title: 结构体和模块属性
description: defstruct 被编译为冻结数据类、结构字面量、模式与更新，以及模块属性作为导入时绑定，从而实现有状态的 Python 装饰器。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Interop
created: 2026-08-01
updated: 2026-08-01
revision: 1
requires: [1, 3]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0004-structs-and-module-attributes.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0004-structs-and-module-attributes.md](../../0004-structs-and-module-attributes.md)。

# GEP-0004: 结构体和模块属性

## 摘要

本提案在 v0 表面增加了两个模块级数据声明。
`defstruct` 声明其模块的结构体，并编译为冻结的 Python `dataclass`，为 Gandora 提供类型化记录的易用性——构造 `%User{name: "MJ"}`、模式匹配 `%User{name: n}` 以及函数式更新 `%User{u | age: 2}`——这些功能基于任何 Python 库都能理解的类实现。模块属性 `@name expr` 编译为在导入时求值的模块级赋值，这使得有状态的 Python 装饰器（`@app :flask.Flask(...)` 后的 `@decorate @app.route("/")`）无需包装模块即可表达。

## 动机

GEP-0001 有意排除了 `defstruct`，而 GEP-0003 的 `@decorate` 仅覆盖可作为纯表达式访问的装饰器。实际的 Python 重用很快遇到这两个限制：结构化数据需要带有名称和默认值的字段，而不是原始映射；Python 生态中最常见的装饰器惯用法——Flask、FastAPI、Celery、atexit 风格的注册表——使用模块级对象的方法来装饰函数。这两个缺口都可以通过一对一映射到地道 Python 声明的声明来填补：dataclasses 和模块级常量。

## 范围

`defstruct` 声明；结构体字面量、模式匹配和更新语法；纯映射更新形式 `%{m | k: v}`；模块属性和属性读取。跨模块的编译时字段验证、结构体类型化、协议和派生不在范围之内。

## Terminology

- **Struct**：模块可使用 `defstruct` 声明的单一命名记录类型。
- **Struct class**：生成的表示结构体的 Python 数据类。
- **Module attribute**：使用 `@name expr` 声明的命名模块级绑定，与保留属性 `@doc`、`@moduledoc` 和 `@decorate` 不同。

## 规范

### 结构体声明

**GEP-0004-R001：** `defstruct` 必须仅出现在模块顶层，且每个模块最多出现一次。其参数为 `field: default` 对构成的关键字列表、原子列表（字段默认值为 `nil`），或两者混合的列表。字段顺序为声明顺序。

**GEP-0004-R002：** 模块 `App.User` 的结构体必须编译为 Python 的 `dataclasses.dataclass`，且 `frozen=True`，以最终模块段（`User`）命名，定义在该模块的生成文件中，位于模块属性和函数之前。字段默认值按声明顺序编译；若默认值为列表、元组或映射字面量，则必须编译为 `default_factory`，以确保可变默认值为每个实例所独有。

**GEP-0004-R003：** 结构体实例是不可变的：对字段赋值会引发标准 `dataclasses.FrozenInstanceError`。函数式更新（R006）是派生变更值所支持的方式。

### 结构体表达式与模式

**GEP-0004-R004：** `%Mod{field: value, ...}` 构造一个结构体。`Mod` 的解析方式与任何模块引用相同（别名适用）；当它解析为封闭模块时，生成的代码使用本地类名，否则导入目标模块。构造编译为关键字参数的构造函数调用；省略的字段取其默认值。

**GEP-0004-R005：** 在 v0 版本中，编译器不执行跨模块字段验证；引用不存在的字段会在运行时失败，并引发 Python 的标准 `TypeError`。未来的修订版可能会增加静态验证。

**GEP-0004-R006：** `%Mod{expr | field: value, ...}` 是函数式更新：它求值为一个新结构体，该结构体等于 `expr` 但替换了指定字段，编译为 `dataclasses.replace`。字段访问仍使用 GEP-0003-R004 的后缀形式（`user.name`）。

**GEP-0004-R007：** 处于模式位置的 `%Mod{field: pattern, ...}` 必须精确匹配结构体类的实例（包括类型检查），然后逐一匹配每个指定字段的值与其子模式。省略的字段不受约束。

**GEP-0004-R008：** `%{expr | key: value, ...}`（无模块）是纯映射更新，编译为字典合并（`{**expr, "key": value}`）。与 Elixir 不同，v0 版本在键不存在时不会引发异常；此差异必须记录在文档中，并且未来 GEP 可能会收紧此行为。

### 模块属性

**GEP-0004-R009：** 在模块顶层，`@name expr` 其中 `name` 不是保留属性（`doc`、`moduledoc`、`decorate`）时，声明模块属性 `name`。它必须编译为模块级的 Python 赋值语句，在导入时求值，按源码顺序生成在导入语句和结构体类之后、函数定义之前。

**GEP-0004-R010：** 属性初始化器可以引用导入、`pyimport` 别名、模块的结构体以及模块中较早声明的属性。每个属性必须恰好声明一次；重复声明会导致编译错误。

**GEP-0004-R011：** 处于表达式位置的 `@name`——包括在 `@decorate` 表达式内部以及后缀链（如 `@decorate @app.route("/")`）中——读取该属性并编译为模块级名称。读取未声明的属性会导致编译错误。

**GEP-0004-R012：** 与 Elixir 不同，Gandora 模块属性是导入时的运行时绑定，而非内联的编译时常量。此差异是有意为之（参见 Rationale），并且必须在面向用户的材料中记录。

## 原理

冻结的数据类（frozen dataclass）是 Python 中最接近 Elixir 结构体（struct）的对象：带默认值的命名字段、结构相等性、支持关键字模式的 `match`，以及规范的 repr——所有这些均由标准库生成，输出可读，且能被任何 Python 代码使用。`frozen=True` 保留了 Elixir 的不可变性契约，并使 `dataclasses.replace` 成为自然的更新形式。

模块属性（module attributes）与 Elixir 的编译时语义不同，因为主要用例——如 Flask 应用中的装饰器状态——需要一个共享的运行时对象，而不是在每个使用点内联的值。内联 `@app :flask.Flask(...)` 会为每个引用创建一个应用，而这绝不是作者的意图。导入时赋值（import-time assignment）与 Python 作者手动编写的方式一致。

结构体更新写法 `%Mod{expr | ...}`（而非扩展 `%{expr | ...}`）是必需的，因为结构体被编译为类而非字典，因此两种更新需要不同的生成代码，且编译器无法在零运行时成本下区分它们；Elixir 接受相同的限定写法。

## 向后兼容性

纯粹是 GEP-0001/0003 的增量扩展。现有程序编译不变。保留属性集保持不变；只有先前被拒绝的语法变得有意义。

## 安全性与确定性

属性初始值设定项和结构体默认值是由 Python 在导入时执行的普通编译表达式，与手写的模块级代码完全相同；编译器在编译时仍然从不导入或执行任何内容。生成的输出保持确定性。

## 工具与AI使用

一旦某个形状有了名称，智能体应优先使用结构体而非临时映射，使用 `%Mod{expr | ...}` 代替逐个字段重建结构体，并将装饰器状态（`Flask`、注册表、会话）声明为模块属性，而不是生成包装器 Python 模块。

## 被拒绝的备选方案

### 将结构体编译为带类型标签的普通字典

更接近 Elixir 基于映射的结构体，并支持 `%{... | ...}` 更新，但每个 Python 使用者看到的将是字典而非类型化对象，从而失去属性访问、`isinstance` 以及库互操作能力——而这正是编译到 Python 的主要原因。

### 保持 Elixir 语义的编译时属性及内联展开

能保持 `@name` 与 Elixir 完全一致的语义，但使得标志性的装饰器用例（一个共享的 app 对象）无法表达，而常量内联是一种 Python 并不需要我们提供的优化。

### 可变数据类（`frozen=False`）

对可变 Python 库更友好，但会悄悄放弃 Elixir 的数据模型；需要可变性的库可以通过互操作机制接收一个字典或一个专用 Python 端类。

## Open Questions

本修订版无开放问题。

## 一致性

测试 MUST 涵盖：声明形式（关键字列表、原子列表、混合形式）；可变默认值的默认工厂；带省略字段和不带省略字段的构造；同模块和跨模块的构造与模式；冻结性；函数式更新；映射更新；属性声明顺序、重复和未声明的诊断；函数体中和 `@decorate` 链中的属性读取；以及一个端到端程序，练习存储在模块属性中的装饰器。

## 变更历史

- 修订版本 1, 2026-08-01: 初始版本。

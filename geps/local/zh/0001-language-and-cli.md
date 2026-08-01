---
gep: 1
title: Gandora语言与gan CLI
description: Gandora的核心身份，Elixir风格的表面语法，编译到Python，模块命名，项目配置，以及gan命令行。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - CLI
  - Configuration
created: 2026-08-01
updated: 2026-08-01
revision: 1
requires: [0]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0001-language-and-cli.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0001-language-and-cli.md](../../0001-language-and-cli.md)。

# GEP-0001: Gandora 语言与 gan CLI

## 摘要

Gandora 是一种编程语言，采用 Elixir 风格的语法，编译为可读的 Python。本提案定义了语言标识、源文件格式、模块命名及其到生成 Python 的映射、项目配置文件 `gandora.jsonc`、与 `uv` 管理的 Python 项目的集成，以及 `gan` 命令行界面。

Gandora 刻意采用了 Elixir 的表面语法：`defmodule`/`def` 配合 `do ... end` 块、原子、模式匹配、`|>` 管道，以及基于 `defmacro` 的元编程。Elixir 通过 `:erlang` 原子调用访问其宿主平台，而 Gandora 以相同方式访问 Python：`:math`、`:json` 或任何已安装的模块都可以直接调用，无需包装代码。

## 动机

Python 拥有最大的库生态系统；Elixir 拥有最令人愉悦的函数式、面向流水线编程的表面之一，以及最好的宏系统之一。Gandora 将两者结合：开发者编写 Elixir 风格的模块和宏，而部署仍然是由 `uv` 管理的普通 Python，生成的代码足够可读，可以使用标准 Python 工具进行审查、调试和性能分析。

需要一个精确的初始提案，以便编译器、包集成和未来工具共享一个契约，并且 AI 代理可以基于稳定的需求而非口头传说来实现。

## 范围

本提案涵盖语言标识、源文件、表层语法清单、模块命名与Python名称映射、项目配置、`uv`集成以及`gan`命令行界面。宏系统由GEP-0002规定，Python互操作合同由GEP-0003规定。标准库、格式化器、LSP和包发布格式不在本提案范围内，需要未来的GEP定义。

## Terminology

- **源文件**: 一个 UTF-8 文本文件，扩展名为 `.gan`。
- **模块**: 由 `defmodule` 声明的命名空间单元。
- **项目**: 由一份 `gandora.jsonc` 和一份
  `pyproject.toml` 管理的目录树。
- **生成模块**: 为一个 Gandora 模块生成的 Python 文件。
- **远程原子调用**: `:module.function(args)` 形式，调用
  Python（GEP-0003）。
- **入口函数**: `gan run` 使用的零参数或单参数 `main` 函数。

## 规范

### 标识

**GEP-0001-R001：** 语言 MUST 命名为 **Gandora**。编译器可执行文件 MUST 命名为 **`gan`**。源文件 MUST 使用扩展名 **`.gan`**。

**GEP-0001-R002：** 编译器 MUST 实现为原生可执行文件，其唯一的构建产物是 Python 源代码。生成的 Python MUST NOT 在运行时需要 Gandora 编译器、Gandora 运行时包或任何导入钩子——仅需项目声明的 Python 依赖。

**GEP-0001-R003：** 编译器的 Python 发行版 MUST 命名为 `gandora-lang`，且 MUST 安装原生 `gan` 可执行文件。Python 依赖管理、虚拟环境和发布仍由标准 Python 工具链负责；默认文档化工作流使用 `uv` 和标准 `.venv` 布局。

**GEP-0001-R004：** 一次编译器调用 MUST 针对恰好一个 Python 版本，配置为 `targetPython`，默认值为 `3.12`。生成的代码 MAY 使用目标版本中可用的任何语法，包括 `match` 语句。

### 表层语法

**GEP-0001-R005：** Gandora 表层语法 MUST 遵循 Elixir 的表层文法（针对语言支持的构造），以便 Elixir 语法高亮和编辑器辅助功能保持可用。与 Elixir 的差异 MUST 记录在 GEP 中。

**GEP-0001-R006：** v0 表层 MUST 包含：

- `defmodule Name do ... end`，使用点分隔的驼峰式模块名；
- `def`、`defp`（私有），使用 `do ... end` 主体，以及关键字简写形式 `def f(x), do: expr`；
- `defmacro` 和 `quote`/`unquote`/`unquote_splicing`（GEP-0002）；
- 模块属性 `@moduledoc` 和 `@doc`，值为字符串；
- 整数（含 `_` 分隔符）、浮点数、布尔值、`nil`、支持 `#{...}` 插值的字符串、原子（`:ok`、`:"quoted"`）、列表、元组（`{a, b}`）、映射（`%{"k" => v, a: 1}`）、关键字列表（`[a: 1, b: 2]`）、区间（`a..b`）；
- 运算符 `+ - * / == != < > <= >= and or not ++ <> |> = // rem div`，采用 Elixir 优先级，其中 `//` 和 `div`/`rem` 遵循 Elixir 的整数除法语义；
- `if/else`、`unless`、`case`、`cond`、`fn args -> body end` 匿名函数及 `&Mod.fun/1` / `&(&1 + 1)` 捕获、`with` 用于链式匹配；
- 模式匹配，通过 `=`、在 `case` 子句中、在函数头中，包括字面量、变量、`_`、元组、列表（`[h | t]`）、映射和 pin（`^x`）模式；
- 多子句函数定义，按模式从上到下调度，并可选 `when` 守卫；
- `alias`、`import` 和 `require`，支持 `as:`、`only:` 和 `except:` 选项，遵循 Elixir 语义；
- 远程原子调用和 `expr.name`/`expr.name(args)` 后缀访问（GEP-0003）。

**GEP-0001-R007：** 对于 v0 表层之外的构造，MUST 产生一个诊断信息，指明不支持的构造；编译器 MUST NOT 静默地错误翻译其未实现的 Elixir 语法。显著的 v0 排除项：协议、行为、`defstruct`、`receive` 及所有进程原语、`try/rescue`、sigil、二进制/位串（`<<>>`）和推导式（`for`）。

**GEP-0001-R008：** 注释使用 `#` 到行尾。源文件 MUST 为 UTF-8，标识符 MUST 遵循 Elixir 规则：函数和变量为 `snake_case`（允许 Unicode 字母），可选以 `?` 或 `!` 结尾；模块为点分隔的驼峰式段。

### 求值语义

**GEP-0001-R009：** Gandora 数据类型 MUST 映射到 Python 值，使得互操作边界上无需包装类型：

| Gandora | Python |
| --- | --- |
| 整数、浮点数、布尔值、字符串 | `int`、`float`、`bool`、`str` |
| `nil` | `None` |
| 原子 `:name` | 内部字符串 `"name"` |
| 列表 | `list` |
| 元组 | `tuple` |
| 映射 | `dict` |
| 关键字列表 | 由 2 元素元组组成的 `list` |
| 区间 `a..b` | 包含性 `range(a, b + 1)` |
| 匿名函数 | Python 可调用对象 |

**GEP-0001-R010：** 原子编译为 Python 字符串字面量。`true`、`false` 和 `nil` 是 Python 单例，而非原子。相等性 `==` 为 Python 相等性。真值判定遵循 Elixir：仅 `false` 和 `nil` 为假，因此条件语句和布尔形式 MUST 编译为显式的 `is not falsy` 检查，而非 Python 的真值判定。

**GEP-0001-R011：** 变量为不可变绑定；在顺序主体中重新绑定名称会创建新绑定，与 Elixir 相同。`case`、`if` 或 `cond` 表达式返回其采纳分支的值。每个函数返回其最后一个表达式的值；编译器插入显式的 `return` 语句。

**GEP-0001-R012：** 失败的模式匹配 MUST 引发一个 Python 异常，其类型名称和消息标识为 Gandora 匹配错误。

### 模块与生成的 Python

**GEP-0001-R013：** 每个源文件 MUST 包含恰好一个 `defmodule`，其名称 MUST 等于文件路径相对于配置的源根目录的驼峰式渲染：路径分隔符变为点，每个 `snake_case` 段变为一个驼峰式模块段。示例：`src/app/hello_web.gan` 在源根目录 `src` 下 MUST 声明 `defmodule App.HelloWeb`。

**GEP-0001-R014：** Gandora 模块 `App.HelloWeb` MUST 编译为输出目录下的 `app/hello_web.py` 模块，每个 `def` 变为顶层 Python 函数。`defp` 函数编译为名称带单个前导下划线的函数。

**GEP-0001-R015：** Gandora 到 Python 的标识符映射 MUST 为单射，并作为公共兼容性契约：`?` 映射为尾随 `_p`，`!` 映射为尾随 `_bang`，任何在 Python 标识符中无效的其他字符映射为 `_u<hex>_`（其码点）。Python 标识符中有效的 Unicode 字母以 NFC 形式保留。映射后发生冲突 MUST 产生编译错误。

**GEP-0001-R016：** `@moduledoc` 和 `@doc` 字符串值 MUST 成为生成的模块和函数上的 Python 文档字符串。

**GEP-0001-R017：** 跨模块引用（`alias`、`import` 和完全限定的 `App.Mod.fun(...)` 调用）MUST 编译为对相应生成模块的普通 Python 导入。Gandora 模块之间的导入循环 MUST 产生编译错误。

### 项目配置

**GEP-0001-R018：** 项目 MUST 由最近的祖先 `gandora.jsonc` 文件配置。该文件允许注释和尾随逗号；重复键 MUST 被拒绝，未知顶层字段 MUST 产生诊断信息。

**GEP-0001-R019：** `gandora.jsonc` 识别以下字段：

- `source`：项目相对源根目录的有序数组，默认为 `["src"]`；
- `outDir`：项目相对输出目录，默认为 `"dist"`，始终从源发现中排除；
- `targetPython`：Python 版本字符串，默认为 `"3.12"`；
- `exclude`：从发现中移除的项目相对 glob 规则数组。

`pyproject.toml` 继续拥有包元数据和依赖项；两个文件 MUST NOT 重复彼此的关注点。

**GEP-0001-R020：** `gan run` MUST 编译到内部缓存目录 `.gandora/cache/` 而非 `outDir`，该缓存 MUST 可随时安全删除，且 MUST NOT 被发布。

### gan CLI

**GEP-0001-R021：** v0 CLI MUST 提供：

- `gan init [path]`：创建一个新的 `uv` 风格项目，包含 `gandora.jsonc`、`pyproject.toml`、`.python-version`、`.gitignore` 和 `src/main.gan`；`gan init --existing [path]` 将 Gandora 文件添加到现有项目，不覆盖任何已有内容；
- `gan check [file...]`：解析、展开并分析，不写入输出；
- `gan build`：编译所有发现的源文件到 `outDir`；
- `gan compile <file> [--out <dir>]`：编译显式指定的文件；
- `gan run <file> [args...]`：编译文件的整个项目，然后用项目的 Python 解释器执行 `<file>` 的生成模块，优先使用 `.venv/bin/python`，回退到 `uv run python`，将 `args` 作为 `sys.argv[1:]` 传递；
- `gan expand <file>`：在宏展开后以表层语法打印源文件；
- `gan --version` / `gan -V`：打印 `gan <semver>`。

**GEP-0001-R022：** 如果 `gan run` 的目标模块定义了 `main/0`，则生成的模块 MUST 在 `if __name__ == "__main__":` 下调用它。编译模块 MUST NOT 执行除 Python 模块级定义机制之外的用户代码。

**GEP-0001-R023：** 退出码：0 表示成功，1 表示编译或运行时失败，2 表示命令行误用。诊断信息 MUST 指明主作用域的文件、行和列，且 MUST 写入 stderr。

**GEP-0001-R024：** 生成的 Python MUST 是确定性的：使用相同的编译器版本和配置编译相同的源文件，会产生字节完全相同的输出。

## 理由

编译为可读的 Python 代码而非字节码或解释器，使得整个 Python 生态系统——调试器、性能分析器、`uv`、部署目标——无需 Gandora 专有支持即可使用，这与 Osiris 项目的经验一致：可读输出加上 `.venv` 兼容性是最便宜的集成方案。

单文件对应单模块的规则（不同于 Elixir 允许一个文件包含多个模块）实现了从模块名称到生成的 Python 模块之间直接、可预测的映射，进而简化了导入、工具链和增量编译。这是 Osiris 通过路径派生的模块名称所做的相同权衡。

原子作为驻留字符串，而非专用的原子类，使得互操作边界无需包装器：`:ok` 与 Python 库返回的 `"ok"` 相等，并且对 Python 数据的模式匹配无需修改即可工作。

Elixir 的真值性得以保留（只有 `false`/`nil` 为假），因为静默采用 Python 的真值性会改变普通 Elixir 风格代码（如 `if list do ... end`）的含义。

## 向后兼容性

这是创始语言提案；没有早期契约需要保留。标识符映射(R015)、数据类型映射(R009)以及模块到路径规则(R013–R014)是未来GEP必须遵守的兼容性表面。

## 安全性与确定性

编译器在编译期间 MUST NOT 执行用户 Python 代码、导入 Python 包或访问网络。宏展开是由 GEP-0002 定义的沙箱化、确定性的编译时求值。确定性输出 (R024) 使得构建可重现和可比较。

## 工具与AI使用

编写Gandora的AI代理在生成代码前应阅读本GEP，以了解表面清单（R006）及其排除项（R007），使用 `gan check` 进行验证，并使用 `gan expand` 检查宏输出。工具可以依赖单射名称映射（R015）来双向关联Gandora名称与生成的Python名称。

## 已否决的备选方案

### 将每个模块编译为 Python 类命名空间

允许每个文件包含多个模块，并生成一个类对应一个模块，这保留了 Elixir 的文件自由度，但生成的代码不够地道，每个调用点都会付出类属性间接访问的开销，而且模块/文件工具链（导入、覆盖率、堆栈跟踪）的质量会下降。因此选择了每个文件一个模块的方案。

### 专用的原子运行时类

一个真正的原子类型会让 `:ok` 与 `"ok"` 可区分，更接近 Elixir 语义，但每次互操作边界都需要转换，且对 Python 数据进行的模式匹配会失效。驻留字符串遵循了数据优先的理念。

### 用 Python 实现 CLI

用 Python 编写的 CLI 可以消除贡献者环境中的 Rust 工具链依赖，但编译速度、单一二进制分发以及独立于目标虚拟环境等优势超过了这种便利性。

## 开放性问题

v0 无；被排除的构造（R007）被有意推迟到未来的 GEP 中，而不是留下歧义。

## 一致性

当以下条件满足时，实现即为一致：`gan init && gan build && gan run` 在新项目上能正常工作；每个 R006 构造都能以指定的语义编译和运行；每个 R007 排除都会产生一个命名的诊断；生成的输出满足 R009、R013–R017 的映射；CLI 满足 R021–R024。仓库的测试套件将测试名称映射到这些需求标识符。

## 变更历史

- 修订版 1，2026-08-01：初始版本。引导接受记录由仓库的初始设计提交记录，而非外部决议 URL。

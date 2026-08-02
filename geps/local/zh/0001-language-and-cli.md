---
gep: 1
title: Gandora语言与gan CLI
description: Gandora 的核心身份，Elixir 风格的表面语法，编译到 Python，模块命名，项目配置，以及 gan 命令行。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - CLI
  - Configuration
created: 2026-08-01
updated: 2026-08-02
revision: 5
requires: [0]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0001-language-and-cli.md
source-revision: 5
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0001-language-and-cli.md](../../0001-language-and-cli.md)。

# GEP-0001: Gandora语言与gan CLI

## 摘要

Gandora 是一门采用 Elixir 风格语法的编程语言，可编译为可读的 Python 代码。本提案定义了语言标识、源文件格式、模块命名及其与生成 Python 的映射关系、项目配置文件 `gandora.jsonc`、与 `uv` 管理的 Python 项目的集成方式，以及 `gan` 命令行接口。

Gandora 有意采用了 Elixir 的表层语法：`defmodule`/`def` 配合 `do ... end` 块、原子（atom）、模式匹配、`|>` 管道符，以及基于 `defmacro` 的元编程。当 Elixir 通过 `:erlang` 调用访问其宿主平台时，Gandora 则通过 `$` 符号访问 Python：`$math`、`$json` 或任何已安装的模块均可直接调用，无需包装代码（GEP-0003）。

## Motivation

Python 拥有最大的库生态系统；Elixir 则具备最令人愉悦的函数式、管道导向编程界面之一，以及最优秀的宏系统之一。Gandora 将二者结合：开发者编写 Elixir 风格的模块与宏，而部署仍由 `uv` 管理的普通 Python 完成，生成的代码可读性足够高，以便使用标准 Python 工具进行审查、调试和性能分析。

需要一份精确的创始提案，以确保编译器、包集成以及未来工具共享同一份契约，并使得 AI 代理能够基于稳定的需求而非经验法则进行实现。

## 范围

本提案涵盖语言标识、源文件、表面语法清单、模块命名与 Python 名称映射、项目配置、`uv` 集成以及 `gan` CLI。宏系统由 GEP-0002 规定，Python 互操作约定由 GEP-0003 规定。标准库、格式化器、LSP 及包发布格式不在本提案范围内，需要由未来的 GEP 来规定。

## 术语

- **Source file**: 一个 UTF-8 文本文件，扩展名为 `.gan`。
- **Module**: 由 `defmodule` 声明的命名空间单元。
- **Project**: 由一个 `gandora.jsonc` 和一个 `pyproject.toml` 管理的目录树。
- **Generated module**: 为一个 Gandora 模块生成的 Python 文件。
- **Remote atom call**: `$module.function(args)` 形式，调用 Python（GEP-0003）。
- **Entry function**: `gan run` 使用的零参数或单参数 `main` 函数。

## 规范

### 标识

**GEP-0001-R001:** 语言 MUST 命名为 **Gandora**。编译器可执行文件 MUST 命名为 **`gan`**。源文件 MUST 使用扩展名 **`.gan`**。

**GEP-0001-R002:** 编译器 MUST 实现为一个原生可执行文件，其唯一的可执行构建产物是 Python 源代码。生成的 Python MUST NOT 在运行时需要 Gandora 编译器、Gandora 运行时包或任何导入钩子，超出项目声明的 Python 依赖。

**GEP-0001-R003:** 编译器的 Python 发行版 MUST 命名为 `gandora-lang`，并且 MUST 安装原生 `gan` 可执行文件。Python 依赖管理、虚拟环境和发布仍由标准 Python 工具链负责；默认的文档化工作流使用 `uv` 和标准的 `.venv` 布局。

**GEP-0001-R004:** 一次编译器调用 MUST 恰好针对一个 Python 版本，配置为 `targetPython`，默认为 `3.12`。生成的代码 MAY 使用目标版本中可用的任何语法，包括 `match` 语句。

### 表层语法

**GEP-0001-R005:** Gandora 表层语法 MUST 遵循 Elixir 的表层语法规则，用于该语言支持的语法结构，以便 Elixir 语法高亮和编辑器辅助功能保持可用。与 Elixir 的差异 MUST 记录在 GEP 中。

**GEP-0001-R006:** v0 表层 MUST 包含：

- `defmodule Name do ... end`，使用点分隔的驼峰式模块名；
- `def`、`defp`（私有），带有 `do ... end` 主体和关键字简写 `def f(x), do: expr`；
- `defmacro` 和 `quote`/`unquote`/`unquote_splicing`（GEP-0002）；
- 模块属性 `@moduledoc` 和 `@doc`，值为字符串；
- 整数（包括 `_` 分隔符）、浮点数、布尔值、`nil`、带有 `#{...}` 插值的字符串、原子（`:ok`、`:"quoted"`）、列表、元组（`{a, b}`）、映射（`%{"k" => v, a: 1}`）、关键字列表（`[a: 1, b: 2]`）、范围（`a..b`）；
- 操作符 `+ - * / == != < > <= >= and or not ++ <> |> = // rem div`，遵循 Elixir 优先级，其中 `//` 和 `div`/`rem` 遵循 Elixir 的整数除法语义；
- `if/else`、`unless`、`case`、`cond`、`fn args -> body end` 匿名函数和 `&Mod.fun/1` / `&(&1 + 1)` 捕获、`with` 用于链式匹配；
- 模式匹配，使用 `=`、在 `case` 子句中、在函数头中，包括字面量、变量、`_`、元组、列表（`[h | t]`）、映射和固定（`^x`）模式；
- 多子句函数定义，按模式从上到下调度，并可选使用 `when` 守卫；
- `alias`、`import` 和 `require`，带有 `as:`、`only:` 和 `except:` 选项，遵循 Elixir 语义；
- 远程 `$` 引用和 `expr.name`/`expr.name(args)` 后缀访问（GEP-0003）。

**GEP-0001-R007:** 在 v0 表层之外的语法结构 MUST 产生一个诊断，指出不支持的语法结构；编译器 MUST NOT 静默地错误翻译它未实现的 Elixir 语法。显著的 v0 排除项：协议、行为、`receive` 以及所有进程原语、`try/rescue`、二进制/位串（`<<>>`）和推导式（`for`）。结构体由 GEP-0004 指定，标记由 GEP-0005 指定。

**GEP-0001-R008:** 注释使用 `#` 到行尾。源文件 MUST 为 UTF-8 编码，标识符 MUST 遵循 Elixir 规则：函数和变量为 `snake_case`（允许 Unicode 字母），可选以 `?` 或 `!` 结尾；模块为由点连接的大驼峰段。

### 求值语义

**GEP-0001-R009:** Gandora 数据类型 MUST 映射到 Python 值，使得没有包装类型跨越互操作边界：

| Gandora | Python |
| --- | --- |
| 整数、浮点数、布尔值、字符串 | `int`、`float`、`bool`、`str` |
| `nil` | `None` |
| 原子 `:name` | 内联字符串 `"name"` |
| 列表 | `list` |
| 元组 | `tuple` |
| 映射 | `dict` |
| 关键字列表 | 由 2 元素元组组成的 `list` |
| 范围 `a..b` | 包含范围 `range(a, b + 1)` |
| 匿名函数 | Python 可调用对象 |

**GEP-0001-R010:** 原子编译为 Python 字符串字面量。`true`、`false` 和 `nil` 是 Python 单例，而非原子。相等性 `==` 是 Python 相等性。真值判断遵循 Elixir：只有 `false` 和 `nil` 为假，因此条件形式和布尔形式 MUST 编译为显式的 `is not falsy` 检查，而非 Python 的真值判断。

**GEP-0001-R011:** 变量是不可变绑定；在顺序体中重新绑定名称会创建一个新绑定，如同 Elixir 一样。`case`、`if` 或 `cond` 表达式返回其采纳分支的值。每个函数返回其最后一个表达式的值；编译器会插入显式的 `return` 语句。

**GEP-0001-R012:** 模式匹配失败 MUST 引发一个 Python 异常，其类型名称和消息标识为 Gandora 匹配错误。

### 模块与生成的 Python

**GEP-0001-R013:** 每个源文件 MUST 恰好包含一个 `defmodule`，且其名称 MUST 等于该文件相对于配置好的源根目录路径的大驼峰渲染：路径分隔符变为点，每个 `snake_case` 段变为一个大驼峰模块段。示例：在源根目录 `src` 下的 `src/app/hello_web.gan` 必须声明 `defmodule App.HelloWeb`。

**GEP-0001-R014:** Gandora 模块 `App.HelloWeb` MUST 编译为输出目录下的 Python 模块 `app/hello_web.py`，每个 `def` 作为顶层 Python 函数。`defp` 函数编译为名称带单个前导下划线的函数。

**GEP-0001-R015:** Gandora 到 Python 标识符映射 MUST 是单射的，并且是一个公开的兼容性契约：`?` 映射为尾随的 `_p`，`!` 映射为尾随的 `_bang`，任何其他在 Python 标识符中无效的字符映射为 `_u<十六进制>_`，其中十六进制表示其码点。在 Python 标识符中有效的 Unicode 字母以 NFC 形式保留。映射后发生冲突 MUST 是一个编译错误。

**GEP-0001-R016:** `@moduledoc` 和 `@doc` 字符串值 MUST 成为生成的模块和函数上的 Python 文档字符串。

**GEP-0001-R017:** 跨模块引用（`alias`、`import` 和完全限定调用 `App.Mod.fun(...)`）MUST 编译为对相应生成模块的普通 Python 导入。Gandora 模块之间的导入循环 MUST 是一个编译错误。

### 项目配置

**GEP-0001-R018:** 项目 MUST 由最近的祖先 `gandora.jsonc` 配置。该文件是 JSON，允许注释和尾随逗号；重复键 MUST 被拒绝，未知的顶层字段 MUST 产生诊断。

**GEP-0001-R019:** `gandora.jsonc` 识别的字段恰好为：

- `source`：有序数组，项目相对源根目录，默认为 `["src"]`；
- `outDir`：项目相对输出目录，默认为 `"dist"`，始终从源发现中排除；
- `targetPython`：Python 版本字符串，默认为 `"3.12"`；
- `exclude`：数组，项目相对 glob 规则，从发现中移除。

`pyproject.toml` 继续拥有包元数据和依赖项；两个文件 MUST NOT 重复对方的职责。

**GEP-0001-R020:** `gan run` MUST 编译到内部缓存目录 `.gandora/cache/` 而非 `outDir`，并且该缓存 MUST 可随时安全删除，MUST NOT 发布。

### gan CLI

**GEP-0001-R021:** v0 CLI MUST 提供：

- `gan init [path]`：创建一个新的 `uv` 风格项目，包含 `gandora.jsonc`、`pyproject.toml`、`.python-version`、`.gitignore` 和 `src/main.gan`；`gan init --existing [path]` 将 Gandora 文件添加到现有项目，不覆盖任何已有内容；
- `gan check [file...]`：解析、展开并分析，不写入输出；
- `gan build`：编译每一个发现的源文件到 `outDir`；
- `gan compile <file> [--out <dir>]`：编译显式指定的文件；
- `gan run <file> [args...]`：编译该文件所在项目，然后对 `<file>` 生成的模块使用项目的 Python 解释器执行，优先使用 `.venv/bin/python`，回退到 `uv run python`，将 `args` 作为 `sys.argv[1:]` 传递；
- `gan expand <file>`：宏展开后以表层语法打印源代码；
- `gan --version` / `gan -V`：打印 `gan <semver>`。

**GEP-0001-R022:** 如果 `gan run` 的目标模块定义了 `main/0`，则生成的模块 MUST 在 `if __name__ == "__main__":` 下调用它。编译模块 MUST NOT 执行用户代码，超出 Python 的模块级定义机制。

**GEP-0001-R023:** 退出码：0 表示成功，1 表示编译或运行时失败，2 表示命令行误用。诊断 MUST 包含主跨度所在文件的名称、行和列，并 MUST 写入 stderr。

**GEP-0001-R025:** 当 `|>` 的右侧以 `.` 开头时，该管道是方法管道：`x |> .name(args)` 求值为对管道值 `x` 的后缀调用 `x.name(args)`（GEP-0003-R004），并且可以跟随更多后缀段。这实现了将 Python 流式 API（pandas、numpy）与 Elixir 管道组合，无需中间绑定。

**GEP-0001-R026:** 三引号字符串（`"""`）是 heredoc，遵循 Elixir 语义：开头的换行符不属于值，且关闭分隔符所在行的空白缩进会从每个内容行的开头剥离。如果 heredoc 的内容从开头行开始，或者关闭分隔符不单独占一行，则按原样处理。这使得缩进的 heredoc（文档、用法文本、脚手架模板）在源代码中保持整洁，同时生成左对齐的值。

**GEP-0001-R024:** 生成的 Python MUST 是确定性的：使用相同的编译器版本和配置编译相同的源代码会产生字节相同的输出。

## 原理

编译为可读的 Python 而非字节码或解释器，使得整个 Python 生态系统——调试器、性能分析器、`uv`、部署目标——无需 Gandora 特定支持即可使用；可读输出加上 `.venv` 兼容性是最廉价的集成方案。

每个文件一个模块的规则（与允许一个文件包含多个模块的 Elixir 不同）带来了从模块名到生成的 Python 模块的直接、可预测的映射，从而简化了导入、工具链和增量编译。基于路径的模块名使这种映射成为机械过程。

原子作为内部字符串（而非专门的原子类）保持了互操作边界无需包装器：`:ok` 与 Python 库返回的 `"ok"` 相等，且对 Python 数据的模式匹配无需修改即可工作。

Elixir 真值被保留（只有 `false`/`nil` 为假），因为静默地采用 Python 真值会改变普通 Elixir 风格代码（如 `if list do ... end`）的含义。

## 向后兼容性

这是创始语言提案；没有需要保留的早期契约。标识符映射（R015）、数据类型映射（R009）以及模块到路径规则（R013–R014）是未来的GEPs MUST 尊重的兼容性表面。

## 安全性与确定性

编译器在编译期间 MUST NOT 执行用户 Python 代码、导入 Python 包或访问网络。宏展开是由 GEP-0002 定义的沙盒化、确定性的编译时求值过程。确定性输出 (R024) 使得构建可重现且可比较差异。

## 工具与AI使用

编写 Gandora 的 AI 代理在生成代码前应阅读本 GEP 以了解表面清单（R006）及其排除项（R007），使用 `gan check` 进行验证，并使用 `gan expand` 检查宏输出。工具可以依赖注入式名称映射（R015）来双向关联 Gandora 名称与生成的 Python 名称。

## 被拒绝的备选方案

### 将每个模块编译为 Python 类命名空间

允许每个文件包含多个模块并为每个模块生成一个类，这样保留了 Elixir 的文件自由度，但生成的代码不够惯用，每个调用点都要付出类属性间接访问的代价，并且模块/文件工具链（导入、覆盖率、堆栈跟踪）会退化。最终选择了每个文件对应一个模块的方式。

### 专用的原子运行时类

真正的原子类型能够使 `:ok` 与 `"ok"` 区分开来，更接近 Elixir 的语义，但每个互操作边界都需要进行转换，而且对 Python 数据进行模式匹配会失效。字符串驻留遵循了数据优先的哲学。

### 使用 Python 实现 CLI

用 Python 实现的 CLI 可以免去贡献者设置 Rust 工具链的麻烦，但编译速度、单一二进制分发以及独立于目标虚拟环境的优势超过了这种便利性。

## 未决问题

v0 阶段无未决问题；被排除的构造（R007）有意推迟到未来的 GEP 处理，而非留作模糊。

## Conformance

一个实现符合以下条件时：在一个新项目上 `gan init && gan build && gan run` 能正常工作；每个 R006 构造体都能按指定的语义编译和运行；每个 R007 排除项都会产生一个命名的诊断信息；生成的输出满足 R009, R013–R017 的映射要求；并且 CLI 满足 R021–R024。仓库的测试套件将测试名称映射到这些需求标识符。

## 变更历史

- 修订版 5，2026-08-02：互操作性引用遵循 GEP-0003 修订版 2（全文使用 `$module` 替代原子调用）。

- 修订版 4，2026-08-02：新增 R026，Elixir 的 heredoc 缩进语义。

- 修订版 3，2026-08-01：新增 R025，即 `|> .method(args)` 方法管道。

- 修订版 2，2026-08-01：R007 排除项列表更新——结构体和特殊符号现由 GEP-0004 和 GEP-0005 规定。

- 修订版 1，2026-08-01：初始版本。启动验收通过仓库的初始设计提交记录，而非外部解决方案 URL。

---
gep: 15
title: 语言服务器
description: gan-lsp — 诊断、文档悬停、定义、符号、补全和格式化，使用Gandora语言编写，基于pygls；以及gan-lsc，用于AI代理的同构JSON命令行。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 7
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0015-the-language-server.md
source-revision: 7
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0015-the-language-server.md](../../0015-the-language-server.md)。

# GEP-0015: 语言服务器

## 摘要

`gan-lsp` 是一个用 Gandora 编写的语言服务器协议实现：即一个 `gan lsp` 插件（GEP-0013-R003），其分发包为 `gandora-lsp`。版本 1 实现了协议生命周期和推送诊断——每次编辑都会通过 `gandora_core.diagnostics` 在内存中编译，错误/警告及其位置范围会显示在编辑器中。它是参考生态系统工具：一个基于 `loop`（GEP-0014）、`try/rescue`、互操作字节 I/O 以及编译器库构建的长时间运行的 Gandora 服务器。

## 动机

编辑器诊断是价值最高的语言服务，且无需任何符号基础设施——这正是当前编译器库的形态。在 Gandora 中编写服务器延续了 GEP-0012 计划：工具链是语言完备性的证明，而 LSP 是其首个长期运行进程。

## Scope

Protocol framing, lifecycle, diagnostics, documentation hover,
go-to-definition, document symbols, `Module.` completion, and
whole-document formatting. Rename and find-references await
span-range enrichment; they MUST reuse the same library queries.

## 规范

**GEP-0015-R001：** 服务器采用 Gandora 编写，基于 **pygls**：框架拥有协议机制（帧结构、JSON-RPC、类型化的 `lsprotocol` 结构），而 Gandora 模块则拥有语言逻辑，通过 `@decorate @server.feature(...)` 在作为模块属性持有的 `LanguageServer` 上附加处理器。该服务器以 `gandora-lsp` 包的形式分发，并暴露 `gan-lsp` 入口点，因此 `gan lsp` 通过插件委托到达它。pygls 仅是该包的依赖项——核心工具链保持依赖精简。

**GEP-0015-R001A：** 同一个包还暴露 `gan-lsc`——语言服务器控制台，面向 AI 的同构表面。每个查询在 stdout 上精确打印一个 JSON 值（引用的术语元组显示为 JSON 数组），并按照 GEP-0001-R023 退出。查询包括：`version`、`diagnostics <file>`、`ast <file>`、`expand <file>`、`compile <file>`、`resolve <module>`、`doc <Mod>[.<fun>]`、`definition <Mod>[.<fun>]`、`symbols <Mod>`，以及 Python 侧镜像 R009 的查询——`pydoc`、`pycomplete`、`pygoto`、`pysig`，它们作用于 `$` 风格的模块链——每个查询都接受 `--root <dir>`。`gan lsc ...` 通过相同的委托到达它；格式化则通过 `gan fmt` 完成。LSP 能力必须能够表示为 lsc 查询。

**GEP-0015-R002：** 版本 1 处理以下消息：`initialize`（宣布全文本同步）、`initialized`、`textDocument/didOpen`、`didChange`、`didClose`（清除诊断信息）、`shutdown` 和 `exit`。未知请求返回 MethodNotFound；未知通知被忽略。

**GEP-0015-R003：** 在打开和每次更改时，服务器运行 `gandora_core.diagnostics(text, path, root)`——root 来自工作区文件夹——并发布 `textDocument/publishDiagnostics`，将严重级别和基于 1 的编译器跨度映射到基于 0 的零长度 LSP 范围。引发异常的请求以错误响应（或对于通知进行日志记录）答复；服务器不得因错误输入而崩溃（GEP-0014 `try/rescue`）。

**GEP-0015-R005：** `textDocument/hover` 提供 GEP-0007 文档通道：光标下的引用（一个 `Module.function` 链、一个模块名、或针对文件自身模块解析的裸局部名称）通过 `gandora_core.doc` 查找，悬停信息渲染子句签名（包括渲染后的头部、守卫和默认值）、默认语言环境的说明文字、所有可用的翻译、元数据以及 `@example` 块（以 Markdown 形式呈现）。`$module` 引用显示模块的规范来源（从不导入该模块）；语言构造（`def`、`loop`、`quote`……）显示内嵌的一段落参考卡片。未文档化或标有 `@doc false` 的目标不产生悬停信息。

**GEP-0015-R006：** `textDocument/definition` 将相同的光标目标通过 `gandora_core.definition` 解析到定义源——模块的 `defmodule` 行、函数的第一个匹配子句——跨越项目源文件以及已安装包的已发布 `.gan` 源文件。

**GEP-0015-R007：** `textDocument/formatting` 对缓冲区运行 GEP-0016 引擎（`Fmt.format_text`），并返回一个完整的文档编辑，该编辑受到 GEP-0016-R006 验证的保护；无法格式化或内容未改变的缓冲区不产生任何编辑。

**GEP-0015-R008：** `textDocument/documentSymbol` 列出模块及其定义（渲染后的头部作为详细信息）；`textDocument/completion` 在模块路径后输入 `.` 时触发，通过 `gandora_core.symbols` 补全其公共函数和宏，附带签名和文档摘要。

**GEP-0015-R009：** 边界的 Python 侧是一等公民：对于 `$module` 引用和 `pyimport` 别名，悬停显示 Python 文档字符串，补全列表模块成员，定义跳转到 Python 源文件，签名帮助显示完整的 Python 签名——这些由项目自身环境中的 jedi 静态解析（服务器从不导入用户模块来回答查询）。

**GEP-0015-R010：** `textDocument/signatureHelp`（在输入 `(` 和 `,` 时触发）提供最内层开放调用：Gandora 目标显示每个子句头作为签名，带有每个参数的标签，`@spec` 行（GEP-0017）作为文档，活动参数按参数位置跟踪；Python 目标按照 R009 提供。

**GEP-0015-R004：** 仓库在 `editors/vscode` 中附带一个极简的 VS Code 客户端，它为 `.gan` 文件启动 `gan lsp`；该客户端不包含任何语言逻辑。

## 论证

使用 pygls 而非手写框架，是用包本地依赖换取 LSP 生态对协议细节的维护——第一版手写服务器证明了该语言能够实现；框架则让工具更廉价地成长。`lsc` 的存在使得代理能以纯 JSON 获得语言智能，而无需使用 JSON-RPC。

诊断优先既符合用户价值，也匹配库的成熟度；更丰富的特性将等待范围区间完善，而非发布半成品。插件方式意味着运行器无需 LSP 知识，任何支持 LSP 的编辑器只需要 `gan lsp` 命令。

## Backwards Compatibility

Additive; new distribution.

## 安全性与确定性

**GEP-0015-R012:** `textDocument/references` 返回光标下函数的每个项目调用点——定义模块或导入模块中的直接调用、别名解析后的 `Mod.fun` 调用、`&` 捕获以及（根据 `includeDeclaration`）子句头——范围覆盖精确的名称词元，包括字符串插值。`textDocument/rename` 应用与跨文件 `WorkspaceEdit` 相同的解析；它验证新名称，并拒绝定义位于项目之外的目标。`gan lsc references <Mod>.<fun>` 镜像原始调用点列表。

**GEP-0015-R013:** `workspace/symbol` 通过不区分大小写的子字符串搜索每个项目定义；`gan lsc wsymbols [query]` 镜像它，`gan lsc check` 镜像整个项目的诊断信息（包括编译器 lint）为一个 JSON 值。

**GEP-0015-R014:** 具有机械修复方法的编译器 lint 在其诊断信息上以快速修复代码操作的形式呈现：在定义上方插入 `@allow :stack_recursion` / `@allow :unused_function`（GEP-0019-R007，GEP-0022-R005），以及为未使用的绑定添加 `_` 前缀（GEP-0022-R002）。

服务器编译它接收到的缓冲区，但不执行任何操作。

## 工具与AI使用

编辑器使用 `gan lsp`。AI代理应优先直接使用 `gandora_core`；LSP 是为人类的编辑器而存在的。

## 被拒绝的备选方案

### 用 Rust 实现服务器

放弃了生态系统的证明；库边界的存在是为了使这个成为 Gandora 程序。

## 开放问题

本轮修订无。

## Conformance

Tests MUST cover a scripted stdio session: initialize handshake, a didOpen with an erroneous buffer producing publishDiagnostics with the right span, a didChange that fixes it producing an empty list, didClose clearing, and shutdown/exit terminating with code 0; the R012 references/rename round-trip; an R013 workspace-symbol query; and an R014 quick fix carrying the @allow edit.

## 变更历史

- 修订版 7，2026-08-03：R012 引用 + 重命名，R013 工作区符号 + `lsc check`，R014 代码检查快速修复。
- 修订版 6，2026-08-02：lsc 镜像完整能力集（R001A 已更新）：文档/定义/符号以及 pydoc/pycomplete/pygoto/pysig Python 查询。
- 修订版 5，2026-08-02：新增 R009（基于 jedi 的 Python 端悬停/补全/定义/签名）和 R010（签名帮助）；悬停显示 GEP-0017 规范。
- 修订版 4，2026-08-02：悬停获得签名、`$module` 和构造卡片（R005 扩展）；新增定义（R006）、通过 GEP-0016 的格式化（R007）以及符号/补全（R008），基于新的 `gandora_core.definition`/`symbols` API。
- 修订版 3，2026-08-02：新增 R005 — 在 `gandora_core.doc` 查找上的文档悬停（新的核心 API）。
- 修订版 2，2026-08-02：采用 pygls 处理协议机制；新增 gan-lsc 同构 JSON 控制台（R001A）。
- 修订版 1，2026-08-02：初始版本。

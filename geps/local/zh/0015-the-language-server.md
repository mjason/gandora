---
gep: 15
title: 语言服务器
description: gan-lsp — 诊断、文档悬停、定义、符号、补全和格式化，使用Gandora基于pygls编写；加上gan-lsc，即面向AI代理的同构JSON命令行。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 6
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0015-the-language-server.md
source-revision: 6
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0015-the-language-server.md](../../0015-the-language-server.md)。

# GEP-0015: 语言服务器

## 摘要

`gan-lsp` 是一个用 Gandora 编写的语言服务器协议实现：一个 `gan lsp` 插件（GEP-0013-R003），其发行版为 `gandora-lsp` 包。版本 1 实现了协议生命周期和推送诊断——每次编辑都会通过 `gandora_core.diagnostics` 在内存中编译，错误/警告及其范围会显示在编辑器中。它是参考生态系统工具：一个基于 `loop` (GEP-0014)、`try/rescue`、互操作字节 I/O 和编译器库构建的长期运行的 Gandora 服务器。

## Motivation

编辑器诊断是最高价值的语言服务，并且不需要符号基础设施——这正是当前编译器库的形态。用 Gandora 编写服务器延续了 GEP-0012 项目：工具链是语言完备性的证明，而 LSP 是它的第一个长期运行的进程。

## Scope

协议框架、生命周期、诊断、文档悬停、跳转到定义、文档符号、`Module.` 补全以及全文档格式化。重命名和查找引用等待范围区间增强；它们 MUST 重用相同的库查询。

## 规范

**GEP-0015-R001：** 服务器使用 Gandora 编写，基于 **pygls**：框架拥有协议机制（帧处理、JSON-RPC、类型化的 `lsprotocol` 结构），而 Gandora 模块拥有语言逻辑，通过在模块属性中持有的 `LanguageServer` 上使用 `@decorate @server.feature(...)` 来附加处理器。该服务器以 `gandora-lsp` 形式分发，暴露 `gan-lsp` 入口点，因此 `gan lsp` 通过插件委托到达它。pygls 仅是该包的一个依赖项——核心工具链保持依赖精简。

**GEP-0015-R001A：** 同一包暴露 `gan-lsc`——语言服务器控制台，即面向 AI 的同构表面。每个查询在标准输出上恰好打印一个 JSON 值（带引号的元组显示为 JSON 数组），并按照 GEP-0001-R023 退出。查询包括：`version`、`diagnostics <file>`、`ast <file>`、`expand <file>`、`compile <file>`、`resolve <module>`、`doc <Mod>[.<fun>]`、`definition <Mod>[.<fun>]`、`symbols <Mod>`，以及镜像 R009 的 Python 侧——`pydoc`、`pycomplete`、`pygoto`、`pysig`（使用 `$` 风格的模块链）——每个都接受 `--root <dir>`。`gan lsc ...` 通过相同的委托到达它；格式化使用 `gan fmt`。LSP 能力 MUST 保持可表达为 lsc 查询。

**GEP-0015-R002：** 版本 1 处理以下请求：`initialize`（宣告完整文本同步）、`initialized`、`textDocument/didOpen`、`didChange`、`didClose`（清除诊断信息）、`shutdown` 和 `exit`。未知请求返回 MethodNotFound；未知通知被忽略。

**GEP-0015-R003：** 在打开和每次更改时，服务器运行 `gandora_core.diagnostics(text, path, root)`（root 来自工作区文件夹），并发布 `textDocument/publishDiagnostics`，将严重级别和基于 1 的编译器跨度映射到基于 0 的零长度 LSP 范围。引发异常的请求以错误响应答复（对于通知则记录日志）；服务器 MUST NOT 因错误输入而崩溃（GEP-0014 `try/rescue`）。

**GEP-0015-R005：** `textDocument/hover` 提供 GEP-0007 文档通道：光标下的引用（一个 `Module.function` 链、模块名或针对文件自身模块解析的裸局部名称）通过 `gandora_core.doc` 查找，悬停显示子句签名（包括渲染头部、守卫和默认值）、默认语言环境的文本、每个可用的翻译、元数据以及 `@example` 块（以 Markdown 格式）。`$module` 引用显示模块的规范来源（从不导入它）；语言构造（`def`、`loop`、`quote` 等）显示嵌入的一段落参考卡片。未文档化或带有 `@doc false` 的目标不产生悬停内容。

**GEP-0015-R006：** `textDocument/definition` 通过 `gandora_core.definition` 将相同的光标目标解析到定义源——模块的 `defmodule` 行、函数的第一匹配子句——涉及项目源代码和已安装包附带的 `.gan` 源代码。

**GEP-0015-R007：** `textDocument/formatting` 对缓冲区运行 GEP-0016 引擎（`Fmt.format_text`），并返回一个整个文档的编辑，受相同 GEP-0016-R006 验证的保护；无法格式化或未更改的缓冲区不产生编辑。

**GEP-0015-R008：** `textDocument/documentSymbol` 列出模块及其定义（渲染头部作为细节）；`textDocument/completion` 在模块路径后的 `.` 上触发，完成其公共函数和宏，附带签名和文档摘要，通过 `gandora_core.symbols` 实现。

**GEP-0015-R009：** 边界上的 Python 侧是一等公民：对于 `$module` 引用和 `pyimport` 别名，悬停显示 Python 文档字符串，补全列出模块成员，定义跳转到 Python 源代码，签名帮助显示完整的 Python 签名——由 jedi 在项目自身环境中静态解析（服务器从不导入用户模块来回答查询）。

**GEP-0015-R010：** `textDocument/signatureHelp`（在 `(` 和 `,` 上触发）提供最内层打开的调用：Gandora 目标将每个子句头显示为带每参数标签的签名，`@spec` 行（GEP-0017）作为文档，活动参数通过参数位置跟踪；Python 目标按照 R009 提供。

**GEP-0015-R004：** 仓库在 `editors/vscode` 中附带一个最小的 VS Code 客户端，它为 `.gan` 文件生成 `gan lsp`；它不包含语言逻辑。

## 原理说明

使用 pygls 而非手写协议帧，是以一个包级依赖换取 LSP 生态系统对协议细节的维护代价——修订版 1 的手写服务器证明了语言能胜任此事；而框架降低了工具扩展的成本。`lsc` 的存在使得代理能够以纯 JSON 形式获取语言智能，而无需使用 JSON-RPC。

以诊断信息为先既符合用户价值，也匹配了库的成熟度；更丰富的特性应等待范围跨度完善，而非交付半成品。插件化路径意味着运行器无需了解 LSP，而任何支持 LSP 的编辑器只需执行 `gan lsp` 命令即可。

## 向后兼容性

增量式；新的发行版。

## Security and Determinism

服务器编译它接收到的缓冲区，但不执行任何操作。

## 工具与AI使用

编辑器使用 `gan lsp`。AI 代理应直接优先使用 `gandora_core`；LSP 是为人类编辑器而存在的。

## 被拒绝的替代方案

### 用 Rust 实现服务器

放弃生态系统证明；库边界的存在是为了使其成为 Gandora 程序。

## 开放问题

此修订版无开放问题。

## 一致性

测试 MUST 覆盖一个脚本化的 stdio 会话：初始化握手，一个带有错误缓冲区的 didOpen，产生带有正确 span 的 publishDiagnostics；一个修复该问题的 didChange，产生空列表；didClose 清除；以及 shutdown/exit 以退出码 0 终止。

## 变更历史

- Revision 6, 2026-08-02: lsc 反映完整能力集（R001A 已更新）：doc/definition/symbols 以及 pydoc/pycomplete/pygoto/pysig Python 查询。

- Revision 5, 2026-08-02: 新增 R009（基于 jedi 的 Python 端悬停/补全/定义/签名）和 R010（签名帮助）；悬停显示 GEP-0017 规范。

- Revision 4, 2026-08-02: 悬停获得签名、`$module` 和构造卡片（R005 扩展）；新增定义（R006）、通过 GEP-0016 实现格式化（R007），以及符号/补全（R008），基于新的 `gandora_core.definition`/`symbols` API。

- Revision 3, 2026-08-02: 新增 R005——通过 `gandora_core.doc` 查找实现文档悬停（新的核心 API）。

- Revision 2, 2026-08-02: 采用 pygls 作为协议机制；新增 gan-lsc 同构 JSON 控制台（R001A）。

- Revision 1, 2026-08-02: 初始版本。

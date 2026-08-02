---
gep: 15
title: 语言服务器
description: gan-lsp — 诊断、文档悬停、定义、符号、补全和格式化，用Gandora编写，基于pygls；外加gan-lsc，面向AI代理的同构JSON命令行。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 5
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0015-the-language-server.md
source-revision: 5
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0015-the-language-server.md](../../0015-the-language-server.md)。

# GEP-0015: 语言服务器

## 摘要

`gan-lsp` 是一个用 Gandora 编写的语言服务器协议实现：一个 `gan lsp` 插件（GEP-0013-R003），其分发包为 `gandora-lsp` 包。版本 1 实现了协议生命周期和推送诊断——每次编辑都会通过 `gandora_core.diagnostics` 在内存中编译，错误/警告出现在编辑器中并附带其代码跨度。它是生态系统的参考工具：一个长期运行的 Gandora 服务器，构建在 `loop`（GEP-0014）、`try/rescue`、互操作字节 I/O 以及编译器库之上。

## 动机

编辑器诊断是最高价值的语言服务，且不需要任何符号基础设施——这正是编译器库当前的形态。用 Gandora 编写服务器延续了 GEP-0012 计划：工具链是该语言完备性的证明，而 LSP 是它的第一个长时运行进程。

## 范围

协议框架、生命周期、诊断、文档悬停、转到定义、文档符号、`Module.` 完成以及整个文档格式化。重命名和查找引用等待跨度范围扩展；它们 MUST 重用相同的库查询。

## Specification

**GEP-0015-R001:** 服务器使用 Gandora 在 **pygls** 上编写：框架负责协议机制（帧封装、JSON-RPC、类型化的 `lsprotocol` 结构），Gandora 模块负责语言逻辑，通过 `@decorate @server.feature(...)` 在模块属性中持有的 `LanguageServer` 上附加处理程序。它作为 `gandora-lsp` 分发，暴露 `gan-lsp` 入口点，因此 `gan lsp` 通过插件委托到达它。pygls 仅是这个包的依赖项——核心工具链保持依赖精简。

**GEP-0015-R001A:** 同一个包暴露 `gan-lsc`——语言服务器控制台，面向 AI 的同构表面。每个查询在 stdout 上精确打印一个 JSON 值（带引号的术语元组作为 JSON 数组）并按照 GEP-0001-R023 退出。版本 2 查询：`version`、`diagnostics <file>`、`ast <file>`、`expand <file>`、`compile <file>`、`resolve <module>`，每个都接受 `--root <dir>`。`gan lsc ...` 通过相同的委托到达它。LSP 能力 MUST 能够表达为 lsc 查询。

**GEP-0015-R002:** 版本 1 处理：`initialize`（宣布全文本同步）、`initialized`、`textDocument/didOpen`、`didChange`、`didClose`（清除诊断）、`shutdown` 和 `exit`。未知请求返回 MethodNotFound；未知通知被忽略。

**GEP-0015-R003:** 在打开和每次更改时，服务器运行 `gandora_core.diagnostics(text, path, root)`——root 来自工作区文件夹——并发布 `textDocument/publishDiagnostics`，将严重性和基于 1 的编译器跨度映射到基于 0 的零长度 LSP 范围。引发异常的请求以错误响应回答（或对于通知，记录日志）；服务器 MUST NOT 因错误输入而崩溃（GEP-0014 `try/rescue`）。

**GEP-0015-R005:** `textDocument/hover` 提供 GEP-0007 文档通道：光标下的引用（一个 `Module.function` 链、一个模块名称、或一个解析为文件自身模块的裸局部名称）通过 `gandora_core.doc` 查找，hover 渲染子句签名（包括渲染的头部、守卫和默认值）、默认语言文档、所有可用翻译、元数据和 `@example` 块作为 Markdown。`$module` 引用显示模块的规范来源（从不导入它）；语言构造（`def`、`loop`、`quote`、...）显示嵌入的段落参考卡片。未文档化或 `@doc false` 的目标不产生 hover。

**GEP-0015-R006:** `textDocument/definition` 通过 `gandora_core.definition` 将相同的光标目标解析到定义源——模块的 `defmodule` 行，函数的第一个匹配子句——跨项目源和已安装包的 `.gan` 源文件。

**GEP-0015-R007:** `textDocument/formatting` 对缓冲区运行 GEP-0016 引擎（`Fmt.format_text`）并返回一个全文档编辑，受相同的 GEP-0016-R006 验证保护；不可格式化或未更改的缓冲区不产生编辑。

**GEP-0015-R008:** `textDocument/documentSymbol` 列出模块及其定义（渲染的头部作为细节）；`textDocument/completion` 在 `.` 后触发，当模块路径完成后，通过 `gandora_core.symbols` 列出其公共函数和宏及其签名和文档摘要。

**GEP-0015-R009:** 边界的 Python 侧是一等公民：对于 `$module` 引用和 `pyimport` 别名，hover 显示 Python 文档字符串，完成列表模块成员，定义跳转到 Python 源文件，签名帮助显示完整的 Python 签名——通过 jedi 在项目自身的环境中静态解析（服务器从不导入用户模块来回答查询）。

**GEP-0015-R010:** `textDocument/signatureHelp`（在 `(` 和 `,` 上触发）提供最内层打开的调用：Gandora 目标显示每个子句头部作为签名，带有每个参数标签，`@spec` 行（GEP-0017）作为文档，活动参数通过参数位置跟踪；Python 目标按照 R009 提供。

**GEP-0015-R004:** 仓库在 `editors/vscode` 中附带一个最小的 VS Code 客户端，它为 `.gan` 文件启动 `gan lsp`；它不包含任何语言逻辑。

## 理由

使用 pygls 而非手写框架，以包本地依赖换取 LSP 生态系统对协议细节的维护——第一版手写服务器证明了该语言能够实现这一点；框架使得工具更易于发展。`lsc` 的存在使得代理程序能够以纯 JSON 形式获取语言智能，而无需使用 JSON-RPC。

诊断优先既符合用户价值也符合库的成熟度；更丰富的功能等待范围跨度，而不是发布半成品。插件方式意味着运行器无需了解 LSP，任何支持 LSP 的编辑器只需使用 `gan lsp` 命令。

## 向后兼容性

添加性的；新的分发方式。

## 安全性与确定性

服务器编译其接收到的缓冲区，且不执行任何内容。

## 工具与 AI 使用

编辑器使用 `gan lsp`。AI 代理应直接选用 `gandora_core`；LSP 是为人类编辑器而存在的。

## 被拒绝的备选方案

### 用 Rust 实现服务器

放弃了生态系统证明；库边界的存在是为了使这成为一个 Gandora 程序。

## 待解决的问题

本次修订无待解决问题。

## 符合性

测试必须涵盖一个脚本化的 stdio 会话：初始化握手、一个带有错误缓冲区的 didOpen 产生具有正确范围的 publishDiagnostics、一个修复它的 didChange 产生空列表、didClose 清除，以及 shutdown/exit 以退出码 0 终止。

## 变更历史

- 修订版 5，2026-08-02：新增 R009（基于 jedi 的 Python 侧悬停/补全/定义/签名）和 R010（签名帮助）；悬停显示 GEP-0017 规范。

- 修订版 4，2026-08-02：悬停获得了签名、`$module` 和构造卡片（R005 扩展）；新增定义（R006）、通过 GEP-0016 的格式化（R007）以及符号/补全（R008），基于新的 `gandora_core.definition`/`symbols` API。

- 修订版 3，2026-08-02：新增 R005——在 `gandora_core.doc` 查找之上提供文档悬停（新的核心 API）。

- 修订版 2，2026-08-02：采用 pygls 作为协议机制；添加了 gan-lsc 同构 JSON 控制台（R001A）。
- 修订版 1，2026-08-02：初始版本。

---
gep: 15
title: 语言服务器
description: gan-lsp — 诊断、文档悬停、定义、符号、完成和格式化，基于pygls用Gandora编写；以及gan-lsc，面向AI代理的同构JSON命令行。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 4
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0015-the-language-server.md
source-revision: 4
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0015-the-language-server.md](../../0015-the-language-server.md)。

# GEP-0015: 语言服务器

## 摘要

`gan-lsp` 是一个用 Gandora 编写的语言服务器协议实现：它是一个 `gan lsp` 插件（GEP-0013-R003），其分发形式为 `gandora-lsp` 包。版本 1 实现了协议生命周期和推送诊断——每次编辑都会通过 `gandora_core.diagnostics` 在内存中编译，错误/警告及其跨度会出现在编辑器中。它是参考生态工具：一个基于 `loop`（GEP-0014）、`try/rescue`、互操作字节 I/O 和编译器库构建的长期运行 Gandora 服务器。

## 动机

编辑器诊断是最高价值的语言服务，且无需任何符号基础设施——这正是当前编译器库的形态。用 Gandora 编写服务器延续了 GEP-0012 计划：工具链是语言完备性的证明，而 LSP 是它的第一个长期运行进程。

## 范围

协议框架、生命周期、诊断、文档悬停、转到定义、文档符号、`Module.` 补全以及全文格式化。重命名和查找引用等待 span-range 丰富化；它们 MUST 重用相同的库查询。

## 规范

**GEP-0015-R001:** 服务器使用 Gandora 编写，基于 **pygls**：框架拥有协议机制（帧、JSON-RPC、类型化的 `lsprotocol` 结构），Gandora 模块拥有语言逻辑，通过 `@decorate @server.feature(...)` 在模块属性中持有的 `LanguageServer` 上附加处理程序。它作为 `gandora-lsp` 分发，暴露 `gan-lsp` 入口点，因此 `gan lsp` 通过插件委托到达它。pygls 仅是该包的依赖——核心工具链保持依赖精简。

**GEP-0015-R001A:** 同一包暴露了 `gan-lsc` —— 语言服务器控制台，面向 AI 的同构界面。每个查询在 stdout 上恰好打印一个 JSON 值（引号项元组显示为 JSON 数组），并按 GEP-0001-R023 退出。版本 2 查询：`version`、`diagnostics <file>`、`ast <file>`、`expand <file>`、`compile <file>`、`resolve <module>`，每个都接受 `--root <dir>`。`gan lsc ...` 通过相同的委托到达它。LSP 能力 MUST 仍可表示为 lsc 查询。

**GEP-0015-R002:** 版本 1 处理：`initialize`（声明完整文本同步）、`initialized`、`textDocument/didOpen`、`didChange`、`didClose`（清除诊断）、`shutdown` 和 `exit`。未知请求返回 MethodNotFound；未知通知被忽略。

**GEP-0015-R003:** 在打开和每次更改时，服务器运行 `gandora_core.diagnostics(text, path, root)` —— root 来自工作区文件夹 —— 并发布 `textDocument/publishDiagnostics`，将严重性和基于 1 的编译器跨度映射到基于 0 的零长度 LSP 范围。引发异常的请求以错误响应回答（或记录日志，对于通知）；服务器 MUST NOT 因错误输入而崩溃（GEP-0014 `try/rescue`）。

**GEP-0015-R005:** `textDocument/hover` 提供 GEP-0007 文档通道：光标下的引用（`Module.function` 链、模块名或解析为文件自身模块的裸局部名称）通过 `gandora_core.doc` 查找，hover 显示子句签名（包括渲染的头部、守卫和默认值）、默认语言环境的散文、每个可用的翻译、元数据和 `@example` 块作为 Markdown。`$module` 引用显示模块的规范来源（从不导入它）；语言构造（`def`、`loop`、`quote`、...）显示嵌入的单段参考卡片。未文档化或 `@doc false` 的目标不产生 hover。

**GEP-0015-R006:** `textDocument/definition` 通过 `gandora_core.definition` 将相同的光标目标解析到定义源 —— 模块的 `defmodule` 行，函数的第一个匹配子句 —— 跨项目源和已安装包附带的 `.gan` 源。

**GEP-0015-R007:** `textDocument/formatting` 在缓冲区上运行 GEP-0016 引擎（`Fmt.format_text`）并返回一个全文档编辑，受相同的 GEP-0016-R006 验证保护；不可格式化或未更改的缓冲区不产生任何编辑。

**GEP-0015-R008:** `textDocument/documentSymbol` 列出模块及其定义（渲染头部作为详情）；`textDocument/completion` 在模块路径后输入`.`时触发，通过 `gandora_core.symbols` 完成其公共函数和宏，并附带签名和文档摘要。

**GEP-0015-R004:** 仓库在 `editors/vscode` 中附带了一个最小的 VS Code 客户端，它为 `.gan` 文件启动 `gan lsp`；它不包含任何语言逻辑。

## 理由

选择 pygls 而非手工实现的帧协议，是用一个包内依赖换取 LSP 生态系统对协议细节的维护——第一版手工实现的服务器已证明语言能够胜任；框架使得工具的扩展成本更低。`lsc` 的存在使得智能体能够以纯 JSON 形式获取语言智能，而无需使用 JSON-RPC。

诊断优先既符合用户价值，也符合库的成熟度；更丰富的功能等待范围跨度，而不是发布半成品。插件路线意味着运行器无需了解 LSP，任何支持 LSP 的编辑器只需 `gan lsp` 命令即可。

## 向后兼容性

增量的；新的分发。

## 安全性与确定性

服务器编译其接收到的缓冲区，但不执行任何操作。

## 工具与 AI 使用

编辑器使用 `gan lsp`。AI 代理应直接偏好 `gandora_core`；LSP 是为人类的编辑器存在的。

## 被否决的备选方案

### 用 Rust 实现服务器

放弃了生态系统证明；库边界的存在是为了使这成为一个 Gandora 程序。

## 开放问题

本次修订无。

## 符合性

测试 MUST 涵盖一个脚本化的 stdio 会话：初始化握手、一个包含错误缓冲区的 didOpen 产生带有正确范围（span）的 publishDiagnostics、一个修复错误的 didChange 产生空列表、didClose 清除，以及 shutdown/exit 以退出码 0 终止。

## 变更历史

- 修订版 4，2026-08-02：悬停获得签名、`$module` 和构造卡片（R005 扩展）；新增定义（R006）、通过 GEP-0016 的格式化（R007）以及符号/补全（R008），基于新的 `gandora_core.definition`/`symbols` API。

- 修订版 3，2026-08-02：新增 R005 —— 通过 `gandora_core.doc` 查找（新的核心 API）实现文档悬停。

- 修订版 2，2026-08-02：采用 pygls 作为协议机制；新增 gan-lsc 同构 JSON 控制台（R001A）。
- 修订版 1，2026-08-02：初始版本。

---
gep: 15
title: 语言服务器
description: gan-lsp — 一个基于pygls用Gandora编写的LSP服务器，加上gan-lsc，即面向AI代理的同构JSON命令行；两者都在一个gan-plugin包中。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 3
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0015-the-language-server.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0015-the-language-server.md](../../0015-the-language-server.md)。

# GEP-0015: 语言服务器

## 摘要

`gan-lsp` 是一个用 Gandora 编写的语言服务器协议实现：它是一个 `gan lsp` 插件（GEP-0013-R003），其发行包名为 `gandora-lsp`。版本 1 实现了协议生命周期和推送诊断——每次编辑都会通过 `gandora_core.diagnostics` 在内存中编译，编辑器中会显示错误/警告及其跨度。它是参考生态系统工具：一个基于 `loop`（GEP-0014）、`try/rescue`、互操作字节 I/O 以及编译器库构建的长期运行的 Gandora 服务器。

## 动机

编辑器诊断是最高价值的语言服务，并且不需要任何符号基础设施——这正是当前编译器库的形态。用 Gandora 编写服务器延续了 GEP-0012 项目：工具链是语言完备性的证明，而 LSP 是其第一个长期运行的进程。

## 范围

协议框架、生命周期、诊断和文档悬停。
补全、定义和符号是后续修订，待跨度范围增强功能落地后实现；它们 MUST 复用相同的库查询。

## 规范

**GEP-0015-R001：** 服务器使用 **pygls** 以 Gandora 编写：该框架拥有协议机制（组帧、JSON-RPC、类型化的 `lsprotocol` 结构），Gandora 模块拥有语言逻辑，通过 `@decorate @server.feature(...)` 在模块属性持有的 `LanguageServer` 上附加处理程序。它作为 `gandora-lsp` 分发，暴露 `gan-lsp` 入口点，因此 `gan lsp` 通过插件委托到达它。pygls 仅是该包的依赖项——核心工具链保持依赖精简。

**GEP-0015-R001A：** 同一包暴露 `gan-lsc`——语言服务器控制台，面向 AI 的同构表面。每个查询在 stdout 上精确打印一个 JSON 值（带引号的术语元组显示为 JSON 数组），并按照 GEP-0001-R023 退出。版本 2 查询包括：`version`、`diagnostics <file>`、`ast <file>`、`expand <file>`、`compile <file>`、`resolve <module>`，每个均接受 `--root <dir>`。`gan lsc ...` 通过相同的委托到达它。LSP 能力 MUST 保持可表达为 lsc 查询。

**GEP-0015-R002：** 版本 1 处理：`initialize`（宣布完整文本同步）、`initialized`、`textDocument/didOpen`、`didChange`、`didClose`（清除诊断）、`shutdown` 和 `exit`。未知请求收到 MethodNotFound；未知通知被忽略。

**GEP-0015-R003：** 在打开及每次更改时，服务器运行 `gandora_core.diagnostics(text, path, root)` —— root 来自工作区文件夹——并发布 `textDocument/publishDiagnostics`，将严重性和基于 1 的编译器范围映射到基于 0 的零长度 LSP 范围。引发异常的请求会被应答为错误响应（或针对通知记录日志）；服务器 MUST NOT 因错误输入而崩溃（GEP-0014 `try/rescue`）。

**GEP-0015-R005：** `textDocument/hover` 提供 GEP-0007 文档通道：光标下的引用（`Module.function` 链、模块名或针对文件自身模块解析的裸局部名称）通过 `gandora_core.doc` 查找，悬停渲染默认语言环境的散文、每个可用的翻译、元数据和 `@example` 块作为 Markdown。无文档或 `@doc false` 的目标不产生悬停。

**GEP-0015-R004：** 仓库在 `editors/vscode` 中附带一个极简的 VS Code 客户端，为 `.gan` 文件生成 `gan lsp`；它不包含语言逻辑。

## 原理说明

pygls 相比于手动构建的框架，用包内依赖换来了 LSP 生态对协议细节的维护——修订版1的手动构建服务器证明了该语言能够做到这一点；而框架使工具的增长成本更低。`lsc` 的存在使得智能体能够以纯 JSON 形式获取语言智能，而无需使用 JSON-RPC 协议。

诊断优先的策略既符合用户价值，也匹配库的成熟度；更丰富的功能有待范围跨度来实现，而非交付半正确的方案。插件路线意味着运行器无需了解 LSP，而任何使用 LSP 的编辑器只需 `gan lsp` 命令即可。

## 向后兼容性

附加性；新分发。

## 安全性与确定性

服务器编译其所接收的缓冲区，并且不执行任何内容。

## 工具与 AI 使用

编辑器使用 `gan lsp`。AI 代理应直接优先使用 `gandora_core`；LSP 是为人类编辑器而存在的。

## 被拒绝的方案

### 用 Rust 实现服务器

放弃了生态证明；库边界的存在是为了使其成为 Gandora 程序。

## 开放问题

此版本无。

## 合规性

测试MUST涵盖一个脚本化的stdio会话：初始化握手、一个带有错误缓冲区的didOpen（产生带有正确范围的publishDiagnostics）、一个修复错误的didChange（产生空列表）、didClose清理，以及shutdown/exit以代码0终止。

## 变更历史

- 修订版 3，2026-08-02：新增 R005 — 对 `gandora_core.doc` 查找（新核心 API）的文档悬停支持。

- 修订版 2，2026-08-02：采用 pygls 实现协议机制；新增 gan-lsc 同构 JSON 控制台（R001A）。
- 修订版 1，2026-08-02：初始版本。

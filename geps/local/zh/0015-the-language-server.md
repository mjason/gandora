---
gep: 15
title: 语言服务器
description: gan-lsp — 一个基于 pygls 使用 Gandora 编写的 LSP 服务器，加上 gan-lsc，即面向 AI 代理的同构 JSON 命令行工具；两者都在同一个 gan-plugin 包中。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 2
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0015-the-language-server.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0015-the-language-server.md](../../0015-the-language-server.md)。

# GEP-0015：语言服务器

## 摘要

`gan-lsp` 是一个用 Gandora 编写的语言服务器协议实现：它是一个 `gan lsp` 插件（GEP-0013-R003），其发行包名为 `gandora-lsp`。版本 1 实现了协议生命周期和推送诊断——每次编辑都通过 `gandora_core.diagnostics` 在内存中编译，编辑器中的错误/警告及其位置跨度将显示出来。它是参考生态工具：基于 `loop`（GEP-0014）、`try/rescue`、互操作字节 I/O 以及编译器库构建的长时间运行 Gandora 服务器。

## 动机

编辑器诊断是最高价值的语言服务，且不需要符号基础设施——这正是当前编译器库的形态。用 Gandora 编写服务器延续了 GEP-0012 计划：工具链是语言完备性的证明，而 LSP 是其首个长期运行进程。

## 范围

协议帧、生命周期和诊断。悬停、补全、定义和符号是后续版本的内容，待跨度范围增强功能落地后；它们 MUST 复用相同的库查询。

## 规范

**GEP-0015-R001:** 服务器使用Gandora编写，基于**pygls**：框架拥有协议机制（帧、JSON-RPC、类型化的`lsprotocol`结构），Gandora模块拥有语言逻辑，通过`@decorate @server.feature(...)`在模块属性中持有的`LanguageServer`上附加处理程序。它以`gandora-lsp`形式分发，暴露`gan-lsp`入口点，因此`gan lsp`通过插件委托到达它。pygls仅是这个包的依赖项——核心工具链保持依赖精简。

**GEP-0015-R001A:** 同一包暴露了`gan-lsc`——语言服务器控制台，面向AI的同构表面。每个查询在stdout上恰好打印一个JSON值（引号术语元组显示为JSON数组），并根据GEP-0001-R023退出。版本2查询：`version`、`diagnostics <file>`、`ast <file>`、`expand <file>`、`compile <file>`、`resolve <module>`，每个接受`--root <dir>`。`gan lsc ...`通过相同的委托到达它。LSP能力 **MUST** 必须能够表示为lsc查询。

**GEP-0015-R002:** 版本1处理：`initialize`（宣布全文本同步）、`initialized`、`textDocument/didOpen`、`didChange`、`didClose`（清除诊断）、`shutdown`和`exit`。未知请求收到MethodNotFound；未知通知被忽略。

**GEP-0015-R003:** 在打开和每次更改时，服务器运行`gandora_core.diagnostics(text, path, root)`——根来自工作区文件夹——并发布`textDocument/publishDiagnostics`，将严重性和基于1的编译器跨度映射到基于0的零长度LSP范围。引发异常的请求会以错误响应（或针对通知记录日志）回答；服务器 **MUST NOT** 不能因错误输入而崩溃（GEP-0014 `try/rescue`）。

**GEP-0015-R004:** 仓库在`editors/vscode`中提供了一个最小的VS Code客户端，它为`.gan`文件生成`gan lsp`；它不包含语言逻辑。

## 原理

使用 pygls 而非手动构建的框架，用一个包级依赖换取 LSP 生态对协议细节的维护——版本 1 手动构建的服务器证明了该语言能够做到；框架使得工具更易于扩展。`lsc` 的存在使得代理能够以纯 JSON 形式获取语言智能，而无需使用 JSON-RPC 通信。

诊断优先符合用户价值与库的成熟度；更丰富的特性等待范围跨度，而不是发布半成品。插件路线意味着运行器不需要了解 LSP，任何支持 LSP 的编辑器只需使用 `gan lsp` 命令。

## 向后兼容性

增量式；新分发。

## 安全性与确定性

服务器编译其接收到的缓冲区，但不执行任何内容。

## 工具与AI使用

编辑器使用`gan lsp`。AI代理应优先直接使用`gandora_core`；LSP是为人类编辑器而存在的。

## 被否决的替代方案

### 用Rust实现服务器

放弃了生态证明；库边界的存在是为了使其成为Gandora程序。

## 开放问题

本修订版无任何开放问题。

## Conformance

Tests MUST cover a scripted stdio session: initialize handshake, a
didOpen with an erroneous buffer producing publishDiagnostics with
the right span, a didChange that fixes it producing an empty list,
didClose clearing, and shutdown/exit terminating with code 0.

## 变更历史

- 修订版 2，2026-08-02：采用 pygls 作为协议机制；新增了 gan-lsc 同构 JSON 控制台（R001A）。
- 修订版 1，2026-08-02：初始版本。

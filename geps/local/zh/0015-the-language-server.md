---
gep: 15
title: 语言服务器
description: gan-lsp — 用Gandora编写的LSP服务器，作为gan插件，发布编译器诊断信息；基于gandora-core构建的第一个生态系统工具。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0015-the-language-server.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0015-the-language-server.md](../../0015-the-language-server.md)。

# GEP-0015: 语言服务器

## 摘要

`gan-lsp` 是一个用 Gandora 编写的语言服务器协议实现：一个 `gan lsp` 插件（GEP-0013-R003），其分发包为 `gandora-lsp` 包。版本 1 实现了协议生命周期和推送诊断——每次编辑都通过 `gandora_core.diagnostics` 在内存中编译，错误和警告及其跨度会出现在编辑器中。它是参考生态工具：一个基于 `loop`（GEP-0014）、`try/rescue`、互操作字节 I/O 和编译器库构建的长期运行的 Gandora 服务器。

## 动机

编辑器诊断是最高价值的语言服务，不需要任何符号基础设施——这正是当前编译器库的形态。用 Gandora 编写服务器延续了 GEP-0012 计划：工具链是语言完备性的证明，而 LSP 是其第一个长期运行进程。

## Scope

协议框架、生命周期和诊断。悬停（Hover）、补全（Completion）、定义（Definition）和符号（Symbols）是后续版本的内容，一旦 span-range enrichment 落地，它们 MUST 重用相同的库查询。

## 规范

**GEP-0015-R001:** 服务器通过标准输入输出（stdio）使用LSP协议，采用`Content-Length`帧格式，在Gandora中通过互操作字节I/O实现。它作为`gandora-lsp`分发，并暴露`gan-lsp`入口点，因此`gan lsp`通过插件委托到达该入口点。

**GEP-0015-R002:** 版本1处理以下请求：`initialize`（宣布全文本同步）、`initialized`、`textDocument/didOpen`、`didChange`、`didClose`（清除诊断）、`shutdown`和`exit`。未知请求收到MethodNotFound；未知通知被忽略。

**GEP-0015-R003:** 在打开和每次更改时，服务器运行`gandora_core.diagnostics(text, path, root)`——其中root来自工作区文件夹——并发布`textDocument/publishDiagnostics`，将严重级别和基于1的编译器跨度映射到基于0的零长度LSP范围。引发异常的请求会以错误响应（或对于通知则记录日志）回答；服务器MUST NOT因错误输入而崩溃（GEP-0014 `try/rescue`）。

**GEP-0015-R004:** 该仓库在`editors/vscode`中附带了一个极简的VS Code客户端，该客户端为`.gan`文件生成`gan lsp`进程；它不包含任何语言逻辑。

## 理由

诊断优先既符合用户价值，也契合库的成熟度；更丰富的功能有待范围跨度，而非发布半成品。插件路线意味着运行器无需了解 LSP，而任何支持 LSP 的编辑器只需 `gan lsp` 命令即可。

## 向后兼容性

新增的；新的分发方式。

## 安全性与确定性

服务器编译其收到的缓冲区，且不执行任何操作。

## 工具和AI使用

编辑器使用 `gan lsp`。AI代理应优先直接使用 `gandora_core`；LSP是为人类编辑器而存在的。

## 被拒绝的备选方案

### 使用 Rust 实现服务器

放弃了生态系统证明；库边界的存在是为了使其成为 Gandora 程序。

## 开放问题

本次修订无。

## 符合性

测试必须涵盖一个脚本化的 stdio 会话：初始化握手，一个带有错误缓冲区的 didOpen 产生具有正确范围的 publishDiagnostics，一个修复它的 didChange 产生一个空列表，didClose 清除，以及 shutdown/exit 以代码 0 终止。

## 变更历史

- 修订版 1，2026-08-02：初始版本。

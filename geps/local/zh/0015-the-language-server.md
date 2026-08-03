---
gep: 15
title: 语言服务器
description: gan-lsp — 诊断、文档悬停、定义、符号、补全和格式化，基于pygls使用Gandora编写；以及gan-lsc，用于AI智能体的同构JSON命令行。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 8
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0015-the-language-server.md
source-revision: 8
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0015-the-language-server.md](../../0015-the-language-server.md)。

# GEP-0015: 语言服务器

## 摘要

`gan-lsp` 是一个用 Gandora 编写的语言服务器协议实现：一个 `gan lsp` 插件（GEP-0013-R003），其分发包为 `gandora-lsp`。版本 1 实现了协议生命周期与推送诊断——每次编辑都会通过 `gandora_core.diagnostics` 在内存中编译，并在编辑器中显示错误/警告及其位置。它是参考生态系统工具：一个基于 `loop`（GEP-0014）、`try/rescue`、互操作字节 I/O 以及编译器库构建的长期运行的 Gandora 服务器。

## 动机

编辑器诊断是最高价值的语言服务，并且不需要任何符号基础设施——这正是编译器库目前的形态。用 Gandora 编写服务器延续了 GEP-0012 计划：工具链是语言完备性的证明，而 LSP 是其第一个长期运行的进程。

## 范围

协议框架、生命周期、诊断、文档悬停、跳转到定义、文档符号、`Module.` 补全以及整个文档的格式化。重命名和查找引用需要等待跨度范围增强；它们 MUST 复用相同的库查询。

## 规范

**GEP-0015-R001:** 服务器用 Gandora 编写，基于 **pygls**：框架拥有协议机制（帧、JSON-RPC、类型化 `lsprotocol` 结构），Gandora 模块拥有语言逻辑，使用 `@decorate @server.feature(...)` 在模块属性中持有的 `LanguageServer` 上附加处理程序。它分发为 `gandora-lsp`，暴露 `gan-lsp` 入口点，因此 `gan lsp` 通过插件委托到达它。pygls 仅是这个包的依赖项——核心工具链保持依赖精简。

**GEP-0015-R001A:** 同一个包暴露 `gan-lsc`——语言服务器控制台，面向 AI 的同构表面。每个查询在 stdout 上打印恰好一个 JSON 值（带引号的术语元组显示为 JSON 数组）并根据 GEP-0001-R023 退出。查询：`version`，`diagnostics <file>`，`ast <file>`，`expand <file>`，`compile <file>`，`resolve <module>`，`doc <Mod>[.<fun>]`，`definition <Mod>[.<fun>]`，`symbols <Mod>`，以及 Python 端镜像 R009——`pydoc`，`pycomplete`，`pygoto`，`pysig` 通过 `$` 风格的模块链——每个接受 `--root <dir>`。`gan lsc ...` 通过相同的委托到达它；格式化是 `gan fmt`。LSP 能力 MUST 保持可表达为 lsc 查询。

**GEP-0015-R002:** 版本 1 处理：`initialize`（宣布全文本同步），`initialized`，`textDocument/didOpen`，`didChange`，`didClose`（清除诊断），`shutdown` 和 `exit`。未知请求返回 MethodNotFound；未知通知被忽略。

**GEP-0015-R003:** 在打开和每次更改时，服务器运行 `gandora_core.diagnostics(text, path, root)`——root 来自工作区文件夹——并发布 `textDocument/publishDiagnostics`，将严重性和基于 1 的编译器跨度映射到基于 0 的零长度 LSP 范围。引发异常的请求返回错误响应（或记录日志，对于通知）；服务器 MUST NOT 因错误输入而崩溃（GEP-0014 `try/rescue`）。

**GEP-0015-R005:** `textDocument/hover` 提供 GEP-0007 文档通道：光标下的引用（一个 `Module.function` 链，一个模块名，或一个根据文件自身模块解析的裸局部名称）通过 `gandora_core.doc` 查找，悬停显示子句签名（包括渲染头部、守卫和默认值），默认语言环境文本，每个可用翻译，元数据，以及 `@example` 块作为 Markdown。`$module` 引用显示模块的规范来源（从不导入它）；语言构造（`def`，`loop`，`quote`，...）显示嵌入的一段落参考卡。未文档化或 `@doc false` 目标不产生悬停。

**GEP-0015-R006:** `textDocument/definition` 将相同的光标目标通过 `gandora_core.definition` 解析到定义源——模块的 `defmodule` 行，函数的第一个匹配子句——跨项目源和已安装包附带的 `.gan` 源。

**GEP-0015-R007:** `textDocument/formatting` 运行 GEP-0016 引擎（`Fmt.format_text`）对缓冲区，并返回一个全文编辑，由相同的 GEP-0016-R006 验证保护；不可格式化或未更改的缓冲区不产生编辑。

**GEP-0015-R008:** `textDocument/documentSymbol` 列出模块及其定义（渲染头部作为细节）；`textDocument/completion` 在模块路径后的 `.` 触发，完成其公共函数和宏，带有签名和文档摘要，通过 `gandora_core.symbols`。

**GEP-0015-R009:** Python 侧边界是一等公民：对于 `$module` 引用和 `pyimport` 别名，悬停显示 Python 文档字符串，补全列出模块成员，定义跳转到 Python 源，签名帮助显示完整的 Python 签名——由 jedi 在项目自身环境中解析，静态（服务器从不导入用户模块来回答查询）。

**GEP-0015-R010:** `textDocument/signatureHelp`（在 `(` 和 `,` 时触发）提供最内层的开放调用：Gandora 目标显示每个子句头作为签名，带有每个参数标签，`@spec` 行（GEP-0017）作为文档，活动参数由参数位置跟踪；Python 目标根据 R009 提供。

**GEP-0015-R004:** 仓库在 `editors/vscode` 中附带一个最小的 VS Code 客户端，它为 `.gan` 文件生成 `gan lsp`；它不包含语言逻辑。

## 理由说明

使用 pygls 而非手写框架，是为了用包本地依赖换取 LSP 生态系统对协议细节的维护——第一版手写服务器证明了该语言能够实现此功能；使用框架则降低了工具的成长成本。`lsc` 的存在使得代理能够以纯 JSON 形式获取语言智能，而无需使用 JSON-RPC。

诊断优先符合用户价值与库的成熟度；更丰富的功能将等待范围跨度，而不是发布半成品。插件路线意味着运行器无需了解 LSP，任何支持 LSP 的编辑器只需执行 `gan lsp` 命令即可。

## 向后兼容性

新增；新的分发。

## 安全性与确定性

**GEP-0015-R012：** `textDocument/references` 返回光标下函数在项目中的每一个调用点——包括定义模块或导入模块中的裸调用、别名解析后的 `Mod.fun` 调用、`&` 捕获，以及（若 `includeDeclaration` 为真）子句头部——其范围覆盖精确的名称令牌，包含字符串插值。`textDocument/rename` 以跨文件 `WorkspaceEdit` 的方式应用相同的解析规则；它会验证新名称，并拒绝定义位于项目之外的目标。`gan lsc references <Mod>.<fun>` 镜像输出原始调用点列表。

**GEP-0015-R013：** `workspace/symbol` 按大小写不敏感的子串搜索项目中的每个定义；`gan lsc wsymbols [query]` 镜像该功能，`gan lsc check` 镜像整个项目的诊断信息——包括编译器 lint——作为一个 JSON 值输出。

**GEP-0015-R014：** 具有机械修复方案的编译器 lint 会作为快速修复代码操作出现在其诊断信息上：在定义上方插入 `@allow :stack_recursion` / `@allow :unused_function`（GEP-0019-R007，GEP-0022-R005），以及为未使用的绑定添加 `_` 前缀（GEP-0022-R002）。

**GEP-0015-R015：** 文档语言是**开发者**偏好设置，而非项目配置。解析顺序为：首先匹配最接近的作用域：显式的 `--locale` 标志；`gandora.local.jsonc` 中的 `docLocale` 键（该文件与 `gandora.jsonc` 同级，属于每个开发者独立文件，会被 git 忽略，其中的未知键被忽略；`gan init` 会写入 gitignore 条目）；`GAN_DOC_LOCALE` 环境变量（VS Code 设置 `gandora.doc.locale` 会将其填入）；最后是默认值——每个语言的章节。特定标签仅渲染该语言内容，包括参数文档，并在每个条目上回退至默认文本。悬停提示和签名帮助会遵循该偏好；`gan lsc doc` 保持为包含所有语言完整信息的 JSON，以便代理自行选择。

服务器会编译接收到的缓冲区，但不会执行任何代码。

## 工具与 AI 使用

编辑器使用 `gan lsp`。AI 代理应直接偏好使用 `gandora_core`；LSP 的存在是为了供人类编辑器使用。

## 被拒绝的替代方案

### 用 Rust 实现服务器

放弃了生态系统证明；库边界的存在是为了使其成为 Gandora 程序。

## 开放问题

此修订版无开放问题。

## 一致性

测试必须覆盖一个脚本化的 stdio 会话：初始化握手（initialize handshake）；一个带有错误缓冲区的 didOpen，产生带有正确跨度的 publishDiagnostics；一个修复该错误的 didChange，产生空列表；didClose 清除；以及 shutdown/exit 以退出码 0 终止；R012 引用/重命名的往返；R013 工作区符号查询；以及一个携带 @allow 编辑的 R014 快速修复。

## 变更历史

- 修订版 8，2026-08-03：R015 — 开发者级文档区域设置（gandora.local.jsonc / GAN_DOC_LOCALE / 编辑器设置），本地化的悬停参数，单语言渲染。
- 修订版 7，2026-08-03：R012 引用 + 重命名，R013 工作区符号 + `lsc check`，R014 代码检查快速修复。
- 修订版 6，2026-08-02：lsc 镜像完整功能集（R001A 已更新）：文档/定义/符号以及 pydoc/pycomplete/pygoto/pysig 的 Python 查询。

- 修订版 5，2026-08-02：新增 R009（基于 jedi 的 Python 侧悬停/补全/定义/签名）和 R010（签名帮助）；悬停显示 GEP-0017 规范。

- 修订版 4，2026-08-02：悬停新增签名、`$module` 和构造卡片（R005 扩展）；新增定义（R006）、通过 GEP-0016 的格式化（R007）以及符号/补全（R008），基于新的 `gandora_core.definition`/`symbols` API。

- 修订版 3，2026-08-02：新增 R005 — 通过 `gandora_core.doc` 查询（新核心 API）的文档悬停。

- 修订版 2，2026-08-02：采用 pygls 作为协议骨架；新增 gan-lsc 同构 JSON 控制台（R001A）。
- 修订版 1，2026-08-02：初始版本。

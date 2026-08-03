---
gep: 15
title: 语言服务器
description: gan-lsp — 诊断、文档悬停、定义、符号、补全和格式化，基于pygls使用Gandora编写；以及gan-lsc，为AI代理提供的同构JSON命令行。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 9
requires: [12, 13, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0015-the-language-server.md
source-revision: 9
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0015-the-language-server.md](../../0015-the-language-server.md)。

# GEP-0015: 语言服务器

## 摘要

`gan-lsp` 是一个用 Gandora 编写的语言服务器协议实现：一个 `gan lsp` 插件（GEP-0013-R003），其发行包为 `gandora-lsp`。版本 1 实现了协议生命周期和推送诊断——每次编辑都会通过 `gandora_core.diagnostics` 在内存中编译，错误/警告会在编辑器中显示，并附有它们的范围。它是参考生态系统工具：一个长时间运行的 Gandora 服务器，基于 `loop`（GEP-0014）、`try/rescue`、互操作字节 I/O 以及编译器库构建。

## 动机

编辑器诊断是最高价值的语言服务，且不需要任何符号基础设施——这正是当前编译器库的形态。用 Gandora 编写服务器延续了 GEP-0012 计划：工具链是语言完备性的证明，而 LSP 是其第一个长期运行的进程。

## 范围

协议帧、生命周期、诊断、文档悬停、转到定义、文档符号、`Module.`自动补全以及整个文档格式化。重命名和查找引用需要跨度范围扩充；它们 MUST 复用同一库查询。

## 规范

**GEP-0015-R001：** 服务器使用 Gandora 编写，基于 **pygls**：框架拥有协议机制（帧、JSON-RPC、类型化的 `lsprotocol` 结构），Gandora 模块拥有语言逻辑，通过 `@decorate @server.feature(...)` 在作为模块属性持有的 `LanguageServer` 上附加处理器。它作为 `gandora-lsp` 分发，暴露 `gan-lsp` 入口点，因此 `gan lsp` 通过插件委托到达它。pygls 仅是该包的依赖项——核心工具链保持依赖精简。

**GEP-0015-R001A：** 同一包暴露 `gan-lsc`——语言服务器控制台，即面向 AI 的同构界面。每个查询在 stdout 上恰好打印一个 JSON 值（带引号的术语元组作为 JSON 数组出现），并根据 GEP-0001-R023 退出。查询：`version`、`diagnostics <file>`、`ast <file>`、`expand <file>`、`compile <file>`、`resolve <module>`、`doc <Mod>[.<fun>]`、`definition <Mod>[.<fun>]`、`symbols <Mod>`，以及 Python 端对应 R009 的镜像——`pydoc`、`pycomplete`、`pygoto`、`pysig`，基于 `$` 风格的模块链——每个都接受 `--root <dir>`。`gan lsc ...` 通过相同的委托到达它；格式化由 `gan fmt` 完成。LSP 能力 MUST 能够表示为 lsc 查询。

**GEP-0015-R002：** 版本 1 处理：`initialize`（宣布完整文本同步）、`initialized`、`textDocument/didOpen`、`didChange`、`didClose`（清除诊断）、`shutdown` 和 `exit`。未知请求收到 MethodNotFound；未知通知被忽略。

**GEP-0015-R003：** 在打开时和每次更改时，服务器运行 `gandora_core.diagnostics(text, path, root)`——root 来自工作区文件夹——并发布 `textDocument/publishDiagnostics`，将严重级别和基于 1 的编译器跨度映射到基于 0 的零长度 LSP 范围。引发异常的请求以错误响应回答（或记录为日志，对于通知）；服务器 MUST NOT 因错误输入而死机（GEP-0014 `try/rescue`）。

**GEP-0015-R005：** `textDocument/hover` 提供 GEP-0007 文档通道：游标下的引用（一个 `Module.function` 链、一个模块名称，或解析为文件自身模块的裸局部名称）通过 `gandora_core.doc` 查找，hover 显示子句签名（包括渲染的头部、守卫和默认值）、默认语言环境散文、每个可用翻译、元数据以及 `@example` 块，格式为 Markdown。`$module` 引用显示模块的规范来源（从不导入它）；语言构造（`def`、`loop`、`quote` 等）显示嵌入的一段式参考卡片。未记录或 `@doc false` 的目标不产生 hover。

**GEP-0015-R006：** `textDocument/definition` 将相同的游标目标通过 `gandora_core.definition` 解析到定义源——模块的 `defmodule` 行，函数的第一个匹配子句——跨项目源和已安装包随附的 `.gan` 源。

**GEP-0015-R007：** `textDocument/formatting` 运行 GEP-0016 引擎（`Fmt.format_text`）对缓冲区，并返回一个全文档编辑，由相同的 GEP-0016-R006 验证守卫；不可格式化或未更改的缓冲区不产生编辑。

**GEP-0015-R008：** `textDocument/documentSymbol` 列出模块及其定义（以渲染头部作为详情）；`textDocument/completion` 在模块路径后输入 `.` 时触发，完成其公共函数和宏，附带签名和文档摘要，通过 `gandora_core.symbols`。

**GEP-0015-R009：** Python 端边界是一等公民：对于 `$module` 引用和 `pyimport` 别名，hover 显示 Python 文档字符串，补全列出模块成员，定义跳转到 Python 源，签名帮助显示完整的 Python 签名——由 jedi 在项目自身环境中静态解析（服务器从不导入用户模块来回答查询）。

**GEP-0015-R010：** `textDocument/signatureHelp`（在输入 `(` 和 `,` 时触发）提供最内层未闭合调用：Gandora 目标将每个子句头部显示为带有每个参数标签的签名，`@spec` 行（GEP-0017）作为文档，活动参数通过参数位置跟踪；Python 目标按 R009 提供。

**GEP-0015-R004：** 仓库在 `editors/vscode` 中附带一个最小的 VS Code 客户端，该客户端为 `.gan` 文件启动 `gan lsp`；它不包含任何语言逻辑。

## 理由

采用 pygls 而不是手写框架，是用一个包级依赖换取 LSP 生态对协议细节的维护——修订版 1 的手写服务器证明了该语言能够做到；框架使得工具更容易扩展。`lsc` 的存在是为了让代理能够以纯 JSON 形式获取语言智能，而无需使用 JSON-RPC。

以诊断为先既符合用户价值，也符合库的成熟度；更丰富的功能等待范围跨度的实现，而不是发布半成品。插件路线意味着运行器不需要 LSP 知识，任何支持 LSP 的编辑器只需要 `gan lsp` 命令。

## 向后兼容性

新增；新的分发方式。

## 安全性与确定性

**GEP-0015-R012：** `textDocument/references` 返回光标所在函数在项目中的每一个调用点——包括定义模块或导入模块中的裸调用、别名解析后的 `Mod.fun` 调用、`&` 捕获，以及（根据 `includeDeclaration`）子句头——其范围覆盖精确的名称标记，字符串插值也包含在内。`textDocument/rename` 应用与跨文件 `WorkspaceEdit` 相同的解析逻辑；它验证新名称，并拒绝定义位于项目之外的目标。`gan lsc references <Mod>.<fun>` 镜像输出原始调用点列表。

**GEP-0015-R013：** `workspace/symbol` 通过不区分大小写的子串匹配搜索项目中的所有定义；`gan lsc wsymbols [query]` 镜像其行为，`gan lsc check` 则以单个 JSON 值镜像整个项目的诊断结果——包括编译器 lint 在内。

**GEP-0015-R014：** 具有机械修复手段的编译器 lint 会以其诊断结果上的快速修复代码动作形式呈现：在定义上方插入 `@allow :stack_recursion` / `@allow :unused_function`（GEP-0019-R007，GEP-0022-R005），以及为未使用的绑定添加 `_` 前缀（GEP-0022-R002）。

**GEP-0015-R015：** 文档语言是**开发者**偏好，而非项目配置。解析顺序为最近作用域优先：显式的 `--locale` 标志；`gandora.local.jsonc` 中的 `docLocale` 键（该文件位于 `gandora.jsonc` 旁，被 gitignore 忽略，属于开发者个人文件，未知键会被忽略；`gan init` 会写入 gitignore 条目）；`GAN_DOC_LOCALE` 环境变量（VS Code 设置 `gandora.doc.locale` 会为其赋值）；最后仅使用默认语言——翻译内容在未请求时不会干扰。特定标签仅渲染该语言，包括参数文档，每个条目会回退到默认文本；`all` 则选择所有语言下的各章节内容。悬停和签名帮助遵循该偏好；`gan lsc doc` 以区域设置完整的 JSON 形式输出，供代理自行选择。

服务器会编译其接收到的缓冲区，但不执行任何操作。

## 工具与AI使用

编辑器使用 `gan lsp`。AI智能体应直接优先使用 `gandora_core`；LSP 是为人类编辑器而存在的。

## 已否决的备选方案

### 使用 Rust 实现服务器

放弃了生态证明；库边界的存在是为了使这个成为 Gandora 程序。

## 开放问题

此版本无开放问题。

## 一致性

测试 MUST 覆盖一个脚本化的 stdio 会话：初始化握手、一个带有错误缓冲区的 didOpen 产生带有正确范围的 publishDiagnostics、修复错误的 didChange 产生空列表、didClose 清除、以及 shutdown/exit 以代码 0 终止；R012 引用/重命名往返；一个 R013 工作区符号查询；以及一个携带 @allow 编辑的 R014 快速修复。

## 变更历史

- 修订版 9，2026-08-03：R015 — 未设置默认值时为默认语言单独生效；“全部”（即各章节中的每种语言）变为显式主动选择。
- 修订版 8，2026-08-03：R015 — 开发者级文档区域设置（`gandora.local.jsonc` / `GAN_DOC_LOCALE` / 编辑器设置）、本地化悬停参数、单语言渲染。
- 修订版 7，2026-08-03：R012 引用 + 重命名，R013 工作区符号 + `lsc check`，R014 lint 快速修复。
- 修订版 6，2026-08-02：`lsc` 镜像完整能力集（R001A 更新）：文档/定义/符号以及 `pydoc`/`pycomplete`/`pygoto`/`pysig` Python 查询。
- 修订版 5，2026-08-02：新增 R009（基于 jedi 的 Python 侧悬停/补全/定义/签名）和 R010（签名帮助）；悬停显示 GEP-0017 规范。
- 修订版 4，2026-08-02：悬停获得签名、`$module` 和构造卡片（R005 扩展）；新增定义（R006）、通过 GEP-0016 的格式化（R007）、以及符号/补全（R008），基于新的 `gandora_core.definition`/`symbols` API。
- 修订版 3，2026-08-02：新增 R005 — 通过 `gandora_core.doc` 查找的文档悬停（新的核心 API）。
- 修订版 2，2026-08-02：采用 pygls 作为协议机制；新增 `gan-lsc` 同构 JSON 控制台（R001A）。
- 修订版 1，2026-08-02：初始版本。

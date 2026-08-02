---
gep: 13
title: gan 任务运行器
description: 开发者入口点 `gan` 成为Gandora程序——mix/cargo角色——基于gandora-core，带有子命令插件，Rust编译器降级为阶段0工具ganc。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [6, 12, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0013-the-gan-task-runner.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0013-the-gan-task-runner.md](../../0013-the-gan-task-runner.md)。

# GEP-0013: gan 任务运行器

## 摘要

开发者输入的不是编译器，而是任务运行器——例如 Elixir 的 `mix`、Rust 的 `cargo`。Gandora 的任务运行器称为 `gan`，它本身是一个 Gandora 程序：一个包（在 PyPI 上为 `gandora-tool`，源代码位于 `tools/gan/`），其入口点驱动 `gandora-core` 在进程内进行构建、运行和评估代码。Rust 二进制文件被重命名为 `ganc`——这是阶段 0 编译器，用于引导工具链并支持尚未重写的命令。未知子命令委托给已安装的 `gan-<name>` 可执行文件（遵循 cargo 插件约定），这样工具生态系统可以在 Gandora 中发展，而无需修改任何一个核心工件。

## 动机

GEP-0012 将编译器做成一个库，正是为了让工具链可以用该语言本身编写。这么做是最强的完整性证明——运行器在每次调用时都会执行互操作、控制流（GEP-0014）、标准库和包系统——并且每天向每个 Gandora 用户展示一个可见的真实 Gandora 代码示例。`gan`/`ganc` 的拆分与 `cargo`/`rustc` 类似：当入口点改变实现语言时，肌肉记忆和文档依然有效。

## 范围

运行器的命令集、其委托规则、`ganc` 重命名以及分发。REPL的行编辑优化、`watch`、`fmt` 以及LSP服务器（一个插件，GEP-0015）不在范围之内。

## 术语

- **运行器**：`gan` 入口点，一个 Gandora 程序。
- **Stage-0 编译器**：`ganc`，Rust 二进制文件，用于构建运行器本身，并作为遗留命令的委托。
- **插件**：可在项目环境中访问的可执行文件 `gan-<name>`。

## 规范

**GEP-0013-R001:** Rust 二进制文件命名为 `ganc`；`gandora-lang` 轮子随附该文件。面向用户的 `gan` 命令由 `gandora-tool` 发行版提供：这是一个 Gandora 包（源文件位于 `tools/gan/`，轮子中编译后的 Python 遵循 GEP-0006），其控制台入口点为运行器。二者与工具链版本同步发布。

**GEP-0013-R002:** 运行器通过 `gandora_core` 原生实现：`version`（报告运行器与核心版本，当版本不匹配时按 GEP-0012-R007 发出警告）、`build`、`check`、`run`、`exec <code>`（编译片段、执行、按 `inspect/1` 方式输出 `_`）以及 `repl`（基于 GEP-0014 对标准输入进行 `loop`，使用相同片段机制，提示符 `gan>`，绑定跨行持续存在）。`init` 按 GEP-0001-R021 所述进行脚手架搭建。

**GEP-0013-R003:** 运行器未实现的子命令按以下顺序解析：环境中路径上的可执行文件 `gan-<name>`（插件，接收剩余参数）；否则委托给 `ganc <name> ...`（涵盖 `expand`、`doc`、`test`、`compile`、`lsc` 及未来的阶段 0 命令）；否则显示用法错误。插件是普通的发行版，暴露 `gan-<name>` 入口点——安装方式为 `uv add`。

**GEP-0013-R004:** 运行器的输出与退出码遵循 GEP-0001-R023。运行器 MUST NOT 在运行时依赖 `gan` Rust 二进制文件（除非通过 R003 委托），并且 MUST 在安装了 `gandora-core` 和 `gandora-tool` 的任何项目中正常工作。

**GEP-0013-R005:** 引导：运行器的轮子通过 CI 中使用 `ganc` 编译 `tools/gan/` 构建（GEP-0006 流水线）。运行器在安装时从不编译自身。

## 理由

带回退的委托机制使得重写可以逐个命令进行，无需标志日（flag day）：从运行器的第一个版本开始，`ganc` 所做的所有事情都可以通过 `gan` 访问。插件约定采用 cargo's 的方式，选择了 registry 或配置文件，因为环境本身就是 registry——`uv add` 安装一个插件，而发现是通过 PATH 查找。

如果首先将 `exec`/`repl` 原生实现（而不是 build/run）将是倒退：它们展示语言，但 `build` 是证明运行器能够端到端驱动编译器库的命令，因此两者一起发布。

## 向后兼容性

`gan` 保持其 CLI 表面（R003 保证这一点）；通过旧标识调用二进制文件的脚本应切换到 `ganc` 或通过运行器继续工作。文档将迁移到 `uv tool install gandora-tool`（其依赖于 `gandora-core`）作为入口安装方式。

## 安全性与确定性

运行器仅执行其命令始终执行的内容（用户自己的项目代码）；委托操作从项目环境中运行可执行文件，其信任边界与任何 Python 控制台脚本相同。

## 工具与AI使用

AI代理持续使用`gan <command>`；表面上看没有任何变化。  
构建Gandora工具集的代理应将其作为`gan-<name>`插件包发布，而非封装运行器。

## 已拒绝的备选方案

### 将运行器保留在 Rust 中

放弃了完备性证明以及生态系统读取自身工具链的能力；库边界（GEP-0012）存在的目的正是为了实现这一转变。

### 为入口点取一个新名称

为了命名纯洁性而打破 `gan` 的肌肉记忆和所有文档；cargo/rustc 的先例表明，在运行器上使用熟悉的名称进行拆分是可行的。

## 未决问题

本修订版暂无。

## 符合性

测试 MUST 覆盖：针对真实项目的每个 R002 命令（包括通过管道输入的 repl 和打印 `_` 的 exec）；在 ganc 回退之前的插件解析以及两者之后的用法错误；版本不匹配警告；以及一个由 wheel 安装的运行器驱动一个没有 Rust 工具链存在的项目。

## Change History

- Revision 1, 2026-08-02: 初始版本。

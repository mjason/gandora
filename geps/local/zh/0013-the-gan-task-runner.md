---
gep: 13
title: gan 任务运行器
description: 开发者入口点 `gan` 成为 Gandora 程序——即 mix/cargo 角色——基于 gandora-core，带有子命令插件，并且 Rust 编译器降级为阶段-0 工具 ganc。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 2
requires: [6, 12, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0013-the-gan-task-runner.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0013-the-gan-task-runner.md](../../0013-the-gan-task-runner.md)。

# GEP-0013: gan 任务运行器

## 摘要

开发者输入的不是编译器，而是任务运行器——Elixir 中的 `mix`，Rust 中的 `cargo`。Gandora 的任务运行器称为 `gan`，它是一个 Gandora 程序：一个包（PyPI 上的 `gandora-tool`，源代码在 `tools/gan/` 中），其入口点驱动 `gandora-core` 在进程内构建、运行和评估代码。Rust 二进制文件被重命名为 `ganc`——这是阶段 0 编译器，用于引导工具链并支持尚未重写的命令。未知子命令委托给已安装的 `gan-<name>` 可执行文件，这是 cargo 插件约定，因此工具生态系统在 Gandora 中增长，而无需触及任一核心工件。

## 动机

GEP-0012 将编译器制作成一个库，正是为了让工具链可以用该语言编写。这样做是现有最强完备性证明——每次调用时，运行器都会执行互操作、控制流（GEP-0014）、标准库和包系统——并且为每个 Gandora 用户提供了一个日常可见的真实 Gandora 代码示例。`gan`/`ganc` 的分割模仿了 `cargo`/`rustc`：在入口点更改实现语言的同时，肌肉记忆和文档仍然有效。

## 范围

运行器的命令集、其委托规则、`ganc` 的重命名以及分发。REPL 的行编辑优化、`watch`、`fmt` 以及 LSP 服务器（一个插件，GEP-0015）不在范围之内。

## 术语

- **Runner**（运行器）：`gan` 入口点，一个 Gandora 程序。
- **Stage-0 compiler**（阶段0编译器）：`ganc`，Rust 二进制文件，用于构建
  Runner 自身，并作为遗留命令的委托。
- **Plugin**（插件）：一个可执行文件 `gan-<name>`，在项目环境中可访问。

## 规范

**GEP-0013-R001:** Rust 二进制文件名为 `ganc`；`gandora-lang` wheel 分发它。面向用户的 `gan` 命令由 `gandora-tool` 发行版提供：这是一个 Gandora 包（源代码位于 `tools/gan/`，按照 GEP-0006 在 wheel 中编译为 Python），其控制台入口点是运行器。两者与工具链版本同步发布。

**GEP-0013-R002:** 运行器原生实现（通过 `gandora_core`）：`version`（报告运行器和核心版本，并带有 GEP-0012-R007 的不匹配警告）、`build`、`check`、`run`、`exec <code>`（编译代码片段，执行，并按 `inspect/1` 的方式打印 `_`）以及 `repl`（一个 GEP-0014 的 `loop`，在 stdin 上使用相同的片段机制，提示符为 `gan>`，绑定跨行持久化）。`init` 按照 GEP-0001-R021 的描述进行脚手架搭建。

**GEP-0013-R003:** 运行器未实现的子命令按以下顺序解析：环境路径上的可执行文件 `gan-<name>`（插件，接收剩余参数）；否则委托给 `ganc <name> ...`（涵盖 `expand`、`doc`、`test`、`compile`、`lsc` 和未来的 stage-0 命令）；否则报使用错误。插件是暴露 `gan-<name>` 入口点的普通发行版——安装一个的方法是通过 `uv add`。

**GEP-0013-R006:** 一个项目在其 `pyproject.toml` 中恰好需要两个 Gandora 条目：`gandora-std` 作为运行时依赖（生成的代码会导入它），以及 `gandora-tool[dev]` 在 `dev` 依赖组中——`dev` extra 聚合了工具链（语言服务器，编译器库通过传递依赖到达）。脚手架（`gan init`、`ganc init`）MUST 准确地生成此结构。

**GEP-0013-R004:** 运行器输出和退出码遵循 GEP-0001-R023。运行器 MUST NOT 依赖于 `gan` Rust 二进制文件在运行时存在，除非通过 R003 委托，且 MUST 在任何安装了 `gandora-core` 和 `gandora-tool` 的项目中工作。

**GEP-0013-R005:** 引导：运行器的 wheel 是通过在 CI 中用 `ganc` 编译 `tools/gan/` 构建的（GEP-0006 流水线）。运行器在安装时从不编译自身。

## 原理

带回退的委托（Delegation-with-fallback）允许重写过程逐条命令进行，无需标志日（flag day）：从运行器的第一个版本开始，`ganc` 所做的所有事情都可以通过 `gan` 访问。插件约定采用 Cargo 的方式，而不是注册表或配置文件，因为环境本身已经是注册表——`uv add` 安装一个插件，而发现方式是通过 PATH 查找。

如果先让 `exec`/`repl` 原生化（而不是 `build`/`run`），那将是倒退：它们展示了语言，但 `build` 是证明运行器能够端到端驱动编译器库的命令，因此两者同时发布。

## 向后兼容性

`gan` 保持其 CLI 界面（R003 保障）；通过旧名称调用该二进制文件的脚本应切换到 `ganc`，或通过运行器保持工作。文档将安装入口迁移至 `uv tool install gandora-tool`（依赖 `gandora-core`）。

## 安全性与确定性

Runner 仅执行其命令始终执行的内容（用户自己的项目代码）；委托运行来自项目环境的可执行文件，与任何 Python 控制台脚本具有相同的信任边界。

## 工具与 AI 使用

AI 代理持续使用 `gan <command>`；表面没有任何变化。
构建 Gandora 工具的代理应当将其作为 `gan-<name>` 插件包发布，而不是包装运行器。

## 已拒绝的替代方案

### 在 Rust 中保留运行器

放弃了完备性证明以及生态系统读取自身工具链的能力；库边界（GEP-0012）的存在正是为了实现这一迁移。

### 入口点的新名称

为了命名纯粹性，打破 `gan` 的肌肉记忆和所有文档；cargo/rustc 的先例表明，在运行器上使用熟悉的名称进行拆分是可行的。

## 开放式问题

本修订版中无。

## 一致性

测试MUST覆盖：针对真实项目的每个R002命令（包括通过管道输入的REPL以及打印`_`的exec）；在ganc回退之前的插件解析以及两者之后的用法错误；版本不匹配警告；以及一个wheel安装的runner驱动一个没有Rust工具链的项目。

## 变更历史

- 修订版 2，2026-08-02：添加了 R006——将 `gandora-tool[dev]` 额外依赖作为唯一的开发依赖项；脚手架会生成它。

- 修订版 1，2026-08-02：初始版本。

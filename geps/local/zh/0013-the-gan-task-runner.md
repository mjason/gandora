---
gep: 13
title: gan 任务运行器
description: 开发者入口点 `gan` 变成一个 Gandora 程序——mix/cargo 角色——基于 gandora-core，带有子命令插件，并且 Rust 编译器被降级为 stage-0 工具 ganc。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-02
updated: 2026-08-07
revision: 3
requires: [6, 12, 14]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0013-the-gan-task-runner.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0013-the-gan-task-runner.md](../../0013-the-gan-task-runner.md)。

# GEP-0013: gan 任务运行器

## 摘要

开发者键入的命令并非编译器；它是任务运行器——Elixir 的 `mix`、Rust 的 `cargo`。Gandora 的任务运行器名为 `gan`，它本身就是一个 Gandora 程序：一个包（PyPI 上的 `gandora-tool`，源码位于 `tools/gan/`），其入口点在进程内驱动 `gandora-core`，用于构建、运行和评估代码。Rust 二进制文件被重命名为 `ganc`——这是 stage-0 编译器，负责引导工具链并支持尚未重写的命令。未知子命令遵循 cargo 插件约定，委托给已安装的 `gan-<name>` 可执行文件，从而使工具生态系统在 Gandora 中得以成长，而无需改动任一核心工件。

## 动机

GEP-0012 将编译器打造为库，正是为了让工具链可以用 Gandora 语言本身来编写。这样做是现有的最强完备性证明——运行器在每次调用时都会执行互操作、控制流（GEP-0014）、标准库和包系统——并且为每位 Gandora 用户提供了一个日常可见的真实 Gandora 代码示例。`gan`/`ganc` 的拆分参照了 `cargo`/`rustc`：肌肉记忆和文档保持有效，而入口点改变了实现语言。

## 范围

运行器的命令集、其委托规则、`ganc` 重命名以及分发。REPL 的行编辑优化、`watch`、`fmt` 以及 LSP 服务器（一个插件，GEP-0015）不在范围内。

## Terminology

- **Runner**：`gan` 入口点，一个 Gandora 程序。
- **Stage-0 compiler**：`ganc`，即用于构建 runner 自身并作为旧版命令代理的 Rust 二进制文件。
- **Plugin**：可在项目环境中访问的可执行 `gan-<name>` 程序。

## Specification

**GEP-0013-R001:** Rust 二进制文件名为 `ganc`；`gandora-lang` wheel 随附该二进制文件。面向用户的 `gan` 命令由 `gandora-tool` 发行版提供：一个 Gandora 包（源码位于 `tools/gan/`，wheel 中为按 GEP-0006 编译得到的 Python），其控制台入口点是运行器。两者与工具链版本同步发布。

**GEP-0013-R002:** 运行器原生实现（通过 `gandora_core`）：`version`（报告运行器与核心版本，并带有 GEP-0012-R007 不匹配警告）、`build`、`check`、`run`、`exec <code>`（编译代码片段、执行、按 `inspect/1` 打印 `_`）以及 `repl`（一个针对 stdin 的 GEP-0014 `loop`，使用相同的代码片段机制，`gan>` 提示符，绑定跨行持续保留）。`init` 按 GEP-0001-R021 所述生成脚手架。

**GEP-0013-R003:** 运行器未实现的子命令按顺序解析：环境路径中的可执行文件 `gan-<name>`（插件，接收剩余参数）；否则委托给 `ganc <name> ...`（涵盖 `expand`、`doc`、`test`、`compile`、`lsc` 以及未来的 stage-0 命令）；否则产生用法错误。插件是暴露 `gan-<name>` 入口点的普通发行版——安装一个插件即 `uv add`。

**GEP-0013-R006:** 一个项目在其 `pyproject.toml` 中需要恰好两个 Gandora 条目：`gandora-std` 作为运行时依赖（生成的代码会导入它），以及 `gandora-tool[dev]` 放在 `dev` 依赖组中——`dev` extra 聚合了工具链（语言服务器，编译器库以传递依赖方式引入），同时包含 `gandora-mcp`，用于脚手架所接入的 MCP 表面（GEP-0028-R012）。脚手架 MUST 精确生成这种结构；当它保留已有 pyproject 时，它 MUST 列出项目仍然需要的条目，而不是让项目的 agent 接线无法解析。

**GEP-0013-R004:** 运行器输出与退出码遵循 GEP-0001-R023。运行器 MUST NOT 依赖 `gan` Rust 二进制文件在运行时存在，除非通过 R003 委托；并且 MUST 能在任何安装了 `gandora-core` 和 `gandora-tool` 的项目中工作。

**GEP-0013-R005:** 自举：运行器的 wheel 是在 CI 中用 `ganc` 编译 `tools/gan/` 构建而成的（即 GEP-0006 流水线）。运行器在安装时绝不会编译自身。

## 理由

带回退的委托让重写可以逐条命令地推进，而无需同步切换日：从 runner 的第一个版本开始，`ganc` 所做的一切就仍然可以通过 `gan` 触达。插件约定采用 cargo 的方式，之所以选择它而非注册表或配置文件，是因为环境本身就已经是注册表——`uv add` 会安装插件，而发现机制就是 PATH 查找。

如果先把 `exec`/`repl` 做成原生命令（而不是 `build`/`run`），那就本末倒置了：它们固然展示了语言，但 `build` 才是证明 runner 能够端到端驱动编译器库的命令，因此两者一同发布。

## 向后兼容性

`gan` 保留其 CLI 接口（R003 对此有保证）；通过旧标识调用该二进制的脚本应切换到 `ganc`，或继续通过 runner 运行。文档改为以 `uv tool install gandora-tool`（依赖 `gandora-core`）作为安装入口。

## 安全性与确定性

运行器只执行其命令始终执行的内容（用户自己的项目代码）；委托从项目环境中运行可执行文件，与任何 Python 控制台脚本具有相同的信任边界。

## 工具与 AI 使用

AI 代理持续使用 `gan <command>`；表面上没有任何变化。
构建 Gandora 工具的代理应当将其作为 `gan-<name>` 插件包发布，而不是包装运行器。

## 被否决的备选方案

### 将运行器保留在 Rust 中

放弃了完备性证明以及生态系统读取自身工具链的能力；库边界（GEP-0012）的存在正是为了实现这一转变。

### 为入口点起一个新名称

为了命名纯粹性而打破对 `gan` 的肌肉记忆并修改所有文档；cargo/rustc 的先例表明，将二者拆分后，在运行器上保留熟悉的名字是可行的。

## 未决问题

本修订版无。

## Conformance

测试 MUST 覆盖：每个 R002 命令在真实项目上的执行（包括通过管道输入 repl，以及 exec 打印 `_`）；在 ganc 回退之前进行插件解析，并覆盖这两者之后的使用错误；版本不匹配警告；以及一个通过 wheel 安装的运行器驱动一个没有 Rust 工具链的项目。

## 变更历史

- 修订版 3，2026-08-07：`init` 仅属于运行器 — `ganc` 移除脚手架，保持为编译器。两个实现早已出现分歧（模板不同，其中一个未能通过 Advisor），而 R006 的“两个脚手架必须一致”规则之所以存在，仅仅是因为当时有两个脚手架。`gan init` 新增 `--package` 并可重新运行，这就是现有项目获取 GEP-0028-R012 代理接线的方式；R006 现在也指名了 `gandora-mcp`。
- 修订版 2，2026-08-02：新增 R006 — 将 `gandora-tool[dev]` extra 作为唯一的开发依赖项；脚手架会生成它。

- 修订版 1，2026-08-02：初始版本。

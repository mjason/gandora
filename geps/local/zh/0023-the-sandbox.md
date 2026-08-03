---
gep: 23
title: The Sandbox
description: 一个查询，用于验证生成的代码——编译、lint、模糊建议、带超时执行——这样代理可以通过尝试来学习语言。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-03
updated: 2026-08-03
revision: 1
requires: [12, 15, 22]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0023-the-sandbox.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0023-the-sandbox.md](../../0023-the-sandbox.md)。

# GEP-0023: 沙盒

## 摘要

`gan lsc try <file|-> [--no-run]` 以单个 JSON 值回答 AI 代理在生成 Gandora 代码后提出的问题：*这是正确的吗？如果不是，我可能的意思是什么？* 该管道会编译、lint、对模块成员进行拼写检查以匹配真实符号、标记常见的跨语言习惯，然后在硬超时限制下在一个临时目录中执行——捕获 stdout，并且对于裸语句片段，捕获最后一个表达式的值。错误的名称会获得来自编辑距离搜索的 Rails 风格的 *did-you-mean* 建议。不会触及项目。

## 动机

对于 AI 代理而言，一门新语言最大的采纳风险在于反馈循环：一个看似合理但拼写错误的代码（`Enum.mpa`、`return`、Python 的 `def f()`）需要耗费完整的编辑-构建-运行往返周期才能发现，而运行时抛出的 `AttributeError` 仅指出症状，并未给出修复方案。一次既能作出裁决又能**传授知识**的快速查询，就能将每一次错误转变为一次学习。

## 范围

`gan lsc` 中的只读验证查询。持久化沙箱、超出墙钟超时的资源配额以及网络隔离不在范围内：沙箱运行开发者自己的代码，使用项目自己的解释器，与 `gan run` 完全一致。

## 规范

**GEP-0023-R001：** `gan lsc try <file|->`（使用 `-` 表示标准输入）接受一个完整模块或裸语句，并输出一个 JSON 对象：`ok`（布尔值）、`stage`（`compile` | `lint` | `run` | `ok`）、`diagnostics`（编译器错误和 GEP-0022 lint 检查，带跨度信息）、`suggestions`（见 R002/R003）、`python`（生成的代码——零运行时承诺的可检查体现）、`stdout` 以及 `value`（代码片段最后一个表达式的 `repr`）。`--no-run` 在诊断后停止执行。执行过程发生在一个全新的临时目录中，使用项目的解释器，并设置硬超时；模块的 `main/0` 恰好运行一次。

**GEP-0023-R002（拼写纠正建议）：** 拼写错误会从真实候选词中获取编辑距离建议：`Mod.fun(...)` 调用会对照模块的实际符号进行检查（`Enum.mpa` → `Enum.map`），未定义变量 lint 检查会对照片段自身的标识符（`valeu` → `value`），编译错误位置会对照关键字列表（`defmodul` → `defmodule`）。建议携带 `"kind": "did_you_mean"`。

**GEP-0023-R003（迁移提示）：** 常见的跨语言习惯会被文本识别，并以 Gandora 拼写方式回应——`return`、`while`、`elif`、Python 的 `def ...():`、`lambda`、`None`/`True`/`False`、`import`/`from ... import`、`self.`、`&&`/`||`、增强赋值、f-string、`switch`、`== nil` 以及已废弃的 `$"a.b"`。建议携带 `"kind": "migration"`；它们是建议性的，绝不阻止对结果的判定。

**GEP-0023-R004（实践提示）：** 文档化的标准会在编译器保持沉默的地方以 `"kind": "practice"` 建议的形式呈现：没有 `@spec` 的公开 `def`，以及反复引用某个 `$module` 而惯用做法是使用 `pyimport` 的情况（GEP-0003 修订版 6）。

## Rationale

将沙盒构建到 `lsc` 中，就保持了一个统一的 AI 界面：使用 `lsc` 进行检查、阅读文档和查找引用的代理，也通过它进行学习和验证。文本错误模式被故意设计得简单——它们的存在是为了教授惯用法，而不是解析 Python——而对真实符号表进行编辑距离搜索，正是使建议可信而非幻觉的原因。

## 向后兼容性

附加性的。

## Security and Determinism

The sandbox executes user-supplied code with the project interpreter —
the same trust boundary as `gan run`/`gan test`. The timeout bounds
wall clock, not capability; verdict output is deterministic apart from
the executed program's own behavior.

## 工具与AI使用

代理（Agents）在将生成的代码写入项目之前，SHOULD 先通过 `gan lsc try` 进行路由，将 `did_you_mean` 建议视为要应用的修正，并在不需要执行时使用 `--no-run`。循环：生成 → `try` → 应用建议 → `try` → 写入。

## 被拒绝的替代方案

### 长期运行的沙箱服务器

状态会积累并与项目产生偏差；每个查询使用新的临时目录可保持裁决结果可重现，且实现代码约 300 行。

### 编译器端的“您是不是要找？”

编译器保持小巧且确定；模糊搜索属于工具链，因为候选对象（符号、标识符）已被索引。

## 一致性

测试 MUST 涵盖：干净代码片段运行，捕获 stdout 和值；成员拼写错误提示正确函数；针对 Python 习惯的迁移提示；未定义变量和关键词的建议（did-you-mean）；实践提示；`main/0` 的单次执行；以及 `--no-run`。

## 变更历史

- 修订版 1，2026-08-03：初始版本。

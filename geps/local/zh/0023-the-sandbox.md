---
gep: 23
title: The Sandbox
description: 一个查询，用于验证生成的代码——编译、检查、模糊建议、带超时执行——以便代理通过尝试来学习语言。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-03
updated: 2026-08-03
revision: 3
requires: [12, 15, 22]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0023-the-sandbox.md
source-revision: 3
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0023-the-sandbox.md](../../0023-the-sandbox.md)。

# GEP-0023: 沙盒

## 摘要

`gan lsc try <file|-> [--no-run]` 以一个 JSON 值回答 AI 智能体在生成 Gandora 代码后提出的问题：*这段代码是否正确？如果不正确，我可能原本想表达什么？* 该流水线依次执行编译、lint 检查、对模块成员与真实符号进行拼写检查、标记常见的跨语言习惯，然后在临时目录中在硬超时限制下运行——并捕获标准输出，同时对于裸语句片段，还会捕获最后一个表达式的值。错误的名称会通过编辑距离搜索得到 Rails 风格的 *did-you-mean* 建议。整个过程不会触及项目本身。

## 动机

一门新语言对 AI 代理最大的采纳风险在于反馈循环：一个看似合理但错误的拼写（`Enum.mpa`、`return`、Python 的 `def f()`）需要完整的编辑-构建-运行往返才能发现，代价高昂；而运行时的 `AttributeError` 只指出症状，而非修复方法。一个快速查询，既能判定*又能教导*，可将每个错误都变成一次学习。

## 范围

一个在 `gan lsc` 中的只读验证查询。持久沙箱、超出挂钟超时的资源配额以及网络隔离不在范围内：沙箱使用项目自身的解释器运行开发者自己的代码，就像 `gan run` 一样。

## 规范

**GEP-0023-R001:** `gan lsc try <file|->`（使用 `-` 表示标准输入）接受一个完整模块或裸语句，并输出一个 JSON 对象：`ok`（布尔值）、`stage`（`compile` | `lint` | `run` | `ok`）、`diagnostics`（编译器错误和 GEP-0022 的 lint 检查结果，带跨度信息）、`suggestions`（见 R002/R003）、`python`（生成的代码——零运行时承诺变得可检查）、`stdout` 和 `value`（代码片段最后一个表达式的 `repr` 值）。`--no-run` 在执行诊断后停止。执行过程在项目的解释器环境下，使用一个临时目录并设置硬超时；模块的 `main/0` 恰好运行一次。

**GEP-0023-R002（是否指提示）：** 拼写错误会从*真实*候选列表中获得编辑距离建议：`Mod.fun(...)` 调用会对照模块的实际符号进行检查（如 `Enum.mpa` → `Enum.map`），未定义变量的 lint 会对照代码片段自身的标识符进行检查（如 `valeu` → `value`），编译错误位置会对照关键字列表进行检查（如 `defmodul` → `defmodule`）。建议携带 `"kind": "did_you_mean"`。

**GEP-0023-R003（迁移提示）：** 常见的跨语言习惯会被文本识别，并以 Gandora 的拼写方式给出回答——例如 `return`、`while`、`elif`、Python 的 `def ...():`、`lambda`、`None`/`True`/`False`、`import`/`from ... import`、`self.`、`&&`/`||`、增强赋值、f-字符串、`switch`、`== nil` 以及已废弃的 `$"a.b"`。建议携带 `"kind": "migration"`；它们仅供参考，不会阻止裁决。

**GEP-0023-R004（实践提示）：** 人工智能很少拼写错误——它只是偷懒。在编译器保持沉默的地方，文档化的标准会以 `"kind": "practice"` 建议的形式呈现：一份综合的注解覆盖率报告（`@spec`/`@doc`/`@moduledoc`，外加缺少 `@example` 的说明）、在 `@spec` 参数位置使用具体的 `list()`/`map()`（“抽象输入，具体输出”）、希望使用 `for` 的 map+filter 管道、希望使用 `&f/1` 的 `fn x -> f(x) end`、希望使用 `Enum.empty?` 的 `count == 0`、希望指定具体异常类型的裸 `rescue`，以及希望使用 `pyimport` 的重复 `$module`（GEP-0003 修订版 6）。

**GEP-0023-R005（信任机制）：** 字符串、heredoc、sigil 和注释内容在进行任何文本检查之前会被屏蔽（保留定界符）——正常文本绝不会触发代码模式，一个符合习惯的模块 MUST 产生 `"suggestions": []`。建议按消息内容去重。

**GEP-0023-R006（人机工程学）：** `gan try` 是同一查询在运行器层面的拼写方式（与 `gan lsc try` 相同）；无目标（或使用 `--help`）时输出技能指南——用法、JSON 契约、建议种类以及代理循环。退出码在 `ok` 时为 0，否则为 1，以便在脚本中形成裁决链。

## Rationale

将沙箱构建到 `lsc` 中可保持一个统一的 AI 面：使用 `lsc` 进行检查、阅读文档和查找引用的智能体，也通过它进行学习和验证。文本错误模式有意保持简单——它们的存在是为了教授惯用法，而非解析 Python——而对真实符号表进行编辑距离搜索，正是让建议可信而非幻觉的关键所在。

## 向后兼容性

增量式。

## Security and Determinism

The sandbox executes user-supplied code with the project interpreter —
the same trust boundary as `gan run`/`gan test`. The timeout bounds
wall clock, not capability; verdict output is deterministic apart from
the executed program's own behavior.

## 工具与 AI 使用

Agents SHOULD 先将生成的代码通过 `gan lsc try` 路由，再写入项目，将 `did_you_mean` 建议视为要应用的修正，并在不需要执行时使用 `--no-run`。  
循环：生成 → `try` → 应用建议 → `try` → 写入。

## 已拒绝的替代方案

### 长期存在的沙箱服务器

状态会累积并偏离项目；每次查询使用全新的临时目录，可保持裁决可重现，且实现仅约300行。

### 编译器端的“你是否要找”

编译器保持小巧和确定性；模糊搜索属于工具链，在那里候选（符号、标识符）已被索引。

## 一致性

BDD 场景套件（给定源 / 当尝试 / 然后判定）MUST 涵盖：干净运行（stdout、值、单个 `main/0`）、运行崩溃与超时、`--no-run`；每一个“您是不是要找”类；每一个迁移模式；每一个实践提示；lint 传递；以及 R005 的静默保证（惯用模块、散文、注释、文档文本）。

## 变更历史

- 修订版 3，2026-08-03：`gan try` 作为第一类运行器的拼写。
- 修订版 2，2026-08-03：R004 扩展了 AI 懒惰模式；R005 字面掩码 + 静默保证；R006 技能风格帮助和退出码；BDD 一致性。
- 修订版 1，2026-08-03：初始版本。

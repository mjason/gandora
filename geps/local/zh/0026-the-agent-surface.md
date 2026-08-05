---
gep: 26
title: 智能体表面
description: AI会话的单次调用上下文 — lsc pack、批处理/简要查询以及gan agent入口点 — 使模型停止为查询循环支付令牌。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-05
updated: 2026-08-05
revision: 1
requires: [15, 25]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0026-the-agent-surface.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0026-the-agent-surface.md](../../0026-the-agent-surface.md)。

# GEP-0026: 代理表面

## 摘要

`gan lsc` 镜像了语言服务器的工作方式：一个问题，一个答案。对于编辑器中的用户而言，这样是正确的；但对于模型而言，它是一个代币熔炉（token furnace）——在写第一行代码之前，五到六次查询的探索阶段就会耗费数千个代币和同样多的往返次数。本 GEP 增加了面向智能体的交互界面：现有查询的批量和精简形式、一次调用的**上下文包（context pack）**，以及作为会话入口点的 `gan agent`。任何内容都不会写入用户的项目中。

## Motivation

根据反复进行的DeepSeek评估，模型的发现阶段（`list_symbols` ×3、`read_doc` ×4，……）每个任务消耗3–8k tokens和4–7次往返——几乎所有这些都用于重新获取每个项目中稳定的事实。裁决已经使*修正*成为一次调用（GEP-0025）；本GEP使*发现*成为一次调用。

## 规范

**GEP-0026-R001（批量目标）：** `gan lsc doc` 和 `gan lsc symbols` 接受多个目标；两个或以上目标时，`doc` 输出一个 JSON 数组，`symbols` 输出一个以名称作为键的对象。单个目标保持当前输出格式。

**GEP-0026-R002（简要模式）：** `doc` 的 `--brief` 选项将每个条目缩减为 `{label, head, summary}` —— 即规范（或标签）及其首条文档句子。详细信息仍可通过一次查询获取。

**GEP-0026-R003（上下文包）：** `gan lsc pack [Mod ...]` 返回一个 JSON 对象，包含：标准库函数列表（按模块列出名称）、每个项目模块及其公开头部（每行一个）、语言构造索引、规范速查表及简短备注列表、结果摘要（`ok`/错误/警告计数），以及指向更深层的 `next` 指针。指定模块名称时，其完整成员文档会加入 `deep` 字段。概览包 MUST（必须）保持提示词可容纳的大小（几千个 token），并且在项目不变的情况下具有确定性 —— 天然适合缓存。

**GEP-0026-R004（入口点）：** `gan agent` 输出一个 Markdown 简报 —— 包含工作循环（构建-结果交通灯、应用每个发现的规则、经验法则），后接渲染后的包；`--json` 则输出原始包。它是任何 AI 会话中推荐的首条命令，并且**不会**向项目写入任何文件 —— 作为生成上下文文件的替代方案，不污染项目。

## Rationale

该包是从已经存在的表面（`symbols`、`doc`、构造卡片、检查）编译而来的——没有第二个真相来源。一个静态生成的文件（`llms.txt`）被否决了：它会污染用户项目并变得过时；`gan agent` 可以根据实时事实按需生成相同的内容。

## 一致性

测试必须覆盖：multi-target doc arrays 和 symbols objects；brief entry shape；pack top-level keys 和确定性；`pack Mod` 深度文档；`gan agent` 打印循环文本以及 pack 和当 `gan-lsc` 缺失时带有清晰消息的降级。

## 变更历史

- 修订版本 1，2026-08-05：初始版本。

---
gep: 24
title: 测试工具
description: 使用标准测试断言的 `tests/*.gan` 文件，与项目一起编译，并由 pytest 执行——一条 `gan test` 命令可同时运行 doctests 和测试文件。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-03
updated: 2026-08-03
revision: 1
requires: [7, 10, 13]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0024-the-test-tool.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0024-the-test-tool.md](../../0024-the-test-tool.md)。

# GEP-0024: 测试工具

## Abstract

`gan test` 是唯一的测试命令：它运行每一个 `@example` doctest (GEP-0007)，然后运行 `tests/*.gan` 中的每一个 `test_*` 函数，这些函数与项目源代码一起编译，并由项目解释器下的 pytest 执行。断言是普通的 std 函数（`Test.assert_eq` 等），在失败时引发异常——pytest 将异常转化为报告。没有发明新的测试运行器；没有引入新的运行时。

## Motivation

Doctests 记录了正常路径；真正的测试套件需要边缘情况、负面情况和不变性。该语言具备这些要素——模块编译为纯 Python 函数，pytest 可以发现 `test_*`——但没有一个公认的惯例。将这一惯例正式化后，每个项目（包括标准库）都能享受相同的单命令体验，并让代理能够像编写代码一样编写测试。

## 范围

项目级测试。覆盖率、fixture/参数化以及测试选择标志仍由pytest自身处理（编译后的树是普通的pytest输入）；基于属性的测试是未来工作。

## Specification

**GEP-0024-R001:** `gan test`（以及 `ganc test`）按顺序运行：已编译项目的每个 `@example` doctest；然后，当存在 `tests/` 目录时，每个 `tests/*.gan` 文件——与项目源代码一起编译（完整模块解析：项目模块、标准库、已安装包）到 `.gandora/tests/` 中，然后使用 `PYTHONPATH` 上的 doctest 缓存运行 `python -m pytest .gandora/tests -q`。当任一阶段失败时，退出码为非零。缺少 pytest 时，报告 `uv add --dev pytest` 作为补救措施。

**GEP-0024-R002:** 测试模块是普通的 Gandora 模块，其 `test_*` 公共函数即为测试（应用 pytest 发现机制）。`Test`（gandora-std）提供断言系列：`assert_eq`、`assert_true`、`assert_false`、`assert_nil`、`assert_raises`、`assert_contains`——每个都会引发异常，消息中命名期望值和实际值。测试文件遵循所有语言规则（lint、fmt、对公共辅助函数有意义的规范）。

**GEP-0024-R003:** `tests/` 从不发布：它在源代码根目录之外，因此 `gan build` 和打包会忽略它；`.gandora/tests` 是构建输出。

## 理由

将测试*与*项目源代码一起编译，使它们获得与被测代码相同的模块解析——无需消费者项目脚手架，无需单独的虚拟环境。使用 pytest 作为执行器，可免费获得故障用户体验、过滤（`-k`）和 CI 集成，而测试本身则保持纯粹的 Gandora。

## 向后兼容性

新增性；没有 `tests/` 目录的项目，如同之前一样，只看到文档测试。

## Security and Determinism

Tests execute project code under the project interpreter — the
existing `gan run`/`gan test` trust boundary.

## 工具与AI使用

代理（Agents）SHOULD 在每次非平凡变更旁添加一个 `tests/*.gan` 文件，断言（assert）doctests 未覆盖的边缘情况，并将红色的 `gan test` 视为停止信号。沙箱像任何标准模块一样教授 `Test`（`gan lsc doc Test.assert_eq`）。

## 被拒绝的替代方案

### 定制测试运行器

pytest 的发现、报告和生态系统为开发组中的一个依赖项提供了支持；而定制运行器则需要重新实现这三者。

### ExUnit 风格的 `test "name" do` 宏

宏 DSL 在增加价值之前先增加了表面复杂度；命名函数已经具备可发现性、可搜索性和可检查性。如果出现需求，则重新审视。

## 符合性

测试 MUST 覆盖：通过和失败的 `tests/*.gan` 经过 `gan test` 运行后的退出码；在测试中对项目模块和标准模块的模块解析；missing-pytest 消息；以及每个 `Test` 断言的通过和失败路径。

## 变更历史

- 修订版 1, 2026-08-03: 初始版本。

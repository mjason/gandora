---
gep: 24
title: 测试工具
description: tests/*.gan 带有标准测试断言，与项目一起编译并由 pytest 执行——一个 `gan test` 会同时运行 doctest 和测试文件。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Tooling
created: 2026-08-03
updated: 2026-08-03
revision: 2
requires: [7, 10, 13]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0024-the-test-tool.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0024-the-test-tool.md](../../0024-the-test-tool.md)。

# GEP-0024: 测试工具

## 摘要

`gan test` 是唯一的测试命令：它运行每一个 `@example` 文档测试（GEP-0007），然后运行 `tests/*.gan` 中的每一个 `test_*` 函数，这些函数与项目源代码一起编译，并由项目解释器下的 pytest 执行。断言是普通的 std 函数（`Test.assert_eq` 等），在失败时引发异常——pytest 将异常转换为报告。没有发明新的测试运行器，也没有引入新的运行时。

## 动机

文档测试（Doctests）记录了正常路径；但真正的测试套件还需要边界情况、负面情况和不变量。该语言已具备相关组件——模块会被编译成纯 Python 函数，pytest 能发现 `test_*` ——但缺少一个公认的约定。将这一约定正式化，就能让每个项目（包括标准库）都拥有相同的“一条命令”体验，并让代理能够像编写代码一样编写测试。

## 范围

项目级测试。覆盖率、fixtures/参数化以及测试选择标志保留在pytest自身的表面（编译后的树是普通的pytest输入）；基于属性的测试是未来的工作。

## 规范

**GEP-0024-R001:** `gan test`（以及 `ganc test`）按顺序运行：首先，编译项目中所有 `@example` 文档测试；然后，当存在 `tests/` 目录时，将所有 `tests/*.gan` 文件与项目源代码一起编译（完整模块解析：项目模块、标准库、已安装包）到 `.gandora/tests/` 中，再运行 `python -m pytest .gandora/tests -q`，并将文档测试缓存置于 `PYTHONPATH` 上。只要任一阶段失败，退出码即为非零。若缺少 pytest，则报告并给出 `uv add --dev pytest` 的补救措施。

**GEP-0024-R002:** 测试模块是普通的 Gandora 模块，其 `test_*` 公共函数即为测试用例（pytest 发现机制适用）。`Test`（gandora-std）提供断言系列：`assert_eq`、`assert_true`、`assert_false`、`assert_nil`、`assert_raises`、`assert_contains`、`assert_raise/2`（有类型）、`assert_in_delta/3`、`flunk/1`——每个断言均会抛出异常，并附带标明期望值与实际值的消息。

**GEP-0024-R004 (ExUnit 接口，修订版 2):** `use Test` 引入宏语法：`test "名称" do ... end` 定义 `test_<slug>`；`describe "前缀" do ... end` 为所有内部测试的名称添加前缀；`assert 表达式` 解构比较，使得失败时能指出双方的值（`assert a == b` 报告左右值；`in` 断言成员关系）；`refute 表达式` 是其否定形式。这些宏编译为 R002 函数——pytest 看到的是普通的定义。

**GEP-0024-R003:** `tests/` 从不发布：它位于源码根目录之外，因此 `gan build` 和打包过程会忽略它；`.gandora/tests` 是构建输出。

## 理由

与项目源码*一起*编译测试，使测试具有与被测代码相同的模块解析——无需消费者项目脚手架，无需单独的虚拟环境。pytest作为执行器，免费提供失败UX、过滤（`-k`）和CI集成，而测试本身保持纯Gandora。

## 向后兼容性

追加的；没有`tests/`的项目只看到doctests，与之前一样。

## 安全性与确定性

测试在项目解释器下执行项目代码——即现有的 `gan run`/`gan test` 信任边界。

## 工具与AI使用

代理人 SHOULD 在每次非平凡变更旁添加一个 `tests/*.gan` 文件，断言边缘情况（doctests 未覆盖的），并将红色 `gan test` 视为停止信号。沙箱像任何标准模块一样教授 `Test`（`gan lsc doc Test.assert_eq`）。

## 被拒绝的替代方案

### 定制的测试运行器

pytest 的发现、报告和生态系统仅用于开发组中的一个依赖项；定制的运行器将重新实现这三者。

### 超越 ExUnit 的定制断言 DSL

ExUnit 的表面是 Elixir 开发者熟悉的词汇；追求对等，而非创新。

## 符合性

测试 MUST 涵盖：通过 `gan test` 运行一个通过的和一个失败的 `tests/*.gan` 的退出码；测试中项目模块与标准模块的模块解析；缺失 pytest 的消息；以及每个 `Test` 断言的通过和失败路径。

## 变更历史

- 修订版2，2026-08-04：R004 — GEP-0002 rev 2宏套件上的ExUnit表面（test/describe/assert/refute宏）。
- 修订版1，2026-08-03：初始版本。

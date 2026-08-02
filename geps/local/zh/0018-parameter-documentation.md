---
gep: 18
title: 参数文档
description: @param 属性——经过验证的、可翻译的每个参数的描述文本，用于生成 Elixir 的 "## Parameters" 部分并提供签名帮助。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Language
  - Tooling
created: 2026-08-02
updated: 2026-08-02
revision: 1
requires: [7, 15, 17]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0018-parameter-documentation.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0018-parameter-documentation.md](../../0018-parameter-documentation.md)。

# GEP-0018: 参数文档

## 摘要

`@param` 用于记录下一个定义中的某一个参数：

```elixir
@param name, "String that represents the name of the person."
@param locale, "BCP 47 tag used for casing rules."
@spec split(string(), string()) :: list(string())
def split(name, locale \\ "en") do
```

参数名会在编译时根据子句头部进行校验——参数文档不会过时。渲染器据此生成 Elixir 风格的 `## Parameters` 章节，翻译通过 `@param_trans` 传递，语言服务器在签名帮助中提供当前参数自身的描述。`@doc` 内手写的 `## Parameters` 章节仍然是合法的 Elixir 代码，且不会被解析；属性是结构化的路径，惯例则是兼容的方式。

## 动机

参数文档是文档中人类和工具*按位置*需要的唯一切片：悬停需要一节，签名帮助需要光标下参数的精确描述，`gan lsc` 的消费者需要将其作为数据。散文约定无法被验证或被寻址；专用通道可以——同样的判断赋予了 `@example` 自己的属性 (GEP-0007)。

## 范围

`@param` 和 `@param_trans` 属性，它们的验证、渲染和工具接口，以及命名的 `@spec` 参数。结构体字段和模块属性的文档注释不在范围内。

## 规范

**GEP-0018-R001:** `@param name, "text"` 位于定义之前（相对于 `@doc`、`@spec` 和 `@example` 的顺序任意），并记录紧随其后的定义组的参数 `name`。`@param` MAY 重复，每个参数一次，子集和顺序任意；文本为 Markdown 散文。重复名称或将 `@param` 放在没有后续定义的位置会导致编译错误。

**GEP-0018-R002:** 每个 `@param` 名称 MUST 作为变量出现在紧随其后的定义的至少一个子句头中（包括默认值，排除以 `_` 为前缀的名称）；否则编译错误会指出该参数和定义。没有 `@param` 的参数仅是不被记录——不要求覆盖。

**GEP-0018-R003:** `@param_trans name, locale: "text"` 为先前声明的 `@param name` 添加翻译，遵循 GEP-0007 的 locale 规则（仅 BCP 47 键，仅散文）。为未声明的参数声明翻译会导致编译错误。

**GEP-0018-R004:** 渲染器从属性生成该部分：`gan doc` 和 hover 附加 `## Parameters`（或其 locale 的翻译输出的标题）列出 `- name: text` 按子句头顺序；生成的 Python 文档字符串携带默认 locale 的部分，位于散文之后、示例之前。带有 `@param` 属性的定义 SHOULD NOT 再手动编写 `## Parameters` 部分；渲染器不会去重。

**GEP-0018-R005:** 文档表面以结构方式携带数据：`gandora_core.doc` 返回 `params: [{name, entries: {locale: text}}]` 按子句头顺序，`gan lsc doc` 传递它，签名帮助（GEP-0015-R010）将每个参数的默认 locale 描述附加到其 `ParameterInformation`。

**GEP-0018-R006:** `@spec` 头 MAY 在 Elixir 的形式中命名其参数：`@spec split(name :: string(), locale :: string()) :: list(string())`。名称是信息性的（在渲染的规范中显示）；类型按 GEP-0017 编译不变。命名参数名称与子句头位置的变量冲突不是错误——子句头在工具标签中获胜。

## 理由

验证的名称是此功能的关键：重命名参数会导致构建失败，而不是静默地使其文档孤立。从属性生成 Elixir 部分保持了单一事实来源，同时精确匹配生态系统的呈现约定。解析手写部分被拒绝，原因与 doctests 相同：散文提取在本地化与格式差异上会出错。

## Backwards Compatibility

添加式。手写的 `## Parameters` 部分保持原样工作。

## 安全性与确定性

属性是编译时数据；渲染是确定性的。

## 工具与AI使用

代理（Agents）SHOULD 对每个非显而易见的公共函数参数输出 `@param`，并从 `gan lsc doc` 读取参数文档，而不是解析Markdown。

## 被拒绝的替代方案

### 解析 `## Parameters` 约定

无法验证，且在不同语言环境和列表样式下脆弱——
GEP-0007 的反解析决定直接适用。

### `@spec` 内的参数文档

用散文使类型通道过载，并违反了 GEP-0017 规则，
即每个规范元素必须是一个类型表达式。

## 一致性

测试 MUST 涵盖：渲染为文档字符串、`gan doc` 和悬停显示；子句标题顺序列表；未知名称、重复名称和孤立翻译错误；`@param_trans` 区域设置查找；显示活动参数文本的签名帮助；以及命名 `@spec` 参数与未命名参数编译结果相同。

## 变更历史

- 修订版本 1, 2026-08-02: 初始版本。

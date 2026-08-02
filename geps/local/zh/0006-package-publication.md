---
gep: 6
title: 包发布
description: 将 Gandora 包作为普通的 PyPI wheels 发布——编译好的自包含 Python 加上附带的用于编译时宏的源代码——无运行时依赖。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Packages
  - Interop
created: 2026-08-01
updated: 2026-08-01
revision: 2
requires: [1, 2]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0006-package-publication.md
source-revision: 2
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0006-package-publication.md](../../0006-package-publication.md)。

# GEP-0006: 包发布

## 摘要

Gandora 包即是一个普通的 Python wheel。它包含编译后的、自包含的 Python 模块（可供任何 Python 使用者使用）、一个静态的 `gandora.toml` 标记，以及原始的 `.gan` 源文件。对已发布包的运行时调用就是普通的 Python 导入——不存在 Gandora 运行时包，这与 GEP-0001-R002 一致。宏是唯一的编译时接口：使用者的编译器通过读取虚拟环境中的标记（从不导入 Python）来发现已安装的包，并在使用者自身的编译过程中从提供的源文件中展开这些宏。

## Motivation

发行版不得分叉生态系统：数据科学团队应该能够 `uv add` 一个由 Gandora 编写的包，而无需知道 Gandora 的存在；同时，Gandora 的消费者应该从同一个工件中获得完整的表面——函数 *和* 宏。核心判断：`pyproject.toml`/`uv` 完全拥有依赖管理，编译器仅从锁定环境中读取静态文件。

## Scope

包项目布局、Wheel 内容契约、标记格式、消费者端解析以及无运行时保证。专用的 PEP 517 构建后端、跨包的 `.gani` 风格的编译接口、版本冲突诊断以及传递性包宏依赖被推迟。

## Terminology

- **Package project**: 一个其 `gandora.jsonc` 设置了 `"package": true` 的项目。
- **Marker**: 标识 wheel 内目录为 Gandora 构建的 `gandora.toml` 文件。
- **Shipped sources**: 复制到 wheel 中 `_gan/` 目录下的 `.gan` 文件。

## 规范

### 包项目与 wheel 内容

**GEP-0006-R001：** `gandora.jsonc` 接受一个可选的布尔字段 `package`（默认值为 `false`），对 GEP-0001-R019 的字段集进行补充。`gan init --package <name>` MUST 创建一个包项目，其 `pyproject.toml` 使用标准 Python 构建后端，配置为打包编译输出目录，从而 `uv build` 生成 wheel 时无需特定于 Gandora 的构建工具。

**GEP-0006-R002：** 在包项目中，`gan build` MUST 额外为每个顶层输出包目录写入：
- 标记文件 `<package>/gandora.toml`；
- 每个模块的源代码，位于 `<package>/_gan/<python-path>.gan`，其中 `<python-path>` 是该模块的 GEP-0001-R014 路径。

**GEP-0006-R003：** 因此，wheel 包含且 MUST NOT 要求更多内容：编译后的 `.py` 模块（每个模块自包含，符合 GEP-0001-R002）、标记文件以及附带的源代码。仅宏模块提供源代码和标记条目，但不提供 `.py` 文件（GEP-0002-R009）。

**GEP-0006-R004：** 标记文件是 TOML 格式，具有以下模式（版本 1）：

```toml
schema = 1
compiler = "<gan version>"

[[modules]]
name = "AcmeText.Slug"
python = "acme_text/slug.py"
source = "acme_text/_gan/acme_text/slug.gan"
```

路径相对于 site-packages 根目录。仅宏模块省略 `python`。消费者 MUST 拒绝其 `schema` 未知的标记文件。

### 消费者解析

**GEP-0006-R005：** 对已发布包（`alias`、限定调用）的运行时引用，编译为普通的 Python 导入语句，与本地模块的处理方式完全相同（GEP-0001-R017）；编译器不执行发现操作，导入在运行时针对已安装的 wheel 进行解析。

**GEP-0006-R005A：** 标记模块名称对运行时解析也具有权威性：当引用的模块不属于构建自身的模块时，编译器会先从已安装的标记文件中解析其导入路径，然后才退回到机械的 GEP-0001-R014 映射。项目模块优先于已安装的名称。

**GEP-0006-R006：** 当 `require` 或 `import` 指定的模块在项目源代码中未找到时，编译器 MUST 搜索项目环境的 site-packages 目录以查找标记文件，并在名称匹配时解析附带的源代码以收集其宏（适用 GEP-0002-R006 可见性规则）。搜索 MUST 仅读取静态文件；MUST NOT 导入或执行 Python 代码。

**GEP-0006-R007：** 在 v0 中，包宏 MUST 是自包含的：附带的宏模块本身不得 `require` 其他模块。展开发生在消费者的编译过程中，使用消费者的卫生上下文和限制；相同的包版本 MUST 产生相同的展开结果。

**GEP-0006-R008：** 一个 `require` 的模块既未在本地找到，也未在已安装的标记文件中找到，MUST 产生一条诊断信息，指明该模块及搜索的环境。

### 无运行时保证

**GEP-0006-R009：** 发布和消费包 MUST NOT 引入任何对 Gandora 的运行时依赖：无共享支持包，无导入钩子，无加载器。每个模块的语义辅助函数保持内联在生成的每个文件中（GEP-0001-R002）。纯 Python 消费者 MAY 使用该 wheel，无需了解 Gandora；从已安装的 wheel 中删除所有 `.gan` 文件和标记文件 MUST NOT 改变其运行时行为。

## Rationale

发布编译后的 Python 而不是在安装时编译，使得 `uv add` 保持即时，保持包可在纯 Python 中使用，并消除了消费者的编译器与包作者的编译器之间的任何版本耦合——wheel 在任何地方表现一致，因为它*就是*其行为本身。

宏不能是运行时产物（它们在运行时不存在，GEP-0002-R009），因此唯一忠实的分发方式是它们的源代码。发布 `.gan` 源代码用一点解析时间换取零新格式，并为未来 GEP 中的编译接口格式留出空间。

从 site-packages 读取标记保持一个硬性规则：发现过程从不执行包代码——安装状态是数据。

## 向后兼容性

增量式。非包项目不受影响。标记模式已版本化以便未来演进；`_gan/` 在包输出中被保留。

## 安全性与确定性

标记扫描从项目自身环境中读取静态文件，从不执行代码。因此，一个恶意的 wheel 最多只能贡献宏 *source*，它在 GEP-0002-R003 的确定性沙箱中展开——它不能在编译时执行 I/O。已安装 wheel 的运行时行为是普通的 Python，可在 wheel 本身中审查。

## 工具与AI使用

发布包的代理：`gan init --package`，编写模块，`gan build`，`uv build`，`uv publish`。消费包的代理：`uv add`，然后像本地包一样`alias`/`require`。代理不应将包代码进行供应商化或包装——wheel已经是接口。

## 被否决的备选方案

### 安装时编译（仅分发 .gan，通过 PEP 517 钩子构建）

这样做可以保证源码/二进制一致性，但会使安装过程需要编译器，破坏纯 Python 消费者的兼容性，并将每个消费者与特定的编译器版本耦合。已否决作为默认方案；后续仍可添加源码构建后端以满足原生扩展风格的需求。

### 共享运行时 wheel 用于辅助函数

可以消除内联的几十行代码重复，但代价是丧失整个无运行时属性，并在不同编译器编译的包之间引入版本偏差。已否决；内联正是实现 R009 的机制。

### 序列化宏 IR 而非分发源码

消费者编译更快且隐藏源码，但当前需要稳定的 IR 格式。推迟到编译接口 GEP 再做；标记的版本化模式保留了可能性。

## 未决问题

此修订版本无。

## 符合性

测试 MUST 涵盖：包的脚手架结构；标记和已发布源代码的输出（包括仅宏模块）；从 site-packages 标记解析消费者宏，无需任何本地副本；R008 诊断；以及一个端到端的消费者运行，其生成的输出导入了已安装的包，并且不包含任何 Gandora 导入。

## 修订历史

- 修订版 2，2026-08-02：新增 R005A — 标记名称控制运行时解析（与 GEP-0010 一起）。
- 修订版 1，2026-08-01：初始版本。

---
gep: 6
title: 包发布
description: 将Gandora包作为普通的PyPI wheels发布——编译的自包含Python加上附带编译时宏的源码——无运行时依赖。
author: MJ
status: Accepted
type: Standards Track
areas:
  - Packages
  - Interop
created: 2026-08-01
updated: 2026-08-01
revision: 1
requires: [1, 2]
replaces: []
superseded-by: null
resolution: null
language: zh-CN
source: ../../0006-package-publication.md
source-revision: 1
translation-status: Current
---

> 本文件是非规范性翻译，仅供评审参考；规范性文本为英文原文 [0006-package-publication.md](../../0006-package-publication.md)。

# GEP-0006: 包发布

## Abstract

Gandora 包是一个普通的 Python wheel。它包含编译后的、自包含的 Python 模块（可供任何 Python 使用者使用）、一个静态的 `gandora.toml` 标记以及原始的 `.gan` 源文件。对已发布包的运行时调用是普通的 Python 导入——不存在 Gandora 运行时包，这与 GEP-0001-R002 一致。宏是唯一的编译时接口：使用者的编译器通过读取虚拟环境中的标记（从不导入 Python）来发现已安装的包，并在使用者自己的编译过程中从附带的源文件中展开它们的宏。

## Motivation

分发不得分裂生态系统：数据科学团队应能通过 `uv add` 添加 Gandora 编写的包，而无需知道 Gandora 的存在；同时，Gandora 用户应从同一个工件中获得完整的表面——函数*和*宏。核心判断：`pyproject.toml` / `uv` 完全拥有依赖管理，而编译器只从锁定环境中读取静态文件。

## 范围

包项目布局、wheel 内容合约、标记格式、消费者端解析以及无运行时保证。专用的 PEP 517 构建后端、跨包 `.gani` 风格的编译接口、版本冲突诊断以及传递包宏依赖被推迟。

## 术语

- **包项目 (Package project)**：其 `gandora.jsonc` 中设置 `"package": true` 的项目。
- **标记文件 (Marker)**：用于标识 wheel 内某个目录为 Gandora 构建的 `gandora.toml` 文件。
- **分发的源码 (Shipped sources)**：复制到 wheel 中 `_gan/` 目录下的 `.gan` 文件。

## 规范

### 包项目与Wheel内容

**GEP-0006-R001:** `gandora.jsonc` 接受一个可选的布尔字段 `package`（默认值为 `false`），补充了 GEP-0001-R019 的字段集。`gan init --package <name>` MUST 搭建一个包项目，其 `pyproject.toml` 使用标准 Python 构建后端，配置为打包编译输出目录，以便 `uv build` 生成不依赖 Gandora 特定构建工具的 wheel。

**GEP-0006-R002:** 在包项目中，`gan build` MUST 额外为每个顶层输出包目录写入：
- 标记文件 `<package>/gandora.toml`；
- 每个模块的源代码位于 `<package>/_gan/<python-path>.gan`，其中 `<python-path>` 是该模块的 GEP-0001-R014 路径。

**GEP-0006-R003:** 因此，wheel 包含且 MUST NOT 要求更多：编译后的 `.py` 模块（每个模块自包含，符合 GEP-0001-R002）、标记文件以及分发的源代码。仅宏模块贡献源代码和标记条目，但不包含 `.py`（GEP-0002-R009）。

**GEP-0006-R004:** 标记文件是 TOML 格式，采用以下模式（版本 1）：

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

**GEP-0006-R005:** 对已发布包的运行时引用（`alias`、限定调用）编译为普通的 Python import，与本地模块相同（GEP-0001-R017）；编译器不执行发现，import 在运行时根据已安装的 wheel 解析。

**GEP-0006-R006:** 当 `require` 或 `import` 引用一个在项目源代码中未找到的模块时，编译器 MUST 在项目环境的 site-packages 目录中搜索标记文件，并在名称匹配时解析分发的源代码以收集其宏（适用 GEP-0002-R006 可见性规则）。搜索 MUST 仅读取静态文件；它 MUST NOT 导入或执行 Python。

**GEP-0006-R007:** 包宏在 v0 中 MUST 是自包含的：分发的宏模块本身不能 `require` 其他模块。宏展开在消费者的编译过程中进行，使用消费者的卫生上下文和限制；相同的包版本 MUST 产生相同的展开。

**GEP-0006-R008:** 一个 `require` 的模块既未在本地找到，也未在已安装的标记文件中找到，MUST 产生一个诊断信息，指出该模块和搜索的环境。

### 无运行时保证

**GEP-0006-R009:** 发布和消费包 MUST NOT 引入任何对 Gandora 的运行时依赖：无共享支持包、无导入钩子、无加载器。每个模块的语义辅助函数保持内联在生成的每个文件中（GEP-0001-R002）。纯 Python 消费者 MAY 在不了解 Gandora 的情况下使用 wheel；从已安装的 wheel 中删除所有 `.gan` 文件和标记文件 MUST NOT 改变其运行时行为。

## 理由

通过分发预编译的 Python 包而非在安装时编译，可使 `uv add` 保持即时响应，保持该包可从普通 Python 使用，并消除消费者编译器与包作者编译器之间的任何版本耦合——因为 wheel 的行为就是其自身，所以它在所有地方行为一致。

宏不能是运行时工件（它们在运行时不存在，GEP-0002-R009），因此唯一忠实的分发方式是它们的源代码。分发 `.gan` 源文件用少量解析时间换取零新格式，并为未来 GEP 中编译接口格式留出了空间。

从 site-packages 读取标记坚守一条硬规则：发现过程绝不执行包代码——安装状态是数据。

## 向后兼容性

增量式。非包项目不受影响。标记模式已版本化以便未来演进；`_gan/` 在包输出中保留。

## 安全性与确定性

标记扫描从项目自身环境中读取静态文件，且从不执行代码。因此，恶意的wheel最多只能贡献宏*源代码*，该宏在GEP-0002-R003的确定性沙箱中展开——它不能在编译时执行I/O。已安装wheel的运行时行为是普通的Python，可在wheel本身中审查。

## 工具与AI使用

发布包的代理：`gan init --package`，编写模块，`gan build`，`uv build`，`uv publish`。使用包的代理：`uv add`，然后像本地模块一样使用 `alias`/`require`。代理 SHOULD NOT vendor 或 wrap 包代码——wheel 已经是接口。

## 被拒绝的替代方案

### 安装时编译（仅分发 .gan，通过PEP 517钩子构建）

能保证源码与二进制的一致性，但会导致安装需要编译器，破坏纯Python用户，并且将每个用户与某个编译器版本耦合。作为默认方案被拒绝；后续仍可添加源码构建后端以满足类似原生扩展的需求。

### 为辅助函数提供共享运行时wheel

能够去重几十行内联代码，但代价是丧失整个无运行时属性，并引入不同编译器编译的包之间的版本偏差面。被拒绝；内联正是使R009成为可能的机制。

### 序列化宏IR替代分发源码

能加快用户编译速度并隐藏源码，但需要立即确定稳定的IR格式。推迟到编译接口GEP时再处理；标记的版本化模式保留了未来变化的可能性。

## 开放问题

此版本无。

## 一致性

测试MUST覆盖：包脚手架形状；标记和分发源码的发出（包括一个仅宏模块）；来自站点包标记的消费者宏解析，且无任何本地副本；R008诊断；以及一个端到端消费者运行，其生成的输出导入了已安装的包，且不包含任何Gandora导入。

## 变更历史

- Revision 1, 2026-08-01: 初始版本。

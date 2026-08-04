# 命令行界面与编辑器

## `gan` — 任务运行器

```console
gan build [--strict]     # the verdict + compile to outDir
gan run <file> [args]    # compile to .gandora/cache and execute
gan test                 # @example doctests + tests/*.gan (pytest under the hood)
gan fmt [--check|--diff] # canonical formatting; `-` reads stdin
gan doc Enum.take        # docs in the terminal (+ --locale zh-CN)
gan repl                 # interactive; state carries across lines
gan exec "1 |> to_string()"
gan init my-app          # new uv-compatible project
```

`gan run` 使用 *项目* Python 执行 — 当存在 `.venv/bin/python` 时 — 因此互操作性能够看到你的真实依赖项。未知子命令将委托给 `gan-<name>` 插件，然后到 `ganc`。

## `ganc` — 第0阶段编译器

底层的Rust编译器：`ganc build`、`ganc run`、`ganc test`、`ganc expand`（宏输出）、`ganc compile`。这是低级管道命令——它不根据判定结果进行门控；`gan` 是执行门控的高级命令。

## `gan lsc` — 语言智能输出为JSON

工具链知道的每一个事实，每次查询返回一个JSON值——专为代理和shell设计：

```console
gan lsc check --root .          # the verdict: {ok, clean, diagnostics, suggestions}
gan lsc doc Enum.map            # docs: spec, prose, translations, examples
gan lsc doc for                 # language-construct cards (for, spec, with, ...)
gan lsc symbols Enum            # every function with rendered heads
gan lsc references Stats.mean   # project-wide call sites
gan lsc compile src/x.gan       # the generated Python, as text
gan lsc expand src/x.gan        # post-macro AST
gan lsc pydoc numpy.array       # Python-side docs via jedi
```

## 语言服务器

`gan-lsp` 通过 stdio 使用 LSP 协议：提供每次编辑时的诊断、悬停提示（文档、类型、编译后的递归形态 ♻/⚠、本地化参数表）、跳转到定义、引用、重命名、工作区符号、补全、签名帮助、快速修复（`@allow` 插入、`_` 前缀）以及整个文档的格式化。

**VS Code**：从[发布页面](https://github.com/mjason/gandora/releases)安装 `gandora-<version>.vsix` — LSP 客户端附带命令面板（Run File、Show Compiled Python 并支持保存时刷新、Expand Macros、Build、Test、REPL）、代码片段、带问题匹配器的任务。

## 文档语言

文档在源文件中是双语的（`@doc` + `@doc_trans`）。你所*看到*的内容取决于你的偏好：`gandora.local.jsonc`（每个开发者自己的，被 git 忽略）中设置 `{"docLocale": "zh-CN"}` → 接着是 `GAN_DOC_LOCALE` 环境变量 → 最后是英文。`--locale all` 会显示所有语言。

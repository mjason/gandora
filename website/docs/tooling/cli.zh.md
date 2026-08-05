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
gan agent [--json]       # the AI-session briefing: working loop +
                         # context pack in one output — no files written
```

`gan run` 使用*项目*的 Python 执行——当存在 `.venv/bin/python` 时——因此互操作能够看到你的实际依赖项。未知子命令会委托给 `gan-<name>` 插件，然后委托给 `ganc`。

## `ganc` — 第0阶段编译器

底层Rust编译器：`ganc build`、`ganc run`、`ganc test`、
`ganc expand`（宏输出）、`ganc compile`。底层命令——它不
根据裁决结果进行门控；`gan`是执行门控的高层命令。

## `gan lsc` — 语言智能以 JSON 形式呈现

工具链知道的每一个事实，每个查询一个 JSON 值 — 为代理和 shell 构建：

```console
gan lsc check --root .          # the verdict: {ok, clean, diagnostics, suggestions}
gan lsc pack [Mod ...]          # ONE-call agent context: std lists, project
                                # signatures, construct index, verdict summary
gan lsc doc Enum.map Enum.take --brief   # many targets, one line each
gan lsc doc Enum.map            # docs: spec, prose, translations, examples
gan lsc doc for                 # language-construct cards (for, spec, with, ...)
gan lsc symbols Enum            # every function with rendered heads
gan lsc references Stats.mean   # project-wide call sites
gan lsc compile src/x.gan       # the generated Python, as text
gan lsc expand src/x.gan        # post-macro AST
gan lsc pydoc numpy.array       # Python-side docs via jedi
```

## 语言服务器

`gan-lsp` 通过 stdio 实现 LSP 协议：每次编辑时提供诊断信息；悬停时显示文档、类型、编译后的递归形状（♻/⚠）、本地化参数表；支持跳转到定义、查找引用、重命名、工作区符号、补全、签名帮助、快速修复（`@allow` 插入、`_` 前缀）以及整文档格式化。

**VS Code**：从 [发布页面](https://github.com/mjason/gandora/releases) 安装 `gandora-<version>.vsix` —— 包含 LSP 客户端与命令面板（运行文件、显示编译后的 Python 代码并支持保存时刷新、展开宏、构建、测试、REPL）、代码片段、带问题匹配器的任务。

## 文档语言

文档在源头上是双语的（`@doc` + `@doc_trans`）。您*看到*的内容取决于您的偏好：`gandora.local.jsonc`（每个开发者，被git忽略）`{"docLocale": "zh-CN"}` → `GAN_DOC_LOCALE` 环境变量 → 英语。`--locale all` 显示所有语言。

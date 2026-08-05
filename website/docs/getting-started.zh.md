# 入门指南

## 安装

Gandora 以五个 PyPI 包的形式发布；一个 `uv` 工具安装命令即可提供整个工具链（`gan` 以及其委托的 `ganc` 编译器）：

```console
uv tool install gandora-tool     # gan — 任务运行器（附带 ganc）
```

为获得编辑器支持，请安装语言服务器和 VS Code 扩展（`gandora-<version>.vsix` 来自 [GitHub releases](https://github.com/mjason/gandora/releases)）：

```console
uv tool install gandora-lsp      # gan-lsp + gan-lsc
```

## 第一个项目

```console
gan init my-app
cd my-app
gan run src/main.gan
```

`gan init` 创建一个与 `uv` 兼容的项目：`pyproject.toml` 管理依赖项，`.venv`（已预添加 `gandora-std`），`gandora.jsonc` 管理编译器配置，源代码位于 `src/` 目录，测试代码位于 `tests/` 目录。

```text
my-app/
├── gandora.jsonc        # {"source": ["src"], "outDir": "dist", ...}
├── pyproject.toml       # uv 管理的依赖项
├── src/
│   └── main.gan
└── tests/
```

## The loop

```console
gan build            # THE verdict: errors, warnings, advice, artifact
                     # verification — then compiled Python in dist/
gan run src/main.gan # compile to .gandora/cache and execute
gan test             # run every @example doctest + tests/*.gan
gan fmt src          # canonical formatting (--check for CI)
gan repl             # interactive, state carries across lines
```

`gan build` 在有错误存在时拒绝写入制品，采用的是重型编译器的方式——并且它打印的所有内容都在教导如何修复：

```console
$ gan build
error: src/main.gan:13: Name `totl` used when not defined — did you mean `total`?
practice: src/main.gan:3: Annotation coverage: missing @spec on: main ...
build aborted: errors in the verdict
```

## 你好，实际上很有用

```elixir
defmodule Main do
  @moduledoc "Word frequencies from the command line."

  @doc "Counts words in `text`, most frequent first."
  @spec frequencies(text :: string()) :: list(tuple(string(), integer()))
  @example """
      gan> Main.frequencies("the quick the lazy the")
      [('the', 3), ('quick', 1), ('lazy', 1)]
  """
  def frequencies(text) do
    text
    |> String.split()
    |> Enum.frequencies()
    |> Map.to_list()
    |> Enum.sort_by(fn {_w, n} -> -n end)
  end

  @spec main() :: nil
  def main() do
    args = Enum.drop($builtins.list($sys.argv), 1)
    IO.puts(frequencies(Enum.join(args, " ")))
  end
end
```

运行它，测试它，发布它：

```console
$ gan run src/main.gan the quick the lazy the
[('the', 3), ('quick', 1), ('lazy', 1)]
$ gan test
doctests: 1 module(s) checked, 0 failed
```

发布就是普通的 Python 打包流程：`gan build && uv build && uv publish` 会生成一个包含编译后 Python 的 wheel（同时包含你的 `.gan` 源码，以便下游宏可以正常工作）——消费者使用 `uv add` 安装它，无需涉及 Gandora 运行时。

## 后续步骤

- [指南](guide/modules.md) 逐步介绍语言各章节。
- [构建结果](tooling/build.md) 解释你的代理应遵循的交通灯。
- [标准库](reference/enum.md) 参考文档由相同的文档字符串生成，`gan doc` 和悬停显示。

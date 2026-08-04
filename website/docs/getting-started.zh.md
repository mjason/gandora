# 入门

## Install

Gandora 以五个 PyPI 包的形式发布；两个 `uv` 工具安装即可提供完整的工具链：

```console
uv tool install gandora-tool     # gan — 任务运行器
uv tool install gandora-lang     # ganc — 其委托的阶段 0 编译器
```

为了获得编辑器支持，请安装语言服务器和 VS Code 扩展（从 [GitHub releases](https://github.com/mjason/gandora/releases) 获取 `gandora-<version>.vsix`）：

```console
uv tool install gandora-lsp      # gan-lsp + gan-lsc
```

## 第一个项目

```console
gan init my-app
cd my-app
gan run src/main.gan
```

`gan init` 创建一个与 `uv` 兼容的项目：`pyproject.toml` 管理依赖和 `.venv`（预置了 `gandora-std`），`gandora.jsonc` 管理编译器配置，源文件位于 `src/` 目录，测试文件位于 `tests/` 目录。

```text
my-app/
├── gandora.jsonc        # {"source": ["src"], "outDir": "dist", ...}
├── pyproject.toml       # uv-managed dependencies
├── src/
│   └── main.gan
└── tests/
```

## 循环

```console
gan build            # THE verdict: errors, warnings, advice, artifact
                     # verification — then compiled Python in dist/
gan run src/main.gan # compile to .gandora/cache and execute
gan test             # run every @example doctest + tests/*.gan
gan fmt src          # canonical formatting (--check for CI)
gan repl             # interactive, state carries across lines
```

`gan build` 拒绝在存在错误时写入工件，这是重型编译器的方式——它打印的所有内容都会教你如何修复：

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

发布是普通的 Python 打包流程：`gan build && uv build &&
uv publish` 产生一个编译后的 Python wheel（加上你的 `.gan`
源文件，以便下游宏能够工作）——消费者 `uv add` 它，无需涉及 Gandora 运行时。

## 下一步

- [指南](guide/modules.md) 逐步介绍语言。
- [构建状态](tooling/build.md) 解释你的代理应该遵循的交通灯。
- [标准库](reference/enum.md) 参考由相同的文档字符串生成，`gan doc` 和悬停显示。

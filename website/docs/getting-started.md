# Getting Started

## Install

Gandora ships as five PyPI packages; two `uv` tool installs give you
the whole toolchain:

```console
uv tool install gandora-tool     # gan — the task runner
uv tool install gandora-lang     # ganc — the stage-0 compiler it delegates to
```

For editor support, add the language server and the VS Code extension
(`gandora-<version>.vsix` from the
[GitHub releases](https://github.com/mjason/gandora/releases)):

```console
uv tool install gandora-lsp      # gan-lsp + gan-lsc
```

## A first project

```console
gan init my-app
cd my-app
gan run src/main.gan
```

`gan init` creates a `uv`-compatible project: `pyproject.toml` owns
dependencies and the `.venv` (with `gandora-std` pre-added),
`gandora.jsonc` owns compiler configuration, sources live in `src/`,
tests in `tests/`.

```text
my-app/
├── gandora.jsonc        # {"source": ["src"], "outDir": "dist", ...}
├── pyproject.toml       # uv-managed dependencies
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

`gan build` refuses to write artifacts while errors exist, the
heavy-compiler way — and everything it prints teaches the fix:

```console
$ gan build
error: src/main.gan:13: Name `totl` used when not defined — did you mean `total`?
practice: src/main.gan:3: Annotation coverage: missing @spec on: main ...
build aborted: errors in the verdict
```

## Hello, actually useful

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

Run it, test it, ship it:

```console
$ gan run src/main.gan the quick the lazy the
[('the', 3), ('quick', 1), ('lazy', 1)]
$ gan test
doctests: 1 module(s) checked, 0 failed
```

Publishing is ordinary Python packaging: `gan build && uv build &&
uv publish` produces a wheel of compiled Python (plus your `.gan`
sources so downstream macros work) — consumers `uv add` it with no
Gandora runtime involved.

## Where next

- The [Guide](guide/modules.md) walks the language a chapter at a time.
- [The Build Verdict](tooling/build.md) explains the traffic light your
  agent should follow.
- The [Standard Library](reference/enum.md) reference is generated from
  the same docstrings `gan doc` and hover show.

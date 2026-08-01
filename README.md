# Gandora

Gandora is the project for `gan`, a compiler for an Elixir-flavored language
that produces readable Python. You write Elixir-style modules, pattern
matching, pipelines, and hygienic `defmacro` macros; deployment is ordinary
Python managed by `uv` with the standard `.venv` layout. Where Elixir reaches
its host platform through `:erlang` atom calls, Gandora reaches Python the
same way: `:math.sqrt(2.0)` compiles to `import math` plus `math.sqrt(2.0)`
with no wrapper code.

```elixir
defmodule App.Mathy do
  @moduledoc "Math helpers showing patterns and interop."

  def fact(0), do: 1
  def fact(n) when n > 0, do: n * fact(n - 1)

  def norm(xs) do
    xs
    |> sum_squares()
    |> then_sqrt()
  end

  defp sum_squares(xs) do
    :functools.reduce(fn acc, x -> acc + x * x end, xs, 0)
  end

  defp then_sqrt(x), do: :math.sqrt(x)
end
```

compiles to plain Python with multi-clause dispatch as a `match` statement,
private functions as `_`-prefixed functions, and the interop as direct
imports — no Gandora runtime is needed to execute the output.

Design decisions are recorded as **Gandora Enhancement Proposals** in
[`geps/`](geps/):

- [GEP-0000](geps/0000-gep-process.md) — the proposal process and translation
  policy ([中文](geps/local/zh/0000-gep-process.md))
- [GEP-0001](geps/0001-language-and-cli.md) — language identity, surface
  syntax, module naming, configuration, and the `gan` CLI
  ([中文](geps/local/zh/0001-language-and-cli.md))
- [GEP-0002](geps/0002-macro-system.md) — the hygienic macro system
  ([中文](geps/local/zh/0002-macro-system.md))
- [GEP-0003](geps/0003-python-interop.md) — Python interop
  ([中文](geps/local/zh/0003-python-interop.md))

English GEPs are normative; the synchronized Chinese translations are
generated with `scripts/translate-gep.py` (a DeepSeek-backed translator
configured by `.env`) and human-reviewed per GEP-0000-R030.

## Requirements

- Rust 1.85 or newer (to build the compiler)
- Python 3.11 or newer
- `uv` for Python development (optional but recommended)

## Quick start

```console
cargo build --release
export PATH="$PWD/target/release:$PATH"

gan init my-app
cd my-app
gan run src/main.gan
```

`gan init` creates a `uv`-compatible project: `pyproject.toml` owns Python
metadata and dependencies, `gandora.jsonc` owns the compiler configuration,
and sources live under `src/`. `gan run` compiles into `.gandora/cache/` and
executes with `.venv/bin/python`, `uv run python`, or `python3` — whichever
is available first.

```console
gan check              # parse, expand, analyze; no output written
gan build              # compile every module into dist/
gan expand src/x.gan   # show a module after macro expansion
gan compile src/x.gan --out build/
```

## The v0 surface

Modules (`defmodule`, one per file, path-derived names), `def`/`defp` with
multi-clause pattern dispatch and `when` guards, `defmacro` with
`quote`/`unquote`/`unquote_splicing` and default hygiene (`var!` escapes),
atoms, strings with `#{}` interpolation, lists, tuples, maps, keyword lists,
ranges, the `|>` pipe, `if`/`unless`/`case`/`cond`/`with`, anonymous
functions and captures (`&Mod.fun/1`, `&(&1 + 1)`), destructuring `=`,
`alias`/`import`/`require`, and the interop forms of GEP-0003 (`:module`
calls, `pyimport`, postfix `expr.name(...)`, `@decorate`).

Anything outside the surface produces a diagnostic naming the construct
(GEP-0001-R007) rather than a silent mistranslation. See
[`examples/tour`](examples/tour) for a working multi-module program.

## Data mapping

| Gandora | Python |
| --- | --- |
| `:atom` | `"atom"` (interned string) |
| `nil` / `true` / `false` | `None` / `True` / `False` |
| list / tuple / map | `list` / `tuple` / `dict` |
| keyword list `[a: 1]` | `[("a", 1)]` |
| `a..b` | `range(a, b + 1)` |

Only `false` and `nil` are falsy (Elixir semantics); the generated code
inserts explicit truthiness checks where needed.

## Repository layout

- `src/` — the Rust compiler (`lexer`, `parser`, `expander`, `codegen`,
  `project`, CLI in `main.rs`)
- `geps/` — Gandora Enhancement Proposals (+ `geps/local/zh/` translations)
- `examples/tour` — a runnable multi-module example
- `tests/e2e.rs` — end-to-end tests driving the real binary
- `scripts/translate-gep.py` — the GEP translation tool

```console
cargo test             # unit + end-to-end tests
```

## Status

v0: the language, compiler, macro system, interop, and CLI described by
GEP-0001..0003 are implemented and tested. Deferred (tracked for future
GEPs): structs, protocols, comprehensions, `try/rescue`, binaries, sigils,
a formatter, and an LSP.

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
- [GEP-0004](geps/0004-structs-and-module-attributes.md) — structs (frozen
  dataclasses) and module attributes
  ([中文](geps/local/zh/0004-structs-and-module-attributes.md))
- [GEP-0005](geps/0005-sigils.md) — sigils (`~w`, `~s`, `~r`, and the raw
  embedded-Python `~python`) ([中文](geps/local/zh/0005-sigils.md))
- [GEP-0006](geps/0006-package-publication.md) — publishing packages as
  ordinary PyPI wheels, with macros shipped as source and zero runtime
  ([中文](geps/local/zh/0006-package-publication.md))

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
`alias`/`import`/`require`, the interop forms of GEP-0003 (`:module`
calls, `pyimport`, postfix `expr.name(...)`, `@decorate`), and the
GEP-0004 data declarations: `defstruct` (a frozen `@dataclass` with
`%Mod{...}` literals, patterns, and `%Mod{s | f: v}` updates) plus module
attributes (`@app :flask.Flask("__main__")` compiles to a module-level
binding, so `@decorate @app.route("/")` works like hand-written Python).
GEP-0005 adds sigils: `~w(a b c)` word lists, `~s(...)` strings,
`~r/\d+/` compiled Python regexes, and `~python(sum(i*i for i in range(n)))`
— a raw sigil that splices one verbatim Python expression into the
output, Gandora's rendering of Osiris's embedded-language sigils.

Anything outside the surface produces a diagnostic naming the construct
(GEP-0001-R007) rather than a silent mistranslation. See
[`examples/tour`](examples/tour) for a working multi-module program.

## Publishing a package

A Gandora package is an ordinary PyPI wheel (GEP-0006):

```console
gan init --package acme-text
cd acme-text            # write modules under src/acme_text/
gan build               # compiles into pkg/, emits marker + .gan sources
uv build && uv publish  # standard hatchling wheel, standard PyPI
```

A live example is [gandora-text](https://github.com/mjason/gandora-text),
installable straight from GitHub with
`uv add git+https://github.com/mjason/gandora-text`. Consumers then use
it from Gandora:

```elixir
require AcmeText.Core     # macros: expanded at compile time from the
alias AcmeText.Core       #         .gan sources shipped in the wheel
Core.hello("world")       # functions: a plain `import acme_text.core`
```

The wheel introduces no runtime: its `.py` modules are self-contained,
Python-only consumers can use them without knowing Gandora exists, and
macro expansion leaves nothing behind at runtime. The compiler discovers
installed packages by reading their `gandora.toml` markers from
`.venv` — it never imports or executes package code.

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
GEP-0001..0005 are implemented and tested. Deferred (tracked for future
GEPs): protocols, comprehensions, `try/rescue`, binaries,
a formatter, and an LSP.

# The Gandora Language Manual

A practical guide to the v0 surface. Normative definitions live in the
GEPs ([`geps/`](../geps/)); this manual shows how the pieces are used.
Every construct here is exercised by [`examples/tour`](../examples/tour),
whose checked-in [`generated/`](../examples/tour/generated/) directory
shows the exact Python each chapter compiles to.

## Modules and functions

One `defmodule` per file; the name must match the path
(`src/app/hello_web.gan` ↔ `App.HelloWeb` ↔ `app/hello_web.py`).

```elixir
defmodule App.Math do
  @moduledoc "Docstrings come from @moduledoc and @doc."

  @doc "Multi-clause dispatch, top to bottom, with guards."
  def fact(0), do: 1
  def fact(n) when n > 0, do: n * fact(n - 1)

  defp helper(x), do: x + 1     # private: leading underscore in Python
end
```

Functions return their last expression. `def f(x), do: expr` is the
one-line form.

## Data

Atoms are interned strings; only `false` and `nil` are falsy.

```elixir
:ok  :"os.path"                      # atoms
"interp #{1 + 1}"                    # f-string in the output
"""
heredocs too
"""
[1, 2, 3]  {:pair, 2}  %{"k" => 1, a: 2}   # list, tuple, map
[timeout: 500, retries: 3]           # keyword list -> [("timeout", 500), ...]
1..10                                # inclusive range
```

## Pattern matching

`=`, `case`, `cond`, `with`, and function heads all match patterns:
literals, variables, `_`, tuples, `[head | tail]`, maps, pin (`^x`),
and structs.

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "the pinned value"
  {:error, %{reason: r}} -> "failed: #{r}"
  _ -> "anything else"
end

with {:ok, a} <- step1(),
     {:ok, b} <- step2(a) do
  {:ok, b}
else
  _ -> :error
end
```

A failed match raises `GanMatchError`.

## Functions as values and pipelines

```elixir
double = fn x -> x * 2 end
add = &(&1 + &2)
sqrt = &$math.sqrt/1
double.(21)

xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
```

Pipelines may continue on the next line when it starts with `|>`.

## Python interop

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$"os.path".join(a, b)               # dotted modules via quoted atoms
$sys.argv                           # attribute read
pyimport numpy, as: np              # aliased import
np.array([1, 2]) * 10               # operators broadcast — it's just Python
value.method(x).attr                # postfix chains on anything
$json.dumps(data, indent: 2)        # trailing keywords become kwargs
```

Decorators attach with `@decorate`, and module attributes hold
import-time state:

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}
```

The generated module is a normal ASGI target:
`uvicorn app.api:app --app-dir dist`.

## Structs

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}           # frozen dataclass instance
%App.User{name: n} = u              # pattern (isinstance + fields)
older = %App.User{u | age: u.age + 1}   # dataclasses.replace
m2 = %{m | count: 2}                # plain-map update: {**m, ...}
```

## Macros

Compile-time, hygienic, Elixir-shaped. Macros run in a deterministic
sandbox inside the compiler and leave no runtime trace.

```elixir
defmacro unless_nil(value, fallback) do
  quote do
    case unquote(value) do
      nil -> unquote(fallback)
      found -> found
    end
  end
end
```

Template variables are renamed per expansion (hygiene); `var!(name)`
deliberately reaches the caller's scope; `unquote_splicing(list)`
splices sequences. Bring macros in with `require Mod` (or `import`),
then call `Mod.some_macro(...)`. Inspect results with `gan expand file`.

## Sigils

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\d+/                      # re.compile(r"\d+"), backslashes verbatim
~python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

`~python` is the escape hatch for Python-only spellings
(comprehensions, boolean indexing); it splices the expression into the
output unchanged.

## Documentation

Three channels, no overlap: `@doc`/`@moduledoc` are Elixir-style
(Markdown string = text, keyword list = metadata, `false` = hidden,
repeated lines accumulate; the text is never parsed). `@example` holds
the runnable examples. `@doc_trans`/`@moduledoc_trans` add prose-only
translations.

```elixir
@doc since: "1.3.0"
@doc "Factorial."
@doc_trans zh_CN: "阶乘。"
@example """
    gan> fact(10)
    3628800
"""
def fact(n), do: ...
```

`gan test` compiles `@example` blocks into native Python doctests and
runs them (expected output is the Python `repr`, i.e. what `inspect/1`
shows); every locale renders the same blocks. A `gan>` line in a
translation is an error; in `@doc` text it is a warning (untested).
`gan doc App.Mathy.fact --locale zh` prints the localized view with
RFC 4647 fallback.

## Projects, packages, and the CLI

`gandora.jsonc` configures the compiler (`source`, `outDir`,
`targetPython`, `exclude`, `package`); `pyproject.toml` + `uv` own
dependencies and `.venv`.

```console
gan init my-app          # new project        gan check    # analyze only
gan run src/main.gan     # compile + execute  gan build    # compile to outDir
gan expand src/x.gan     # show macro output  gan init --package name
```

Packages publish as ordinary wheels (`gan build && uv build &&
uv publish`) carrying compiled Python, a `gandora.toml` marker, and the
`.gan` sources macros expand from — consumers `uv add` them and
`require`/`alias` as if local, with no Gandora runtime introduced
(GEP-0006).

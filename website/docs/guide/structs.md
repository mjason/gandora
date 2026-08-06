# Structs & Annotations

## Structs

`defstruct` declares a frozen dataclass; literals, patterns, and
updates all work on it:

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}               # frozen dataclass instance
%App.User{name: n} = u                  # pattern (isinstance + fields)
older = %App.User{u | age: u.age + 1}   # dataclasses.replace
m2 = %{m | count: 2}                    # plain-MAP update: {**m, ...}
```

`%{x | ...}` is for plain maps; a struct value updates with the struct
spelling `%Mod{x | ...}` — the build reminds you if the two get
crossed. Struct types appear in specs as `App.User()`.

## Documentation annotations

Annotations before a `def` accumulate onto it — any of `@doc`,
`@doc_trans`, `@param`, `@param_trans`, `@spec`, `@example`,
`@decorate`, `@allow`:

```elixir
@doc "Word frequencies of a sentence, as a map."
@doc_trans zh_CN: "统计句子的词频，返回映射。"
@param sentence, "Case-folded and split on whitespace."
@param_trans sentence, zh_CN: "会转小写并按空白切分的句子。"
@spec word_count(string()) :: map(string(), integer())
@example """
    gan> word_count("the quick the")
    {'the': 2, 'quick': 1}
"""
def word_count(sentence), do: ...
```

- `@param` names must match clause-head variables — validated at
  compile time.
- `@example` is the only doctest channel: `gan test` runs the `gan>`
  lines as native Python doctests. Expected output is the Python
  `repr`: atoms print as `'ok'`, tuples as `('ok', 21)`, booleans as
  `True`.
- The standard: **every public `def` carries `@doc` + `@spec`**;
  user-facing surfaces add the `_trans` pair. The build's coverage
  report keeps score.


!!! note "No `## Examples` sections"
    The Elixir habit of example blocks *inside* `@doc` does not apply:
    `@doc` is prose only. Examples always go in the separate
    `@example` attribute — runnable doctests on defs, displayed
    documentation on macros. The build teaches this on sight.

Documentation *language* is a developer preference, never project
config: `gandora.local.jsonc` (`{"docLocale": "zh-CN"}`, gitignored) →
`GAN_DOC_LOCALE` → English.

## Module attributes and decorators

Attributes hold import-time state; `@decorate` attaches Python
decorators:

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

The generated module is a normal ASGI target:
`uvicorn app.api:app --app-dir dist`.

## Decorators: two tiers

Runtime decoration belongs to the `$` world; compile-time decoration
belongs to macros. Concretely:

- **`@decorate <expr>`** attaches any Python decorator to the next
  def — a library's (`$functools.lru_cache(maxsize: 64)`,
  `@app.get("/")`), or a Gandora function returning a wrapper.
  Several stack; the one nearest the def wraps first, as in Python.
- A **Gandora-written wrapper is arity-exact** (`fn x -> ... f.(x) end`
  wraps 1-arg functions only) — Gandora has no `*args`, deliberately.
  A *general* any-arity decorator is Python's job: put it in a `.py`
  next to your sources and reference it as `$mymod.deco`.
- **Compile-time rewriting** — the Elixir-flavored decorator — is
  `defattr :name` + an `@on_definition` macro (GEP-0008): it sees the
  real head, keeps zero runtime, and can itself emit `@decorate` for
  the Python side. The tour's `@cache` chapter is the worked example.
- A wrapper built from a Gandora `fn` is a lambda underneath — it
  drops `__name__`/`__doc__`; if introspection matters, write that
  decorator in Python.


## Error handling

`try/rescue/after` maps onto Python exceptions; rescue clauses match
by exception class, spelled through their module:

```elixir
try do
  risky()
rescue
  e in $builtins.ValueError -> {:bad_value, to_string(e)}
  e in $requests.HTTPError -> {:http, e.response.status_code}
  _e -> :error                     # deliberate catch-all
after
  cleanup()                        # always runs, contributes no value
end
```

`try` is an expression. Tail calls inside `try` are *not* optimized
(the frame must survive for the handlers) — `recur` there is a compile
error, not a silent stack eater.

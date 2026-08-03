# The Gandora Language Manual

The practical guide to writing Gandora — for humans and, deliberately,
for AI agents. Normative definitions live in the GEPs
([`geps/`](../geps/)); this manual shows how the pieces are used and
which spelling to prefer when several exist. Every construct here is
exercised by [`examples/tour`](../examples/tour) (whose checked-in
[`generated/`](../examples/tour/generated/) shows the exact Python each
chapter compiles to) and battle-tested by the playground's
self-checking suite.

Ground rules that shape everything below:

- **Zero runtime.** Generated Python is self-contained and readable —
  what a reviewer would have written by hand. Helpers are inlined per
  module; deployment never depends on Gandora.
- **Elixir surface, Python semantics underneath.** Where Elixir has a
  construct, Gandora spells it the Elixir way; values are ordinary
  Python objects.
- **The compiler talks back.** Warnings are statically provable facts,
  not opinions; hover shows how recursion compiled; `gan lsc` serves
  every fact as JSON.

## Modules and functions

One `defmodule` per file; the name must match the path
(`src/app/hello_web.gan` ↔ `App.HelloWeb` ↔ `app/hello_web.py`).

```elixir
defmodule App.Math do
  @moduledoc "Docstrings come from @moduledoc and @doc."

  @doc "Multi-clause dispatch, top to bottom, with guards."
  @spec fact(integer()) :: integer()
  @allow :stack_recursion
  def fact(0), do: 1
  def fact(n) when n > 0, do: n * fact(n - 1)

  def greet(name, greeting \\ "hi"), do: "#{greeting} #{name}"  # defaults: every arity generated

  def empty?(xs), do: length(xs) == 0   # ? and ! are part of names
  defp helper(x), do: x + 1             # private: leading underscore in Python
end
```

Functions return their last expression. `def f(x), do: expr` is the
one-line form. Guards (`when`) may use the boolean-shaped builtins:
`is_list is_map is_tuple is_binary is_integer is_float is_number
is_atom is_nil is_function`, comparisons, `and or not`, arithmetic.

## Data

Atoms are interned strings and **pure data** — they never name modules
(that is `$module`'s job). Only `false` and `nil` are falsy; `0`, `""`,
and `[]` are truthy (Elixir semantics, not Python's).

```elixir
:ok  :"quoted atom"                  # atoms -> Python strings
"interp #{1 + 1}"                    # f-string in the output
"""
heredocs too (dedented)
"""
[1, 2, 3]  {:pair, 2}  %{"k" => 1, a: 2}   # list, tuple, map
[timeout: 500, retries: 3]           # keyword list -> [("timeout", 500), ...]
1..10                                # inclusive range
10 / 4                               # 2.5 — / is true division
10 // 4                              # 2 — truncated division
rem(-7, 2)                           # -1 — truncated remainder (not Python %)
"a" <> "b"                           # string concatenation
```

## Pattern matching

`=`, `case`, function heads, `with`, and `for` generators all match
patterns: literals, variables, `_`, tuples, `[head | tail]`, maps,
pin (`^x` — match the *existing* value of x), and structs.

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "the pinned value"
  {:error, %{reason: r}} -> "failed: #{r}"
  _ -> "anything else"
end

cond do                       # first truthy condition (not patterns)
  x > 90 -> :a
  true -> :c
end

with {:ok, a} <- step1(),
     {:ok, b} <- step2(a) do
  {:ok, b}
else
  _ -> :error
end
```

A failed `=`/`case` match raises `GanMatchError`. Rebinding a name
(`x = 1; x = 2`) creates a new binding — closures created in between
keep the old value (see below).

## Functions as values

```elixir
double = fn x -> x * 2 end           # -> lambda
classify = fn                        # multi-clause + guards -> hoisted def
  0 -> :zero
  n when n > 0 -> :pos
  _ -> :neg
end
add = &(&1 + &2)                     # capture with placeholders
sqrt = &($math.sqrt/1)               # capture a Python function
mine = &fact/1                       # capture a module function
double.(21)                          # calling a function value uses .()
```

**Closures capture by value at creation time** (GEP-0021), exactly as
in Elixir — a later rebinding, a tail-recursion loop iteration, or a
comprehension step never leaks into a closure made earlier. The
compiler realizes this with Python's own idiom
(`lambda x, *, n=n: x + n`); calling arity stays strict.

## Pipelines

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

Pipelines may continue on the next line when it starts with `|>`.

## Iteration: comprehensions and recursion

There is no `loop` and no `while`. Iteration is `for`, the `Enum`
family, or recursion — and the compiler makes recursion safe.

### `for` comprehensions (GEP-0020)

Compile to native Python comprehensions:

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # multiple generators
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # dict comprehension
for {k, v} <- [{"a", 1}, :bad], do: k          # non-matching elements are SKIPPED
```

The body is one expression; `into: %{}` needs a `{key, value}` tuple
body. A comprehension **builds a collection** — using it for side
effects is a compiler warning; use `Enum.each` instead.

### Tail recursion compiles to a loop (GEP-0019)

A call to the enclosing function in tail position becomes parameter
rebinding inside `while True:` — constant stack at any depth:

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # a million frames is fine
```

`recur(args)` is the **checked** spelling of the same jump: it must be
in tail position and match a clause arity, or the build fails —
use it when constant stack is a requirement, not a hope:

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

Non-tail recursion (`n * fact(n - 1)`) stays a real call and uses the
Python stack (~1000 frames); the compiler **warns** at the definition.
Structural recursion — depth bounded by the data, like tree walks —
is legitimate: acknowledge it and the warning goes away:

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

Every function's compiled shape is visible: hover shows
`♻ tail recursion → while loop` or `⚠ native call stack`, `gan doc`
prints it, and `gan lsc doc` returns `"tco": "loop" | "stack"`.

## The type system (`@spec`)

`@spec` declares a function's type; the compiler validates it against
the clauses and emits **PEP 484 annotations** in the generated Python,
so `pyright`/`mypy` check callers and hover shows real types. One
`@spec` per definition group, placed with the other annotations before
the first clause.

```elixir
@spec mean(xs :: sequence(number()), precision :: integer()) :: float()
def mean(xs, precision \\ 2) do ... end
# -> def mean(xs: collections.abc.Sequence[int | float], precision: int = 2) -> float:
```

### Scalars

| Gandora | Python annotation |
| --- | --- |
| `integer()` | `int` |
| `float()` | `float` |
| `number()` | `int \| float` |
| `string()` | `str` |
| `boolean()` | `bool` |
| `atom()` | `str` (atoms are interned strings) |
| `nil` | `None` |
| `term()` / `any()` | `object` |
| `fun()` | `Callable` (unparametrized) |

### Containers — concrete when you build, abstract when you accept

Concrete containers say "exactly this Python type":

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

Abstract containers say "anything that behaves like one" — prefer them
for **parameters** so callers may pass tuples, ranges, or generators,
and because `list` is invariant in Python's type system while
`Sequence` is covariant:

```elixir
iterable(t)                # collections.abc.Iterable[t]  — will be walked once
sequence(t)                # collections.abc.Sequence[t]  — indexed / re-walked
mapping(k, v)              # collections.abc.Mapping[k, v] — read-only dict-like
keyword()                  # a keyword list: list[tuple[str, object]]
```

Rule of thumb: **abstract in, concrete out** — accept `sequence(t)`,
return `list(t)`.

### Unions and `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### Type variables (generics)

A bare lowercase name of **one or two characters** is a type variable;
each becomes a module-level `typing.TypeVar` in the output. The same
letter means "the same type" across the spec:

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

Longer bare names are an error with a hint ("did you mean `name()`?"),
so a typo'd `intger` cannot silently become a generic.

### Named parameters

`name :: type` names the parameter in the spec — self-documenting and
what signature help displays:

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### Host (Python) types

Any Python type can appear via `$module`; parentheses parametrize:

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str], with the import emitted
```

### Struct types

`Mod.t()` is the struct class defined by `defstruct` in that module:

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### Interplay with defaults and multi-arity

One spec covers the whole group; write it for the full parameter list
(defaults included). The generated delegates and the dispatcher carry
the annotations. `@spec` naming a function or arity that doesn't exist
is a compile error, so specs cannot rot silently.

## Documentation and doctests

Annotation order before a `def`: any of `@doc`, `@doc_trans`,
`@param`, `@param_trans`, `@spec`, `@example`, `@decorate`, `@allow` —
all accumulate onto the next definition.

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
- `@example` is the only doctest channel: `gan test` compiles the
  `gan>` lines into native Python doctests and runs them. Expected
  output is the Python `repr` (what `inspect/1` shows): atoms print as
  `'ok'`, maps as `{'k': 1}`, booleans as `True`.
- The standard: **every public `def` carries `@doc` + `@spec`** (and
  `@param` when it has parameters); user-facing surfaces add the
  `_trans` pair.

Documentation language is a **developer** preference, never project
config: `gandora.local.jsonc` (`{"docLocale": "zh-CN"}`, gitignored,
next to `gandora.jsonc`) > the `GAN_DOC_LOCALE` environment variable /
editor setting > default language only. `gan doc Mod.fun --locale zh-CN`
asks explicitly; `gan lsc doc` always returns every locale as JSON.

## Testing (GEP-0024)

One command, two layers: `gan test` runs every `@example` doctest,
then every `test_*` function of `tests/*.gan` — compiled with the
project's full module resolution and executed by pytest (add it once:
`uv add --dev pytest`).

```elixir
# tests/test_stats.gan
defmodule TestStats do
  @moduledoc "Edge cases the doctests don't cover."

  use Test

  describe "mean" do
    test "averages evenly" do
      assert Stats.mean([1, 2, 3, 4]) == 2.5   # failure names left and right
    end
  end

  test "membership and negation" do
    assert 16 in Stats.even_squares(1, 8)
    refute 9 in Stats.even_squares(1, 8)
  end

  test "typed raises" do
    _ = Test.assert_raise($builtins.KeyError, fn -> Map.fetch!(%{}, "no") end)
    nil
  end
end
```

The ExUnit surface: `test "name" do` (defines `test_<slug>`),
`describe` (prefixes inner names), `assert`/`refute` (comparisons
report both operands), plus `Test.assert_eq / assert_nil /
assert_raise / assert_in_delta / flunk` as plain functions. `tests/`
never ships — it lives outside the source roots.

## Python interop

`$module` is a first-class module object; `$` marks the interop
boundary visibly.

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # dotted chain: imports importlib.metadata
$(PIL.Image).open(f)                # $(...) locks the module boundary explicitly
$(sys).stderr                       # ...single-segment too: import sys, attr stderr
pyimport numpy, as: np              # aliased import
pyimport sys                        # bare import binds `sys` as a plain name
np.array([1, 2]) * 10               # operators broadcast — it's just Python
sys.stderr.write("...")             # bare-name chains have no import ambiguity
$json.dumps(data, indent: 2)        # trailing keywords become kwargs
```

Which spelling, when:

| Situation | Spelling |
| --- | --- |
| one-off reference | `$math.sqrt(x)` |
| one-off where the chain heuristic guesses wrong | `$(os.path).sep`, `$(sys).stderr` |
| module used repeatedly in a file | `pyimport sys` (or `, as:`) + bare names |
| a deep attribute used often | `@environ $(os).environ` module attribute |

Repeated `$(...)` spellings in one file are a smell — declare a
`pyimport`. Never write wrapper modules around Python APIs; the
absence of wrappers is the design.

Decorators attach with `@decorate`; module attributes hold import-time
state:

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

The generated module is a normal ASGI target:
`uvicorn app.api:app --app-dir dist`.

## Error handling

`try/rescue/after` maps onto Python exceptions; rescue clauses match
by exception class:

```elixir
try do
  risky()
rescue
  e in $builtins.ValueError -> {:bad_value, to_string(e)}
  e in $requests.HTTPError -> {:http, e.response.status_code}
  e -> {:error, to_string(e)}      # bare variable: any Exception
after
  cleanup()                        # always runs, contributes no value
end

raise "message"                    # -> raise RuntimeError("message")
```

`try` is an expression. Tail calls inside `try` are *not* optimized
(the frame must survive for the handlers) — `recur` there is a
compile error, not a silent stack eater.

## Structs

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}               # frozen dataclass instance
%App.User{name: n} = u                  # pattern (isinstance + fields)
older = %App.User{u | age: u.age + 1}   # dataclasses.replace
m2 = %{m | count: 2}                    # plain-map update: {**m, ...}
```

Struct types appear in specs as `App.User.t()`.

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
splices sequences; `def unquote(head)` builds definitions. Bring
macros in with `require Mod` (or `import`/`use`). Inspect results with
`gan expand file` or the editor's *Expand Macros* command.

## Sigils and embedded languages

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\\d+/                     # re.compile("\\d+") — string escapes apply
~python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

Uppercase-free embedded-language sigils carry whole documents with
`<%= expr %>` splices back into Gandora (editors highlight the inner
language):

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

`~python` is the escape hatch for Python-only spellings; splices are
compiled Gandora expressions, everything else passes through verbatim.

## Compiler lints — warnings are provable facts

Each fires only on something statically certain, lands on the
definition line in `gan check`/`gan build`/editor squiggles, and has a
mechanical remedy (often a one-click quick fix):

| Warning | Meaning | Remedy |
| --- | --- | --- |
| undefined variable | a read nothing binds — guaranteed `NameError` | fix the name |
| unused binding | bound, never read | `_` prefix (`_meta`) |
| unreachable clause | a guard-less all-variable head shadows later same-arity clauses (also `case` wildcards) | reorder or delete |
| discarded comprehension | `for` in statement position | `Enum.each` |
| unused `defp` | dead private function | delete, or `@allow :unused_function` |
| stack recursion | self-recursive, never in tail position | accumulator form, `recur`, or `@allow :stack_recursion` |

`@allow` targets are checked — a typo is a compile error. Treat
warnings as defects: the codebase standard is zero.

## Projects and the CLI

`gandora.jsonc` configures the compiler (`source`, `outDir`,
`targetPython`, `exclude`, `package`, `pyPackage`); `pyproject.toml` +
`uv` own dependencies and `.venv`; `gandora.local.jsonc` holds
per-developer preferences and stays out of git.

```console
gan init my-app          # new project        gan check      # analyze + lints only
gan run src/main.gan     # compile + execute  gan build      # compile to outDir
gan test                 # run @example doctests
gan fmt src              # format in place    gan fmt --check src   # CI gate
gan fmt --diff src       # show the diff      echo ... | gan fmt -  # stdin -> stdout
gan doc Enum.take        # docs (+ --locale)  gan repl       # interactive
gan expand src/x.gan     # macro output       gan try <file|->  # sandbox
gan init --package name
```

Packages publish as ordinary wheels (`gan build && uv build &&
uv publish`) carrying compiled Python, a `gandora.toml` marker, and the
`.gan` sources macros expand from — consumers `uv add` them with no
Gandora runtime introduced.

## The AI toolbox: `gan lsc`

Every language fact is one JSON value on stdout — built for agents:

```console
gan lsc check --root .                  # whole-project diagnostics, lints included
gan lsc review --root .                 # check + per-file practice/migration suggestions
gan lsc diagnostics src/x.gan --root .  # one file
gan lsc doc Enum.take --root .          # specs, prose (all locales), params, tco shape
gan lsc doc for --root .                # language constructs answer too (for/recur/with...)
gan lsc references Stats.mean --root .  # every call site (+ definitions)
gan lsc wsymbols mean --root .          # project-wide symbol search
gan lsc symbols Stats --root .          # one module's outline
gan lsc definition Stats.mean --root .  # where it's defined
gan lsc compile src/x.gan --root .      # the generated Python, as text
gan lsc expand src/x.gan --root .       # post-macro quoted AST
gan lsc ast src/x.gan --root .          # parse tree (Elixir encoding)
gan lsc pydoc numpy.array --root .      # Python-side docs via jedi
```

And the **sandbox** — validate generated code before it touches the
project (GEP-0023):

```console
echo 'Enum.mpa([1,2], fn x -> x end)' | gan try -
# -> {"ok": false, ..., "suggestions": [{"kind": "did_you_mean",
#     "message": "`Enum.mpa` is not a function of Enum — did you mean `Enum.map`?"}]}
```

`try` compiles, lints, spell-checks names against real symbols
(edit-distance), flags cross-language habits (`return`, `lambda`,
`None`, Python `def ...():` …) with the Gandora spelling, then runs in
a temp dir under a timeout — returning the generated Python, stdout,
and a snippet's last value. `--no-run` skips execution.

A productive loop for an agent: **generate → `gan try -` → apply
the suggestions → `try` again → write into the project** → then
`gan lsc check` (fix every finding) → `gan test` → `gan fmt src`.

## Style checklist for agents

1. Every public `def`: `@doc` + `@spec` (+ `@param` per parameter);
   `@example` for anything with interesting behavior — `gan test` keeps
   them honest.
2. Specs: abstract containers for inputs (`sequence`, `mapping`),
   concrete for outputs; type variables for genuinely generic flow;
   `Mod.t()` for structs; `$mod.Type()` at Python boundaries.
3. Iteration: `for`/`Enum` for mapping-shape work; accumulator tail
   recursion for unbounded loops (`recur` when constant stack is a
   requirement); `@allow :stack_recursion` only for structure-bounded
   recursion, with the reason obvious.
4. Interop: `$` one-offs, `pyimport` for repeated use, no wrappers.
5. Zero warnings, `gan fmt` clean, doctests passing — the toolchain,
   std, tour, and playground all hold this line; match them.

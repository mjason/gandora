# Modules & Functions

One `defmodule` per file, and the name must match the path:
`src/app/hello_web.gan` ↔ `App.HelloWeb` ↔ `app/hello_web.py`.

```elixir
defmodule App.Math do
  @moduledoc "Docstrings come from @moduledoc and @doc."

  @doc "Multi-clause dispatch, top to bottom, with guards."
  @spec fact(integer()) :: integer()
  @allow :stack_recursion
  def fact(0), do: 1
  def fact(n) when n > 0, do: n * fact(n - 1)

  def empty?(xs), do: length(xs) == 0   # ? and ! are part of names
  defp helper(x), do: x + 1             # private: leading underscore in Python
end
```

Functions return their **last expression** — there is no `return`.
`def f(x), do: expr` is the one-line form; a `do ... end` block holds
several expressions.

## Multi-clause heads and guards

Clauses of the same name and arity dispatch by pattern, top to
bottom. Guards (`when`) may use the boolean-shaped builtins —
`is_list is_map is_tuple is_binary is_integer is_float is_number
is_atom is_nil is_function` — plus comparisons, `and or not`, and
arithmetic.

```elixir
def classify(n) when n < 0, do: :negative
def classify(0), do: :zero
def classify(_), do: :positive
```

The compiler proves clause reachability: a clause after a catch-all is
a warning (GEP-0022).

## Default parameters

Trailing defaults use `\\`; callers omit suffix arguments:

```elixir
def greet(name, greeting \\ "hi", mark \\ "!") do
  "#{greeting} #{name}#{mark}"
end

greet("Ada")            # "hi Ada!"
greet("Ada", "yo")      # "yo Ada!"
```

Immutable-literal defaults compile to a **native Python signature**
(`def greet(name, greeting="hi", mark="!")`) — typed, honest, and
arity-checkable by the build's verification layer. Mutable defaults
(`\\ []`) keep call-time evaluation, Elixir-style.

## Cross-module calls

```elixir
alias App.Stats            # then Stats.mean(...)
import App.Stats           # bare mean(...) — sparingly
Stats.mean([1, 2, 3])
```

## Annotation discipline

Every public `def` carries `@doc` + `@spec` (and `@param` per
parameter in user-facing tools); `@example` doctests document
interesting behavior and are executed by `gan test`. The build's
practice pass keeps you honest — see
[The Build Verdict](../tooling/build.md).

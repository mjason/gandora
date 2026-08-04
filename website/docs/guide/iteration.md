# Iteration & Recursion

There is no `loop` and no `while`. Iteration is `for`, the `Enum`
family, or recursion — and the compiler makes recursion safe.

## `for` comprehensions

Compile to native Python comprehensions (GEP-0020):

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # multiple generators
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # dict comprehension
for {k, v} <- [{"a", 1}, :bad], do: k          # non-matching elements are SKIPPED
```

The body is one expression; `into: %{}` needs a `{key, value}` tuple
body. A comprehension **builds a collection** — using it for side
effects is a compiler warning; use `Enum.each` instead.

## Tail recursion compiles to a loop

A call to the enclosing function in tail position becomes parameter
rebinding inside `while True:` — constant stack at any depth
(GEP-0019):

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # a million frames is fine
```

`recur(args)` is the **checked** spelling of the same jump: it must be
in tail position and match a clause arity, or the build fails — use it
when constant stack is a requirement, not a hope:

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

## When recursion really recurses

Non-tail recursion (`n * fact(n - 1)`) stays a real call and uses the
Python stack (~1000 frames); the compiler **warns at the definition**
with an accumulator recipe. Structural recursion — depth bounded by
the data, like tree walks — is legitimate: acknowledge it and the
warning goes away:

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

Every function's compiled shape is visible: hover shows
`♻ tail recursion → while loop` or `⚠ native call stack`, `gan doc`
prints it, and `gan lsc doc` returns `"tco": "loop" | "stack"`.

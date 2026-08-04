# Functions & Pipelines

## Anonymous functions and captures

```elixir
double = fn x -> x * 2 end           # -> lambda
classify = fn                        # multi-clause + guards -> hoisted def
  0 -> :zero
  n when n > 0 -> :pos
  _ -> :neg
end
add = &(&1 + &2)                     # capture with placeholders
sqrt = &($math.sqrt/1)               # capture a Python function
mine = &fact/1                       # capture a module function (defp too)
double.(21)                          # calling a function value uses .()
```

`&f/1` works for public defs, private defps, and kernel forms
(`&to_string/1` compiles to `str`). When a `fn` merely wraps one call
— `fn x -> f(x) end` — the build suggests the capture.

## Closures capture by value

Exactly as in Elixir, a closure snapshots its free variables **at
creation time** (GEP-0021) — later rebindings, tail-recursion
iterations, and comprehension steps never leak into a closure made
earlier:

```elixir
n = 1
add_n = fn x -> x + n end
n = 100
add_n.(1)    # 2, not 101
```

The compiler realizes this with Python's own idiom
(`lambda x, *, n=n: x + n`); calling arity stays strict.

## Pipelines

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

`|>` pipes into the **first argument** of a Gandora call; `|> .m(...)`
pipes into a Python *method of the piped value* — that one form covers
the whole pandas/numpy fluent world. Pipelines may continue on the
next line when it starts with `|>`.

There is no `then/2` — pipe into an anonymous function
(`x |> (fn v -> f(v, 1) end).()`) or just bind a variable.

# Data & Pattern Matching

## Values

Atoms are interned strings and **pure data** — they never name modules
(that is `$module`'s job). Only `false` and `nil` are falsy; `0`,
`""`, and `[]` are truthy (Elixir semantics, not Python's).

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

Maps are the **one way to write data** — a JSON document from API docs
becomes a map by swapping `:` for `=>` (the build teaches this on
sight); runtime JSON text is `$json.loads(s)`:

```elixir
%{"type" => "function",
  "function" => %{"name" => "ping",
                  "parameters" => %{"type" => "object", "properties" => %{}}}}
```

## Pattern matching

`=`, `case`, function heads, `with`, and `for` generators all match
patterns: literals, variables, `_`, tuples, `[head | tail]`, maps, pin
(`^x` — match the *existing* value of `x`), and structs.

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "the pinned value"
  {:error, %{reason: r}} -> "failed: #{r}"
  _ -> "anything else"
end
```

A failed `=` or `case` match raises `GanMatchError`. `cond` picks the
first truthy *condition* (not a pattern):

```elixir
cond do
  x > 90 -> :a
  x > 80 -> :b
  true -> :c
end
```

## `with` — chaining fallible steps

```elixir
with {:ok, a} <- parse(s),
     {:ok, b} <- check(a) do
  {:ok, b}
else
  {:error, why} -> {:error, why}
end
```

The first pattern that fails to match falls to `else`. This is the
idiom for ok/error pipelines — no exceptions needed.

## Rebinding

`x = 1; x = 2` creates a *new binding*; closures created in between
keep the old value (see
[Functions & Pipelines](functions.md#closures-capture-by-value)).

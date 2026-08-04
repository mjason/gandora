# List

List-shape helpers; element-wise work lives in Enum (GEP-0010).

## first

```elixir
@spec first(sequence(a)) :: a | nil
```

The first element, or nil for an empty list.

| | |
| --- | --- |
| `xs` | The list. |

```elixir
    gan> List.first([1, 2, 3])
    1
```

## last

```elixir
@spec last(sequence(a)) :: a | nil
```

The last element, or nil for an empty list.

| | |
| --- | --- |
| `xs` | The list. |

```elixir
    gan> List.last([1, 2, 3])
    3
```

## flatten

```elixir
@spec flatten(sequence()) :: list()
```

Flattens nested lists to any depth.

| | |
| --- | --- |
| `xs` | Nested lists collapse to one level. |

```elixir
    gan> List.flatten([1, [2, [3, 4]], 5])
    [1, 2, 3, 4, 5]
```

## wrap

```elixir
@spec wrap(any()) :: list()
```

Wraps a value in a list: nil becomes [], lists pass through, anything else becomes [value].

| | |
| --- | --- |
| `value` | nil becomes [], lists pass through, others become [value]. |

```elixir
    gan> List.wrap(1)
    [1]
```

## duplicate

```elixir
@spec duplicate(a, integer()) :: list(a)
```

A list of `n` copies of `value`.

| | |
| --- | --- |
| `value` | The value to repeat. |
| `n` | How many copies. |

```elixir
    gan> List.duplicate("x", 3)
    ['x', 'x', 'x']
```

## insert_at

```elixir
@spec insert_at(sequence(a), integer(), a) :: list(a)
```

Inserts `value` at `index` (negative counts from the end, as in Elixir).

| | |
| --- | --- |
| `xs` | The list. |
| `index` | Where the value lands. |
| `value` | The value to insert. |

```elixir
    gan> List.insert_at([1, 3], 1, 2)
    [1, 2, 3]
```

## delete_at

```elixir
@spec delete_at(sequence(a), integer()) :: list(a)
```

Removes the element at `index`; out-of-range leaves the list unchanged.

| | |
| --- | --- |
| `xs` | The list. |
| `index` | The position to remove. |

```elixir
    gan> List.delete_at([1, 2, 3], 1)
    [1, 3]
```

## to_tuple

```elixir
@spec to_tuple(sequence()) :: tuple()
```

The list as a tuple.

| | |
| --- | --- |
| `xs` | The list. |

```elixir
    gan> List.to_tuple([1, 2])
    (1, 2)
```

## starts_with?

```elixir
@spec starts_with?(sequence(a), sequence(a)) :: boolean()
```

Whether the list starts with `prefix`.

| | |
| --- | --- |
| `xs` | The list. |
| `prefix` | The candidate prefix. |

```elixir
    gan> List.starts_with?([1, 2, 3], [1, 2])
    True
```

## replace_at

```elixir
@spec replace_at(sequence(a), integer(), a) :: list(a)
```

Replaces the element at `index` (negative counts from the end).

| | |
| --- | --- |
| `xs` | The list. |
| `index` | The position to replace. |
| `value` | The new value. |

```elixir
    gan> List.replace_at([1, 2, 3], 1, 9)
    [1, 9, 3]
```

## update_at

```elixir
@spec update_at(sequence(a), integer(), fun()) :: list(a)
```

Updates the element at `index` by `f`.

| | |
| --- | --- |
| `xs` | The list. |
| `index` | The position to update. |
| `f` | Applied to the current value. |

```elixir
    gan> List.update_at([1, 2, 3], 1, fn v -> v * 10 end)
    [1, 20, 3]
```

# Keyword

Functions over keyword lists — lists of {atom, value} tuples (GEP-0010).

## get

```elixir
@spec get(keyword(), atom(), any()) :: any()
```

The first value for `key`, or nil.

| | |
| --- | --- |
| `kw` | The keyword list. |
| `key` | The key to read; first match wins. |
| `default` | Returned when the key is absent. |

```elixir
    gan> Keyword.get([a: 1, b: 2], :a)
    1
```

## put

```elixir
@spec put(keyword(), atom(), any()) :: keyword()
```

Replaces `key` with `value`, prepending it (Elixir Keyword.put).

| | |
| --- | --- |
| `kw` | The keyword list. |
| `key` | The key to set. |
| `value` | The value to store. |

```elixir
    gan> Keyword.put([a: 1], :b, 2)
    [('b', 2), ('a', 1)]
```

## keys

```elixir
@spec keys(keyword()) :: list(atom())
```

The keys, in order, duplicates included.

| | |
| --- | --- |
| `kw` | The keyword list. |

```elixir
    gan> Keyword.keys([a: 1, b: 2])
    ['a', 'b']
```

## values

```elixir
@spec values(keyword()) :: list()
```

The values, in order.

| | |
| --- | --- |
| `kw` | The keyword list. |

```elixir
    gan> Keyword.values([a: 1, b: 2])
    [1, 2]
```

## has_key?

```elixir
@spec has_key?(keyword(), atom()) :: boolean()
```

Whether `key` is present.

| | |
| --- | --- |
| `kw` | The keyword list. |
| `key` | The key to test. |

```elixir
    gan> Keyword.has_key?([a: 1], :a)
    True
```

## delete

```elixir
@spec delete(keyword(), atom()) :: keyword()
```

Removes every entry for `key`.

| | |
| --- | --- |
| `kw` | The keyword list. |
| `key` | Every entry with this key is removed. |

```elixir
    gan> Keyword.delete([a: 1, b: 2], :a)
    [('b', 2)]
```

## merge

```elixir
@spec merge(keyword(), keyword()) :: keyword()
```

Merges `kw2` into `kw1`; `kw2` wins, its entries appended.

| | |
| --- | --- |
| `kw1` | The base list. |
| `kw2` | Its entries win on conflicts. |

```elixir
    gan> Keyword.merge([a: 1, b: 2], [b: 9, c: 3])
    [('a', 1), ('b', 9), ('c', 3)]
```

# Keyword

关键字列表（{原子, 值} 元组的列表）上的函数（GEP-0010）。

## get

```elixir
@spec get(keyword(), atom(), any()) :: any()
```

键 `key` 的首个值，不存在时为 nil。

| | |
| --- | --- |
| `kw` | 关键字列表。 |
| `key` | 要读取的键，取首个匹配。 |
| `default` | 键缺失时返回的值。 |

```elixir
    gan> Keyword.get([a: 1, b: 2], :a)
    1
```

## put

```elixir
@spec put(keyword(), atom(), any()) :: keyword()
```

以 `value` 替换 `key` 并将其前置（Elixir 的 Keyword.put）。

| | |
| --- | --- |
| `kw` | 关键字列表。 |
| `key` | 要写入的键。 |
| `value` | 要存储的值。 |

```elixir
    gan> Keyword.put([a: 1], :b, 2)
    [('b', 2), ('a', 1)]
```

## keys

```elixir
@spec keys(keyword()) :: list(atom())
```

按序的全部键（含重复）。

| | |
| --- | --- |
| `kw` | 关键字列表。 |

```elixir
    gan> Keyword.keys([a: 1, b: 2])
    ['a', 'b']
```

## values

```elixir
@spec values(keyword()) :: list()
```

按序的全部值。

| | |
| --- | --- |
| `kw` | 关键字列表。 |

```elixir
    gan> Keyword.values([a: 1, b: 2])
    [1, 2]
```

## has_key?

```elixir
@spec has_key?(keyword(), atom()) :: boolean()
```

是否存在键 `key`。

| | |
| --- | --- |
| `kw` | 关键字列表。 |
| `key` | 要检测的键。 |

```elixir
    gan> Keyword.has_key?([a: 1], :a)
    True
```

## delete

```elixir
@spec delete(keyword(), atom()) :: keyword()
```

移除键为 `key` 的全部条目。

| | |
| --- | --- |
| `kw` | 关键字列表。 |
| `key` | 删除所有该键的条目。 |

```elixir
    gan> Keyword.delete([a: 1, b: 2], :a)
    [('b', 2)]
```

## merge

```elixir
@spec merge(keyword(), keyword()) :: keyword()
```

将 `kw2` 并入 `kw1`；`kw2` 胜出，其条目附加在后。

| | |
| --- | --- |
| `kw1` | 基础列表。 |
| `kw2` | 冲突时以其条目为准。 |

```elixir
    gan> Keyword.merge([a: 1, b: 2], [b: 9, c: 3])
    [('a', 1), ('b', 9), ('c', 3)]
```

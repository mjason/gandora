# Map

数据优先的 dict 函数；所有更新都返回新映射（GEP-0010）。

## get

```elixir
@spec get(mapping(k, v), k, d) :: v | d
```

键 `key` 的值；不存在时为 `default`（未给出则为 nil）。

| | |
| --- | --- |
| `m` | 映射。 |
| `key` | 要读取的键。 |
| `default` | 键缺失时的返回值。 |

```elixir
    gan> Map.get(%{a: 1}, :b, 0)
    0
```

## put

```elixir
@spec put(mapping(k, v), k, v) :: map(k, v)
```

设置 `key` 为 `value` 后的新映射。

| | |
| --- | --- |
| `m` | 映射。 |
| `key` | 要写入的键。 |
| `value` | 要存储的值。 |

```elixir
    gan> Map.put(%{a: 1}, :b, 2)
    {'a': 1, 'b': 2}
```

## delete

```elixir
@spec delete(mapping(k, v), k) :: map(k, v)
```

移除 `key` 后的新映射。

| | |
| --- | --- |
| `m` | 映射。 |
| `key` | 要删除的键。 |

```elixir
    gan> Map.delete(%{"a" => 1, "b" => 2}, "a")
    {'b': 2}
```

## keys

```elixir
@spec keys(mapping(k, v)) :: list(k)
```

键的列表。

| | |
| --- | --- |
| `m` | 映射。 |

```elixir
    gan> Map.keys(%{"a" => 1, "b" => 2})
    ['a', 'b']
```

## values

```elixir
@spec values(mapping(k, v)) :: list(v)
```

值的列表。

| | |
| --- | --- |
| `m` | 映射。 |

```elixir
    gan> Map.values(%{"a" => 1, "b" => 2})
    [1, 2]
```

## merge

```elixir
@spec merge(mapping(k, v), mapping(k, v)) :: map(k, v)
```

将 `m2` 并入 `m1`；冲突时 `m2` 胜出。

| | |
| --- | --- |
| `m1` | 基础映射。 |
| `m2` | 冲突时以其条目为准。 |

```elixir
    gan> Map.merge(%{"a" => 1}, %{"b" => 2})
    {'a': 1, 'b': 2}
```

## has_key?

```elixir
@spec has_key?(mapping(k, v), k) :: boolean()
```

是否存在键 `key`。

| | |
| --- | --- |
| `m` | 映射。 |
| `key` | 要检测的键。 |

```elixir
    gan> Map.has_key?(%{"a" => 1}, "a")
    True
```

## to_list

```elixir
@spec to_list(mapping(k, v)) :: list(tuple(k, v))
```

以 `{键, 值}` 元组列表表示的条目。

| | |
| --- | --- |
| `m` | 映射。 |

```elixir
    gan> Map.to_list(%{"a" => 1, "b" => 2})
    [('a', 1), ('b', 2)]
```

## new

```elixir
@spec new() :: map()
```

空映射。

```elixir
    gan> Map.new([{"a", 1}, {"b", 2}])
    {'a': 1, 'b': 2}
```

## new

```elixir
@spec new() :: map()
```

空映射。

```elixir
    gan> Map.new([{"a", 1}, {"b", 2}])
    {'a': 1, 'b': 2}
```

## update

```elixir
@spec update(mapping(k, v), k, v, fun()) :: map(k, v)
```

以 `f` 更新 `key`；键不存在时使用 `default`（Elixir 的 Map.update/4）。

| | |
| --- | --- |
| `m` | 映射。 |
| `key` | 要更新的键。 |
| `default` | 键缺失时写入的初值。 |
| `f` | 键存在时作用于当前值。 |

```elixir
    gan> Map.update(%{a: 1}, :a, 0, fn v -> v + 10 end)
    {'a': 11}
```

## fetch!

```elixir
@spec fetch!(mapping(k, v), k) :: v
```

键 `key` 的值；不存在时抛 Python KeyError。

| | |
| --- | --- |
| `m` | 映射。 |
| `key` | 必须存在的键。 |

```elixir
    gan> Map.fetch!(%{"a" => 1}, "a")
    1
```

## put_new

```elixir
@spec put_new(mapping(k, v), k, v) :: map(k, v)
```

仅当 `key` 不存在时设置。

| | |
| --- | --- |
| `m` | 映射。 |
| `key` | 仅当缺失时写入的键。 |
| `value` | 要存储的值。 |

```elixir
    gan> Map.put_new(%{"a" => 1}, "a", 9)
    {'a': 1}
```

## take

```elixir
@spec take(mapping(k, v), sequence(k)) :: map(k, v)
```

仅含 `keys` 的子映射（缺失的键忽略）。

| | |
| --- | --- |
| `m` | 映射。 |
| `keys` | 要保留的键。 |

```elixir
    gan> Map.take(%{"a" => 1, "b" => 2}, ["a"])
    {'a': 1}
```

## drop

```elixir
@spec drop(mapping(k, v), sequence(k)) :: map(k, v)
```

移除 `keys` 后的映射。

| | |
| --- | --- |
| `m` | 映射。 |
| `keys` | 要移除的键。 |

```elixir
    gan> Map.drop(%{"a" => 1, "b" => 2}, ["a"])
    {'b': 2}
```

## filter

```elixir
@spec filter(mapping(k, v), fun()) :: map(k, v)
```

保留使 `f`（接收 `{键, 值}` 元组）为真值的条目。

| | |
| --- | --- |
| `m` | 映射。 |
| `f` | 接收 {键, 值}；保留使其为真的条目。 |

```elixir
    gan> Map.filter(%{"a" => 1, "b" => 5}, fn pair -> elem(pair, 1) > 2 end)
    {'b': 5}
```

# List

列表形状辅助函数；逐元素操作在 Enum 中（GEP-0010）。

## first

```elixir
@spec first(sequence(a)) :: a | nil
```

首元素，空列表为 nil。

| | |
| --- | --- |
| `xs` | 列表。 |

```elixir
    gan> List.first([1, 2, 3])
    1
```

## last

```elixir
@spec last(sequence(a)) :: a | nil
```

末元素，空列表为 nil。

| | |
| --- | --- |
| `xs` | 列表。 |

```elixir
    gan> List.last([1, 2, 3])
    3
```

## flatten

```elixir
@spec flatten(sequence()) :: list()
```

任意深度展平嵌套列表。

| | |
| --- | --- |
| `xs` | 要完全展平的嵌套列表。 |

```elixir
    gan> List.flatten([1, [2, [3, 4]], 5])
    [1, 2, 3, 4, 5]
```

## wrap

```elixir
@spec wrap(any()) :: list()
```

包装为列表：nil 变 []，列表原样返回，其余变 [值]。

| | |
| --- | --- |
| `value` | nil 变 []，列表原样返回，其余包成 [值]。 |

```elixir
    gan> List.wrap(1)
    [1]
```

## duplicate

```elixir
@spec duplicate(a, integer()) :: list(a)
```

由 `n` 个 `value` 构成的列表。

| | |
| --- | --- |
| `value` | 要重复的值。 |
| `n` | 份数。 |

```elixir
    gan> List.duplicate("x", 3)
    ['x', 'x', 'x']
```

## insert_at

```elixir
@spec insert_at(sequence(a), integer(), a) :: list(a)
```

在 `index` 处插入 `value`（负数从尾部数起，同 Elixir）。

| | |
| --- | --- |
| `xs` | 列表。 |
| `index` | 插入位置。 |
| `value` | 要插入的值。 |

```elixir
    gan> List.insert_at([1, 3], 1, 2)
    [1, 2, 3]
```

## delete_at

```elixir
@spec delete_at(sequence(a), integer()) :: list(a)
```

删除 `index` 处的元素；越界时列表原样返回。

| | |
| --- | --- |
| `xs` | 列表。 |
| `index` | 要删除的位置。 |

```elixir
    gan> List.delete_at([1, 2, 3], 1)
    [1, 3]
```

## to_tuple

```elixir
@spec to_tuple(sequence()) :: tuple()
```

转换为元组的列表。

| | |
| --- | --- |
| `xs` | 列表。 |

```elixir
    gan> List.to_tuple([1, 2])
    (1, 2)
```

## starts_with?

```elixir
@spec starts_with?(sequence(a), sequence(a)) :: boolean()
```

是否以 `prefix` 开头。

| | |
| --- | --- |
| `xs` | 列表。 |
| `prefix` | 候选前缀。 |

```elixir
    gan> List.starts_with?([1, 2, 3], [1, 2])
    True
```

## replace_at

```elixir
@spec replace_at(sequence(a), integer(), a) :: list(a)
```

替换 `index` 处的元素（负数从尾部数起）。

| | |
| --- | --- |
| `xs` | 列表。 |
| `index` | 要替换的位置。 |
| `value` | 新值。 |

```elixir
    gan> List.replace_at([1, 2, 3], 1, 9)
    [1, 9, 3]
```

## update_at

```elixir
@spec update_at(sequence(a), integer(), fun()) :: list(a)
```

以 `f` 更新 `index` 处的元素。

| | |
| --- | --- |
| `xs` | 列表。 |
| `index` | 要更新的位置。 |
| `f` | 作用于当前值。 |

```elixir
    gan> List.update_at([1, 2, 3], 1, fn v -> v * 10 end)
    [1, 20, 3]
```

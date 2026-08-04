# Map

Data-first dict functions; all updates return new maps (GEP-0010).

## get

```elixir
@spec get(mapping(k, v), k, d) :: v | d
```

The value for `key`, or `default` (nil unless given) when absent.

| | |
| --- | --- |
| `m` | The map. |
| `key` | The key to read. |
| `default` | Returned when the key is absent. |

```elixir
    gan> Map.get(%{a: 1}, :b, 0)
    0
```

## put

```elixir
@spec put(mapping(k, v), k, v) :: map(k, v)
```

A new map with `key` set to `value`.

| | |
| --- | --- |
| `m` | The map. |
| `key` | The key to set. |
| `value` | The value to store. |

```elixir
    gan> Map.put(%{a: 1}, :b, 2)
    {'a': 1, 'b': 2}
```

## delete

```elixir
@spec delete(mapping(k, v), k) :: map(k, v)
```

A new map without `key`.

| | |
| --- | --- |
| `m` | The map. |
| `key` | The key to remove. |

```elixir
    gan> Map.delete(%{"a" => 1, "b" => 2}, "a")
    {'b': 2}
```

## keys

```elixir
@spec keys(mapping(k, v)) :: list(k)
```

The keys as a list.

| | |
| --- | --- |
| `m` | The map. |

```elixir
    gan> Map.keys(%{"a" => 1, "b" => 2})
    ['a', 'b']
```

## values

```elixir
@spec values(mapping(k, v)) :: list(v)
```

The values as a list.

| | |
| --- | --- |
| `m` | The map. |

```elixir
    gan> Map.values(%{"a" => 1, "b" => 2})
    [1, 2]
```

## merge

```elixir
@spec merge(mapping(k, v), mapping(k, v)) :: map(k, v)
```

Merges `m2` into `m1`; `m2` wins on conflicts.

| | |
| --- | --- |
| `m1` | The base map. |
| `m2` | Its entries win on conflicts. |

```elixir
    gan> Map.merge(%{"a" => 1}, %{"b" => 2})
    {'a': 1, 'b': 2}
```

## has_key?

```elixir
@spec has_key?(mapping(k, v), k) :: boolean()
```

Whether `key` is present.

| | |
| --- | --- |
| `m` | The map. |
| `key` | The key to test. |

```elixir
    gan> Map.has_key?(%{"a" => 1}, "a")
    True
```

## to_list

```elixir
@spec to_list(mapping(k, v)) :: list(tuple(k, v))
```

The entries as a list of `{key, value}` tuples.

| | |
| --- | --- |
| `m` | The map. |

```elixir
    gan> Map.to_list(%{"a" => 1, "b" => 2})
    [('a', 1), ('b', 2)]
```

## new

```elixir
@spec new() :: map()
```

An empty map.

```elixir
    gan> Map.new([{"a", 1}, {"b", 2}])
    {'a': 1, 'b': 2}
```

## new

```elixir
@spec new() :: map()
```

An empty map.

```elixir
    gan> Map.new([{"a", 1}, {"b", 2}])
    {'a': 1, 'b': 2}
```

## update

```elixir
@spec update(mapping(k, v), k, v, fun()) :: map(k, v)
```

Updates `key` by `f`; uses `default` when the key is absent (Elixir Map.update/4).

| | |
| --- | --- |
| `m` | The map. |
| `key` | The key to update. |
| `default` | Stored when the key is absent. |
| `f` | Applied to the current value when present. |

```elixir
    gan> Map.update(%{a: 1}, :a, 0, fn v -> v + 10 end)
    {'a': 11}
```

## fetch!

```elixir
@spec fetch!(mapping(k, v), k) :: v
```

The value for `key`; raises Python KeyError when absent.

| | |
| --- | --- |
| `m` | The map. |
| `key` | The key that must exist. |

```elixir
    gan> Map.fetch!(%{"a" => 1}, "a")
    1
```

## put_new

```elixir
@spec put_new(mapping(k, v), k, v) :: map(k, v)
```

Sets `key` only when absent.

| | |
| --- | --- |
| `m` | The map. |
| `key` | The key to set only if absent. |
| `value` | The value to store. |

```elixir
    gan> Map.put_new(%{"a" => 1}, "a", 9)
    {'a': 1}
```

## take

```elixir
@spec take(mapping(k, v), sequence(k)) :: map(k, v)
```

The submap with only `keys` (missing keys ignored).

| | |
| --- | --- |
| `m` | The map. |
| `keys` | The keys to keep. |

```elixir
    gan> Map.take(%{"a" => 1, "b" => 2}, ["a"])
    {'a': 1}
```

## drop

```elixir
@spec drop(mapping(k, v), sequence(k)) :: map(k, v)
```

The map without `keys`.

| | |
| --- | --- |
| `m` | The map. |
| `keys` | The keys to remove. |

```elixir
    gan> Map.drop(%{"a" => 1, "b" => 2}, ["a"])
    {'b': 2}
```

## filter

```elixir
@spec filter(mapping(k, v), fun()) :: map(k, v)
```

Keeps entries for which `f` (receiving a `{key, value}` tuple) is truthy.

| | |
| --- | --- |
| `m` | The map. |
| `f` | Receives {key, value}; keeps entries it returns truthy for. |

```elixir
    gan> Map.filter(%{"a" => 1, "b" => 5}, fn pair -> elem(pair, 1) > 2 end)
    {'b': 5}
```

# Enum

Eager, data-first collection functions (GEP-0010). Every subject comes first, so everything pipes.

## map

```elixir
@spec map(sequence(a), fun()) :: list(b)
```

Applies `f` to every element.

| | |
| --- | --- |
| `xs` | The source list. |
| `f` | Applied to each element; the results form the new list. |

```elixir
    gan> Enum.map([1, 2, 3], fn x -> x * 10 end)
    [10, 20, 30]
```

## filter

```elixir
@spec filter(sequence(a), fun()) :: list(a)
```

Keeps elements for which `f` is truthy.

| | |
| --- | --- |
| `xs` | The source list. |
| `f` | Keeps the elements it returns truthy for. |

```elixir
    gan> Enum.filter([1, 2, 3, 4], fn x -> rem(x, 2) == 0 end)
    [2, 4]
```

## reject

```elixir
@spec reject(sequence(a), fun()) :: list(a)
```

Drops elements for which `f` is truthy.

| | |
| --- | --- |
| `xs` | The source list. |
| `f` | Drops the elements it returns truthy for. |

```elixir
    gan> Enum.reject([1, 2, 3, 4], fn x -> rem(x, 2) == 0 end)
    [1, 3]
```

## reduce

```elixir
@spec reduce(sequence(a), b, fun()) :: b
```

Folds left with Elixir argument order: `f` receives (element, acc).

| | |
| --- | --- |
| `xs` | The list to fold. |
| `acc` | The starting accumulator. |
| `f` | Receives an element and the accumulator, returns the next accumulator. |

```elixir
    gan> Enum.reduce([1, 2, 3, 4], 0, fn x, acc -> acc + x end)
    10
```

## sum

```elixir
@spec sum(sequence()) :: number()
```

Sum of the elements.

| | |
| --- | --- |
| `xs` | The numbers to add. |

```elixir
    gan> Enum.sum([1, 2, 3])
    6
```

## count

```elixir
@spec count(sequence()) :: integer()
```

Number of elements.

| | |
| --- | --- |
| `xs` | The list to measure. |

```elixir
    gan> Enum.count([10, 20, 30])
    3
```

## sort

```elixir
@spec sort(sequence()) :: list()
```

Ascending sort.

| | |
| --- | --- |
| `xs` | The list to order ascending. |

```elixir
    gan> Enum.sort([3, 1, 2])
    [1, 2, 3]
```

## sort_by

```elixir
@spec sort_by(sequence(a), fun()) :: list(a)
```

Sorts by the key computed by `f` (Python `key=`, not a comparator).

| | |
| --- | --- |
| `xs` | The list to order. |
| `f` | Maps each element to its sort key. |

```elixir
    gan> Enum.sort_by(["ccc", "a", "bb"], fn s -> String.length(s) end)
    ['a', 'bb', 'ccc']
```

## reverse

```elixir
@spec reverse(sequence(a)) :: list(a)
```

Elements in reverse order.

| | |
| --- | --- |
| `xs` | The list to reverse. |

```elixir
    gan> Enum.reverse([1, 2, 3])
    [3, 2, 1]
```

## join

```elixir
@spec join(sequence(), string()) :: string()
```

Joins elements (converted by `to_string`) with `sep`.

| | |
| --- | --- |
| `xs` | The elements to concatenate. |
| `sep` | Placed between elements. |

```elixir
    gan> Enum.join([1, 2, 3], "-")
    '1-2-3'
```

## at

```elixir
@spec at(sequence(a), integer()) :: a | nil
```

The element at `index` (negative counts from the end), or nil.

| | |
| --- | --- |
| `xs` | The list to index. |
| `index` | Zero-based; negative counts from the end. |

```elixir
    gan> Enum.at([10, 20, 30], 1)
    20
```

## take

```elixir
@spec take(sequence(a), integer()) :: list(a)
```

The first `n` elements; a negative `n` takes the last `-n` (Elixir semantics).

| | |
| --- | --- |
| `xs` | The source list. |
| `n` | How many elements to keep — leading when positive, trailing when negative. |

```elixir
    gan> Enum.take([1, 2, 3], 2)
    [1, 2]
    gan> Enum.take([1, 2, 3, 4, 5], -2)
    [4, 5]
```

## drop

```elixir
@spec drop(sequence(a), integer()) :: list(a)
```

The elements after the first `n`; a negative `n` drops the last `-n` (Elixir semantics).

| | |
| --- | --- |
| `xs` | The source list. |
| `n` | How many elements to skip — leading when positive, trailing when negative. |

```elixir
    gan> Enum.drop([1, 2, 3], 2)
    [3]
    gan> Enum.drop([1, 2, 3, 4, 5], -2)
    [1, 2, 3]
```

## zip

```elixir
@spec zip(sequence(a), sequence(b)) :: list(tuple(a, b))
```

Pairs up two lists into `{a, b}` tuples, stopping at the shorter.

| | |
| --- | --- |
| `xs` | The first list. |
| `ys` | The second list. |

```elixir
    gan> Enum.zip([1, 2], ["a", "b"])
    [(1, 'a'), (2, 'b')]
```

## with_index

```elixir
@spec with_index(sequence(a)) :: list(tuple(a, integer()))
```

Each element paired with its index: `{element, index}`.

| | |
| --- | --- |
| `xs` | The list to enumerate. |

```elixir
    gan> Enum.with_index([:a, :b])
    [('a', 0), ('b', 1)]
```

## member?

```elixir
@spec member?(sequence(a), a) :: boolean()
```

Whether `x` is an element.

| | |
| --- | --- |
| `xs` | The list to search. |
| `x` | The value to look for. |

```elixir
    gan> Enum.member?([1, 2, 3], 2)
    True
```

## all?

```elixir
@spec all?(sequence(a), fun()) :: boolean()
```

Whether `f` is truthy for every element.

| | |
| --- | --- |
| `xs` | The list to test. |
| `f` | The predicate every element must satisfy. |

```elixir
    gan> Enum.all?([1, 2, 3], fn x -> x > 0 end)
    True
```

## any?

```elixir
@spec any?(sequence(a), fun()) :: boolean()
```

Whether `f` is truthy for any element.

| | |
| --- | --- |
| `xs` | The list to test. |
| `f` | The predicate at least one element must satisfy. |

```elixir
    gan> Enum.any?([1, 2, 3], fn x -> x > 2 end)
    True
```

## empty?

```elixir
@spec empty?(sequence()) :: boolean()
```

Whether the collection has no elements.

| | |
| --- | --- |
| `xs` | The list to test. |

```elixir
    gan> Enum.empty?([])
    True
```

## uniq

```elixir
@spec uniq(sequence(a)) :: list(a)
```

Removes duplicates, keeping first occurrences in order.

| | |
| --- | --- |
| `xs` | The list to deduplicate, keeping first occurrences. |

```elixir
    gan> Enum.uniq([1, 2, 1, 3, 2])
    [1, 2, 3]
```

## flat_map

```elixir
@spec flat_map(sequence(a), fun()) :: list(b)
```

Maps `f` (which returns a list) and concatenates the results.

| | |
| --- | --- |
| `xs` | The source list. |
| `f` | Returns a list per element; all are concatenated. |

```elixir
    gan> Enum.flat_map([1, 2], fn x -> [x, x * 10] end)
    [1, 10, 2, 20]
```

## each

```elixir
@spec each(sequence(a), fun()) :: atom()
```

Runs `f` on each element for its side effects; returns :ok.

| | |
| --- | --- |
| `xs` | The list to walk. |
| `f` | Called on each element for its side effect. |

```elixir
    gan> Enum.each(["a", "b"], fn x -> IO.puts(x) end)
    a
    b
    'ok'
```

## min

```elixir
@spec min(sequence()) :: any()
```

The smallest element.

| | |
| --- | --- |
| `xs` | The non-empty list. |

```elixir
    gan> Enum.min([3, 1, 2])
    1
```

## max

```elixir
@spec max(sequence()) :: any()
```

The largest element.

| | |
| --- | --- |
| `xs` | The non-empty list. |

```elixir
    gan> Enum.max([3, 1, 2])
    3
```

## find

```elixir
@spec find(sequence(a), fun()) :: a | nil
```

The first element for which `f` is truthy, or nil.

| | |
| --- | --- |
| `xs` | The list to search. |
| `f` | The predicate; the first truthy match is returned. |

```elixir
    gan> Enum.find([1, 2, 3, 4], fn x -> x > 2 end)
    3
```

## find_index

```elixir
@spec find_index(sequence(), fun()) :: integer() | nil
```

The index of the first element for which `f` is truthy, or nil.

| | |
| --- | --- |
| `xs` | The list to search. |
| `f` | The predicate; the index of the first match is returned. |

```elixir
    gan> Enum.find_index(["a", "b", "c"], fn x -> x == "b" end)
    1
```

## frequencies

```elixir
@spec frequencies(sequence(a)) :: map(a, integer())
```

A map from each distinct element to its occurrence count.

| | |
| --- | --- |
| `xs` | The elements to tally. |

```elixir
    gan> Enum.frequencies(["a", "b", "a"])
    {'a': 2, 'b': 1}
```

## group_by

```elixir
@spec group_by(sequence(a), fun()) :: map(b, list(a))
```

Groups elements by the key computed by `f`, preserving order.

| | |
| --- | --- |
| `xs` | The elements to group. |
| `f` | Maps an element to its group key. |

```elixir
    gan> Enum.group_by([1, 2, 3, 4, 5], fn x -> rem(x, 2) end)
    {1: [1, 3, 5], 0: [2, 4]}
```

## max_by

```elixir
@spec max_by(sequence(a), fun()) :: a
```

The element maximizing the key computed by `f`; raises on empty.

| | |
| --- | --- |
| `xs` | The non-empty list. |
| `f` | Maps an element to its comparison key. |

```elixir
    gan> Enum.max_by(["a", "bbb", "cc"], fn s -> String.length(s) end)
    'bbb'
```

## min_by

```elixir
@spec min_by(sequence(a), fun()) :: a
```

The element minimizing the key computed by `f`; raises on empty.

| | |
| --- | --- |
| `xs` | The non-empty list. |
| `f` | Maps an element to its comparison key. |

```elixir
    gan> Enum.min_by(["a", "bbb", "cc"], fn s -> String.length(s) end)
    'a'
```

## product

```elixir
@spec product(sequence()) :: number()
```

The product of the elements (1 for an empty collection).

| | |
| --- | --- |
| `xs` | The numbers to multiply. |

```elixir
    gan> Enum.product([2, 3, 4])
    24
```

## take_while

```elixir
@spec take_while(sequence(a), fun()) :: list(a)
```

Leading elements while `f` stays truthy.

| | |
| --- | --- |
| `xs` | The source list. |
| `f` | Elements are taken while it returns truthy. |

```elixir
    gan> Enum.take_while([1, 2, 0, 3], fn x -> x > 0 end)
    [1, 2]
```

## drop_while

```elixir
@spec drop_while(sequence(a), fun()) :: list(a)
```

Drops leading elements while `f` stays truthy, keeps the rest.

| | |
| --- | --- |
| `xs` | The source list. |
| `f` | Elements are dropped while it returns truthy. |

```elixir
    gan> Enum.drop_while([1, 2, 0, 3], fn x -> x > 0 end)
    [0, 3]
```

## chunk_every

```elixir
@spec chunk_every(sequence(a), integer(), integer() | nil) :: list(list(a))
```

Chunks of `n` elements; `step` slides the window (Elixir chunk_every/3, default n). The tail chunk may be shorter.

| | |
| --- | --- |
| `xs` | The list to split. |
| `n` | The chunk size; the last chunk may be shorter. |
| `step` | How far the window advances between chunks (defaults to `n`). |

```elixir
    gan> Enum.chunk_every([1, 2, 3, 4, 5], 2)
    [[1, 2], [3, 4], [5]]
    gan> Enum.chunk_every([2, 1, 5, 1, 3, 2], 3, 1)
    [[2, 1, 5], [1, 5, 1], [5, 1, 3], [1, 3, 2], [3, 2], [2]]
```

## concat

```elixir
@spec concat(sequence(sequence(a))) :: list(a)
```

Concatenates a list of lists (one level).

| | |
| --- | --- |
| `xss` | A list of lists to flatten one level. |

```elixir
    gan> Enum.concat([[1, 2], [3], []])
    [1, 2, 3]
```

## intersperse

```elixir
@spec intersperse(sequence(a), a) :: list(a)
```

Puts `sep` between every two elements.

| | |
| --- | --- |
| `xs` | The source list. |
| `sep` | Inserted between every two elements. |

```elixir
    gan> Enum.intersperse([1, 2, 3], :x)
    [1, 'x', 2, 'x', 3]
```

## slice

```elixir
@spec slice(sequence(a), integer(), integer()) :: list(a)
```

`count` elements starting at `start` (negative start counts from the end).

| | |
| --- | --- |
| `xs` | The source list. |
| `start` | Zero-based start position. |
| `count` | Maximum number of elements. |

```elixir
    gan> Enum.slice([1, 2, 3, 4], 1, 2)
    [2, 3]
```

## dedup

```elixir
@spec dedup(sequence(a)) :: list(a)
```

Collapses consecutive duplicate elements.

| | |
| --- | --- |
| `xs` | The list whose adjacent duplicates collapse. |

```elixir
    gan> Enum.dedup([1, 1, 2, 2, 2, 1])
    [1, 2, 1]
```

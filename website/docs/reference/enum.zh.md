# Enum

急切求值、数据优先的集合函数（GEP-0010）。主语在首位，全部可入管道。

## map

```elixir
@spec map(sequence(a), fun()) :: list(b)
```

对每个元素应用 `f`。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `f` | 作用于每个元素，其结果构成新列表。 |

```elixir
    gan> Enum.map([1, 2, 3], fn x -> x * 10 end)
    [10, 20, 30]
```

## filter

```elixir
@spec filter(sequence(a), fun()) :: list(a)
```

保留 `f` 为真值的元素。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `f` | 保留使其返回真值的元素。 |

```elixir
    gan> Enum.filter([1, 2, 3, 4], fn x -> rem(x, 2) == 0 end)
    [2, 4]
```

## reject

```elixir
@spec reject(sequence(a), fun()) :: list(a)
```

丢弃 `f` 为真值的元素。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `f` | 丢弃使其返回真值的元素。 |

```elixir
    gan> Enum.reject([1, 2, 3, 4], fn x -> rem(x, 2) == 0 end)
    [1, 3]
```

## reduce

```elixir
@spec reduce(sequence(a), b, fun()) :: b
```

左折叠，参数顺序与 Elixir 一致：`f` 接收 (元素, 累积值)。

| | |
| --- | --- |
| `xs` | 要折叠的列表。 |
| `acc` | 初始累加值。 |
| `f` | 接收元素与累加值，返回新的累加值。 |

```elixir
    gan> Enum.reduce([1, 2, 3, 4], 0, fn x, acc -> acc + x end)
    10
```

## sum

```elixir
@spec sum(sequence()) :: number()
```

元素之和。

| | |
| --- | --- |
| `xs` | 要求和的数字。 |

```elixir
    gan> Enum.sum([1, 2, 3])
    6
```

## count

```elixir
@spec count(sequence()) :: integer()
```

元素个数。

| | |
| --- | --- |
| `xs` | 要计数的列表。 |

```elixir
    gan> Enum.count([10, 20, 30])
    3
```

## sort

```elixir
@spec sort(sequence()) :: list()
```

升序排序。

| | |
| --- | --- |
| `xs` | 要升序排序的列表。 |

```elixir
    gan> Enum.sort([3, 1, 2])
    [1, 2, 3]
```

## sort_by

```elixir
@spec sort_by(sequence(a), fun()) :: list(a)
```

按 `f` 计算的键排序（Python 的 `key=`，非比较器）。

| | |
| --- | --- |
| `xs` | 要排序的列表。 |
| `f` | 将元素映射为排序键。 |

```elixir
    gan> Enum.sort_by(["ccc", "a", "bb"], fn s -> String.length(s) end)
    ['a', 'bb', 'ccc']
```

## reverse

```elixir
@spec reverse(sequence(a)) :: list(a)
```

逆序排列的元素。

| | |
| --- | --- |
| `xs` | 要反转的列表。 |

```elixir
    gan> Enum.reverse([1, 2, 3])
    [3, 2, 1]
```

## join

```elixir
@spec join(sequence(), string()) :: string()
```

以 `sep` 连接各元素（先经 `to_string` 转换）。

| | |
| --- | --- |
| `xs` | 要拼接的元素。 |
| `sep` | 插入元素之间的分隔串。 |

```elixir
    gan> Enum.join([1, 2, 3], "-")
    '1-2-3'
```

## at

```elixir
@spec at(sequence(a), integer()) :: a | nil
```

位于 `index` 的元素（负数从尾部数起），越界返回 nil。

| | |
| --- | --- |
| `xs` | 要取值的列表。 |
| `index` | 从 0 开始；负数自末尾计。 |

```elixir
    gan> Enum.at([10, 20, 30], 1)
    20
```

## take

```elixir
@spec take(sequence(a), integer()) :: list(a)
```

前 `n` 个元素;负数 `n` 取末尾 `-n` 个(Elixir 语义)。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `n` | 保留数量——正数取头部,负数取尾部。 |

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

跳过前 `n` 个之后的元素;负数 `n` 丢弃末尾 `-n` 个(Elixir 语义)。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `n` | 跳过数量——正数跳头部,负数丢尾部。 |

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

将两个列表配成 `{a, b}` 元组，止于较短者。

| | |
| --- | --- |
| `xs` | 第一个列表。 |
| `ys` | 第二个列表。 |

```elixir
    gan> Enum.zip([1, 2], ["a", "b"])
    [(1, 'a'), (2, 'b')]
```

## with_index

```elixir
@spec with_index(sequence(a)) :: list(tuple(a, integer()))
```

每个元素与其下标配对：`{元素, 下标}`。

| | |
| --- | --- |
| `xs` | 要编号的列表。 |

```elixir
    gan> Enum.with_index([:a, :b])
    [('a', 0), ('b', 1)]
```

## member?

```elixir
@spec member?(sequence(a), a) :: boolean()
```

`x` 是否为其中元素。

| | |
| --- | --- |
| `xs` | 被查找的列表。 |
| `x` | 要查找的值。 |

```elixir
    gan> Enum.member?([1, 2, 3], 2)
    True
```

## all?

```elixir
@spec all?(sequence(a), fun()) :: boolean()
```

`f` 是否对所有元素为真值。

| | |
| --- | --- |
| `xs` | 被检验的列表。 |
| `f` | 每个元素都需满足的谓词。 |

```elixir
    gan> Enum.all?([1, 2, 3], fn x -> x > 0 end)
    True
```

## any?

```elixir
@spec any?(sequence(a), fun()) :: boolean()
```

`f` 是否对任一元素为真值。

| | |
| --- | --- |
| `xs` | 被检验的列表。 |
| `f` | 至少一个元素需满足的谓词。 |

```elixir
    gan> Enum.any?([1, 2, 3], fn x -> x > 2 end)
    True
```

## empty?

```elixir
@spec empty?(sequence()) :: boolean()
```

集合是否为空。

| | |
| --- | --- |
| `xs` | 被检验的列表。 |

```elixir
    gan> Enum.empty?([])
    True
```

## uniq

```elixir
@spec uniq(sequence(a)) :: list(a)
```

去重，按序保留首次出现者。

| | |
| --- | --- |
| `xs` | 要去重的列表，保留首次出现。 |

```elixir
    gan> Enum.uniq([1, 2, 1, 3, 2])
    [1, 2, 3]
```

## flat_map

```elixir
@spec flat_map(sequence(a), fun()) :: list(b)
```

对每个元素应用返回列表的 `f`，并拼接全部结果。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `f` | 对每个元素返回一个列表，随后拼接。 |

```elixir
    gan> Enum.flat_map([1, 2], fn x -> [x, x * 10] end)
    [1, 10, 2, 20]
```

## each

```elixir
@spec each(sequence(a), fun()) :: atom()
```

对每个元素执行 `f`（取其副作用）；返回 :ok。

| | |
| --- | --- |
| `xs` | 要遍历的列表。 |
| `f` | 对每个元素调用以产生副作用。 |

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

最小元素。

| | |
| --- | --- |
| `xs` | 非空列表。 |

```elixir
    gan> Enum.min([3, 1, 2])
    1
```

## max

```elixir
@spec max(sequence()) :: any()
```

最大元素。

| | |
| --- | --- |
| `xs` | 非空列表。 |

```elixir
    gan> Enum.max([3, 1, 2])
    3
```

## find

```elixir
@spec find(sequence(a), fun()) :: a | nil
```

首个使 `f` 为真值的元素，无则 nil。

| | |
| --- | --- |
| `xs` | 被查找的列表。 |
| `f` | 谓词；返回第一个使其为真的元素。 |

```elixir
    gan> Enum.find([1, 2, 3, 4], fn x -> x > 2 end)
    3
```

## find_index

```elixir
@spec find_index(sequence(), fun()) :: integer() | nil
```

首个使 `f` 为真值的元素下标，无则 nil。

| | |
| --- | --- |
| `xs` | 被查找的列表。 |
| `f` | 谓词；返回首个匹配的下标。 |

```elixir
    gan> Enum.find_index(["a", "b", "c"], fn x -> x == "b" end)
    1
```

## frequencies

```elixir
@spec frequencies(sequence(a)) :: map(a, integer())
```

从每个不同元素到其出现次数的映射。

| | |
| --- | --- |
| `xs` | 要统计频次的元素。 |

```elixir
    gan> Enum.frequencies(["a", "b", "a"])
    {'a': 2, 'b': 1}
```

## group_by

```elixir
@spec group_by(sequence(a), fun()) :: map(b, list(a))
```

按 `f` 计算的键分组，保持顺序。

| | |
| --- | --- |
| `xs` | 要分组的元素。 |
| `f` | 将元素映射为分组键。 |

```elixir
    gan> Enum.group_by([1, 2, 3, 4, 5], fn x -> rem(x, 2) end)
    {1: [1, 3, 5], 0: [2, 4]}
```

## max_by

```elixir
@spec max_by(sequence(a), fun()) :: a
```

使 `f` 计算的键最大的元素；空集合抛错。

| | |
| --- | --- |
| `xs` | 非空列表。 |
| `f` | 将元素映射为比较键。 |

```elixir
    gan> Enum.max_by(["a", "bbb", "cc"], fn s -> String.length(s) end)
    'bbb'
```

## min_by

```elixir
@spec min_by(sequence(a), fun()) :: a
```

使 `f` 计算的键最小的元素；空集合抛错。

| | |
| --- | --- |
| `xs` | 非空列表。 |
| `f` | 将元素映射为比较键。 |

```elixir
    gan> Enum.min_by(["a", "bbb", "cc"], fn s -> String.length(s) end)
    'a'
```

## product

```elixir
@spec product(sequence()) :: number()
```

元素之积（空集合为 1）。

| | |
| --- | --- |
| `xs` | 要相乘的数字。 |

```elixir
    gan> Enum.product([2, 3, 4])
    24
```

## take_while

```elixir
@spec take_while(sequence(a), fun()) :: list(a)
```

自头部起 `f` 持续为真值的元素。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `f` | 持续取元素直到其返回假值。 |

```elixir
    gan> Enum.take_while([1, 2, 0, 3], fn x -> x > 0 end)
    [1, 2]
```

## drop_while

```elixir
@spec drop_while(sequence(a), fun()) :: list(a)
```

丢弃头部 `f` 持续为真值的元素，保留其余。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `f` | 持续丢弃元素直到其返回假值。 |

```elixir
    gan> Enum.drop_while([1, 2, 0, 3], fn x -> x > 0 end)
    [0, 3]
```

## chunk_every

```elixir
@spec chunk_every(sequence(a), integer(), integer() | nil) :: list(list(a))
```

按每组 `n` 个元素分块;`step` 为滑动步长(Elixir chunk_every/3,默认为 n);末块可较短。

| | |
| --- | --- |
| `xs` | 要切分的列表。 |
| `n` | 每块大小；末块可更短。 |
| `step` | 相邻块之间的步长(默认为 `n`)。 |

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

拼接列表的列表（单层）。

| | |
| --- | --- |
| `xss` | 要展平一层的列表的列表。 |

```elixir
    gan> Enum.concat([[1, 2], [3], []])
    [1, 2, 3]
```

## intersperse

```elixir
@spec intersperse(sequence(a), a) :: list(a)
```

在每两个元素之间放入 `sep`。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `sep` | 插入相邻元素之间的值。 |

```elixir
    gan> Enum.intersperse([1, 2, 3], :x)
    [1, 'x', 2, 'x', 3]
```

## slice

```elixir
@spec slice(sequence(a), integer(), integer()) :: list(a)
```

自 `start` 起的 `count` 个元素（负的 start 从尾部数起）。

| | |
| --- | --- |
| `xs` | 来源列表。 |
| `start` | 起始下标（从 0 起）。 |
| `count` | 最多取出的元素个数。 |

```elixir
    gan> Enum.slice([1, 2, 3, 4], 1, 2)
    [2, 3]
```

## dedup

```elixir
@spec dedup(sequence(a)) :: list(a)
```

折叠连续的重复元素。

| | |
| --- | --- |
| `xs` | 相邻重复元素折叠的列表。 |

```elixir
    gan> Enum.dedup([1, 1, 2, 2, 2, 1])
    [1, 2, 1]
```

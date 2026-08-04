# String

基于 Python str 的数据优先字符串函数(GEP-0010)。数字格式化直接用 Python 的 format 迷你语言:"{:.2f}".format(3.14159) 得 '3.14'。

## upcase

```elixir
@spec upcase(string()) :: string()
```

转为大写。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.upcase("abc")
    'ABC'
```

## downcase

```elixir
@spec downcase(string()) :: string()
```

转为小写。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.downcase("AbC")
    'abc'
```

## capitalize

```elixir
@spec capitalize(string()) :: string()
```

首字符大写，其余小写。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.capitalize("heLLo")
    'Hello'
```

## split

```elixir
@spec split(string()) :: list(string())
```

按连续空白切分并丢弃空段（Elixir 语义）。

| | |
| --- | --- |
| `s` | 按连续空白切分的字符串。 |

```elixir
    gan> String.split("  a  b c ")
    ['a', 'b', 'c']
```

## split_on

```elixir
@spec split_on(string(), string()) :: list(string())
```

按分隔符 `sep` 切分。

| | |
| --- | --- |
| `s` | 字符串。 |
| `sep` | 切分用的分隔串。 |

```elixir
    gan> String.split_on("a,b,c", ",")
    ['a', 'b', 'c']
```

## trim

```elixir
@spec trim(string()) :: string()
```

去除首尾空白。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.trim("  x  ")
    'x'
```

## replace

```elixir
@spec replace(string(), string(), string()) :: string()
```

将每处 `pattern` 替换为 `replacement`。

| | |
| --- | --- |
| `s` | 字符串。 |
| `pattern` | 要查找的子串。 |
| `replacement` | 替换每处出现的内容。 |

```elixir
    gan> String.replace("a-b-a", "a", "x")
    'x-b-x'
```

## contains?

```elixir
@spec contains?(string(), string()) :: boolean()
```

是否包含子串 `part`。

| | |
| --- | --- |
| `s` | 字符串。 |
| `part` | 要查找的子串。 |

```elixir
    gan> String.contains?("hello", "ell")
    True
```

## starts_with?

```elixir
@spec starts_with?(string(), string()) :: boolean()
```

是否以 `prefix` 开头。

| | |
| --- | --- |
| `s` | 字符串。 |
| `prefix` | 候选前缀。 |

```elixir
    gan> String.starts_with?("file.gan", "file")
    True
```

## ends_with?

```elixir
@spec ends_with?(string(), string()) :: boolean()
```

是否以 `suffix` 结尾。

| | |
| --- | --- |
| `s` | 字符串。 |
| `suffix` | 候选后缀。 |

```elixir
    gan> String.ends_with?("file.gan", ".gan")
    True
```

## length

```elixir
@spec length(string()) :: integer()
```

字符数（Unicode 码点，即 Python 的 `len`）。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.length("héllo")
    5
```

## slice

```elixir
@spec slice(string(), integer(), integer()) :: string()
```

自 `start` 起、长 `len` 的子串（负的 start 从尾部数起）。

| | |
| --- | --- |
| `s` | 字符串。 |
| `start` | 起始下标（从 0 起）。 |
| `len` | 最多取出的字符数。 |

```elixir
    gan> String.slice("gandora", 3, 4)
    'dora'
```

## pad_leading

```elixir
@spec pad_leading(string(), integer()) :: string()
```

左侧以空格填充至 `width`。

| | |
| --- | --- |
| `s` | 字符串。 |
| `width` | 目标最小总宽度。 |

```elixir
    gan> String.pad_leading("5", 3)
    '  5'
```

## pad_trailing

```elixir
@spec pad_trailing(string(), integer()) :: string()
```

右侧以空格填充至 `width`。

| | |
| --- | --- |
| `s` | 字符串。 |
| `width` | 目标最小总宽度。 |

```elixir
    gan> String.pad_trailing("5", 3)
    '5  '
```

## to_integer

```elixir
@spec to_integer(string()) :: integer()
```

解析整数；非法输入抛 Python ValueError。

| | |
| --- | --- |
| `s` | 要解析的十进制数字串。 |

```elixir
    gan> String.to_integer("42")
    42
```

## to_float

```elixir
@spec to_float(string()) :: float()
```

解析浮点数；非法输入抛 Python ValueError。

| | |
| --- | --- |
| `s` | 要解析的数字字面串。 |

```elixir
    gan> String.to_float("2.5")
    2.5
```

## at

```elixir
@spec at(string(), integer()) :: string() | nil
```

位于 `index` 的字符（负数从尾部数起），越界为 nil。

| | |
| --- | --- |
| `s` | 字符串。 |
| `index` | 从 0 开始；负数自末尾计。 |

```elixir
    gan> String.at("abc", 1)
    'b'
```

## first

```elixir
@spec first(string()) :: string() | nil
```

第一个字符;空串为 nil。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.first("ant")
    'a'
    gan> String.first("")
```

## last

```elixir
@spec last(string()) :: string() | nil
```

最后一个字符;空串为 nil。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.last("ant")
    't'
```

## reverse

```elixir
@spec reverse(string()) :: string()
```

反转后的字符串。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.reverse("abc")
    'cba'
```

## duplicate

```elixir
@spec duplicate(string(), integer()) :: string()
```

重复 `n` 次的字符串。

| | |
| --- | --- |
| `s` | 要重复的字符串。 |
| `n` | 重复次数。 |

```elixir
    gan> String.duplicate("ab", 2)
    'abab'
```

## trim_leading

```elixir
@spec trim_leading(string()) :: string()
```

去除首部空白。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.trim_leading("  x")
    'x'
```

## trim_trailing

```elixir
@spec trim_trailing(string()) :: string()
```

去除尾部空白。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.trim_trailing("x  ")
    'x'
```

## codepoints

```elixir
@spec codepoints(string()) :: list(string())
```

以单字符字符串列表表示的全部字符。

| | |
| --- | --- |
| `s` | 字符串。 |

```elixir
    gan> String.codepoints("héllo") |> Enum.take(2)
    ['h', 'é']
```

## match?

```elixir
@spec match?(string(), $re.Pattern) :: boolean()
```

编译后的正则（`~r/.../`）是否在字符串中命中。

| | |
| --- | --- |
| `s` | 被检验的字符串。 |
| `regex` | 编译后的正则，如 ~r// 的结果。 |

```elixir
    gan> String.match?("gandora-2026", ~r/\d+/)
    True
```

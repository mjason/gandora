# String

Data-first string functions over Python str (GEP-0010).

Number formatting is Python's own format mini-language, called as a
method on a string literal: `"{:.2f}".format(3.14159)` gives '3.14',
`"{:>8}".format(x)` pads — no wrapper needed.


## upcase

```elixir
@spec upcase(string()) :: string()
```

Uppercases the string.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.upcase("abc")
    'ABC'
```

## downcase

```elixir
@spec downcase(string()) :: string()
```

Lowercases the string.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.downcase("AbC")
    'abc'
```

## capitalize

```elixir
@spec capitalize(string()) :: string()
```

Uppercases the first character, lowercases the rest.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.capitalize("heLLo")
    'Hello'
```

## split

```elixir
@spec split(string()) :: list(string())
```

Splits on whitespace runs, dropping empty parts (Elixir semantics).

| | |
| --- | --- |
| `s` | Split on runs of whitespace. |

```elixir
    gan> String.split("  a  b c ")
    ['a', 'b', 'c']
```

## split_on

```elixir
@spec split_on(string(), string()) :: list(string())
```

Splits on the separator `sep`.

| | |
| --- | --- |
| `s` | The string. |
| `sep` | The separator to split on. |

```elixir
    gan> String.split_on("a,b,c", ",")
    ['a', 'b', 'c']
```

## trim

```elixir
@spec trim(string()) :: string()
```

Removes leading and trailing whitespace.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.trim("  x  ")
    'x'
```

## replace

```elixir
@spec replace(string(), string(), string()) :: string()
```

Replaces every occurrence of `pattern` with `replacement`.

| | |
| --- | --- |
| `s` | The string. |
| `pattern` | The substring to find. |
| `replacement` | Substituted for every occurrence. |

```elixir
    gan> String.replace("a-b-a", "a", "x")
    'x-b-x'
```

## contains?

```elixir
@spec contains?(string(), string()) :: boolean()
```

Whether the string contains `part`.

| | |
| --- | --- |
| `s` | The string. |
| `part` | The substring to look for. |

```elixir
    gan> String.contains?("hello", "ell")
    True
```

## starts_with?

```elixir
@spec starts_with?(string(), string()) :: boolean()
```

Whether the string starts with `prefix`.

| | |
| --- | --- |
| `s` | The string. |
| `prefix` | The candidate prefix. |

```elixir
    gan> String.starts_with?("file.gan", "file")
    True
```

## ends_with?

```elixir
@spec ends_with?(string(), string()) :: boolean()
```

Whether the string ends with `suffix`.

| | |
| --- | --- |
| `s` | The string. |
| `suffix` | The candidate suffix. |

```elixir
    gan> String.ends_with?("file.gan", ".gan")
    True
```

## length

```elixir
@spec length(string()) :: integer()
```

The number of characters (Unicode code points, Python `len`).

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.length("héllo")
    5
```

## slice

```elixir
@spec slice(string(), integer(), integer()) :: string()
```

The substring of `len` characters starting at `start` (negative start counts from the end).

| | |
| --- | --- |
| `s` | The string. |
| `start` | Zero-based start position. |
| `len` | Maximum number of characters. |

```elixir
    gan> String.slice("gandora", 3, 4)
    'dora'
```

## pad_leading

```elixir
@spec pad_leading(string(), integer()) :: string()
```

Pads on the left with spaces to `width`.

| | |
| --- | --- |
| `s` | The string. |
| `width` | The minimum total width. |

```elixir
    gan> String.pad_leading("5", 3)
    '  5'
```

## pad_trailing

```elixir
@spec pad_trailing(string(), integer()) :: string()
```

Pads on the right with spaces to `width`.

| | |
| --- | --- |
| `s` | The string. |
| `width` | The minimum total width. |

```elixir
    gan> String.pad_trailing("5", 3)
    '5  '
```

## to_integer

```elixir
@spec to_integer(string()) :: integer()
```

Parses an integer; raises Python ValueError on bad input.

| | |
| --- | --- |
| `s` | The decimal digits to parse. |

```elixir
    gan> String.to_integer("42")
    42
```

## to_float

```elixir
@spec to_float(string()) :: float()
```

Parses a float; raises Python ValueError on bad input.

| | |
| --- | --- |
| `s` | The number literal to parse. |

```elixir
    gan> String.to_float("2.5")
    2.5
```

## at

```elixir
@spec at(string(), integer()) :: string() | nil
```

The character at `index` (negative counts from the end), or nil.

| | |
| --- | --- |
| `s` | The string. |
| `index` | Zero-based; negative counts from the end. |

```elixir
    gan> String.at("abc", 1)
    'b'
```

## first

```elixir
@spec first(string()) :: string() | nil
```

The first character, or nil for the empty string.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.first("ant")
    'a'
    gan> String.first("")
```

## last

```elixir
@spec last(string()) :: string() | nil
```

The last character, or nil for the empty string.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.last("ant")
    't'
```

## reverse

```elixir
@spec reverse(string()) :: string()
```

The string reversed.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.reverse("abc")
    'cba'
```

## duplicate

```elixir
@spec duplicate(string(), integer()) :: string()
```

The string repeated `n` times.

| | |
| --- | --- |
| `s` | The string to repeat. |
| `n` | How many times. |

```elixir
    gan> String.duplicate("ab", 2)
    'abab'
```

## trim_leading

```elixir
@spec trim_leading(string()) :: string()
```

Removes leading whitespace.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.trim_leading("  x")
    'x'
```

## trim_trailing

```elixir
@spec trim_trailing(string()) :: string()
```

Removes trailing whitespace.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.trim_trailing("x  ")
    'x'
```

## codepoints

```elixir
@spec codepoints(string()) :: list(string())
```

The characters as a list of one-character strings.

| | |
| --- | --- |
| `s` | The string. |

```elixir
    gan> String.codepoints("héllo") |> Enum.take(2)
    ['h', 'é']
```

## match?

```elixir
@spec match?(string(), $re.Pattern) :: boolean()
```

Whether the compiled regex (`~r/.../`) matches anywhere in the string.

| | |
| --- | --- |
| `s` | The string to test. |
| `regex` | A compiled pattern, e.g. from ~r//. |

```elixir
    gan> String.match?("gandora-2026", ~r/\d+/)
    True
```

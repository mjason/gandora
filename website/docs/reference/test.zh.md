# Test

The official test assertions (GEP-0024): write `tests/*.gan` modules
whose `test_*` functions call these — `gan test` compiles them and
runs pytest underneath. A failure raises; pytest reports it.


## assert_eq

```elixir
@spec assert_eq(term(), term()) :: nil
```

断言相等,失败时同时报出期望值与实际值。

| | |
| --- | --- |
| `got` | 实际值。 |
| `want` | 期望值。 |

```elixir
    gan> Test.assert_eq(1 + 1, 2)
```

## assert_true

```elixir
@spec assert_true(term()) :: nil
```

断言值恰为 true。

| | |
| --- | --- |
| `value` | 被检验的布尔值。 |

```elixir
    gan> Test.assert_true(2 > 1)
```

## assert_false

```elixir
@spec assert_false(term()) :: nil
```

断言值恰为 false。

| | |
| --- | --- |
| `value` | 被检验的布尔值。 |

```elixir
    gan> Test.assert_false(1 > 2)
```

## assert_nil

```elixir
@spec assert_nil(term()) :: nil
```

断言值为 nil。

| | |
| --- | --- |
| `value` | 被检验的值。 |

```elixir
    gan> Test.assert_nil(Map.get(%{}, "missing"))
```

## assert_raises

```elixir
@spec assert_raises(fun()) :: string()
```

断言零元函数会抛异常,返回异常消息。

| | |
| --- | --- |
| `f` | 预期抛异常的零元函数。 |

## assert_contains

```elixir
@spec assert_contains(term(), term()) :: nil
```

断言字符串或列表包含给定元素。

| | |
| --- | --- |
| `haystack` | 被搜索的字符串或列表。 |
| `needle` | 必须存在的内容。 |

```elixir
    gan> Test.assert_contains([1, 2, 3], 2)
```

## assert_op

```elixir
@spec assert_op(string(), term(), term()) :: nil
```



## assert_truthy

```elixir
@spec assert_truthy(term()) :: nil
```



## refute_truthy

```elixir
@spec refute_truthy(term()) :: nil
```



## refute_in

```elixir
@spec refute_in(term(), term()) :: nil
```



## assert_raise

```elixir
@spec assert_raise(term(), fun()) :: string()
```

Fails unless `f` raises an instance of `type` (ExUnit assert_raise/2).

| | |
| --- | --- |
| `type` | The exception class, e.g. $builtins.ValueError. |
| `f` | A zero-arity function expected to raise it. |

## assert_in_delta

```elixir
@spec assert_in_delta(number(), number(), number()) :: nil
```

Fails unless `abs(a - b) <= delta` (ExUnit assert_in_delta/3).

| | |
| --- | --- |
| `a` | One value. |
| `b` | The other value. |
| `delta` | The allowed absolute difference. |

```elixir
    gan> Test.assert_in_delta(3.14159, 3.14, 0.01)
```

## flunk

```elixir
@spec flunk(string()) :: nil
```

Fails immediately with the given message (ExUnit flunk/1).

| | |
| --- | --- |
| `message` | Why the test fails. |

# Test

The official test assertions (GEP-0024): write `tests/*.gan` modules
whose `test_*` functions call these — `gan test` compiles them and
runs pytest underneath. A failure raises; pytest reports it.


## assert_eq

```elixir
@spec assert_eq(term(), term()) :: nil
```

Fails unless `got == want`, naming both sides.

| | |
| --- | --- |
| `got` | The actual value. |
| `want` | The expected value. |

```elixir
    gan> Test.assert_eq(1 + 1, 2)
```

## assert_true

```elixir
@spec assert_true(term()) :: nil
```

Fails unless the value is exactly `true`.

| | |
| --- | --- |
| `value` | The boolean under test. |

```elixir
    gan> Test.assert_true(2 > 1)
```

## assert_false

```elixir
@spec assert_false(term()) :: nil
```

Fails unless the value is exactly `false`.

| | |
| --- | --- |
| `value` | The boolean under test. |

```elixir
    gan> Test.assert_false(1 > 2)
```

## assert_nil

```elixir
@spec assert_nil(term()) :: nil
```

Fails unless the value is `nil`.

| | |
| --- | --- |
| `value` | The value under test. |

```elixir
    gan> Test.assert_nil(Map.get(%{}, "missing"))
```

## assert_raises

```elixir
@spec assert_raises(fun()) :: string()
```

Runs `f` and fails unless it raises; returns the message.

| | |
| --- | --- |
| `f` | A zero-arity function expected to raise. |

## assert_contains

```elixir
@spec assert_contains(term(), term()) :: nil
```

Fails unless `haystack` contains `needle` (string or list).

| | |
| --- | --- |
| `haystack` | The string or list searched. |
| `needle` | What must be present. |

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

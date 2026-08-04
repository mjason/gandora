# Testing

One command, two layers: `gan test` runs every `@example` doctest,
then every `test_*` function of `tests/*.gan` — compiled with the
project's full module resolution and executed by pytest (add it once:
`uv add --dev pytest`).

## Doctests

`@example` is the only doctest channel — documentation that cannot
rot:

```elixir
@example """
    gan> Stats.mean([1, 2, 3, 4])
    2.5
"""
```

Expected output is the Python `repr` of the value: atoms print as
`'ok'`, tuples as `('ok', 21)`, maps as `{'k': 1}`, booleans as
`True`.

## The ExUnit surface

```elixir
# tests/test_stats.gan
defmodule TestStats do
  @moduledoc "Edge cases the doctests don't cover."

  use Test

  describe "mean" do
    test "averages evenly" do
      assert Stats.mean([1, 2, 3, 4]) == 2.5   # failure names left and right
    end
  end

  test "membership and negation" do
    assert 16 in Stats.even_squares(1, 8)
    refute 9 in Stats.even_squares(1, 8)
  end

  test "typed raises" do
    _ = Test.assert_raise($builtins.KeyError, fn -> Map.fetch!(%{}, "no") end)
    nil
  end
end
```

- `test "name" do` defines `test_<slug>`; `describe` prefixes inner
  names; `assert a == b` reports **both operands** on failure;
  `refute` negates; `in` asserts membership.
- Plain-function assertions are also there: `Test.assert_eq`,
  `assert_nil`, `assert_true`, `assert_raises`, `assert_contains`,
  `assert_raise/2` (typed), `assert_in_delta/3`, `flunk/1` — see the
  [Test reference](../reference/test.md).
- The macros compile to ordinary defs, so pytest sees ordinary tests —
  and they work from installed wheels, because packages ship their
  `.gan` sources.

`tests/` never ships — it lives outside the source roots. The build's
verdict covers top-level `tests/*.gan` with the teaching pass (but
test modules are exempt from library annotation coverage), and the
whole conformance suite of Gandora itself — check BDD, formatter
contract, a JSON-RPC client driving the language server — is written
exactly this way.

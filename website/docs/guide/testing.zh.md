# Testing

一条命令，两层测试：`gan test` 运行所有 `@example` 文档测试，接着运行 `tests/*.gan` 中的所有 `test_*` 函数——这些函数使用项目的完整模块解析进行编译，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

## Doctests

`@example` 是唯一的文档测试通道——不会过时的文档：

```elixir
@example """
    gan> Stats.mean([1, 2, 3, 4])
    2.5
"""
```

预期输出是值的 Python `repr`：原子打印为 `'ok'`，元组为 `('ok', 21)`，映射为 `{'k': 1}`，布尔值为 `True`。

## ExUnit 表面

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

- `test "name" do` 定义 `test_<slug>`；`describe` 为内部测试名添加前缀；`assert a == b` 在失败时报告**两个操作数**；`refute` 取反；`in` 断言成员关系。
- 纯函数断言也同样存在：`Test.assert_eq`、`assert_nil`、`assert_true`、`assert_raises`、`assert_contains`、`assert_raise/2`（带类型）、`assert_in_delta/3`、`flunk/1`——参见 [Test 参考](../reference/test.md)。
- 这些宏在编译后变成普通的 def，因此 pytest 看到的也是普通测试——并且它们可以从安装的 wheel 中正常工作，因为包中包含了 `.gan` 源码。

`tests/` 永远不会被打包发行——它位于源码根目录之外。构建的结论覆盖了顶层 `tests/*.gan` 及其教学通过（但测试模块不受库注解覆盖率约束），而 Gandora 本身的整个一致性套件——包括 BDD 检查、格式化器契约、驱动语言服务器的 JSON-RPC 客户端——完全按照这种方式编写。

# 宏

编译时、卫生、Elixir风格。宏在编译器内一个确定性的沙箱中运行，并且**不留任何运行时痕迹**——生成的 Python 仅包含其展开结果。

```elixir
defmacro unless_nil(value, fallback) do
  quote do
    case unquote(value) do
      nil -> unquote(fallback)
      found -> found
    end
  end
end
```

## 工具包

- `quote do ... end` 返回其块的 AST；`unquote(x)` 将值插回；`unquote_splicing(list)` 拼接序列。
- 模板变量根据展开进行重命名（**卫生**）；`var!(name)` 有意地访问调用者的作用域。
- `def unquote(head)(...)` 构建定义——这是 `use` 风格代码注入背后的模式：

```elixir
defmacro __using__(_opts) do
  quote do
    def unquote(:injected)(), do: :from_using
  end
end
```

- 引用的代码解构为普通数据——`{name, meta, args}` 三元组和 do 块对——因此宏可以重写它们接收的内容。构建器如 `slug/1`、`downcase/1`、`replace/3` 和 `to_atom/1` 在展开时可用。
- `compile_warn/1` 允许库宏在其调用点发出**带范围的编译器警告**——扩展与内核以相同的声音发言。

## 引入宏

`require Mod` 使得 `Mod` 的宏可用；`import Mod` 还会直接引入宏名称；`use Mod` 会调用 `Mod.__using__/1` 来注入代码。标准库中基于 ExUnit 风格的测试接口（[Testing](testing.md)）完全由这些构件组成。

## 检查宏展开

```console
gan expand src/x.gan        # 宏展开后的模块
gan lsc expand src/x.gan    # 等同于引用的 AST（JSON）
```

或者编辑器中的 **Expand Macros** 命令。发布在 wheel 中的宏对消费者仍然有效——包携带其 `.gan` 源文件，正是为了下游展开能够运行。

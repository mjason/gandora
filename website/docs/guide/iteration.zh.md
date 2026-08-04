## 迭代与递归

不存在 `loop` 和 `while`。迭代是 `for`、`Enum` 系列或递归——并且编译器确保递归的安全性。

## `for` 推导式

编译为原生 Python 推导式 (GEP-0020):

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # multiple generators
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # dict comprehension
for {k, v} <- [{"a", 1}, :bad], do: k          # non-matching elements are SKIPPED
```

主体是一个表达式；`into: %{}` 需要一个 `{key, value}` 元组作为主体。推导式 **构建一个集合** — 用其进行副作用操作会引发编译器警告；应使用 `Enum.each` 替代。

## 尾递归编译为循环

对尾位置处封闭函数的调用将变为 `while True:` 内的参数重新绑定——无论深度多大，栈空间恒定不变（GEP-0019）：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万个栈帧也没问题
```

`recur(args)` 是同一跳转的**检查过的**写法：它必须位于尾位置且与子句元数匹配，否则构建失败——当你需要恒定栈而不是期望时，请使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

## 当递归真正递归时

非尾递归（`n * fact(n - 1)`）保持真实调用，并使用 Python 栈（约 1000 帧）；编译器会在定义处**发出警告**并附带累加器配方。结构递归——深度由数据边界决定，如树的遍历——是合法的：承认它，警告就会消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态都是可见的：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 会打印它，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

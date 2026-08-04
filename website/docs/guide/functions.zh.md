# 函数与管道

## 匿名函数与捕获

```elixir
double = fn x -> x * 2 end           # -> lambda
classify = fn                        # multi-clause + guards -> hoisted def
  0 -> :zero
  n when n > 0 -> :pos
  _ -> :neg
end
add = &(&1 + &2)                     # capture with placeholders
sqrt = &($math.sqrt/1)               # capture a Python function
mine = &fact/1                       # capture a module function (defp too)
double.(21)                          # calling a function value uses .()
```

`&f/1` 适用于公共 def、私有 defp 以及内核形式（`&to_string/1` 编译为 `str`）。当 `fn` 仅包装一个调用时——`fn x -> f(x) end`——构建工具会建议使用捕获语法。

## 闭包按值捕获

与 Elixir 完全相同，闭包在其**创建时**（GEP-0021）快照其自由变量——后续的重新绑定、尾递归迭代以及推导步骤永远不会渗入到更早创建的闭包中：

```elixir
n = 1
add_n = fn x -> x + n end
n = 100
add_n.(1)    # 2, not 101
```

编译器通过 Python 自身的惯用法（`lambda x, *, n=n: x + n`）实现这一点；调用参数数量保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # 首参数管道
df |> .groupby("k") |> .agg(spec)      # 方法管道：在管道值上调用
" gan " |> .strip() |> .upper()        # 也适用于字面量
value.method(x).attr                   # 在任何对象上的后缀链
```

`|>` 将值传递给 Gandora 调用的**首参数**；`|> .m(...)` 将值传递给管道值的 Python *方法* —— 这一种形式覆盖了 pandas/numpy 的全部流畅风格。当下一行以 `|>` 开头时，管道可以延续到下一行。

没有 `then/2` —— 通过匿名函数进行管道传递（`x |> (fn v -> f(v, 1) end).()`），或者直接绑定一个变量即可。

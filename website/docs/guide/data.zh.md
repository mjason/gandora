# 数据与模式匹配

## Values

原子是驻留字符串和**纯数据**——它们从不命名模块（那是`$module`的职责）。只有`false`和`nil`为假值；`0`、`""`和`[]`为真值（采用 Elixir 语义，而非 Python）。

```elixir
:ok  :"quoted atom"                  # atoms -> Python strings
"interp #{1 + 1}"                    # f-string in the output
"""
heredocs too (dedented)
"""
[1, 2, 3]  {:pair, 2}  %{"k" => 1, a: 2}   # list, tuple, map
[timeout: 500, retries: 3]           # keyword list -> [("timeout", 500), ...]
1..10                                # inclusive range
10 / 4                               # 2.5 — / is true division
10 // 4                              # 2 — truncated division
rem(-7, 2)                           # -1 — truncated remainder (not Python %)
"a" <> "b"                           # string concatenation
```

映射是**写入数据的唯一方式**——来自 API 文档的 JSON 文档通过将 `:` 替换为 `=>` 变成映射（构建工具能即时识别）；运行时 JSON 文本通过 `$json.loads(s)` 处理：

```elixir
%{"type" => "function",
  "function" => %{"name" => "ping",
                  "parameters" => %{"type" => "object", "properties" => %{}}}}
```

## 模式匹配

`=`、`case`、函数头、`with` 以及 `for` 生成器都会匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、引脚（`^x` —— 匹配 `x` 的*现有*值）以及结构体。

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "被固定的值"
  {:error, %{reason: r}} -> "失败: #{r}"
  _ -> "其他任何情况"
end
```

失败的 `=` 或 `case` 匹配会引发 `GanMatchError`。`cond` 选择第一个为真的*条件*（而非模式）：

```elixir
cond do
  x > 90 -> :a
  x > 80 -> :b
  true -> :c
end
```

## `with` — 链式处理可失败步骤

```elixir
with {:ok, a} <- parse(s),
     {:ok, b} <- check(a) do
  {:ok, b}
else
  {:error, why} -> {:error, why}
end
```

第一个匹配失败的模式会落入 `else` 分支。这是处理 ok/error 管道的惯用方式——无需使用异常。

## Rebinding

`x = 1; x = 2` 创建一个*新绑定*；期间创建的闭包保留旧值（参见
[函数与管道](functions.md#closures-capture-by-value)）。

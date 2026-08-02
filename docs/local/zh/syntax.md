# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

v0 表面的实用指南。规范定义位于 GEPs ([`geps/`](../../../geps/)) 中；本手册展示了这些部分如何使用。这里的每个构造都由 [`examples/tour`](../../../examples/tour) 进行练习，其已检入的 [`generated/`](../../../examples/tour/generated/) 目录展示了每个章节编译成的确切 Python 代码。

## 模块与函数

每个文件一个 `defmodule`；名称必须与路径匹配（`src/app/hello_web.gan` ↔ `App.HelloWeb` ↔ `app/hello_web.py`）。

```elixir
defmodule App.Math do
  @moduledoc "Docstrings come from @moduledoc and @doc."

  @doc "Multi-clause dispatch, top to bottom, with guards."
  def fact(0), do: 1
  def fact(n) when n > 0, do: n * fact(n - 1)

  defp helper(x), do: x + 1     # private: leading underscore in Python
end
```

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。

## 数据

原子是内部化的字符串；只有 `false` 和 `nil` 是假值。

```elixir
:ok  :"os.path"                      # atoms
"interp #{1 + 1}"                    # f-string in the output
"""
heredocs too
"""
[1, 2, 3]  {:pair, 2}  %{"k" => 1, a: 2}   # list, tuple, map
[timeout: 500, retries: 3]           # keyword list -> [("timeout", 500), ...]
1..10                                # inclusive range
```

## 模式匹配

`=`、`case`、`cond`、`with` 以及函数头均匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、固定（`^x`）和结构体。

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "the pinned value"
  {:error, %{reason: r}} -> "failed: #{r}"
  _ -> "anything else"
end

with {:ok, a} <- step1(),
     {:ok, b} <- step2(a) do
  {:ok, b}
else
  _ -> :error
end
```

匹配失败会抛出 `GanMatchError`。

## 函数作为值与管道

```elixir
double = fn x -> x * 2 end
add = &(&1 + &2)
sqrt = &$math.sqrt/1
double.(21)

xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
```

当管道以 `|>` 开始时，可以延续到下一行。

## Python 互操作

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$"os.path".join(a, b)               # dotted modules via quoted atoms
$sys.argv                           # attribute read
pyimport numpy, as: np              # aliased import
np.array([1, 2]) * 10               # operators broadcast — it's just Python
value.method(x).attr                # postfix chains on anything
$json.dumps(data, indent: 2)        # trailing keywords become kwargs
```

装饰器通过 `@decorate` 附加，模块属性持有导入时状态：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}
```

生成的模块是一个普通的 ASGI 目标：
`uvicorn app.api:app --app-dir dist`。

## 结构体

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}           # frozen dataclass instance
%App.User{name: n} = u              # pattern (isinstance + fields)
older = %App.User{u | age: u.age + 1}   # dataclasses.replace
m2 = %{m | count: 2}                # plain-map update: {**m, ...}
```

## 宏

编译时、卫生、Elixir 风格。宏在编译器内部的确定性沙箱中运行，不留运行时痕迹。

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

模板变量每次展开都会重命名（卫生性）；`var!(name)` 有意地触及调用者的作用域；`unquote_splicing(list)` 拼接序列。通过 `require Mod`（或 `import`）引入宏，然后调用 `Mod.some_macro(...)`。使用 `gan expand file` 检查结果。

## Sigils

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\d+/                      # re.compile(r"\d+"), backslashes verbatim
~python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

`~python` 是用于 Python 特有拼写（推导式、布尔索引）的逃逸出口；它将表达式原样拼接至输出中。

## 文档

三个通道，互不重叠：`@doc`/`@moduledoc` 是 Elixir 风格（Markdown
字符串=文本、keyword 列表=元数据、`false`=隐藏，多行累积；文本永不
被解析）。`@example` 承载可运行示例。`@doc_trans`/`@moduledoc_trans`
添加纯散文翻译。

```elixir
@doc since: "1.3.0"
@doc "Factorial."
@doc_trans zh_CN: "阶乘。"
@example """
    gan> fact(10)
    3628800
"""
def fact(n), do: ...
```

`gan test` 把 `@example` 块编译成原生 Python doctest 并运行（期望输出
是 Python 的 `repr`，即 `inspect/1` 打印的内容）；所有语言渲染同一份
示例。翻译里出现 `gan>` 是错误；`@doc` 文本里出现则是警告（不会被
测试）。`gan doc App.Mathy.fact --locale zh` 按 RFC 4647 回退打印对应
语言视图。

## 项目、包与 CLI

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`）；`pyproject.toml` + `uv` 管理依赖和 `.venv`。

```console
gan init my-app          # 新建项目          gan check    # 仅分析
gan run src/main.gan     # 编译 + 执行       gan build    # 编译到 outDir
gan expand src/x.gan     # 展示宏展开结果    gan init --package name
```

包以标准 wheel 格式发布（`gan build && uv build && uv publish`），携带编译后的 Python 代码、一个 `gandora.toml` 标记文件以及宏展开所依赖的 `.gan` 源文件——使用者通过 `uv add` 安装，并像本地包一样使用 `require`/`alias`，无需引入 Gandora 运行时（GEP-0006）。

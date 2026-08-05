# Gandora语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

编写Gandora的实用指南——面向人类，也特意为AI智能体编写。规范性定义存在于GEPs（`geps/`）中；本手册展示各个部分如何使用，以及当存在多种写法时优先选择哪一种。这里的每个结构都通过 `examples/tour`（其已检查的 `generated/` 显示每个章节编译成的确切Python代码）进行练习，并由游乐场的自检套件进行实战测试。

塑造以下所有内容的基本规则：

- **零运行时。** 生成的Python是自包含且可读的——评审者会手写的代码。辅助函数在每个模块内联；部署从不依赖Gandora。
- **Elixir表面，Python语义底层。** 当Elixir有某个结构时，Gandora以Elixir方式拼写；值是普通的Python对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非意见；悬停显示递归如何编译；`gan lsc`以JSON形式提供每个事实。

## 模块与函数

每个文件一个 `defmodule`；名称必须与路径匹配（`src/app/hello_web.gan` ↔ `App.HelloWeb` ↔ `app/hello_web.py`）。

```elixir
defmodule App.Math do
  @moduledoc "Docstrings come from @moduledoc and @doc."

  @doc "Multi-clause dispatch, top to bottom, with guards."
  @spec fact(integer()) :: integer()
  @allow :stack_recursion
  def fact(0), do: 1
  def fact(n) when n > 0, do: n * fact(n - 1)

  def greet(name, greeting \\ "hi"), do: "#{greeting} #{name}"  # defaults: every arity generated

  def empty?(xs), do: length(xs) == 0   # ? and ! are part of names
  defp helper(x), do: x + 1             # private: leading underscore in Python
end
```

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔型内置函数：`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`、比较运算、`and or not`、算术运算。

## 数据

原子是驻留字符串和**纯数据**——它们从不命名模块（那是`$module`的职责）。只有`false`和`nil`是假值；`0`、`""`和`[]`是真值（Elixir语义，而非Python的）。

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

## 模式匹配

`=`、`case`、函数头、`with` 和 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、固定（`^x` — 匹配 x 的*现有*值）以及结构体。

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "the pinned value"
  {:error, %{reason: r}} -> "failed: #{r}"
  _ -> "anything else"
end

cond do                       # first truthy condition (not patterns)
  x > 90 -> :a
  true -> :c
end

with {:ok, a} <- step1(),
     {:ok, b} <- step2(a) do
  {:ok, b}
else
  _ -> :error
end
```

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新的绑定——在此期间创建的闭包保留旧值（见下文）。

## 函数作为值

```elixir
double = fn x -> x * 2 end           # -> lambda
classify = fn                        # multi-clause + guards -> hoisted def
  0 -> :zero
  n when n > 0 -> :pos
  _ -> :neg
end
add = &(&1 + &2)                     # capture with placeholders
sqrt = &($math.sqrt/1)               # capture a Python function
mine = &fact/1                       # capture a module function
double.(21)                          # calling a function value uses .()
```

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 完全相同——后续的重新绑定、尾递归循环迭代或推导式步骤绝不会泄露到之前创建的闭包中。编译器通过 Python 自身的惯用法（`lambda x, *, n=n: x + n`）来实现这一点；调用时的参数数量保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

当管道以 `|>` 开始时，可以延续到下一行。

## 迭代：推导式与递归

没有 `loop`，也没有 `while`。迭代使用 `for`、`Enum` 家族或递归——编译器会确保递归的安全。

### `for` 推导式（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被跳过
```

主体是一个表达式；`into: %{}` 需要主体返回 `{key, value}` 元组。推导式 **构建一个集合**——若将其用于副作用，编译器会发出警告；请改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

对尾位置的封闭函数的调用转化为 `while True:` 内部的参数重新绑定——无论深度如何，栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万个栈帧也没问题
```

`recur(args)` 是同一跳转的 **受检查的** 写法：它必须位于尾位置且与某个子句的参数数量匹配，否则构建失败——当恒定栈是要求而非期望时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用并使用 Python 栈（约 1000 个栈帧）；编译器会在定义处 **发出警告**。结构递归——深度由数据（如树遍历）决定——是合法的：确认后警告消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态均可见：悬停时显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 打印该信息，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

一条拼写规则：**类型即调用** —— `integer()`、`list(t)`、`Mod.t()`、`$mod.Type()` 都必须带括号；唯一允许裸写的是类型变量（1-2 个小写字母）和字面量 `nil`。任何其他写法都会导致编译错误，并附带修复提示。

`@spec` 声明函数的类型；编译器会对照子句对其进行验证，并在生成的 Python 中输出 **PEP 484 类型注解**，使得 `pyright`/`mypy` 能够检查调用方，悬停时显示真实类型。每个定义组只能有一个 `@spec`，且与其他注解一同放在第一个子句之前。

```elixir
@spec mean(xs :: sequence(number()), precision :: integer()) :: float()
def mean(xs, precision \\ 2) do ... end
# -> def mean(xs: collections.abc.Sequence[int | float], precision: int = 2) -> float:
```

### 标量类型

| Gandora | Python 注解 |
| --- | --- |
| `integer()` | `int` |
| `float()` | `float` |
| `number()` | `int \| float` |
| `string()` | `str` |
| `boolean()` | `bool` |
| `atom()` | `str`（原子是驻留字符串） |
| `nil` | `None` |
| `term()` / `any()` | `object` |
| `fun()` | `Callable`（无参数化） |

### 容器 —— 构建时具体，接收时抽象

具体容器表示“就是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何具有该行为的东西”——在**参数**中优先使用它们，这样调用方可以传入元组、区间或生成器；同时因为 `list` 在 Python 类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  —— 只会被遍历一次
sequence(t)                # collections.abc.Sequence[t]  —— 可索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] —— 只读的类字典
keyword()                  # 关键词列表：list[tuple[str, object]]
```

经验法则：**接收时抽象，返回时具体** —— 接受 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

**一个或两个字符**的裸写小写名称即为类型变量；每个变量在输出中都会成为模块级别的 `typing.TypeVar`。同一个字母在 spec 中表示“同一类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸写名称会报错，并附带提示（“你是不是想写 `name()`？”），因此拼写错误的 `intger` 不会静默地变成一个泛型变量。

### 命名参数

`name :: type` 在 spec 中为参数命名——既可自文档化，也能在签名提示中显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 引入；括号用于参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并会生成相应的 import 语句
```

### 结构体类型

`Mod.t()` 表示该模块中由 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数量之间的交互

一个 spec 覆盖整个组；应针对完整参数列表（包括默认值）编写。生成的委托函数和分发器会携带这些注解。如果 `@spec` 中指定的函数或参数量不存在，则会导致编译错误，因此 spec 不会无声地腐烂。

## Documentation and doctests

Annotation order before a `def`: any of `@doc`, `@doc_trans`,
`@param`, `@param_trans`, `@spec`, `@example`, `@decorate`, `@allow` —
all accumulate onto the next definition.

```elixir
@doc "Word frequencies of a sentence, as a map."
@doc_trans zh_CN: "统计句子的词频，返回映射。"
@param sentence, "Case-folded and split on whitespace."
@param_trans sentence, zh_CN: "会转小写并按空白切分的句子。"
@spec word_count(string()) :: map(string(), integer())
@example """
    gan> word_count("the quick the")
    {'the': 2, 'quick': 1}
"""
def word_count(sentence), do: ...
```

- `@param` 名称必须与子句头部的变量匹配——编译时验证。
- `@example` 是唯一的 doctest 渠道：`gan test` 将 `gan>` 行编译为原生 Python doctest 并执行它们。预期输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子打印为 `'ok'`，映射为 `{'k': 1}`，布尔值为 `True`。
- 标准：**每个公共 `def` 必须附带 `@doc` + `@spec`**（以及有参数时的 `@param`）；面向用户的表面需要添加 `_trans` 对。

文档语言是一种**开发者**偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，位于 `.gitignore` 中，与 `gandora.jsonc` 相邻）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 默认语言。`gan doc Mod.fun --locale zh-CN` 显式指定；`gan lsc doc` 始终以 JSON 形式返回所有语言环境。

## 测试 (GEP-0024)

一条命令，两层结构：`gan test` 运行所有 `@example` doctest，然后运行 `tests/*.gan` 中的所有 `test_*` 函数——编译时使用项目的完整模块解析，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

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

ExUnit 的测试表面：`test "name" do`（定义 `test_<slug>`），`describe`（为内部测试名称添加前缀），`assert`/`refute`（比较操作会报告两个操作数），此外，还提供 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk` 作为普通函数。`tests/` 目录从不随项目发布——它位于源代码根目录之外。

## Python 互操作

`$module` 是一个一等模块对象；`$` 显式标记了互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # 点链：导入 importlib.metadata
$(PIL.Image).open(f)                # $(...) 显式锁定模块边界
$(sys).stderr                       # ...单段同样：import sys, 属性 stderr
pyimport numpy, as: np              # 别名导入
pyimport sys                        # 裸导入将 `sys` 绑定为普通名称
np.array([1, 2]) * 10               # 运算符广播——它就是 Python
sys.stderr.write("...")             # 裸名称链没有导入歧义
$json.dumps(data, indent: 2)        # 尾部关键字变为 kwargs
```

何时使用何种拼写：

| 场景 | 拼写 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性且链式启发式猜测错误时 | `$(os.path).sep`, `$(sys).stderr` |
| 在文件中反复使用的模块 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 经常使用的深层属性 | `@environ $(os).environ` 模块属性 |

在同一个文件中重复使用 `$(...)` 拼写是一种不良实践——应声明 `pyimport`。切勿围绕 Python API 编写包装模块；不编写包装器正是设计所在。

装饰器通过 `@decorate` 附加；模块属性保存导入时的状态：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是一个普通的 ASGI 目标：`uvicorn app.api:app --app-dir dist`。

## 错误处理

`try/rescue/after` 映射到 Python 异常；rescue 子句按异常类匹配：

```elixir
try do
  risky()
rescue
  e in $builtins.ValueError -> {:bad_value, to_string(e)}
  e in $requests.HTTPError -> {:http, e.response.status_code}
  e -> {:error, to_string(e)}      # 裸变量：任何 Exception
after
  cleanup()                        # 总是运行，不贡献值
end

raise "message"                    # -> raise RuntimeError("message")
```

`try` 是一个表达式。`try` 内部的尾调用 *不* 会优化（帧必须保留给处理程序）——在其中使用 `recur` 会导致编译错误，而非静默消耗栈空间。

## 结构体

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}               # frozen dataclass instance
%App.User{name: n} = u                  # pattern (isinstance + fields)
older = %App.User{u | age: u.age + 1}   # dataclasses.replace
m2 = %{m | count: 2}                    # plain-map update: {**m, ...}
```

结构体类型在规范中以 `App.User.t()` 的形式出现。

## 宏

编译时、卫生的、Elixir 风格的。宏在编译器的确定性沙箱中运行，不留下任何运行时痕迹。

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

模板变量在每次展开时被重命名（卫生性）；`var!(name)` 有意地进入调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *Expand Macros* 命令检查结果。

## 魔符与嵌入语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\\d+/                     # re.compile("\\d+") — string escapes apply
$python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

无大写字母的嵌入语言魔符可以携带整个文档，并通过 `<%= expr %>` 插值回 Gandora（编辑器会高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`~p` 是用于 AI 散文的推荐名称** (GEP-0009-R006)：主体是原始的，因此引号、花括号、反斜杠和内联 JSON 无需转义——再也不用与 `\\\"` 斗争了：

```elixir
@task ~p(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~p"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

**数据就是映射——包括粘贴的 JSON**：Gandora 的 `%{}` 字面量是唯一的数据拼写；来自 API 文档的 JSON 文档通过将 `:` 替换为 `=>` 变成映射（顾问会即时教你），而运行时 JSON 文本使用 `$json.loads(s)`：

```elixir
@tool %{"type" => "function",
        "function" => %{"name" => "ping", "description" => "Health check.",
                        "parameters" => %{"type" => "object", "properties" => %{}}}}
```

`$python` 是用于纯 Python 拼写的转义出口；插值会被编译为 Gandora 表达式，其他所有内容逐字传递。

## 编译器 lint——警告是可证明的事实

每个警告只在静态确定的情况下触发，在 `gan build`/编辑器波浪线中定位到定义行，并具有机械性修复（通常是一键快速修复）：

| 警告 | 含义 | 修复方式 |
| --- | --- | --- |
| 未定义变量 | 读取一个未绑定的变量 —— 保证会引发 `NameError` | 修正名称 |
| 未使用的绑定 | 已绑定，但从未读取 | 加 `_` 前缀（`_meta`） |
| 不可达子句 | 无守卫的全变量头部遮蔽了后续同参数量级的子句（也包括 `case` 通配符） | 重排序或删除 |
| 被丢弃的推导式 | `for` 位于语句位置 | 改用 `Enum.each` |
| 未使用的 `defp` | 死私有函数 | 删除，或使用 `@allow :unused_function` |
| 栈递归 | 自递归，但从未处于尾位置 | 改用累加器形式、`recur`，或使用 `@allow :stack_recursion` |

`@allow` 目标会被检查——拼写错误会导致编译错误。将警告视为缺陷：代码库标准为零警告。

## 项目与命令行界面

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理自身依赖和 `.venv`；`gandora.local.jsonc` 存放每个开发者的偏好设置，且不纳入 git 版本控制。

```console
gan init my-app          # 新项目
gan run src/main.gan     # 编译 + 执行
gan build                # 裁决 + 编译到 outDir
gan test                 # 运行 @example 文档测试
gan fmt src              # 原地格式化
gan fmt --check src      # CI 关卡
gan fmt --diff src       # 显示差异
echo ... | gan fmt -     # 标准输入 -> 标准输出
gan doc Enum.take        # 文档（+ --locale）
gan repl                 # 交互式
gan expand src/x.gan     # 宏展开输出
gan init --package name
```

包以普通的 wheel 包形式发布（`gan build && uv build && uv publish`），携带编译后的 Python 代码、`gandora.toml` 标记文件、以及宏展开所需的 `.gan` 源文件——消费者通过 `uv add` 添加，无需引入 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一个 JSON 值——专为智能体构建：

```console
gan lsc check --root .                  # the verdict: {diagnostics, suggestions}
gan lsc diagnostics src/x.gan --root .  # one file
gan lsc doc Enum.take --root .          # specs, prose (all locales), params, tco shape
gan lsc doc for --root .                # language constructs answer too (for/recur/with...)
gan lsc references Stats.mean --root .  # every call site (+ definitions)
gan lsc wsymbols mean --root .          # project-wide symbol search
gan lsc symbols Stats --root .          # one module's outline
gan lsc definition Stats.mean --root .  # where it's defined
gan lsc compile src/x.gan --root .      # the generated Python, as text
gan lsc expand src/x.gan --root .       # post-macro quoted AST
gan lsc ast src/x.gan --root .          # parse tree (Elixir encoding)
gan lsc pydoc numpy.array --root .      # Python-side docs via jedi
```

**`gan build` 是裁决**（GEP-0025 rev 3）：编译器诊断、Advisor 建议（实践差距、跨语言迁移提示、针对拼写错误名称的“你是不是要指”），以及**工件验证**——生成的 Python 会使用解析规则（ty）进行类型检查，因此未定义的函数、无用的导入或缺失的模块成员是构建**错误**，而不是运行时意外。错误会阻止工件的生成，这是重型编译器的方式；警告和建议则打印出来并允许继续。`gan lsc check` 返回相同的裁决，作为一个 JSON 对象。

```console
gan lsc check --root .
# {"ok": true, "clean": false, "diagnostics": [...],
#  "suggestions": [{"kind": "did_you_mean", "line": 3,
#   "message": "`Enum.mpa` is not a function of Enum — did you mean `Enum.map`?", ...}]}
```

裁决以交通信号灯的方式呈现：`ok`（编译通过——无错误）和 `clean`（ok **并且**零警告 **并且**零建议）。红色 → 修复错误；黄色 → 阅读建议；绿色 → 提交。建议带有其第一个证据的行号，跨多个文件的相同发现会合并为一个带注释的条目，覆盖范围包括 `src/` 和顶层 `tests/*.gan`（测试模块会获得迁移和惯用法提示，但免于库注解覆盖）。

智能体的高效循环：**编写 → `gan build`（修复所有发现） → `gan test` → 发布**。当不确定某物生成什么时，运行 `gan lsc compile file`；当不确定语法时，运行 `gan lsc doc <construct>`（`for`、`spec`、`test`、……）。

## 代理风格检查清单

1. 每个公共 `def`：`@doc` + `@spec`（+ 每个参数的 `@param`）；对任何具有有趣行为的代码添加 `@example` —— `gan test` 确保它们真实可靠。
2. 规范：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正泛型的流程；结构体使用 `Mod.t()`；在 Python 边界使用 `$mod.Type()`。
3. 迭代：对于映射形状的工作使用 `for`/`Enum`；对于无界循环使用累加器尾递归（当需要恒定栈时使用 `recur`）；仅对结构有界的递归使用 `@allow :stack_recursion`，且原因需明显。
4. 互操作：一次性使用 `$`，重复使用 `pyimport`，无需包装器。
5. 零警告，`gan fmt` 整洁，文档测试通过 —— 工具链、标准库、教程和游乐场都坚持这一标准；请保持一致。

# The Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

实用的 Gandora 编写指南——面向人类，也特意面向 AI 智能体。规范性定义存在于 GEP 中（[`geps/`](../geps/)）；本手册展示各个部分如何被使用，以及当存在多种写法时优先选择哪种拼写。这里的每个构造都通过 [`examples/tour`](../examples/tour) 进行了练习（其检入的 [`generated/`](../examples/tour/generated/) 展示了每章编译出的精确 Python 代码），并经过游乐场自检套件的实战测试。

塑造以下所有内容的基本原则：

- **零运行时。** 生成的 Python 是自包含且可读的——审阅者会手写出的样子。辅助函数按模块内联；部署从不依赖 Gandora。
- **Elixir 表象，Python 语义底层。** 对于 Elixir 存在的构造，Gandora 以 Elixir 方式拼写；值则是普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非意见；悬停显示递归如何编译；`gan lsc` 以 JSON 格式提供每个事实。

## 模块与函数

每个文件一个 `defmodule`；名称必须与路径匹配
（`src/app/hello_web.gan` ↔ `App.HelloWeb` ↔ `app/hello_web.py`）。

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

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔型的内置函数：`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`、比较操作、`and or not`、算术运算。

## 数据

原子是内联字符串，属于**纯数据** —— 它们从不命名模块（那是 `$module` 的职责）。只有 `false` 和 `nil` 是假值；`0`、`""` 和 `[]` 是真值（采用 Elixir 语义，而非 Python 的）。

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

`=`、`case`、函数头、`with` 以及 `for` 生成器都进行模式匹配：字面量、变量、`_`、元组、`[head | tail]`、映射、固定（`^x`——匹配 x 的*现有*值）以及结构体。

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "the pinned value"
  {:error, %{reason: r}} -> "failed: #{r}"
  _ -> "anything else"
end

cond do                       # 第一个为真的条件（而非模式）
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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新绑定——在这之间创建的闭包会保留旧值（见下文）。

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

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 完全一致——后续的重新绑定、尾递归循环迭代或推导步骤绝不会泄露到之前创建的闭包中。编译器使用 Python 自身的惯用法（`lambda x, *, n=n: x + n`）来实现这一点；调用参数个数保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

管道可以延续到下一行，当它以 `|>` 开头时。

## 迭代：推导式与递归

Gandora 中没有 `loop` 和 `while`。迭代通过 `for`、`Enum` 族或递归实现——编译器会确保递归的安全性。

### `for` 推导式（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被跳过
```

主体是一个表达式；`into: %{}` 需要主体返回 `{key, value}` 元组。推导式 **构建一个集合**——若将其用于副作用，编译器会发出警告；应改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

尾位置对当前函数的调用会被编译为 `while True:` 内部的参数重绑定——无论深度多大，栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万次调用也没问题
```

`recur(args)` 是同一跳转的 **带检查** 写法：它必须位于尾位置且匹配某个子句的元数，否则构建失败——当你需要确保栈空间恒定而非仅凭希望时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（如 `n * fact(n - 1)`）仍为真实调用，使用 Python 栈（约 1000 帧）；编译器会在函数定义处 **警告**。结构递归——深度受数据限制，如树遍历——是合理的：通过注解确认即可消除警告：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态都可见：悬停时显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 会打印，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

一条拼写规则：**类型即调用**——`integer()`、`list(t)`、`Mod()`、`$mod.Type()` 均需使用括号；唯一允许裸写的是类型变量（1–2个小写字母）和字面量 `nil`。其他任何写法都会导致编译错误，并附带修正建议。

`@spec` 声明函数的类型；编译器会将其与函数子句进行验证，并在生成的 Python 代码中输出 **PEP 484 注解**，从而使 `pyright`/`mypy` 能够检查调用方，并让悬停提示显示真实类型。每个定义组只能有一个 `@spec`，且与其他注解一起放在第一个子句之前。

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
| `fun()` | `Callable`（未参数化） |

### 容器——构建时具体，接受时抽象

具体容器表示“精确的 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为类似的对象”——优先在**参数**中使用它们，以便调用方可以传递元组、范围或生成器，并且因为 `list` 在 Python 类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — will be walked once
sequence(t)                # collections.abc.Sequence[t]  — indexed / re-walked
mapping(k, v)              # collections.abc.Mapping[k, v] — read-only dict-like
keyword()                  # a keyword list: list[tuple[str, object]]
```

经验法则：**抽象输入，具体输出**——接受 `sequence(t)`，返回 `list(t)`。

## 命名类型（`@type`）

`@type` 为类型命名——同时也是**泛型的声明点**：参数在头部声明，引用处进行元数检查，所有内容在编译时展开（零运行时开销）：

```elixir
@type age() :: integer()
@type result(t) :: tuple(atom(), t)
@type scores() :: map(string(), age())

@spec parse(string()) :: result(integer())    # -> tuple[str, int]
@spec load(string()) :: Mod.result(string())  # 跨模块引用
```

`@type` 体中未声明的变量、引用处错误的元数、重复定义、遮蔽内置类型以及递归，均会引发编译错误并附带修复提示；`@doc` 置于 `@type` 上方可为其添加文档。在 `@spec` 内部，简短的类型变量保持隐式作用域——但若某规格的整个返回值是一个在其他地方未使用的变量，则会给出实践提示（因为它未约束任何内容）。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

**一个或两个字符**的裸小写名称即类型变量；每个变量在输出中成为模块级别的 `typing.TypeVar`。同一字母在规格内表示“同一类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸名称会引发错误并附带提示（“你的意思是 `name()` 吗？”），因此拼写错误的 `intger` 不会默默地变成泛型。

### 命名参数

`name :: type` 为规格中的参数命名——具备自文档功能，并会在签名帮助中显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型均可通过 `$module` 出现；括号用于参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并自动生成导入语句
```

### 结构体类型

`Mod()` 表示该模块中由 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop()) :: App.Shop()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多元数函数的交互

一个规格覆盖整个函数组；请针对完整参数列表（包含默认值）编写规格。生成的委托函数和分发器将携带这些注解。`@spec` 命名一个不存在的函数或元数会引发编译错误，因此规格不会悄然失效。

## 文档与文档测试

在 `def` 之前的注解顺序：可以是 `@doc`、`@doc_trans`、
`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow` 中的任意几个——它们都会累积到下一个定义上。

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

- `@param` 的名称必须与子句头部的变量名一致——在编译时验证。
- 永远不要在 `@doc` 内部编写散文形式的 `Example:` 部分（这是 Elixir 的习惯）——`@example` 是唯一的渠道；在 `def` 上它会运行，在 `defmacro` 上它会被展示。
- `@example` 是唯一的 doctest 渠道：`gan test` 会将 `gan>` 行编译成原生的 Python doctest 并运行它们。期望的输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子打印为 `'ok'`，映射打印为 `{'k': 1}`，布尔值打印为 `True`。
- 标准：**每个公开的 `def` 都带有 `@doc` + `@spec`**（如果有参数，则加上 `@param`）；面向用户的接口需要添加 `_trans` 配对。

文档语言是**开发者**的偏好，绝不是项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，被 git 忽略，与 `gandora.jsonc` 相邻）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 仅默认语言。`gan doc Mod.fun --locale zh-CN` 明确指定语言；`gan lsc doc` 始终将所有语言环境以 JSON 格式返回。

## 测试 (GEP-0024)

一条命令，两层结构：`gan test` 运行每个 `@example` 文档测试，然后运行 `tests/*.gan` 中的每个 `test_*` 函数——使用项目的完整模块解析进行编译，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

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

ExUnit 表面：`test "name" do`（定义 `test_<slug>`），`describe`（为内部名称添加前缀），`assert`/`refute`（比较报告两个操作数），以及 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk` 作为普通函数。`tests/` 目录永远不会被打包——它位于源码根目录之外。

## Python 互操作

`$module` 是一个一等模块对象；`$` 明确标记了互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # 点链：导入 importlib.metadata
$(PIL.Image).open(f)                # $(...) 明确锁定模块边界
$(sys).stderr                       # ...单段也一样：import sys, 属性 stderr
pyimport numpy, as: np              # 别名导入
pyimport sys                        # 裸导入将 `sys` 绑定为普通名称
np.array([1, 2]) * 10               # 运算符广播——它只是 Python
sys.stderr.write("...")             # 裸名称链无导入歧义
$json.dumps(data, indent: 2)        # 尾部关键字变为 kwargs
```

何时使用何种拼写：

| 情况 | 拼写 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性引用，但链式启发式猜测错误 | `$(os.path).sep`, `$(sys).stderr` |
| 模块在文件中反复使用 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 深层属性频繁使用 | `@environ $(os).environ` 模块属性 |

同一文件中反复出现 `$(...)` 拼写是坏味道——应声明 `pyimport`。永远不要为 Python API 编写包装模块；没有包装器正是设计意图。

装饰器通过 `@decorate` 附着；模块属性保存导入时状态：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是一个普通的 ASGI 目标：`uvicorn app.api:app --app-dir dist`。

### 装饰器：两个层级

运行时装饰属于 `$` 世界；编译时装饰属于宏。具体来说：

- **`@decorate <expr>`** 将任意 Python 装饰器附着到下一个 def 上——可以是库的装饰器（`$functools.lru_cache(maxsize: 64)`、`@app.get("/")`），也可以是返回包装器的 Gandora 函数。多个装饰器可以堆叠；离 def 最近的装饰器最先包装，与 Python 相同。
- **Gandora 编写的包装器是精确元数的**（`fn x -> ... f.(x) end` 仅包装 1 参数函数）——Gandora 故意没有 `*args`。*通用*任意元数装饰器是 Python 的任务：将其放在源码旁边的 `.py` 文件中，并引用为 `$mymod.deco`。
- **编译时重写**——Elixir 风格的装饰器——是 `defattr :name` 加上 `@on_definition` 宏（GEP-0008）：它看到真实的函数头部，保持零运行时开销，并且可以自身为 Python 侧发出 `@decorate`。教程中的 `@cache` 章节是完整示例。
- 由 Gandora `fn` 构建的包装器本质上是 lambda——它会丢失 `__name__`/`__doc__`；如果自省很重要，请在 Python 中编写该装饰器。

## 错误处理

`try/rescue/after` 映射到 Python 异常；rescue 子句按异常类匹配：

```elixir
try do
  risky()
rescue
  e in $builtins.ValueError -> {:bad_value, to_string(e)}
  e in $requests.HTTPError -> {:http, e.response.status_code}
  e -> {:error, to_string(e)}      # bare variable: any Exception
after
  cleanup()                        # always runs, contributes no value
end

raise "message"                    # -> raise RuntimeError("message")
```

`try` 是一个表达式。`try` 内部的尾调用 *不* 会被优化（框架必须为处理程序存活）——其中的 `recur` 是编译错误，而不是静默的栈消耗。

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

结构体类型在规范中表示为 `App.User()`。

## 宏

编译时、卫生的、Elixir 风格的。宏在编译器内部的一个确定性沙箱中运行，不留下任何运行时痕迹。

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

模板变量在每次展开时被重命名（卫生）；`var!(name)` 有意地触及调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *展开宏* 命令检查结果。

## 标记与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\\d+/                     # re.compile("\\d+") — string escapes apply
$python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

无大写嵌入式语言标记承载整个文档，并带有 `<%= expr %>` 拼接回 Gandora（编辑器高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`~p` 是AI散文的受赐名** (GEP-0009-R006): 主体是原始的，因此引号、花括号、反斜杠和内联JSON无需转义——不再与 `\\\"` 斗争：

```elixir
@task ~p(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~p"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

**数据是映射——包括粘贴的JSON**：Gandora的 `%{}` 字面量是唯一的数据拼写；来自API文档的JSON文档通过将 `:` 交换为 `=>` 变成映射（Advisor会即时教授），运行时JSON文本是 `$json.loads(s)`：

```elixir
@tool %{"type" => "function",
        "function" => %{"name" => "ping", "description" => "Health check.",
                        "parameters" => %{"type" => "object", "properties" => %{}}}}
```

`$python` 是仅Python拼写的逃生口；拼接是编译后的Gandora表达式，其他所有内容原样传递。

## 编译器 lint — 警告是可证明的事实

每条 lint 仅针对静态确定的项触发，定位在 `gan build`/编辑器波浪线中的定义行上，并具有机械化的修复方法（通常是一键快速修复）：

| 警告 | 含义 | 修复方法 |
| --- | --- | --- |
| 未定义变量 | 读取未绑定的变量 — 保证会引发 `NameError` | 修正名称 |
| 未使用的绑定 | 已绑定但从未读取 | `_` 前缀（`_meta`） |
| 不可达分支 | 无守卫的全变量头部遮蔽了后面相同元数的分支（也包括 `case` 通配符） | 重新排序或删除 |
| 丢弃的推导式 | 语句位置的 `for` | `Enum.each` |
| 未使用的 `defp` | 未使用的私有函数 | 删除，或使用 `@allow :unused_function` |
| 栈递归 | 自递归，且不在尾部位置 | 累加器形式、`recur` 或 `@allow :stack_recursion` |

`@allow` 目标会被检查 — 拼写错误会导致编译错误。将警告视为缺陷：代码库标准为零。

## 项目与命令行界面

`gandora.jsonc` 配置编译器（`source`, `outDir`, `targetPython`, `exclude`, `package`, `pyPackage`）；`pyproject.toml` + `uv` 管理依赖和 `.venv`；`gandora.local.jsonc` 保存开发者个人偏好，不纳入版本控制。

```console
gan init my-app          # new project
gan run src/main.gan     # compile + execute  gan build      # verdict + compile to outDir
gan test                 # run @example doctests
gan fmt src              # format in place    gan fmt --check src   # CI gate
gan fmt --diff src       # show the diff      echo ... | gan fmt -  # stdin -> stdout
gan doc Enum.take        # docs (+ --locale)  gan repl       # interactive
gan expand src/x.gan     # macro output
gan init --package name
```

包以普通 wheel 格式发布（`gan build && uv build && uv publish`），包含编译后的 Python、一个 `gandora.toml` 标记以及宏展开所需的 `.gan` 源文件——使用者通过 `uv add` 安装它们，无需引入 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是一个 JSON 值输出到 stdout——专为代理构建：

```console
gan lsc check --root .                  # 裁决：{diagnostics, suggestions}
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格说明、文档（所有语言环境）、参数、tco 形状
gan lsc doc for --root .                # 语言构造也会回答（for/recur/with……）
gan lsc references Stats.mean --root .  # 所有调用点（+ 定义）
gan lsc wsymbols mean --root .          # 项目级符号搜索
gan lsc symbols Stats --root .          # 单个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python（文本形式）
gan lsc expand src/x.gan --root .       # 宏展开后的带引号 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取的 Python 侧文档
```

**`gan build` 就是裁决**（GEP-0025 rev 3）：编译器诊断、Advisor 建议（实践差距、跨语言迁移提示、拼写错误名称的“您是不是要找”），以及**验证生成制品**——生成的 Python 会使用解析规则（ty）进行类型检查，因此未定义的函数、死导入或缺失的模块成员都会成为构建**错误**，而不是运行时意外。错误会阻止制品生成，这是重量级编译器的方式；警告和建议会打印并允许继续。`gan lsc check` 返回相同的裁决，作为一个 JSON 对象。

```console
gan lsc check --root .
# {"ok": true, "clean": false, "diagnostics": [...],
#  "suggestions": [{"kind": "did_you_mean", "line": 3,
#   "message": "`Enum.mpa` 不是 Enum 的函数——您是不是要找 `Enum.map`？", ...}]}
```

裁决以交通灯开头：`ok`（编译通过——无错误）和 `clean`（ok **且**零警告 **且**零建议）。红色→修复错误；黄色→阅读建议；绿色→提交。建议携带其第一个证据的行号，跨多个文件的相同发现会合并为一个带注释的条目，覆盖范围包括 `src/` 以及顶层 `tests/*.gan`（测试模块会获得迁移和惯用法提示，但免于库注释覆盖）。

一个高效的代理循环：**编写 → `gan build`（修复所有发现）→ `gan test` → 发布**。当不确定某物生成什么时，使用 `gan lsc compile file`；当不确定语法时，使用 `gan lsc doc <construct>`（`for`、`spec`、`test`……）。

## 代理风格检查清单

1. 每个公共 `def`：`@doc` + `@spec`（每个参数对应 `@param`）；任何具有有趣行为的代码均需 `@example` —— `gan test` 会确保其正确性。
2. 规格说明：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正泛化的流程；结构体使用 `Mod()`；Python 边界处使用 `$mod.Type()`。
3. 迭代：映射形状类工作使用 `for`/`Enum`；无界循环使用累加器尾递归（当需要恒定栈时使用 `recur`）；仅对结构有界递归使用 `@allow :stack_recursion`，且原因必须显而易见。
4. 互操作：一次性使用 `$`，重复使用 `pyimport`，无需封装器。
5. 零警告，`gan fmt` 干净，doctest 通过 —— 工具链、标准库、教程及 playground 均遵循此标准；请与之保持一致。

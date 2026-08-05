# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

为人类（以及有意地，为AI代理）编写的Gandora实用指南。规范性定义位于GEPs（`geps/`）中；本手册展示各部分的用法，以及在存在多种拼写时推荐使用哪一种。此处每个构造都由`examples/tour`（其已检入的`generated/`目录展示了每章编译成的确切Python代码）进行验证，并通过playground的自检套件进行实战测试。

以下所有内容遵循的基本原则：

- **零运行时。** 生成的Python代码自包含且可读——类似于审阅者手动编写的内容。辅助函数按模块内联；部署从不依赖Gandora。
- **Elixir 表层语法，Python 底层语义。** 凡Elixir具有的构造，Gandora皆以Elixir方式书写；值则是普通的Python对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非观点；悬停显示递归如何编译；`gan lsc` 以 JSON 形式提供所有事实。

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

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔型内置函数：`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`、比较运算符、`and or not`、算术运算。

## Data

原子是驻留字符串和**纯数据**——它们从不命名模块（那是`$module`的工作）。只有`false`和`nil`是假值；`0`、`""`和`[]`是真值（遵循Elixir语义，而非Python的）。

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

`=`, `case`、函数头、`with` 和 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、固定（`^x` — 匹配 x 的*现有*值）以及结构体。

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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新的绑定——在中间创建的闭包会保留旧值（见下文）。

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

**闭包在创建时通过值捕获** (GEP-0021)，与 Elixir 完全相同——后续的重新绑定、尾递归循环迭代或推导步骤永远不会泄漏到之前创建的闭包中。编译器通过 Python 自身的惯用法（`lambda x, *, n=n: x + n`）实现这一点；调用参数数量保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

当管道以 `|>` 开始时，可以继续到下一行。

## 迭代：推导式与递归

没有 `loop`，也没有 `while`。迭代通过 `for`、`Enum` 家族或递归实现——而编译器使递归变得安全。

### `for` 推导式 (GEP-0020)

编译为原生的 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被跳过
```

主体是一个表达式；`into: %{}` 需要一个 `{key, value}` 元组作为主体。一个推导式会**构建一个集合**——若将其用于副作用，编译器会发出警告；请改用 `Enum.each`。

### 尾递归编译为循环 (GEP-0019)

在尾位置上对当前函数的调用会变成 `while True:` 内部的参数重新绑定——无论栈深度多少，栈帧数恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万个栈帧也没问题
```

`recur(args)` 是同一种跳转的**受检查**写法：它 MUST 处于尾位置并且与某个子句的参数数量匹配，否则构建会失败——当恒定的栈帧是一个要求、而非期望时，请使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用，并使用 Python 栈（约 1000 帧）；编译器会在定义处**发出警告**。结构递归——深度由数据自身限制，如树遍历——是合法的：确认后，警告便会消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态都是可见的：悬停时显示 `♻ 尾递归 → while 循环` 或 `⚠ 原生调用栈`，`gan doc` 会打印该信息，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

一个拼写规则：**类型就是调用** — `integer()`, `list(t)`, `Mod()`, `$mod.Type()` 都需要括号；唯一允许不写括号的是类型变量（1–2个小写字母）和字面量 `nil`。其他任何写法都会导致编译错误，并附带修复提示。

`@spec` 声明函数的类型；编译器会根据子句对其进行验证，并在生成的 Python 中输出 **PEP 484 注解**，因此 `pyright`/`mypy` 可以检查调用者，悬停时显示真实类型。每个定义组只能有一个 `@spec`，放在第一个子句之前，与其他注解一起。

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

### 容器 — 构建时具体，接受时抽象

具体容器表示“精确的 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为类似的东西”——在**参数**中优先使用它们，这样调用者可以传递元组、范围或生成器，而且因为 `list` 在 Python 类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 将被遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 可索引/可重新遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读的类字典对象
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**抽象输入，具体输出** — 接受 `sequence(t)`，返回 `list(t)`。

## 命名类型（`@type`）

`@type` 为类型命名，并且是**泛型的声明位置**：参数在头部声明，引用处进行元数检查，所有内容在编译期展开（零运行时开销）：

```elixir
@type age() :: integer()
@type result(t) :: tuple(atom(), t)
@type scores() :: map(string(), age())

@spec parse(string()) :: result(integer())    # -> tuple[str, int]
@spec load(string()) :: Mod.result(string())  # 跨模块引用
```

`@type` 主体中未声明的变量、引用处的错误元数、重复、遮蔽内置类型以及递归，都会导致编译错误并附带修复提示；`@doc` 置于 `@type` 之上可以为其添加文档。在 `@spec` 内部，短类型变量保持隐式作用域——但如果某个规范的整体返回类型是一个在别处未使用的变量，则会给出实践提示（因为它未约束任何东西）。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

**一或两个字符**的裸小写名称即为类型变量；每个变量在输出中会成为模块级别的 `typing.TypeVar`。同一个字母在同一规范中表示“同一类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸名称会报错并附带提示（“你的意思是 `name()` 吗？”），因此拼写错误的 `intger` 不会静默地变成一个泛型。

### 具名参数

`name :: type` 为规范中的参数命名——既实现自文档化，也用于签名帮助显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 出现；括号用于参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并生成相应的 import 语句
```

### 结构体类型

`Mod()` 表示该模块中由 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop()) :: App.Shop()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数的交互

一个规范覆盖整个函数组；请针对完整参数列表（包含默认值）编写规范。生成的委托函数和分发器会携带这些注解。若 `@spec` 指定了不存在的函数或元数，会产生编译错误，因此规范不会无声地腐烂。

## 文档与文档测试

在 `def` 之前的注解顺序：`@doc`, `@doc_trans`, `@param`, `@param_trans`, `@spec`, `@example`, `@decorate`, `@allow` 中的任意一个——所有这些注解都会累积到下一个定义上。

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

- `@param` 名称必须与子句头部变量匹配——在编译时验证。
- `@example` 是唯一的文档测试通道：`gan test` 将 `gan>` 行编译成原生的 Python 文档测试并运行它们。预期输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子显示为 `'ok'`，映射为 `{'k': 1}`，布尔值为 `True`。
- 标准：**每个公开的 `def` 都带有 `@doc` + `@spec`**（当有参数时还需 `@param`）；面向用户的接口添加 `_trans` 对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，被 git 忽略，位于 `gandora.jsonc` 旁边）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 仅默认语言。`gan doc Mod.fun --locale zh-CN` 会明确要求；`gan lsc doc` 始终返回所有语言环境为 JSON。

## 测试 (GEP-0024)

一个命令，两层：`gan test` 运行每个 `@example` doctest，
然后运行 `tests/*.gan` 中的每个 `test_*` 函数——使用项目的完整模块解析编译并由 pytest 执行（添加一次：`uv add --dev pytest`）。

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

ExUnit 表面：`test "name" do`（定义 `test_<slug>`），
`describe`（为内部名称添加前缀），`assert`/`refute`（比较报告两个操作数），
以及 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk` 作为普通函数。
`tests/` 永远不会发布——它位于源代码根目录之外。

## Python 互操作

`$module` 是一等模块对象；`$` 显式标记互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # dotted chain: imports importlib.metadata
$(PIL.Image).open(f)                # $(...) locks the module boundary explicitly
$(sys).stderr                       # ...single-segment too: import sys, attr stderr
pyimport numpy, as: np              # aliased import
pyimport sys                        # bare import binds `sys` as a plain name
np.array([1, 2]) * 10               # operators broadcast — it's just Python
sys.stderr.write("...")             # bare-name chains have no import ambiguity
$json.dumps(data, indent: 2)        # trailing keywords become kwargs
```

何时使用哪种拼写形式：

| 场景 | 拼写形式 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 链式启发式猜测错误时的一次性引用 | `$(os.path).sep`, `$(sys).stderr` |
| 文件中重复使用的模块 | `pyimport sys`（或 `, as:`）+ 裸名 |
| 经常使用的深层属性 | `@environ $(os).environ` 模块属性 |

在同一个文件中重复使用 `$(...)` 拼写形式是一种代码异味——应声明 `pyimport`。切勿编写围绕 Python API 的包装模块；不存在包装器正是设计所在。

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

`try/rescue/after` 映射到 Python 异常；rescue 子句按异常类进行匹配：

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

`try` 是一个表达式。`try` 内部的尾调用 *不* 会被优化（帧必须保留以用于处理程序）——在其中使用 `recur` 会导致编译错误，而非静默耗尽栈空间。

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

结构体类型在规范中以 `App.User()` 的形式出现。

## 宏

编译时、卫生的、Elixir 风格的。宏在编译器内部的确定性沙箱中运行，并且不留下运行时痕迹。

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

模板变量根据展开而重命名（卫生性）；`var!(name)` 有意地访问调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。使用 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *Expand Macros* 命令检查结果。

## 符号和嵌入语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\\d+/                     # re.compile("\\d+") — string escapes apply
$python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

无大写字母的嵌入语言符号携带整个文档，通过 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`~p` 是面向 AI 散文的官方名称（GEP-0009-R006）**：其主体是原始文本，因此引号、花括号、反斜杠和内联 JSON 无需转义——再也不必与 `\\\"` 搏斗：

```elixir
@task ~p(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~p"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

**数据就是映射——包括粘贴的 JSON**：Gandora 的 `%{}` 字面量是唯一的数据拼写；来自 API 文档的 JSON 文档通过将 `:` 替换为 `=>` 即成为映射（Advisor 会即时指导），运行时 JSON 文本则为 `$json.loads(s)`：

```elixir
@tool %{"type" => "function",
        "function" => %{"name" => "ping", "description" => "Health check.",
                        "parameters" => %{"type" => "object", "properties" => %{}}}}
```

`$python` 是唯一 Python 拼写的转义出口；拼接内容是编译后的 Gandora 表达式，其余所有内容按原样传递。

## 编译器 lint — 警告乃可证明事实

每条警告仅针对静态确定的情况触发，定位到 `gan build` 或编辑器波浪线中的定义行，并提供机械性修复方案（通常通过一键快速修复）：

| 警告 | 含义 | 修复方法 |
| --- | --- | --- |
| 未定义变量 | 读取无绑定对象 — 保证触发 `NameError` | 修正名称 |
| 未使用绑定 | 已绑定，从未读取 | 添加 `_` 前缀（`_meta`） |
| 不可达子句 | 无守卫的全变量头部遮蔽了后续同元数子句（也包括 `case` 通配符） | 重新排序或删除 |
| 丢弃的推导式 | `for` 出现在语句位置 | `Enum.each` |
| 未使用的 `defp` | 死私有函数 | 删除，或使用 `@allow :unused_function` |
| 栈递归 | 自递归，且永远不在尾位置 | 累加器形式、`recur`，或使用 `@allow :stack_recursion` |

`@allow` 目标会被检查 — 拼写错误将导致编译错误。将警告视为缺陷：代码库的标准是零警告。

## Projects and the CLI

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理依赖和 `.venv`；`gandora.local.jsonc` 保存开发者个人偏好，不纳入 git 管理。

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

包以普通 wheel 形式发布（`gan build && uv build && uv publish`），携带编译后的 Python 代码、一个 `gandora.toml` 标记文件以及宏展开所需的 `.gan` 源文件——使用者通过 `uv add` 引入，不引入任何 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一条 JSON 值 —— 专为智能体构建：

```console
gan lsc check --root .                  # 判决结果：{diagnostics, suggestions}
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格、说明（所有语言区域）、参数、tco 结构
gan lsc doc for --root .                # 语言构造也能回答（for/recur/with...）
gan lsc references Stats.mean --root .  # 每个调用点（+ 定义）
gan lsc wsymbols mean --root .          # 项目级符号搜索
gan lsc symbols Stats --root .          # 单个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python 代码（文本形式）
gan lsc expand src/x.gan --root .       # 宏展开后的带引号 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取的 Python 端文档
```

**`gan build` 就是判决结果**（GEP-0025 rev 3）：编译器诊断、Advisor 建议（实践差距、跨语言迁移提示、拼写错误名称的“你是要查找…吗”），以及**制品验证**——生成的 Python 代码会按解析规则（ty）进行类型检查，因此未定义的函数、死掉的导入或缺失的模块成员都会成为构建**错误**，而不是运行时意外。错误会阻止制品生成，这是重型编译器的方式；警告和建议会打印出来，并让构建继续执行。`gan lsc check` 返回相同的判决结果，但以单个 JSON 对象的形式。

```console
gan lsc check --root .
# {"ok": true, "clean": false, "diagnostics": [...],
#  "suggestions": [{"kind": "did_you_mean", "line": 3,
#   "message": "`Enum.mpa` 不是 Enum 的函数 —— 你是要查找 `Enum.map` 吗？", ...}]}
```

判决结果以交通灯形式呈现：`ok`（编译通过 —— 无错误）和 `clean`（ok **并且** 零警告 **并且** 零建议）。红色 → 修复错误；黄色 → 阅读建议；绿色 → 提交。建议会携带首次证据所在的行，跨多个文件的相同发现会合并为一条带注释的条目，覆盖范围包括 `src/` 以及顶层的 `tests/*.gan`（测试模块会获得迁移和惯用写法提示，但免除库注解覆盖率要求）。

智能体的高效循环：**编写 → `gan build`（修复所有发现）→ `gan test` → 发布**。当不确定某个内容会生成什么时，使用 `gan lsc compile file`；当不确定语法时，使用 `gan lsc doc <construct>`（`for`、`spec`、`test`……）。

## 代理风格检查清单

1. 每个公开的 `def`：`@doc` + `@spec`（以及每个参数的 `@param`）；对任何具有有趣行为的代码添加 `@example`——`gan test` 能确保它们保持诚实。
2. 类型规格：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正泛型的流程；结构体使用 `Mod()`；Python 边界处使用 `$mod.Type()`。
3. 迭代：`for`/`Enum` 用于映射形状的工作；累加器尾递归用于无界循环（当需要恒定栈时使用 `recur`）；仅对结构有界的递归使用 `@allow :stack_recursion`，且原因必须显而易见。
4. 互操作：一次性使用 `$`，重复使用用 `pyimport`，无需包装器。
5. 零警告，`gan fmt` 整洁，文档测试通过——工具链、标准库、教程和游乐场均遵循这一标准；请与之保持一致。

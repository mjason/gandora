# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

编写 Gandora 的实用指南——面向人类，也特意面向 AI 智能体。规范性定义见 GEP ([`geps/`](../geps/))；本手册展示各部件如何使用，以及当存在多种拼写时应优先选用哪一种。这里的每种构造都在 [`examples/tour`](../examples/tour) 中演练过（其已签入的 [`generated/`](../examples/tour/generated/) 目录展示了每个章节编译成的精确 Python 代码），并通过 playground 的自检套件进行了实战检验。

决定以下所有内容的基本原则：

- **零运行时。** 生成的 Python 代码是自包含且可读的——评审者手动编写也会如此。辅助函数按模块内联；部署永远不依赖 Gandora。
- **Elixir 语法，Python 语义在内。** 当 Elixir 存在某种构造时，Gandora 就按 Elixir 的方式拼写；值都是普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非观点；悬停提示显示递归如何编译；`gan lsc` 以 JSON 形式提供全部事实。

## 模块和函数

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

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔型内建函数：`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`、比较操作符、`and or not`、算术运算。

## 数据

原子（Atoms）是驻留字符串，属于**纯数据**——它们从不命名模块（这是 `$module` 的职责）。只有 `false` 和 `nil` 为假值；`0`、`""` 和 `[]` 为真值（遵循 Elixir 语义，而非 Python）。

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

`=`、`case`、函数头、`with` 和 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、pin（`^x` — 匹配 x 的*现有*值）以及结构体。

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "the pinned value"
  {:error, %{reason: r}} -> "failed: #{r}"
  _ -> "anything else"
end

cond do                       # 第一个为真的条件（不是模式）
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

失败的 `=`/`case` 匹配会抛出 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新的绑定——中间创建的闭包保留旧值（见下文）。

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

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 完全一致——后续的重绑定、尾递归循环迭代或推导式步骤绝不会泄露到之前创建的闭包中。编译器通过 Python 自身的惯用写法（`lambda x, *, n=n: x + n`）实现这一点；调用时的 arity 保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

管道可以换行继续，但续行必须以 `|>` 开头。

## 迭代：推导式与递归

没有 `loop` 和 `while` 语句。迭代通过 `for`、`Enum` 家族或递归实现——编译器会保证递归的安全性。

### `for` 推导式（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被 SKIPPED
```

主体是一个表达式；`into: %{}` 需要 `{key, value}` 元组作为主体。推导式用于**构建集合**——若将其用于副作用，编译器会发出警告；应改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

对尾位置封闭函数的调用会变成 `while True:` 内部的参数重新绑定——无论深度如何，栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 百万级帧数也能正常工作
```

`recur(args)` 是同一跳转的**受检**写法：它必须位于尾位置且与某个子句的元数匹配，否则构建失败——当你需要恒定栈（而非仅期望）时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用，使用 Python 栈（约 1000 帧）；编译器会在定义处**发出警告**。结构递归——深度受数据边界限制，如树遍历——是合理的：确认后警告消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数编译后的形态都可见：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 会打印该信息，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

一条拼写规则：**类型即调用**——`integer()`、`list(t)`、`Mod()`、`$mod.Type()` 都需要括号；唯一裸写法是类型变量（1–2 个小写字母）和字面量 `nil`。其他任何写法都会产生编译错误，并附带修复建议。

`@spec` 声明函数的类型；编译器会对照各个子句对其进行验证，并在生成的 Python 中输出 **PEP 484 注解**，因此 `pyright`/`mypy` 会检查调用方，悬停提示也会显示真实类型。每个定义组只能有一个 `@spec`，并与其他注解一起放在第一个子句之前。

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
| `atom()` | `str`（原子是内部字符串） |
| `nil` | `None` |
| `term()` / `any()` | `object` |
| `fun()` | `Callable`（未参数化） |

### 容器——构建时具体，接受时抽象

具体容器表示“就是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为类似该容器的类型”——优先将其用于**参数**，这样调用方可以传入元组、区间或生成器；同时因为 `list` 在 Python 类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 只会被遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 支持索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读的类字典对象
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**抽象入，具体出**——接受 `sequence(t)`，返回 `list(t)`。

## 命名类型 (`@type`)

`@type` 为一个类型命名——并且是**泛型的声明点**：参数在头部声明，引用处进行元数检查，所有内容在编译时展开（零运行时开销）：

```elixir
@type age() :: integer()
@type result(t) :: tuple(atom(), t)
@type scores() :: map(string(), age())

@spec parse(string()) :: result(integer())    # -> tuple[str, int]
@spec load(string()) :: Mod.result(string())  # 跨模块引用
```

在 `@type` 主体中未声明的变量、引用处错误的元数、重复定义、遮蔽内置类型以及递归，均属于编译错误，并会给出修复提示；`@type` 上方的 `@doc` 用于记录该类型。在 `@spec` 内部，短类型变量保持隐式作用域——但如果某个规范的整体返回类型是一个在其他地方未使用的变量，则会收到一个实践提示（因为它没有约束任何东西）。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个**一或两个字符**的裸小写名称即为类型变量；每个变量在输出中都会成为模块级别的 `typing.TypeVar`。相同的字母在规范中表示“相同的类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸名称会报错并给出提示（“你是指 `name()` 吗？”），因此拼写错误的 `intger` 不会静默地成为泛型。

### 命名参数

`name :: type` 为规范中的参数命名——有助于自文档化，并且会显示在签名提示中：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 使用；括号用于参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并会生成对应的导入语句
```

### 结构体类型

`Mod()` 表示该模块中由 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop()) :: App.Shop()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数法的交互

一个规范覆盖整个函数组；请针对完整的参数列表（包括默认值）编写规范。生成的委托函数和调度器会携带这些注解。为一个不存在的函数或元数命名 `@spec` 属于编译错误，因此规范不会静默地腐烂。

## 文档与文档测试

在 `def` 之前的注解顺序：`@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow` 中的任意一个——所有这些注解都会累积到下一个定义上。

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
- `@example` 是唯一的文档测试通道：`gan test` 将 `gan>` 行编译为原生 Python 文档测试并运行。预期输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子打印为 `'ok'`，映射打印为 `{'k': 1}`，布尔值打印为 `True`。
- 标准：**每个公开的 `def` 都带有 `@doc` + `@spec`**（当参数存在时还包括 `@param`）；面向用户的接口则添加 `_trans` 对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，已加入 .gitignore，位于 `gandora.jsonc` 旁边）> `GAN_DOC_LOCALE` 环境变量/编辑器设置 > 仅默认语言。`gan doc Mod.fun --locale zh-CN` 显式指定；`gan lsc doc` 始终以 JSON 形式返回所有语言区域。

## Testing (GEP-0024)

一条命令，两个层次：`gan test` 运行所有 `@example` 文档测试，然后运行 `tests/*.gan` 中所有 `test_*` 函数——这些函数会以项目的完整模块解析进行编译，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

```elixir
# tests/test_stats.gan
defmodule TestStats do
  @moduledoc "文档测试未覆盖的边缘情况。"

  use Test

  describe "mean" do
    test "averages evenly" do
      assert Stats.mean([1, 2, 3, 4]) == 2.5   # 失败时左右两侧均报告名称
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

ExUnit 表面：`test "name" do`（定义 `test_<slug>`），`describe`（为内部名称添加前缀），`assert`/`refute`（比较失败时报告两个操作数），以及 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk` 作为普通函数。`tests/` 永远不会被发布——它位于源代码根目录之外。

## Python 互操作

`$module` 是一等模块对象；`$` 清晰标记了互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # 点链：导入 importlib.metadata
$(PIL.Image).open(f)                # $(...) 显式锁定模块边界
$(sys).stderr                       # ...单段也一样：import sys, 属性 stderr
pyimport numpy, as: np              # 别名导入
pyimport sys                        # 裸导入将 `sys` 绑定为普通名称
np.array([1, 2]) * 10               # 运算符广播——它只是 Python
sys.stderr.write("...")             # 裸名称链没有导入歧义
$json.dumps(data, indent: 2)        # 尾随关键字变为 kwargs
```

何时使用何种写法：

| 情况 | 写法 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性引用且链式猜测错误时 | `$(os.path).sep`, `$(sys).stderr` |
| 模块在文件中多次使用 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 深层属性频繁使用 | `@environ $(os).environ` 模块属性 |

在同一文件中重复使用 `$(...)` 写法是一种不良信号——应声明 `pyimport`。切勿编写 Python API 的包装模块；没有包装正是设计初衷。

装饰器通过 `@decorate` 挂载；模块属性保存导入时状态：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是一个普通的 ASGI 目标：`uvicorn app.api:app --app-dir dist`。

### 装饰器：两个层次

运行时装饰属于 `$` 世界；编译时装饰属于宏。具体来说：

- **`@decorate <expr>`** 将任何 Python 装饰器附加到下一个 def 上——可以是库的（`$functools.lru_cache(maxsize: 64)`、`@app.get("/")`），也可以是返回包装器的 Gandora 函数。多个装饰器可以堆叠；离 def 最近的那个最先包装，与 Python 一致。
- **Gandora 编写的包装器是严格元数匹配的**（`fn x -> ... f.(x) end` 仅包装 1 参数函数）——Gandora 没有 `*args`，这是有意为之。*通用*任意元数装饰器是 Python 的职责：将其放在源码旁边的 `.py` 文件中，并引用为 `$mymod.deco`。
- **编译时重写**——即 Elixir 风格的装饰器——是 `defattr :name` + `@on_definition` 宏（GEP-0008）：它看到真实函数头，保持零运行时开销，并且自身可以为 Python 侧发出 `@decorate`。指南中的 `@cache` 章节是完整示例。
- 由 Gandora `fn` 构建的包装器本质上是 lambda——它会丢失 `__name__`/`__doc__`；如果内省很重要，请用 Python 编写该装饰器。

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

`try` 是一个表达式。`try` 内部的尾调用 *不会* 被优化（框架必须保留给处理程序）——`recur` 在那里是一个编译错误，而不是静默的堆栈消耗者。

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

编译时、卫生的、Elixir 风格的。宏在编译器的确定性沙箱中运行，不会留下任何运行时痕迹。

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

模板变量会在每次展开时重命名（卫生性）；`var!(name)` 有意地访问调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *展开宏* 命令检查结果。

## 符号与嵌入语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # 字符串，自由定界符选择
~r/\\d+/                     # re.compile("\\d+") — 字符串转义仍然生效
$python(sum(i*i for i in range(n)))   # 一个逐字的 Python 表达式
```

无大写形式的嵌入语言符号可以携带整个文档，并通过 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`~p` 是用于 AI 散文的受祝福名称**（GEP-0009-R006）：主体是原始的，因此引号、大括号、反斜杠和内联 JSON 都不需要转义 — 再也不用与 `\\\"` 斗争了：

```elixir
@task ~p(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~p"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

**数据是映射 — 包括粘贴的 JSON**：Gandora 的 `%{}` 字面量是唯一的数据拼写；来自 API 文档的 JSON 文档通过将 `:` 交换为 `=>` 成为映射（Advisor 会即时教授这一点），运行时 JSON 文本是 `$json.loads(s)`：

```elixir
@tool %{"type" => "function",
        "function" => %{"name" => "ping", "description" => "Health check.",
                        "parameters" => %{"type" => "object", "properties" => %{}}}}
```

`$python` 是 Python 专属拼写的退路；拼接部分是编译后的 Gandora 表达式，其他所有内容原样通过。

## Compiler lints — warnings are provable facts

Each fires only on something statically certain, lands on the
definition line in `gan build`/editor squiggles, and has a
mechanical remedy (often a one-click quick fix):

| Warning | Meaning | Remedy |
| --- | --- | --- |
| undefined variable | a read nothing binds — guaranteed `NameError` | fix the name |
| unused binding | bound, never read | `_` prefix (`_meta`) |
| unreachable clause | a guard-less all-variable head shadows later same-arity clauses (also `case` wildcards) | reorder or delete |
| discarded comprehension | `for` in statement position | `Enum.each` |
| unused `defp` | dead private function | delete, or `@allow :unused_function` |
| stack recursion | self-recursive, never in tail position | accumulator form, `recur`, or `@allow :stack_recursion` |

`@allow` targets are checked — a typo is a compile error. Treat
warnings as defects: the codebase standard is zero.

## 项目与命令行界面

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 拥有依赖和 `.venv`；`gandora.local.jsonc` 存放开发者个人偏好，且不纳入 Git 管理。

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

包的发布形式为普通 wheel 文件（`gan build && uv build && uv publish`），其中包含已编译的 Python、一个 `gandora.toml` 标记，以及宏展开时依赖的 `.gan` 源文件——使用者通过 `uv add` 添加它们，不会引入 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一条 JSON 值——专为智能体构建：

```console
gan lsc check --root .                  # 检查结果：{diagnostics, suggestions}
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格说明、文档文本（所有语言）、参数、TCO 形态
gan lsc doc for --root .                # 语言构造同样回答（for/recur/with...）
gan lsc references Stats.mean --root .  # 所有调用点（+ 定义）
gan lsc wsymbols mean --root .          # 项目级符号搜索
gan lsc symbols Stats --root .          # 一个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python 代码（文本形式）
gan lsc expand src/x.gan --root .       # 宏展开后的引用 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取的 Python 端文档
```

**`gan build` 就是检查结果**（GEP-0025 修订版 3）：编译器诊断、Advisor 建议（实践缺口、跨语言迁移提示、拼写错误名称的“你是不是要查找”）、以及**工件验证**——生成的 Python 代码会经过解析规则的类型检查（ty），因此未定义函数、死 import 或缺失模块成员都会导致构建**错误**，而非运行时意外。错误会阻止构建工件（重型编译器方式）；警告和建议则打印输出并继续构建。`gan lsc check` 以单个 JSON 对象返回相同的检查结果。

```console
gan lsc check --root .
# {"ok": true, "clean": false, "diagnostics": [...],
#  "suggestions": [{"kind": "did_you_mean", "line": 3,
#   "message": "`Enum.mpa` 不是 Enum 的函数——你是不是要查找 `Enum.map`？", ...}]}
```

检查结果以交通灯开头：`ok`（编译通过——无错误）和 `clean`（ok **且**零警告 **且** 零建议）。红色→修复错误；黄色→阅读建议；绿色→提交。建议携带其首个证据的行号，跨多个文件的相同发现会合并为一条带注释的条目，覆盖范围包括 `src/` 以及顶层 `tests/*.gan`（测试模块会获得迁移和惯用法提示，但不受库注解覆盖约束）。

智能体的高效循环：**编写 → `gan build`（修复所有发现）→ `gan test` → 发布**。当不确定某物生成什么时，使用 `gan lsc compile file`；当不确定语法时，使用 `gan lsc doc <construct>`（`for`、`spec`、`test`……）。

## 代理风格检查清单

1. 每个公开的 `def`：`@doc` + `@spec`（+ 每个参数对应 `@param`）;  
   对任何存在有趣行为的内容添加 `@example`——`gan test` 确保它们保持诚实。
2. 规格：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正泛型的流程；结构体使用 `Mod()`；Python 边界处使用 `$mod.Type()`。
3. 迭代：映射形状的工作使用 `for`/`Enum`；无界循环使用累加器尾递归（当需要恒定栈时使用 `recur`）；仅在结构有界递归时使用 `@allow :stack_recursion`，且原因必须显而易见。
4. 互操作：一次性使用 `$`，重复使用用 `pyimport`，不编写包装器。
5. 零警告，`gan fmt` 格式整洁，文档测试通过——工具链、标准库、教程和 playground 均遵循此标准；请与之保持一致。

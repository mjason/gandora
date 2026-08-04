# The Gandora Language Manual

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

The practical guide to writing Gandora — for humans and, deliberately,
for AI agents. Normative definitions live in the GEPs
([`geps/`](../geps/)); this manual shows how the pieces are used and
which spelling to prefer when several exist. Every construct here is
exercised by [`examples/tour`](../examples/tour) (whose checked-in
[`generated/`](../examples/tour/generated/) shows the exact Python each
chapter compiles to) and battle-tested by the playground's
self-checking suite.

Ground rules that shape everything below:

- **Zero runtime.** Generated Python is self-contained and readable —
  what a reviewer would have written by hand. Helpers are inlined per
  module; deployment never depends on Gandora.
- **Elixir surface, Python semantics underneath.** Where Elixir has a
  construct, Gandora spells it the Elixir way; values are ordinary
  Python objects.
- **The compiler talks back.** Warnings are statically provable facts,
  not opinions; hover shows how recursion compiled; `gan lsc` serves
  every fact as JSON.

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

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔形状的内置函数：`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`、比较运算符、`and or not`、算术运算。

## 数据

原子是驻留字符串和**纯数据**——它们从不命名模块（那是`$module`的工作）。只有`false`和`nil`是假值；`0`、`""`和`[]`是真值（Elixir语义，而非Python的）。

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

`=`、`case`、函数头、`with` 以及 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、pin（`^x`——匹配 x 的*现有*值）以及结构体。

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

失败的 `=`/`case` 匹配会抛出 `GanMatchError`。重新绑定一个名称（`x = 1; x = 2`）会创建一个新的绑定——介于之间创建的闭包会保留旧值（见下文）。

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

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 完全一致——后续的重新绑定、尾递归循环迭代或推导步骤绝不会泄漏到先前创建的闭包中。编译器使用 Python 自身的惯用法（`lambda x, *, n=n: x + n`）来实现这一点；调用参数数量保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

当管道以 `|>` 开头时，可以延续到下一行。

## 迭代：推导与递归

没有 `loop` 和 `while`。迭代使用 `for`、`Enum` 家族或递归——而编译器确保递归安全。

### `for` 推导（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # multiple generators
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # dict comprehension
for {k, v} <- [{"a", 1}, :bad], do: k          # non-matching elements are SKIPPED
```

主体是一个表达式；`into: %{}` 需要一个 `{key, value}` 元组主体。推导式**构建一个集合**——将其用于副作用会导致编译器警告；请改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

对尾位置外部函数的调用变成 `while True:` 内部的参数重新绑定——在任何深度下栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # a million frames is fine
```

`recur(args)` 是相同跳转的**受检**写法：它必须位于尾位置且匹配子句的参数数量，否则构建失败——当恒定栈空间是要求而非希望时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用并使用 Python 栈（约 1000 帧）；编译器在定义处**发出警告**。结构递归——深度受数据限制，如树遍历——是合法的：确认后警告消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态可见：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 会打印它，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

`@spec` 声明函数的类型；编译器会将其与函数子句进行验证，并在生成的 Python 中生成 **PEP 484 注解**，从而让 `pyright`/`mypy` 检查调用方，悬浮提示也能显示真实类型。每个定义组一个 `@spec`，放在该组第一个子句之前的其他注解之后。

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
| `fun()` | `Callable`（无参数化） |

### 容器——构建时具体，接收时抽象

具体容器表示“正是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为类似它的东西”——优先在**参数**中使用它们，这样调用方可以传入元组、范围或生成器，同时因为 `list` 在 Python 类型系统中是抗变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 只会被遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 可索引/可重新遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读字典
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**抽象入、具体出**——接收 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个由**一或两个字符**组成的裸小写名称是一个类型变量；每个变量在输出中会变成一个模块级的 `typing.TypeVar`。同一个字母在 spec 中表示“同一类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

较长的裸名称会报错，并附带提示（“是否是指 `name()`？”），因此拼写错误的 `intger` 不会无声地变成泛型变量。

### 命名参数

`name :: type` 在 spec 中为参数命名——既可自文档化，也能在签名帮助中显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 出现；括号可以参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并会生成相应的导入
```

### 结构体类型

`Mod.t()` 表示该模块中由 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数列表的交互

一个 spec 覆盖整个函数组；针对完整的参数列表编写（包含默认值）。生成的委托和分发器会携带这些注解。若 `@spec` 指定了一个不存在的函数或参数数量，会在编译时报错，因此 spec 不会静默地腐烂。

## 文档与文档测试

`def` 之前的注解顺序：`@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow` —— 所有注解都会累积到下一个定义上。

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

- `@param` 的名称必须与子句头部的变量匹配 —— 在编译时进行验证。
- `@example` 是唯一的文档测试通道：`gan test` 将 `gan>` 行编译为原生 Python 文档测试并执行。预期输出必须是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子打印为 `'ok'`，映射打印为 `{'k': 1}`，布尔值打印为 `True`。
- 标准：**每个公共 `def` 必须带有 `@doc` 和 `@spec`**（若含有参数，则还需 `@param`）；面向用户的接口需添加 `_trans` 配对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，位于 `gandora.jsonc` 旁，应被 git 忽略） > `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 默认语言。`gan doc Mod.fun --locale zh-CN` 显式指定语言；`gan lsc doc` 始终以 JSON 格式返回所有语言区域。

## 测试 (GEP-0024)

一条命令，两层作用：`gan test` 先运行所有 `@example` 文档测试，然后运行 `tests/*.gan` 中所有 `test_*` 函数——这些函数会以项目的完整模块解析进行编译，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

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

ExUnit 接口：`test "name" do`（定义 `test_<slug>`）、`describe`（为内部名称添加前缀）、`assert`/`refute`（比较会报告两个操作数），以及作为普通函数的 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk`。`tests/` 永远不会被打包——它位于源根目录之外。

## Python 互操作

`$module` 是一等模块对象；`$` 清晰地标记了互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # 点链：导入 importlib.metadata
$(PIL.Image).open(f)                # $(...) 显式锁定模块边界
$(sys).stderr                       # ...单段也可：import sys，属性 stderr
pyimport numpy, as: np              # 带别名的导入
pyimport sys                        # 裸导入将 `sys` 绑定为普通名称
np.array([1, 2]) * 10               # 运算符广播——它依然是 Python
sys.stderr.write("...")             # 裸名称链无导入歧义
$json.dumps(data, indent: 2)        # 尾随关键字变为关键字参数
```

何时使用何种拼写：

| 场景 | 拼写 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性引用且链式启发式猜测错误时 | `$(os.path).sep`, `$(sys).stderr` |
| 模块在文件中多次使用 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 深层属性频繁使用 | `@environ $(os).environ` 模块属性 |

同一文件中重复出现 `$(...)` 拼写是一种坏味道——应声明 `pyimport`。切勿为 Python API 编写包装模块；无包装正是设计所在。

装饰器通过 `@decorate` 附加；模块属性保存导入时状态：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是一个普通的 ASGI 目标：
`uvicorn app.api:app --app-dir dist`。

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

`try` 是一个表达式。`try` 内部的尾调用 *不* 会优化（栈帧必须为处理程序存活）——其中的 `recur` 会导致编译错误，而非静默地耗尽栈空间。

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

结构体类型在规范中表示为 `App.User.t()`。

## 宏

编译时、卫生的、Elixir 风格的。宏在编译器内部的确定性沙箱中运行，不留运行时痕迹。

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

模板变量按扩展重命名（卫生性）；`var!(name)` 有意触及调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。使用 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *Expand Macros* 命令检查结果。

## 符号（Sigils）与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # 字符串，可自由选择分隔符
~r/\\d+/                     # re.compile("\\d+") — 字符串转义规则适用
~python(sum(i*i for i in range(n)))   # 一条逐字嵌入的 Python 表达式
```

大写字母开头的嵌入式语言符号可携带整个文档，并通过 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

`~python` 是仅用于 Python 拼写的逃生出口；拼接处是编译后的 Gandora 表达式，其余部分逐字通过。

## Compiler lint——警告是可证明的事实

每个 lint 仅针对静态确定的项触发，定位在 `gan check`/`gan build`/编辑器波浪线中的定义行，并具有机械修复方案（通常是一键快速修复）：

| 警告 | 含义 | 修复方法 |
| --- | --- | --- |
| 未定义变量 | 读取的变量未绑定——保证引发 `NameError` | 修正名称 |
| 未使用的绑定 | 已绑定，从未读取 | `_` 前缀（`_meta`） |
| 不可达的子句 | 无守卫的纯变量头部遮蔽了后续相同元组的子句（也包括 `case` 通配符） | 重新排序或删除 |
| 被丢弃的推导 | `for` 处于语句位置 | `Enum.each` |
| 未使用的 `defp` | 死掉的私有函数 | 删除，或 `@allow :unused_function` |
| 栈递归 | 自递归，但从未在尾位置 | 累加器形式、`recur` 或 `@allow :stack_recursion` |

`@allow` 目标会被检查——拼写错误是编译错误。将警告视为缺陷：代码库标准为零。

## 项目与命令行界面

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理依赖项和 `.venv`；`gandora.local.jsonc` 保存开发者个人偏好，不应纳入版本控制。

```console
gan init my-app          # 新建项目           gan check      # 仅分析 + 检查
gan run src/main.gan     # 编译并执行         gan build      # 编译到 outDir
gan test                 # 运行 @example 文档测试
gan fmt src              # 原地格式化         gan fmt --check src   # CI 门禁
gan fmt --diff src       # 显示差异           echo ... | gan fmt -  # 标准输入 -> 标准输出
gan doc Enum.take        # 文档（+ --locale） gan repl       # 交互式
gan expand src/x.gan     # 宏展开输出
gan init --package name
```

包以普通 wheel 格式发布（`gan build && uv build && uv publish`），包含编译后的 Python 代码、一个 `gandora.toml` 标记文件，以及宏展开所需的 `.gan` 源文件——使用者通过 `uv add` 安装，无需引入 Gandora 运行时环境。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一个 JSON 值——专为代理设计：

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

**检查判定是编译器的教学通道**（GEP-0025）：`gan check` 输出编译器诊断信息 *和* Advisor 建议（实践差距、跨语言迁移提示、拼写错误名称的“你是否要查找”）；`gan lsc check` 返回相同内容但作为一个 JSON 对象 `{diagnostics, suggestions}`。**`gan build` 首先运行 check** —— 一个重量级编译器，以 Rust 的方式：错误停止构建，警告和建议打印并允许继续。

```console
gan lsc check --root .
# {"diagnostics": [...], "suggestions": [{"kind": "did_you_mean",
#   "message": "`Enum.mpa` is not a function of Enum — did you mean `Enum.map`?", ...}]}
```

对代理而言的高效循环：**编写 → `gan check`（修复每个发现）→ `gan test` → `gan build`**。当不确定某物生成什么时，使用 `gan lsc compile file`；当不确定语法时，使用 `gan lsc doc <construct>`（`for`、`spec`、`test`、……）。

## 针对代理的样式检查清单

1. 每个公开的 `def`：`@doc` + `@spec`（每个参数对应 `@param`）；对于任何有有趣行为的代码，添加 `@example`——`gan test` 会让它们保持诚实。
2. 规格：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；对真正泛型的流程使用类型变量；结构体使用 `Mod.t()`；在 Python 边界处使用 `$mod.Type()`。
3. 迭代：`for`/`Enum` 用于映射形状的工作；无界循环使用累加器尾递归（当需要恒定栈时用 `recur`）；`@allow :stack_recursion` 仅用于结构限界递归，且原因必须显而易见。
4. 互操作：一次性使用 `$`，重复使用用 `pyimport`，不编写包装器。
5. 零警告，`gan fmt` 整洁，文档测试通过——工具链、标准库、教程和游乐场均坚守此标准；与之保持一致。

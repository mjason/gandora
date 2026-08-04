# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

Gandora 的实用写作指南——面向人类，同样也刻意面向 AI 代理。规范性定义位于 GEP 中（[`geps/`](../geps/)）；本手册展示各个部分如何使用，以及当存在多种拼写时，应优先选用哪一种。这里的每个构造都在 [`examples/tour`](../examples/tour) 中得到实践（其已签入的 [`generated/`](../examples/tour/generated/) 显示了每章编译成的确切 Python 代码），并通过 playground 的自检套件进行了实战测试。

以下基本规则贯穿始终：

- **零运行时。** 生成的 Python 是自包含且可读的——与审阅者手写的内容无异。辅助函数按模块内联；部署从不依赖 Gandora。
- **Elixir 语法，Python 语义。** 凡是 Elixir 具有的构造，Gandora 都按 Elixir 的方式拼写；值则是普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非意见；悬停时显示递归如何编译；`gan lsc` 以 JSON 格式提供每个事实。

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

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔型的内置函数：`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`、比较运算、`and or not`、算术运算。

## 数据

原子（Atom）是驻留字符串（interned strings）和**纯数据**——它们从不命名模块（那是`$module`的职责）。只有`false`和`nil`为假值；`0`、`""`和`[]`为真值（遵循 Elixir 语义，而非 Python 的）。

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

`=`, `case`, 函数头, `with` 和 `for` 生成器都可以匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、固定操作符（`^x` —— 匹配变量 x 的*现有*值）以及结构体。

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

失败的 `=`/`case` 匹配会抛出 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新的绑定 —— 在中间创建的闭包保留旧值（见下文）。

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

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 中完全相同 — 后续的重新绑定、尾递归循环迭代或推导步骤永远不会泄漏到之前创建的闭包中。编译器通过 Python 自身的惯用法（`lambda x, *, n=n: x + n`）实现这一点；调用参数数量保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

管道可以继续到下一行，当它以 `|>` 开头时。

## 迭代：推导式与递归

没有 `loop` 和 `while`。迭代通过 `for`、`Enum` 系列或递归实现——而编译器使递归变得安全。

### `for` 推导式 (GEP-0020)

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # multiple generators
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # dict comprehension
for {k, v} <- [{"a", 1}, :bad], do: k          # non-matching elements are SKIPPED
```

主体是一个表达式；`into: %{}` 需要一个 `{key, value}` 元组形式的主体。推导式会**构建一个集合**——将其用于副作用会导致编译器警告；应改用 `Enum.each`。

### 尾递归编译为循环 (GEP-0019)

尾位置中对封闭函数的调用会变成 `while True:` 内部的参数重新绑定——无论深度如何，栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # a million frames is fine
```

`recur(args)` 是同一跳转的**受检**写法：它必须位于尾位置且匹配某个子句的参数数量，否则构建失败——当恒定栈空间是要求而非期望时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用并使用 Python 栈（约 1000 帧）；编译器会在定义处**发出警告**。结构递归——深度受数据限制，如树遍历——是合理的：确认它后警告便会消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态可见：悬停时显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 会打印它，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

`@spec` 声明函数类型；编译器会将其与子句进行验证，并在生成的 Python 代码中生成 **PEP 484 注解**，因此 `pyright`/`mypy` 可以检查调用方，鼠标悬停时显示真实类型。每个定义组只有一个 `@spec`，与其他注解一起放在第一个子句之前。

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

### 容器 —— 构建时具体，接受时抽象

具体容器表示“恰好是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为类似的东西”——在**参数**中优先使用它们，以便调用方可以传入元组、范围或生成器，同时因为 `list` 在 Python 类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  —— 只会被遍历一次
sequence(t)                # collections.abc.Sequence[t]  —— 可索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] —— 只读的类字典
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**抽象输入，具体输出** —— 接受 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个**一或两个字符**的裸小写名称是类型变量；每个变量在输出中会成为模块级别的 `typing.TypeVar`。同一个字母在 spec 中表示“同一个类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸名称会报错并给出提示（“你是不是想写 `name()`？”），因此拼写错误的 `intger` 不会偷偷变成泛型。

### 命名参数

`name :: type` 在 spec 中为参数命名——既自我说明，又在签名提示中显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 出现；括号提供参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并自动生成导入语句
```

### 结构体类型

`Mod.t()` 是由该模块中 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数数量的交互

一个 spec 覆盖整个组；为完整参数列表（含默认值）编写 spec。生成的委托函数和分发器会携带这些注解。如果 `@spec` 指定的函数或参数数量不存在，则会产生编译错误，因此 spec 不会无声地腐烂。

## 文档与文档测试

`def` 之前的注释顺序：`@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow` —— 所有注解均会累计到下一个定义上。

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

- `@param` 的名称必须与子句头部的变量名保持一致 —— 会在编译时校验。
- `@example` 是唯一的文档测试通道：`gan test` 会将 `gan>` 行编译为原生 Python 文档测试并执行。预期输出为 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子输出为 `'ok'`，映射为 `{'k': 1}`，布尔值为 `True`。
- 标准要求：**每个公开的 `def` 都应带有 `@doc` + `@spec`**（如果带有参数，还应包含 `@param`）；面向用户的接口需添加 `_trans` 配对。

文档语言是**开发者**的个人偏好，而非项目配置：优先级顺序为 `gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，应被 git 忽略，位于 `gandora.jsonc` 同级）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 默认语言。`gan doc Mod.fun --locale zh-CN` 用于显式指定语言；`gan lsc doc` 始终以 JSON 格式返回所有语言环境。

## Testing (GEP-0024)

一个命令，两层：`gan test` 运行每一个 `@example` 文档测试，然后运行 `tests/*.gan` 中每一个 `test_*` 函数——使用项目的完整模块解析进行编译，并由 pytest 执行（一次性添加：`uv add --dev pytest`）。

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

ExUnit 表面：`test "name" do`（定义 `test_<slug>`），`describe`（前缀内部名称），`assert`/`refute`（比较报告两个操作数），以及 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk` 作为普通函数。`tests/` 从不发布——它位于源代码根目录之外。

## Python 互操作

`$module` 是一个一等模块对象；`$` 显式地标记互操作边界。

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

何时使用哪种拼写：

| 情况 | 拼写 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性且链式启发式猜测错误时 | `$(os.path).sep`, `$(sys).stderr` |
| 在文件中重复使用的模块 | `pyimport sys` (或 `, as:`) + 裸名称 |
| 经常使用的深层属性 | `@environ $(os).environ` 模块属性 |

在同一个文件中重复使用 `$(...)` 拼写是一种坏味道——应声明一个 `pyimport`。永远不要为 Python API 编写包装模块；没有包装器正是设计所在。

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

`try/rescue/after` 映射到 Python 异常；`rescue` 子句按异常类匹配：

```elixir
try do
  risky()
rescue
  e in $builtins.ValueError -> {:bad_value, to_string(e)}
  e in $requests.HTTPError -> {:http, e.response.status_code}
  e -> {:error, to_string(e)}      # 裸变量：任何 Exception
after
  cleanup()                        # 总是执行，不贡献值
end

raise "message"                    # -> raise RuntimeError("message")
```

`try` 是一个表达式。`try` 内部的尾调用 **不会** 被优化（栈帧必须保留给处理程序）——其中的 `recur` 是编译期错误，而非静默的栈消耗。

## Structs

```elixir
defmodule App.User do
  defstruct name: nil, age: 0, tags: []
end

u = %App.User{name: "MJ"}               # frozen dataclass instance
%App.User{name: n} = u                  # pattern (isinstance + fields)
older = %App.User{u | age: u.age + 1}   # dataclasses.replace
m2 = %{m | count: 2}                    # plain-map update: {**m, ...}
```

结构体类型在规约中表现为 `App.User.t()`。

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

模板变量在每次展开时被重命名（卫生）；`var!(name)` 有意触及调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *展开宏* 命令检查结果。

## 符号与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # 字符串，自由选择分隔符
~r/\\d+/                     # re.compile("\\d+") — 字符串转义生效
~python(sum(i*i for i in range(n)))   # 一条原样传递的 Python 表达式
```

大写字母开头的嵌入式语言符号借助 `~` 和 `<%= expr %>` 插值将整个文档回馈到 Gandora（编辑器会高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`~prompt` 是 AI 文本的受名符号**（GEP-0009-R006）：主体为原始文本，因此引号、花括号、反斜杠和内联 JSON 均无需转义——再也不用与 `\\\"` 搏斗了：

```elixir
@task ~prompt(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~prompt"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

`~python` 是仅用于 Python 写法的逃生口；插值部分为编译后的 Gandora 表达式，其余内容原样传递。

## 编译器lint — 警告是可证明的事实

每个lint仅针对静态确定的内容触发，在`gan build`中定位到定义行/编辑器波浪线，并且具有机械式的修复方法（通常是一键快速修复）：

| 警告 | 含义 | 修复方法 |
| --- | --- | --- |
| 未定义变量 | 读取未绑定任何值 — 保证引发`NameError` | 修正名称 |
| 未使用的绑定 | 已绑定但从未读取 | `_`前缀（`_meta`） |
| 不可达子句 | 无守卫的全变量头部遮蔽了后续相同元数的子句（也包括`case`通配符） | 重新排序或删除 |
| 被丢弃的推导式 | `for`处于语句位置 | `Enum.each` |
| 未使用的`defp` | 无用的私有函数 | 删除，或使用`@allow :unused_function` |
| 栈递归 | 自递归，从未处于尾位置 | 累加器形式、`recur`或`@allow :stack_recursion` |

`@allow`目标会被检查 — 拼写错误会导致编译错误。将警告视为缺陷：代码库标准为零缺陷。

## 项目与命令行界面

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理依赖和 `.venv`；`gandora.local.jsonc` 保存开发者个人偏好，不纳入版本控制。

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

包以普通 wheel 格式发布（`gan build && uv build && uv publish`），其中包含编译后的 Python 代码、`gandora.toml` 标记文件以及宏展开所需的 `.gan` 源文件——使用者通过 `uv add` 引入它们，无需引入 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一个 JSON 值——专为智能体（agent）构建：

```console
gan lsc check --root .                  # 诊断结果：{diagnostics, suggestions}
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格、文档（所有语言区域）、参数、tco 形状
gan lsc doc for --root .                # 语言构造也会给出答案（for/recur/with...）
gan lsc references Stats.mean --root .  # 所有调用点（+ 定义）
gan lsc wsymbols mean --root .          # 项目范围内的符号搜索
gan lsc symbols Stats --root .          # 单个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python 代码（文本形式）
gan lsc expand src/x.gan --root .       # 宏展开后的引号 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取的 Python 侧文档
```

**`gan build` 就是诊断结果**（GEP-0025 修订版 3）：编译器诊断、Advisor 建议（实践差距、跨语言迁移提示、拼写错误名称的“您是不是要查找”），以及**工件验证**——生成的 Python 代码会经过解析规则（ty）的类型检查，因此未定义的函数、无用的导入或缺失的模块成员都会导致构建**错误**，而非运行时意外。错误会阻止工件生成，这是重型编译器的方式；警告和建议则打印出来并继续执行。`gan lsc check` 以单个 JSON 对象的形式返回相同的诊断结果。

```console
gan lsc check --root .
# {"ok": true, "clean": false, "diagnostics": [...],
#  "suggestions": [{"kind": "did_you_mean", "line": 3,
#   "message": "`Enum.mpa` 不是 Enum 的函数——您是不是要查找 `Enum.map`？", ...}]}
```

诊断结果以交通灯信号开头：`ok`（编译通过——无错误）和 `clean`（ok **且** 零警告 **且** 零建议）。红色→修复错误；黄色→阅读建议；绿色→提交。建议会携带其第一个证据的行号，跨多个文件的相同发现会合并为一条带注释的条目，覆盖范围包括 `src/` 以及顶级目录下的 `tests/*.gan`（测试模块会获得迁移和惯用法提示，但不受库注解覆盖范围限制）。

智能体（agent）的高效循环：**编写 → `gan build`（修复所有发现）→ `gan test` → 发布**。当不确定某个内容生成什么时，使用 `gan lsc compile file`；当不确定语法时，使用 `gan lsc doc <construct>`（`for`、`spec`、`test`……）。

## 面向智能体的风格检查清单

1. 每个公开的 `def`：`@doc` + `@spec`（每个参数另加 `@param`）；对于任何具有有趣行为的代码，添加 `@example` —— `gan test` 会确保它们保持诚实。
2. 类型规约：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正泛型的流程；结构体使用 `Mod.t()`；Python 边界处使用 `$mod.Type()`。
3. 迭代：`for`/`Enum` 用于映射形态的工作；累加器尾递归用于无界循环（当要求恒定栈时使用 `recur`）；`@allow :stack_recursion` 仅用于结构有界递归，且理由必须显而易见。
4. 互操作：一次性使用 `$`，重复使用则用 `pyimport`，不包装。
5. 零警告，`gan fmt` 保持整洁，doctest 通过 —— 工具链、标准库、教程以及 playground 均遵循此标准；请与之保持一致。

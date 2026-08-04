# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

编写 Gandora 的实用指南——面向人类，也刻意面向 AI 智能体。规范性定义位于 GEP 中（[`geps/`](../geps/)）；本手册展示各个部分如何使用，以及当存在多种写法时应优先选择哪种拼写。此处每个结构均在 [`examples/tour`](../examples/tour) 中进行了实践（其已检入的 [`generated/`](../examples/tour/generated/) 展示了每个章节编译成的精确 Python 代码），并通过游乐场的自检套件进行了实战验证。

指导以下所有内容的基本规则：

- **零运行时。** 生成的 Python 代码是自包含且可读的——审阅者手工编写也会如此。辅助函数按模块内联；部署从不依赖 Gandora。
- **Elixir 表面，Python 语义底层。** 当 Elixir 有某个构造时，Gandora 以 Elixir 方式拼写它；值则为普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非意见；悬停显示递归如何编译；`gan lsc` 以 JSON 形式提供所有事实。

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

原子是驻留字符串和**纯数据**——它们从不命名模块（那是 `$module` 的工作）。只有 `false` 和 `nil` 为假值；`0`、`""` 和 `[]` 为真值（Elixir 语义，而非 Python 的）。

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

`=`、`case`、函数头、`with` 和 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、pin (`^x` — 匹配 *现有的* x 值) 以及结构体。

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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重新绑定名称 (`x = 1; x = 2`) 会创建一个新绑定 — 在重新绑定之间创建的闭包保留旧值（见下文）。

## 函数作为值

```elixir
double = fn x -> x * 2 end           # -> lambda
classify = fn                        # 多子句 + 守卫 -> 提升后的 def
  0 -> :zero
  n when n > 0 -> :pos
  _ -> :neg
end
add = &(&1 + &2)                     # 使用占位符捕获
sqrt = &($math.sqrt/1)               # 捕获一个 Python 函数
mine = &fact/1                       # 捕获一个模块函数
double.(21)                          # 调用函数值使用 .()
```

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 中的行为完全一致——后续的重新绑定、尾递归循环迭代或推导式步骤永远不会泄漏到之前创建的闭包中。编译器通过 Python 自身的惯用写法（`lambda x, *, n=n: x + n`）来实现这一点；调用时的参数个数保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # 第一个参数管道
df |> .groupby("k") |> .agg(spec)      # 方法管道：在管道值上调用
" gan " |> .strip() |> .upper()        # 同样适用于字面量
value.method(x).attr                   # 任何值上的后缀链式调用
```

管道可以延续到下一行，前提是下一行以 `|>` 开头。

## 迭代：推导式与递归

没有 `loop` 和 `while`。迭代通过 `for`、`Enum` 系列或递归实现——编译器确保递归安全。

### `for` 推导式 (GEP-0020)

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # multiple generators
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # dict comprehension
for {k, v} <- [{"a", 1}, :bad], do: k          # non-matching elements are SKIPPED
```

主体是一个表达式；`into: %{}` 需要一个 `{key, value}` 元组主体。推导式**构建集合**——用于副作用会产生编译器警告；应使用 `Enum.each` 替代。

### 尾递归编译为循环 (GEP-0019)

在尾位置调用包围函数会变成 `while True:` 内的参数重新绑定——任意深度栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # a million frames is fine
```

`recur(args)` 是同一跳转的**检查**形式：它必须位于尾位置且匹配子句参数数量，否则构建失败——在需要恒定栈空间而非期望时使用它：

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

每个函数的编译形态可见：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 打印它，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统 (`@spec`)

`@spec` 声明函数的类型；编译器会对照子句对其进行验证，并在生成的 Python 中输出 **PEP 484 注解**，因此 `pyright`/`mypy` 能够检查调用方，悬停时也可显示真实类型。每个定义组只能有一个 `@spec`，放在第一个子句之前的其他注解之间。

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

### 容器 —— 构建时具体，接受时抽象

具体容器表达“正好是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表达“任何具有类似行为的类型” —— 在**参数**中优先使用它们，以便调用方可以传入元组、区间或生成器，同时因为 `list` 在 Python 类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 只会被遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 支持索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读的类字典结构
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**接受用抽象，返回用具体** —— 参数接受 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个**一到两个字符**的裸小写名称即为类型变量；每个变量在输出中变成模块级的 `typing.TypeVar`。同一个字母在 spec 中代表“相同类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸名称会报错并附带提示（“你是不是想写 `name()`？”），因此拼写错误的 `intger` 不会悄悄变成泛型。

### 命名参数

`name :: type` 在 spec 中为参数命名 —— 自文档化，并且签名提示中会显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可通过 `$module` 出现；括号可带参数：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并输出相应的 import 语句
```

### 结构体类型

`Mod.t()` 表示该模块中由 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数组的交互

一个 spec 覆盖整个组；请为完整的参数列表（含默认值）编写 spec。生成的委托和分发器会携带这些注解。如果 `@spec` 命名的函数或参数个数不存在，则会在编译时报错，因此 spec 不会默默失效。

## 文档和文档测试

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

- `@param` 名称必须与子句头变量匹配——在编译时验证。
- `@example` 是唯一的文档测试通道：`gan test` 将 `gan>` 行编译为原生 Python 文档测试并运行。预期输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子显示为 `'ok'`，映射显示为 `{'k': 1}`，布尔值显示为 `True`。
- 标准：**每个公开的 `def` 都应带有 `@doc` + `@spec`**（以及有参数时的 `@param`）；面向用户的接口需添加 `_trans` 配对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，已被 git 忽略，位于 `gandora.jsonc` 旁边）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 仅默认语言。`gan doc Mod.fun --locale zh-CN` 显式指定；`gan lsc doc` 始终返回所有语言环境为 JSON。

## 测试 (GEP-0024)

一条命令，两层测试：`gan test` 运行所有 `@example` 文档测试，然后运行 `tests/*.gan` 中的所有 `test_*` 函数——这些测试使用项目的完整模块解析进行编译，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

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

ExUnit 表面：`test "name" do`（定义 `test_<slug>`），`describe`（为内部名称添加前缀），`assert`/`refute`（比较运算符会报告两个操作数），以及 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk` 作为普通函数。`tests/` 从不发布——它位于源码根目录之外。

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

何时使用何种拼写：

| Situation | Spelling |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 当链式启发式猜测错误时的一次性使用 | `$(os.path).sep`, `$(sys).stderr` |
| 在文件中反复使用的模块 | `pyimport sys` (或 `, as:`) + 裸名 |
| 经常使用的深层属性 | `@environ $(os).environ` 模块属性 |

在一个文件中重复使用 `$(...)` 拼写是一种代码异味——应声明 `pyimport`。永远不要为 Python API 编写包装模块；不编写包装器正是设计所在。

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

`try/rescue/after` 映射到 Python 异常；`rescue` 子句按异常类进行匹配：

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

raise "message"                    # -> 引发 RuntimeError("message")
```

`try` 是一个表达式。`try` 内部的尾调用**不会**被优化（帧必须为处理程序保留）——此处使用 `recur` 会引发编译错误，而不是静默消耗堆栈。

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

在规范中，结构体类型表示为 `App.User.t()`。

## 宏

编译期、卫生性、Elixir 风格的。宏在编译器的确定性沙箱中运行，不留运行时痕迹。

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

模板变量在每次展开时被重命名（卫生性）；`var!(name)` 有意地访问调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *展开宏* 命令检查结果。

## 符号与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # 字符串，自由选择分隔符
~r/\\d+/                     # re.compile("\\d+") — 字符串转义依然有效
~python(sum(i*i for i in range(n)))   # 一条原样传递的 Python 表达式
```

不带大写字母的嵌入式语言符号可携带整个文档，并通过 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内嵌语言内容）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

`~python` 是仅使用 Python 写法的后门；拼接处是已编译的 Gandora 表达式，其余部分原样传递。

## 编译器 lint — 警告是可证明的事实

每个警告只针对静态确定的情况触发，定位在 `gan build` / 编辑器波浪线下的定义行，并具有机械性的补救措施（通常是一键快速修复）：

| 警告 | 含义 | 补救措施 |
| --- | --- | --- |
| 未定义变量 | 读取未绑定的值 — 保证引发 `NameError` | 修正名称 |
| 未使用的绑定 | 已绑定，从未读取 | 使用 `_` 前缀（`_meta`） |
| 不可达的子句 | 无守卫的全变量头部掩盖了后续相同元数的子句（也包括 `case` 通配符） | 重排序或删除 |
| 丢弃的推导式 | 位于语句位置的 `for` | 使用 `Enum.each` |
| 未使用的 `defp` | 死私有函数 | 删除，或使用 `@allow :unused_function` |
| 栈递归 | 自递归，且不在尾位置 | 累加器形式、`recur`，或使用 `@allow :stack_recursion` |

`@allow` 目标会被检查 — 拼写错误会导致编译错误。将警告视为缺陷：代码库标准为零。

## 项目与 CLI

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理依赖项和 `.venv`；`gandora.local.jsonc` 保存开发者个人偏好，不纳入 git。

```console
gan init my-app          # 新建项目
gan run src/main.gan     # 编译 + 执行
gan build                # 验证 + 编译至 outDir
gan test                 # 运行 @example 文档测试
gan fmt src              # 原地格式化
gan fmt --check src      # CI 检查
gan fmt --diff src       # 显示差异
echo ... | gan fmt -     # 标准输入 -> 标准输出
gan doc Enum.take        # 文档（+ --locale）
gan repl                 # 交互式
gan expand src/x.gan     # 宏输出
gan init --package name
```

包以普通 wheel 的形式发布（`gan build && uv build && uv publish`），携带编译后的 Python、一个 `gandora.toml` 标记文件以及宏所展开的 `.gan` 源码——消费者使用 `uv add` 引入它们，无需引入任何 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一条 JSON 值——专为智能体构建：

```console
gan lsc check --root .                  # 结果：{diagnostics, suggestions}
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格说明、文档（所有语言区域）、参数、tco 形状
gan lsc doc for --root .                # 语言构造同样回答（for/recur/with...）
gan lsc references Stats.mean --root .  # 所有调用点（+ 定义）
gan lsc wsymbols mean --root .          # 项目范围的符号搜索
gan lsc symbols Stats --root .          # 单个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python，以文本形式
gan lsc expand src/x.gan --root .       # 宏展开后的引用 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取 Python 侧文档
```

**`gan build` 就是结果**（GEP-0025 rev 3）：编译器诊断、Advisor 建议（实践缺口、跨语言迁移提示、拼写错误名称的“您是不是要查找？”）、以及**制品验证**——生成的 Python 使用解析规则（ty）进行类型检查，因此未定义的函数、死导入或缺失的模块成员会构成构建**错误**，而非运行时意外。错误以重量级编译器的方式阻止制品生成；警告和建议则打印出来并继续执行。`gan lsc check` 返回与 `gan build` 相同的结果，作为一个 JSON 对象。

```console
gan lsc check --root .
# {"ok": true, "clean": false, "diagnostics": [...],
#  "suggestions": [{"kind": "did_you_mean", "line": 3,
#   "message": "`Enum.mpa` 不是 Enum 的函数——您是不是要查找 `Enum.map`？", ...}]}
```

结果以红绿灯开头：`ok`（编译通过——无错误）和 `clean`（ok **且** 零警告 **且** 零建议）。红色 → 修复错误；黄色 → 阅读建议；绿色 → 提交。建议携带其第一条证据所在的行号，跨多个文件的相同发现会折叠成一条带注释的条目，覆盖范围包括 `src/` 和顶层 `tests/*.gan`（测试模块会收到迁移和惯用法提示，但免于库注解覆盖）。

智能体的高效循环：**编写 → `gan build`（修复所有发现的问题）→ `gan test` → 发布**。当不确定某物生成什么时，使用 `gan lsc compile file`；当不确定语法时，使用 `gan lsc doc <construct>`（`for`、`spec`、`test`……）。

## 面向代理的样式检查清单

1. 每个公共的 `def`：`@doc` + `@spec`（+ 每个参数的 `@param`）；为任何有有趣行为的内容添加 `@example`——`gan test` 确保它们真实可靠。
2. 规范：输入使用抽象容器（`sequence`，`mapping`），输出使用具体容器；类型变量用于真正通用的流程；结构体使用 `Mod.t()`；在 Python 边界使用 `$mod.Type()`。
3. 迭代：`for`/`Enum` 用于映射形状的工作；累加器尾递归用于无界循环（当需要恒定栈时使用 `recur`）；`@allow :stack_recursion` 仅用于结构有界的递归，且原因必须显而易见。
4. 互操作：`$` 一次性使用，`pyimport` 用于重复使用，无包装器。
5. 零警告，`gan fmt` 整洁，文档测试通过——工具链、标准库、教程和游乐场均保持这一标准；与其保持一致。

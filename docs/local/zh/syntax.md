# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

编写 Gandora 的实用指南——面向人类，也特意面向 AI 智能体。规范性定义位于 GEPs 中（[`geps/`](../geps/)）；本手册展示了各个部分的使用方式，以及当存在多种写法时应优先选择哪种拼写。这里的每个构造都通过 [`examples/tour`](../examples/tour) 进行练习（其已检入的 [`generated/`](../examples/tour/generated/) 目录显示了每章编译后生成的精确 Python 代码），并经过 playground 的自检套件的实战检验。

贯穿以下所有内容的基本规则：

- **零运行时。** 生成的 Python 代码自包含且可读——即审阅者手动编写的结果。辅助函数按模块内联；部署从未依赖于 Gandora。
- **Elixir 表面，Python 语义底层。** 当 Elixir 具有某个构造时，Gandora 以 Elixir 的方式拼写它；值是普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非观点；悬停显示递归如何编译；`gan lsc` 以 JSON 形式提供每个事实。

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

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔型内置函数：`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`、比较运算符、`and or not`、算术运算符。

## 数据

原子是驻留字符串，属于**纯数据**——它们从不命名模块（那是 `$module` 的工作）。只有 `false` 和 `nil` 是假值；`0`、`""` 和 `[]` 是真值（遵循 Elixir 语义，而非 Python 的）。

```elixir
:ok  :"quoted atom"                  # 原子 -> Python 字符串
"interp #{1 + 1}"                    # 输出中的 f-string
"""
heredocs too (dedented)             # 此处为 heredoc 字符串（已缩进取消）
"""
[1, 2, 3]  {:pair, 2}  %{"k" => 1, a: 2}   # 列表、元组、映射
[timeout: 500, retries: 3]           # 关键字列表 -> [("timeout", 500), ...]
1..10                                # 包含区间
10 / 4                               # 2.5 — / 是真实除法
10 // 4                              # 2 — 截断除法
rem(-7, 2)                           # -1 — 截断余数（并非 Python 的 %）
"a" <> "b"                           # 字符串拼接
```

## Pattern matching

`=`, `case`, 函数头, `with` 和 `for` 生成器都可以匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、固定（`^x` — 匹配 x 的*现有*值）以及结构体。

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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重绑定名称（`x = 1; x = 2`）会创建一个新绑定——在此期间创建的闭包保留旧值（见下文）。

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

**闭包在创建时通过值捕获**（GEP-0021），与 Elixir 完全相同——后续的重新绑定、尾递归循环迭代或推导步骤永远不会泄漏到之前创建的闭包中。编译器通过 Python 自身的惯用法实现这一点（`lambda x, *, n=n: x + n`）；调用参数数量保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

当管道以 `|>` 开始时，可以延续到下一行。

## 迭代：推导式与递归

Gandora 中没有 `loop` 和 `while`。迭代使用 `for`、`Enum` 家族或递归——编译器会确保递归的安全性。

### `for` 推导式（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被跳过
```

主体是一个表达式；`into: %{}` 需要主体返回 `{key, value}` 元组。推导式用于**构建集合**——若将其用于副作用，编译器会发出警告；请改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

对尾调用位置上的封闭函数的调用，会变成 `while True:` 内的参数重新绑定——无论深度如何，栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万层栈也没问题
```

`recur(args)` 是同一跳转的**受检查**写法：它必须位于尾调用位置，且参数个数与某个子句匹配，否则构建失败——当恒定栈空间是要求而非期望时，请使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（如 `n * fact(n - 1)`）保持为真实调用，并使用 Python 栈（约 1000 层）；编译器会在定义处**发出警告**。结构递归——深度受数据限制（如树遍历）——是合理的：声明后警告消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态均可见：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 会打印该信息，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统 (`@spec`)

一条拼写规则：**类型就是调用**——`integer()`、`list(t)`、`Mod()`、`$mod.Type()` 都必须带括号；唯一允许裸写的是类型变量（1-2 个小写字母）和字面量 `nil`。其他任何写法都会导致编译错误，并附带修正建议。

`@spec` 声明函数的类型；编译器会对照子句验证该声明，并在生成的 Python 中输出 **PEP 484 类型注解**，这样 `pyright`/`mypy` 就能检查调用方，悬停提示也能显示真实类型。每个定义组只能有一个 `@spec`，放在其他注解之前，位于第一个子句前面。

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
| `atom()` | `str`（原子是 interned 字符串） |
| `nil` | `None` |
| `term()` / `any()` | `object` |
| `fun()` | `Callable`（无参数化） |

### 容器——构造时具体，接收时抽象

具体容器表示“就是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何像它的东西”——在**参数**中优先使用，这样调用方可以传入元组、范围或生成器；同时因为 `list` 在 Python 类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 只会被遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 可索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读的 dict-like
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**抽象接收，具体返回**——接受 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

裸写的一到两个小写字母就是类型变量；每个变量在输出中都会成为模块级别的 `typing.TypeVar`。同一个字母在 spec 中表示“同一类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸写名称会报错并附带提示（“你是不是想写 `name()`？”），这样拼写错误的 `intger` 就不会悄悄变成泛型。

### 命名参数

`name :: type` 为 spec 中的参数命名——既起到自文档作用，也用于签名提示的显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 引入；括号用于参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并生成对应的 import
```

### 结构体类型

`Mod()` 表示该模块中由 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop()) :: App.Shop()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数组的交互

一个 spec 覆盖整个定义组；请针对完整的参数列表（包括默认值）编写 spec。生成的委托函数和调度器会携带这些注解。如果 `@spec` 指向不存在的函数或参数个数，则属于编译错误，因此 spec 不会悄然失效。

## 文档与文档测试

在 `def` 之前的注解顺序：`@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow` 中的任意一个——这些注解都会累积到下一个定义上。

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
- `@example` 是唯一的文档测试通道：`gan test` 会将 `gan>` 行编译为原生 Python 文档测试并运行它们。期望的输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子打印为 `'ok'`，映射打印为 `{'k': 1}`，布尔值打印为 `True`。
- 标准：**每个公开的 `def` 都带有 `@doc` 和 `@spec`**（如果有参数则加上 `@param`）；面向用户的接口需添加 `_trans` 对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，已加入 gitignore，位于 `gandora.jsonc` 旁边）> `GAN_DOC_LOCALE` 环境变量/编辑器设置 > 仅默认语言。`gan doc Mod.fun --locale zh-CN` 显式指定语言；`gan lsc doc` 始终返回所有语言环境为 JSON。

## 测试 (GEP-0024)

一条命令，两个层面：`gan test` 运行所有 `@example` 文档测试，然后运行 `tests/*.gan` 中所有 `test_*` 函数——使用项目的完整模块解析进行编译，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

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

ExUnit 表面：`test "name" do`（定义 `test_<slug>`），`describe`（为内部名称添加前缀），`assert`/`refute`（比较会报告两个操作数），以及 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk` 作为普通函数。`tests/` 永远不会随包发布——它位于源代码根目录之外。

## Python 互操作

`$module` 是一等模块对象；`$` 直观地标记了互操作边界。

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

| 情境 | 拼写 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 链式启发式猜测错误时的一次性使用 | `$(os.path).sep`，`$(sys).stderr` |
| 文件中重复使用的模块 | `pyimport sys`（或 `, as:`）加裸名 |
| 经常使用的深层属性 | `@environ $(os).environ` 模块属性 |

在一个文件中重复使用 `$(...)` 拼写是不好的做法——应声明 `pyimport`。切勿编写围绕 Python API 的包装模块；没有包装正是设计所在。

装饰器通过 `@decorate` 附加；模块属性保存导入时的状态：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是普通的 ASGI 目标：`uvicorn app.api:app --app-dir dist`。

## 错误处理

`try/rescue/after` 映射到 Python 异常；`rescue` 子句按异常类进行匹配：

```elixir
try do
  risky()
rescue
  e in $builtins.ValueError -> {:bad_value, to_string(e)}
  e in $requests.HTTPError -> {:http, e.response.status_code}
  e -> {:error, to_string(e)}      # 裸变量：任何异常
after
  cleanup()                        # 始终执行，不贡献值
end

raise "message"                    # -> raise RuntimeError("message")
```

`try` 是一个表达式。`try` 内部的尾调用 *不会* 被优化（帧必须保留以用于处理程序）——在此处使用 `recur` 会导致编译错误，而非静默的栈溢出。

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

结构体类型在规范中显示为 `App.User()`。

## 宏

编译时、卫生的、Elixir 风格。宏在编译器内部的确定性沙箱中运行，不留下运行时痕迹。

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

模板变量在每次展开时重命名（卫生性）；`var!(name)` 有意访问调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *Expand Macros* 命令检查结果。

## 符号与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # 字符串，自由选择定界符
~r/\\d+/                     # re.compile("\\d+") — 字符串转义仍适用
$python(sum(i*i for i in range(n)))   # 一段逐字传递的 Python 表达式
```

全小写的嵌入式语言符号可携带完整的文档，并通过 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内嵌语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`~p` 是 AI 叙事的官方指定名称**（GEP-0009-R006）：其主体为原始文本，因此引号、花括号、反斜杠和内联 JSON 均无需转义——再也不必与 `\\\"` 纠缠：

```elixir
@task ~p(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~p"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

**数据即映射——包括粘贴的 JSON**：Gandora 的 `%{}` 字面量是唯一的数据写法；从 API 文档粘贴的 JSON 文档只需将 `:` 换成 `=>` 即可变为映射（Advisor 会立即识别），运行时的 JSON 文本则使用 `$json.loads(s)`：

```elixir
@tool %{"type" => "function",
        "function" => %{"name" => "ping", "description" => "Health check.",
                        "parameters" => %{"type" => "object", "properties" => %{}}}}
```

`$python` 是专用于 Python 写法的逃生出口；拼接部分为编译后的 Gandora 表达式，其余内容逐字传递。

## 编译器 lint —— 警告即为可证明的事实

每条警告仅在静态确定的情况下触发，定位在 `gan build`/编辑器波浪线中的定义行上，并带有机械化的修复方式（通常是一键快速修复）：

| 警告 | 含义 | 修复方式 |
| --- | --- | --- |
| 未定义变量 | 读取的绑定未绑定任何内容——保证会引发 `NameError` | 修正名称 |
| 未使用的绑定 | 已绑定但从未读取 | 使用 `_` 前缀（如 `_meta`） |
| 不可达的分支 | 无守卫的全变量头部遮蔽了后续同元数子句（也包括 `case` 通配符） | 重新排序或删除 |
| 丢弃的推导式 | `for` 出现在语句位置 | 使用 `Enum.each` |
| 未使用的 `defp` | 死私有函数 | 删除，或使用 `@allow :unused_function` |
| 栈递归 | 自递归，且从未处于尾位置 | 使用累加器形式、`recur`，或使用 `@allow :stack_recursion` |

`@allow` 的目标会被检查——拼写错误会导致编译错误。将警告视为缺陷：代码库的标准是零警告。

## 项目和 CLI

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理自身依赖和 `.venv`；`gandora.local.jsonc` 存放开发者个人偏好，不纳入 Git 管理。

```console
gan init my-app          # 新建项目
gan run src/main.gan     # 编译 + 执行
gan build                # 验证 + 编译到 outDir
gan test                 # 运行 @example 文档测试
gan fmt src              # 原地格式化
gan fmt --check src      # CI 门禁
gan fmt --diff src       # 显示差异
echo ... | gan fmt -     # 标准输入 -> 标准输出
gan doc Enum.take        # 文档（+ --locale）
gan repl                 # 交互式
gan expand src/x.gan     # 宏展开输出
gan init --package name
```

包以标准 wheel 形式发布（`gan build && uv build && uv publish`），携带编译后的 Python、一个 `gandora.toml` 标记文件以及宏展开所需的 `.gan` 源文件——使用者通过 `uv add` 安装它们，无需引入 Gandora 运行时。

## 人工智能工具箱：`gan lsc`

每个语言事实都是 stdout 上的一个 JSON 值——为智能体而生：

```console
gan lsc check --root .                  # 裁决结果：{diagnostics, suggestions}
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格说明、散文（所有语言环境）、参数、tco 形状
gan lsc doc for --root .                # 语言构造同样可以回答（for/recur/with……）
gan lsc references Stats.mean --root .  # 每个调用点（+ 定义）
gan lsc wsymbols mean --root .          # 项目级符号搜索
gan lsc symbols Stats --root .          # 单个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python 代码（文本形式）
gan lsc expand src/x.gan --root .       # 宏展开后的带引号 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # Python 侧的文档（通过 jedi）
```

**`gan build` 就是裁决结果**（GEP-0025 修订版 3）：编译器诊断信息、Advisor 建议（实践缺陷、跨语言迁移提示、拼写错误名称的“您是否是指”提示），以及**制品验证**——生成的 Python 代码会按照解析规则（ty）进行类型检查，因此未定义的函数、无用的导入或缺失的模块成员都会导致构建**错误**，而不会在运行时才暴露。错误会阻止制品生成，这是重型编译器的做法；警告和建议则会打印出来并允许继续执行。`gan lsc check` 返回与 `gan build` 相同的裁决结果，以 JSON 对象形式呈现。

```console
gan lsc check --root .
# {"ok": true, "clean": false, "diagnostics": [...],
#  "suggestions": [{"kind": "did_you_mean", "line": 3,
#   "message": "`Enum.mpa` 不是 Enum 的函数——您是否是指 `Enum.map`？", ...}]}
```

裁决结果以交通灯模式开头：`ok`（编译通过——无错误）和 `clean`（ok **且** 零警告 **且** 零建议）。红色 → 修复错误；黄色 → 阅读建议；绿色 → 提交。建议会携带其第一个证据所在的行号，跨多个文件的相同发现会合并为一个带注释的条目，覆盖范围包括 `src/` 以及顶层 `tests/*.gan`（测试模块会获得迁移和习惯用法提示，但免于库注解覆盖范围）。

对智能体而言，一个高效的循环是：**编写 → `gan build`（修复所有发现）→ `gan test` → 发布**。当不确定某个内容会生成什么时，使用 `gan lsc compile file`；当不确定语法时，使用 `gan lsc doc <construct>`（`for`、`spec`、`test`……）。

## 面向智能体的风格检查清单

1. 每个公开的 `def`：`@doc` + `@spec`（每个参数加 `@param`）；对于任何具有有趣行为的内容添加 `@example` —— `gan test` 确保它们保持诚实。
2. 类型规范：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正泛型的数据流；`Mod()` 用于结构体；在 Python 边界使用 `$mod.Type()`。
3. 迭代：`for`/`Enum` 用于映射形状的工作；累加器尾递归用于无界循环（当需要恒定栈时使用 `recur`）；仅在结构有界递归时使用 `@allow :stack_recursion`，且原因应显而易见。
4. 互操作：一次性使用 `$`，重复使用使用 `pyimport`，不要包装器。
5. 零警告，`gan fmt` 干净，doctests 通过 —— 工具链、标准库、教程和游乐场都保持这一标准；请与之保持一致。

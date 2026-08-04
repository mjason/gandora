# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

编写 Gandora 的实用指南——面向人类，也特意面向 AI 智能体。规范定义位于 GEP 中（[`geps/`](../geps/)）；本手册展示各部件如何使用，以及当存在多种写法时优先选用哪种。每个构造均在 [`examples/tour`](../examples/tour) 中得到了实践（其已检入的 [`generated/`](../examples/tour/generated/) 展示了每一章编译成的确切 Python 代码），并经过 playground 的自检套件充分测试。

影响以下所有内容的基本原则：

- **零运行时。** 生成的 Python 代码自包含且可读——相当于审阅者手写的样子。辅助函数按模块内联；部署从不依赖 Gandora。
- **Elixir 的外观，Python 的语义在底层。** 凡是 Elixir 具有的构造，Gandora 都按 Elixir 的方式拼写；值则是普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非意见；悬停提示展示递归的编译方式；`gan lsc` 将每个事实以 JSON 形式提供。

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

原子是驻留字符串和**纯数据**——它们从不命名模块（那是 `$module` 的工作）。只有 `false` 和 `nil` 是假值；`0`、`""` 和 `[]` 是真值（遵循 Elixir 语义，而非 Python 的语义）。

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

`=`、`case`、函数头、`with` 和 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、固定（`^x` — 匹配 *现有* 值 x），以及结构体。

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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重绑定名称（`x = 1; x = 2`）会创建一个新绑定——其间创建的闭包会保留旧值（参见下文）。

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

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 中完全相同——稍后的重新绑定、尾递归循环迭代或推导式步骤绝不会泄露到之前创建的闭包中。编译器通过 Python 自身的惯用法（`lambda x, *, n=n: x + n`）实现这一点；调用元数保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

管道可以在下一行继续，当下一行以 `|>` 开头时。

## 迭代：推导式与递归

不存在 `loop` 和 `while`。迭代是通过 `for`、`Enum` 家族或递归实现的——而编译器会确保递归的安全性。

### `for` 推导式 (GEP-0020)

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素被跳过
```

主体是一个表达式；`into: %{}` 需要返回 `{key, value}` 元组的主体。
推导式**构建一个集合**——将其用于副作用会触发编译器警告；请改用 `Enum.each`。

### 尾递归编译为循环 (GEP-0019)

处于尾位置的调用将变为参数重绑定，置于 `while True:` 内部——无论深度如何，栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万帧也没问题
```

`recur(args)` 是同一跳转的**检查**写法：它必须位于尾位置且与某个子句的参数数量匹配，否则构建失败——当恒定的栈空间是要求而非期望时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用，使用 Python 栈（约 1000 帧）；编译器会在定义处**发出警告**。结构递归——深度受数据边界限制，如树遍历——是合理的：显式认可后，警告消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数编译后的形态都可见：悬停显示 `♻ 尾递归 → while 循环` 或 `⚠ 原生调用栈`，`gan doc` 会打印它，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

`@spec` 声明函数的类型；编译器会依据子句验证该声明，并在生成的 Python 中发出 **PEP 484 类型注解**，因此 `pyright`/`mypy` 可检查调用方，悬停时显示真实类型。每个定义组只有一个 `@spec`，与其他注解一起放在第一个子句之前。

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

### 容器——构建时具体，接受时抽象

具体容器表述“恰好是此 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表述“任何行为类似它的东西”——在**参数**中优先使用它们，这样调用方可以传入元组、区间或生成器，同时也因为 `list` 在 Python 类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  —— 会被遍历一次
sequence(t)                # collections.abc.Sequence[t]  —— 可索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] —— 只读的类字典对象
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**入参抽象，出参具体**——接受 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个**一到两个字符**的裸小写名称是类型变量；每个变量在输出中会变成一个模块级的 `typing.TypeVar`。同一个字母在同一个 spec 中表示“同一类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸名称会报错并给出提示（“你是不是想写 `name()`？”），因此拼写错误的 `intger` 不会悄悄变成泛型。

### 命名参数

`name :: type` 在 spec 中为参数命名——既自文档化，也用于签名帮助显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 出现；括号内可参数化：

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

### 与默认参数及多参数性的交互

一个 spec 覆盖整个定义组；请为完整参数列表（包含默认参数）编写 spec。生成的委托和分发器会携带这些注解。若 `@spec` 命名了一个不存在的函数或参数数量，则编译时出错，因此 spec 不会悄悄腐烂。

## 文档与文档测试（doctests）

`def` 之前的注解顺序：`@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow` —— 这些都会累积到下一个定义上。

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

- `@param` 的名称必须与子句头部的变量名一致 —— 这在编译时会被验证。
- `@example` 是唯一的文档测试通道：`gan test` 会将 `gan>` 行编译成原生 Python 文档测试并执行。预期输出必须是 Python 的 `repr`（即 `inspect/1` 展示的内容）：原子输出为 `'ok'`，映射输出为 `{'k': 1}`，布尔值输出为 `True`。
- 标准要求：**每个公开的 `def` 都必须携带 `@doc` + `@spec`**（如果带有参数，则还需加上 `@param`）；面向用户的接口需添加对应的 `_trans` 配对。

文档语言属于**开发者**的个人偏好，而非项目配置：优先级为 `gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，应被 git 忽略，放在 `gandora.jsonc` 旁边）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 默认语言。`gan doc Mod.fun --locale zh-CN` 用于显式指定语言；`gan lsc doc` 始终以 JSON 格式返回所有语言。

## 测试（GEP-0024）

一个命令，两层：`gan test` 运行每个 `@example` 文档测试，然后运行 `tests/*.gan` 中的每个 `test_*` 函数——使用项目的完整模块解析进行编译，并由 pytest 执行（一次添加：`uv add --dev pytest`）。

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

ExUnit 表面：`test "name" do`（定义 `test_<slug>`），`describe`（为内部名称添加前缀），`assert`/`refute`（比较会报告两个操作数），以及 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk` 作为普通函数。`tests/` 永远不会打包——它位于源代码根目录之外。

## Python 互操作性

`$module` 是一等模块对象；`$` 清晰标记了互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # 点链：导入 importlib.metadata
$(PIL.Image).open(f)                # $(...) 显式锁定模块边界
$(sys).stderr                       # ...单段也适用：import sys，属性 stderr
pyimport numpy, as: np              # 别名导入
pyimport sys                        # 裸导入将 `sys` 绑定为普通名称
np.array([1, 2]) * 10               # 运算符广播——它仍然是 Python
sys.stderr.write("...")             # 裸名称链没有导入歧义
$json.dumps(data, indent: 2)        # 尾部关键字变为关键字参数
```

何时使用何种写法：

| 情况 | 写法 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性引用且链式启发式猜测错误时 | `$(os.path).sep`、`$(sys).stderr` |
| 在文件中多次使用模块 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 频繁使用的深层属性 | `@environ $(os).environ` 模块属性 |

同一文件中反复出现 `$(...)` 写法是代码异味——应声明 `pyimport`。切勿为 Python API 编写包装器模块；零包装器正是设计意图。

装饰器通过 `@decorate` 附加；模块属性持有导入时状态：

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

`try` 是一个表达式。`try` 内部的尾调用*不*会被优化（框架必须为处理程序存活）——`recur` 在那里是编译错误，而不是静默的栈消耗器。

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

结构体类型在规格说明中表示为 `App.User.t()`。

## 宏

编译时、卫生的、类似Elixir的宏。宏在编译器内部的确定性沙箱中运行，不留下任何运行时痕迹。

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

模板变量在每次展开时被重命名（卫生）；`var!(name)` 有意地访问调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的*展开宏*命令检查结果。

## 符咒与嵌入语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\\d+/                     # re.compile("\\d+") — string escapes apply
$python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

小写的嵌入语言符咒携带整个文档，通过 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`~prompt` 是 AI 散文的专用名称**（GEP-0009-R006）：主体是原始文本，因此引号、花括号、反斜杠和内联 JSON 无需转义 — 再也不用与 `\\\"` 抗争了：

```elixir
@task ~prompt(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~prompt"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

**`%json` 是编译期数据字面量**（GEP-0009-R007）：主体是 JSON(C) — 允许注释和尾随逗号 — 由编译器解析并作为纯数据输出，因此 OpenAI 工具模式可以原样粘贴，而拼写错误会变成编译错误，而不是运行时意外：

```elixir
@tools %json"""
[{"type": "function",
  "function": {"name": "ping", "description": "Health check.",
               "parameters": {"type": "object", "properties": {}}}}]
"""
```

`$python` 是 Python 专属用法的逃生门；拼接部分是编译后的 Gandora 表达式，其余部分原样传递。

## Compiler lints — warnings are provable facts

Each fires only on something statically certain, lands on the
definition line in `gan build`/editor squiggles, and has a
mechanical remedy (often a one-click quick fix):

| 警告 | 含义 | 修复方法 |
| --- | --- | --- |
| undefined variable | 读取未绑定的变量——保证引发 `NameError` | 修正名称 |
| unused binding | 已绑定，但从未读取 | `_` 前缀（`_meta`） |
| unreachable clause | 无守卫的全变量头覆盖了后续同参数数量的分支（也包括 `case` 通配符） | 重新排序或删除 |
| discarded comprehension | 处于语句位置的 `for` | 使用 `Enum.each` |
| unused `defp` | 无用的私有函数 | 删除，或 `@allow :unused_function` |
| stack recursion | 自递归，从未处于尾位置 | 累加器形式、`recur`，或 `@allow :stack_recursion` |

`@allow` 目标会被检查——拼写错误会导致编译错误。将警告视为缺陷：代码库标准为零。

## 项目与 CLI

`gandora.jsonc` 配置编译器（`source`, `outDir`, `targetPython`, `exclude`, `package`, `pyPackage`）；`pyproject.toml` + `uv` 管理依赖和 `.venv`；`gandora.local.jsonc` 保存每个开发者的偏好设置，并排除在 git 之外。

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

包以普通的 wheel 格式发布（`gan build && uv build && uv publish`），包含编译后的 Python、一个 `gandora.toml` 标记以及宏展开所需的 `.gan` 源文件——消费者通过 `uv add` 添加它们，无需引入 Gandora 运行时。

## 人工智能工具箱：`gan lsc`

每个语言事实都是标准输出上的一个 JSON 值——专为 agent 设计：

```console
gan lsc check --root .                  # 裁决：{diagnostics, suggestions}
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格说明、文档（所有语言环境）、参数、tco 形状
gan lsc doc for --root .                # 语言构造同样回答（for/recur/with...）
gan lsc references Stats.mean --root .  # 每个调用点（+ 定义）
gan lsc wsymbols mean --root .          # 项目范围的符号搜索
gan lsc symbols Stats --root .          # 单个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python 代码（文本形式）
gan lsc expand src/x.gan --root .       # 宏展开后的带引号 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取的 Python 端文档
```

**`gan build` 是最终裁决**（GEP-0025 rev 3）：编译器诊断、
Advisor 建议（实践缺陷、跨语言迁移提示、拼写错误名称的“你是不是想找”），
以及 **产物验证**——生成的 Python 使用解析规则（ty）进行类型检查，
因此未定义的函数、死导入或缺失的模块成员是构建 **错误**，而非运行时意外。
错误会阻止产物生成，这是重型编译器的方式；警告和建议会打印并允许继续。
`gan lsc check` 返回相同的裁决，作为一个 JSON 对象。

```console
gan lsc check --root .
# {"ok": true, "clean": false, "diagnostics": [...],
#  "suggestions": [{"kind": "did_you_mean", "line": 3,
#   "message": "`Enum.mpa` 不是 Enum 的函数——你是不是想找 `Enum.map`？", ...}]}
```

裁决以交通灯开头：`ok`（编译成功——无错误）
和 `clean`（ok **且** 零警告 **且** 零建议）。
红色 → 修复错误；黄色 → 阅读建议；绿色 → 提交。
建议携带其首个证据的行号，多个文件中相同的发现会合并为一个带注释的条目，
覆盖范围包括 `src/` 以及顶层 `tests/*.gan`（测试模块会获得迁移和惯用法提示，
但免于库注释覆盖率检查）。

对于 agent 而言，高效的工作循环是：**编写 → `gan build`（修复所有发现）→ `gan test` → 发布**。
当不确定某个内容生成什么时，使用 `gan lsc compile file`；当不确定语法时，
使用 `gan lsc doc <construct>`（`for`、`spec`、`test`……）。

## Agent 的代码风格检查清单

1. 每个公开的 `def`：`@doc` + `@spec`（+ 每个参数对应的 `@param`）；对于任何有有趣行为的内容使用 `@example`——`gan test` 会确保它们保持正确。
2. 类型规约：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正泛型的流程；结构体使用 `Mod.t()`；在 Python 边界处使用 `$mod.Type()`。
3. 迭代：映射形状的工作使用 `for`/`Enum`；无界循环使用累加器尾递归（当需要恒定栈时使用 `recur`）；仅对结构有界的递归使用 `@allow :stack_recursion`，且原因必须显而易见。
4. 互操作：一次性使用 `$`，重复使用使用 `pyimport`，不包装。
5. 零警告，`gan fmt` 整洁，doctest 通过——工具链、标准库、教程以及 playground 均遵守此规则；请遵循它们。

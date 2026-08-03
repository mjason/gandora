# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

编写 Gandora 的实用指南——面向人类，也特意面向 AI 智能体。规范性定义见 GEP（[`geps/`](../geps/)）；本手册展示各组件如何使用，以及在存在多种写法时应优先选用哪种。每一种构造都在 [`examples/tour`](../examples/tour) 中得到了实践（其已签入的 [`generated/`](../examples/tour/generated/) 展示了各章节编译成的精确 Python 代码），并在游乐场的自检套件中经受了实战检验。

下列基本规则贯穿始终：

- **零运行时。** 生成的 Python 代码自包含且可读——如同审阅者手写的内容。辅助函数按模块内联；部署从不依赖 Gandora。
- **Elixir 表层，Python 语义之下。** 若 Elixir 中存在某种构造，Gandora 便以 Elixir 的方式拼写它；值则为普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非观点；悬停显示递归如何编译；`gan lsc` 将所有事实以 JSON 形式提供。

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

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔型内建函数：`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`、比较运算符、`and or not`、算术运算符。

## 数据

原子（Atom）是内部化字符串且为**纯数据**——它们从不命名模块（那是 `$module` 的职责）。只有 `false` 和 `nil` 为假值；`0`、`""` 和 `[]` 为真值（采用 Elixir 语义，而非 Python 的）。

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

`=`、`case`、函数头、`with` 和 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、固定（`^x`——匹配 x 的*现有*值）以及结构体。

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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新的绑定——在两者之间创建的闭包保留旧值（见下文）。

## Functions as values

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

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 完全一致——后续的重新绑定、尾递归循环迭代或推导式步骤永远不会泄露到早期创建的闭包中。编译器通过 Python 自身的惯用法（`lambda x, *, n=n: x + n`）来实现这一点；调用元数保持严格。

## Pipelines

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

当管道以 `|>` 开头时，可以延续到下一行。

## 迭代：推导式和递归

没有 `loop` 和 `while`。迭代通过 `for`、`Enum` 家族或递归完成——编译器使递归安全。

### `for` 推导式（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被跳过
```

主体是一个表达式；`into: %{}` 需要 `{key, value}` 元组形式的主体。推导式**构建一个集合**——将其用于副作用会触发编译器警告；应改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

对尾位置上的封闭函数的调用会成为 `while True:` 内部的参数重新绑定——在任何深度下栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万个栈帧也没问题
```

`recur(args)` 是同一跳转的**受检**写法：它必须在尾位置且匹配某个子句的元数，否则构建失败——当恒定的栈空间是需求而非期望时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用，并使用 Python 栈（约 1000 帧）；编译器在定义处**警告**。结构递归——深度由数据本身限定，如树遍历——是合理的：确认后，警告消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态可见：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 打印它，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

`@spec` 声明函数的类型；编译器根据子句对其进行验证，并在生成的 Python 中发出 **PEP 484 注解**，从而 `pyright`/`mypy` 可以检查调用方，悬停显示真实类型。每个定义组一个 `@spec`，放在第一个子句之前与其他注解一起。

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

具体容器表示“正好是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为像它的东西”——在**参数**中优先使用它们，以便调用方可以传递元组、范围或生成器，并且因为 `list` 在 Python 的类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — will be walked once
sequence(t)                # collections.abc.Sequence[t]  — indexed / re-walked
mapping(k, v)              # collections.abc.Mapping[k, v] — read-only dict-like
keyword()                  # a keyword list: list[tuple[str, object]]
```

经验法则：**抽象输入，具体输出**——接受 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个**一到两个字符**的裸小写名称是一个类型变量；每个变量在输出中成为模块级别的 `typing.TypeVar`。相同的字母在规范中表示“相同类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸名称是一个错误，并带有提示（“您是指 `name()` 吗？”），因此拼写错误的 `intger` 不会无声地变成泛型。

### 命名参数

`name :: type` 在规范中命名参数——自文档化，并且签名帮助会显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 出现；括号参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str], with the import emitted
```

### 结构体类型

`Mod.t()` 是由该模块中的 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### 与默认值和多参数数量的交互

一个规范覆盖整个组；为完整参数列表（包括默认值）编写它。生成的委托和调度器携带注解。`@spec` 命名一个不存在的函数或参数数量会导致编译错误，因此规范不会无声地腐烂。

## 文档与文档测试

在 `def` 之前的注解顺序：任何 `@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow` — 所有这些都会累积到下一个定义上。

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

- `@param` 名称必须与子句头部变量匹配 — 在编译时验证。
- `@example` 是唯一的文档测试通道：`gan test` 会将 `gan>` 行编译为原生 Python 文档测试并运行。预期输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子打印为 `'ok'`，映射打印为 `{'k': 1}`，布尔值打印为 `True`。
- 标准：**每个公开的 `def` 都应包含 `@doc` + `@spec`**（当有参数时还需包含 `@param`）；面向用户的接口则添加 `_trans` 对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，已被 git 忽略，位于 `gandora.jsonc` 旁边）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 仅默认语言。`gan doc Mod.fun --locale zh-CN` 会明确指定；`gan lsc doc` 始终返回所有语言环境作为 JSON。

## 测试 (GEP-0024)

一个命令，两层结构：`gan test` 会运行每个 `@example` 文档测试，
然后运行 `tests/*.gan` 中每个 `test_*` 函数——这些函数会使用项目的完整模块解析进行编译，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

```elixir
# tests/test_stats.gan
defmodule TestStats do
  @moduledoc "Edge cases the doctests don't cover."

  import Test

  def test_mean(), do: Test.assert_eq(Stats.mean([1, 2, 3, 4]), 2.5)
  def test_missing_key_is_nil(), do: Test.assert_nil(Map.get(%{}, "x"))
  def test_fetch_raises() do
    _ = Test.assert_raises(fn -> Map.fetch!(%{}, "nope") end)
    nil
  end
end
```

`Test` (std) 提供 `assert_eq / assert_true / assert_false / assert_nil / assert_raises / assert_contains`。`tests/` 目录永远不会随项目发布——它位于源根目录之外。

## Python 互操作

`$module` 是一等模块对象；`$` 清晰地标记了互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # 点式链：import importlib.metadata
$(PIL.Image).open(f)                # $(...) 显式锁定模块边界
$(sys).stderr                       # ...单段也一样：import sys，属性 stderr
pyimport numpy, as: np              # 别名导入
pyimport sys                        # 裸导入将 `sys` 绑定为普通名称
np.array([1, 2]) * 10               # 运算符广播——它就是 Python
sys.stderr.write("...")             # 裸名称链无导入歧义
$json.dumps(data, indent: 2)        # 尾部关键字变为 kwargs
```

何时使用何种写法：

| 场景 | 写法 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 当链式启发式猜测错误时的一次性使用 | `$(os.path).sep`, `$(sys).stderr` |
| 在文件中重复使用的模块 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 经常使用的深层属性 | `@environ $(os).environ` 模块属性 |

在一个文件中重复使用 `$(...)` 写法是一种代码异味——应声明 `pyimport`。永远不要为 Python API 编写包装模块；没有包装模块正是设计所在。

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
  e -> {:error, to_string(e)}      # bare variable: any Exception
after
  cleanup()                        # always runs, contributes no value
end

raise "message"                    # -> raise RuntimeError("message")
```

`try` 是一个表达式。`try` 内部的尾调用*不会*被优化（帧必须为处理程序保留）——在那里使用 `recur` 会导致编译错误，而不是悄无声息地消耗栈。

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

编译时、卫生、Elixir 风格。宏在编译器内的确定性沙箱中运行，不留下任何运行时痕迹。

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

模板变量按每次展开重命名（卫生）；`var!(name)` 有意地访问调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *展开宏* 命令检查结果。

## 符号与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # 字符串，自由选择分隔符
~r/\\d+/                     # re.compile("\\d+") — 字符串转义生效
~python(sum(i*i for i in range(n)))   # 一个逐字传递的 Python 表达式
```

无大写的嵌入式语言符号通过 `<%= expr %>` 拼接将整个文档传递回 Gandora（编辑器会高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

`~python` 是仅用于 Python 拼写的转义出口；拼接部分是编译后的 Gandora 表达式，其余所有内容逐字传递。

## 编译器 lint — 警告是可证明的事实

每条 lint 仅针对静态确定的情况触发，定位到 `gan check`/`gan build`/编辑器中的波浪线所指示的定义行，并具有机械化的修复方法（通常是一键快速修复）：

| 警告 | 含义 | 修复方法 |
| --- | --- | --- |
| undefined variable | 读取未绑定的变量 — 保证会引发 `NameError` | 修正名称 |
| unused binding | 已绑定但从未读取 | 添加 `_` 前缀（`_meta`） |
| unreachable clause | 无守卫的全变量头部覆盖了后面同元数的子句（也包括 `case` 的通配符） | 重新排序或删除 |
| discarded comprehension | `for` 出现在语句位置 | 改用 `Enum.each` |
| unused `defp` | 无用的私有函数 | 删除，或使用 `@allow :unused_function` |
| stack recursion | 自递归且从未处于尾位置 | 采用累加器形式、`recur`，或使用 `@allow :stack_recursion` |

`@allow` 的目标会被检查 — 拼写错误将导致编译错误。将警告视为缺陷：代码库的标准是零容忍。

## 项目与命令行界面

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理依赖与 `.venv`；`gandora.local.jsonc` 存放开发者个人偏好，不纳入 git 管理。

```console
gan init my-app          # 新建项目         gan check      # 仅分析 + 代码检查
gan run src/main.gan     # 编译并执行       gan build      # 编译到 outDir
gan test                 # 运行 @example 文档测试
gan fmt src              # 原地格式化       gan fmt --check src   # CI 检查
gan fmt --diff src       # 显示差异       echo ... | gan fmt -  # 标准输入 -> 标准输出
gan doc Enum.take        # 文档（+ --locale）   gan repl       # 交互式环境
gan expand src/x.gan     # 宏展开输出       gan try <file|->  # 沙箱模式
gan init --package name
```

包以标准 wheel 格式发布（`gan build && uv build && uv publish`），其中包含编译后的 Python 代码、一个 `gandora.toml` 标记文件以及宏展开所需的 `.gan` 源文件——消费者使用 `uv add` 添加它们，无需引入 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一个 JSON 值——为智能体构建：

```console
gan lsc check --root .                  # 全项目诊断，包含 lint
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格说明、文档（所有语言区域）、参数、tco 形状
gan lsc doc for --root .                # 语言构造也能回答（for/recur/with...）
gan lsc references Stats.mean --root .  # 每个调用位置（+ 定义）
gan lsc wsymbols mean --root .          # 项目范围的符号搜索
gan lsc symbols Stats --root .          # 单个模块的大纲
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python，纯文本形式
gan lsc expand src/x.gan --root .       # 宏展开后的引用 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取的 Python 端文档
```

以及**沙盒**——在生成的代码接触项目之前对其进行验证（GEP-0023）：

```console
echo 'Enum.mpa([1,2], fn x -> x end)' | gan try -
# -> {"ok": false, ..., "suggestions": [{"kind": "did_you_mean",
#     "message": "`Enum.mpa` is not a function of Enum — did you mean `Enum.map`?"}]}
```

`try` 会编译、lint、对真实符号进行拼写检查（编辑距离），标记跨语言习惯（`return`、`lambda`、`None`、Python `def ...():` ……）并给出 Gandora 拼写，然后在临时目录中设置超时运行——返回生成的 Python、stdout 以及代码片段的最后一个值。`--no-run` 跳过执行。

智能体的高效循环：**生成 → `gan try -` → 应用建议 → 再次 `try` → 写入项目** → 然后 `gan lsc check`（修复所有发现的问题）→ `gan test` → `gan fmt src`。

## 代理的样式检查清单

1. 每个公开的 `def`：`@doc` + `@spec` (+ 每个参数对应 `@param`)；针对任何有趣的行为添加 `@example` —— `gan test` 使其保持诚实。
2. 规范：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正泛型的流程；结构体使用 `Mod.t()`；Python 边界使用 `$mod.Type()`。
3. 迭代：映射形状的工作使用 `for`/`Enum`；无界循环使用累加器尾递归（当需要常量栈时使用 `recur`）；仅对结构有界的递归使用 `@allow :stack_recursion`，且原因需显而易见。
4. 互操作：一次性使用 `$`，重复使用则用 `pyimport`，无需包装器。
5. 零警告，`gan fmt` 清洁，doctests 通过——工具链、标准库、教程以及交互式 playground 都遵守这一原则；请与之保持一致。

# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

面向人类以及（特意）面向 AI 代理编写 Gandora 的实用指南。规范性定义位于 GEP 中（[`geps/`](../geps/)）；本手册展示各个部件如何使用，以及当存在多种拼写时优先选用哪种。本手册中的每个构造都在 [`examples/tour`](../examples/tour) 中得到练习（该目录下已检入的 [`generated/`](../examples/tour/generated/) 展示了每章编译后的精确 Python 代码），并通过游乐场的自检套件进行了实战验证。

以下基本规则决定了所有内容：

- **零运行时。** 生成的 Python 代码是自包含且可读的——与审阅者手写的代码无异。辅助函数按模块内联；部署永远不依赖 Gandora。
- **Elixir 表面，Python 语义底层。** 当 Elixir 有某个构造时，Gandora 以 Elixir 方式拼写它；值则是普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非意见；悬停显示递归如何编译；`gan lsc` 将所有事实以 JSON 形式提供。

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

`=`, `case`, 函数头, `with`, 和 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、固定（`^x` — 匹配 x 的*现有*值），以及结构体。

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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新的绑定 — 在两者之间创建的闭包会保留旧值（见下文）。

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

**闭包在创建时按值捕获**（GEP-0021），正如在 Elixir 中一样——后续的重绑定、尾递归循环迭代或推导步骤永远不会泄漏到之前创建的闭包中。编译器通过 Python 自身的惯用法实现这一点（`lambda x, *, n=n: x + n`）；调用参数数量保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

管道可以继续到下一行，只要下一行以 `|>` 开头。

## 迭代：推导式与递归

没有 `loop` 也没有 `while`。迭代通过 `for`、`Enum` 家族或递归实现——而编译器会确保递归的安全性。

### `for` 推导式（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被跳过
```

主体是一个表达式；`into: %{}` 需要主体为 `{key, value}` 元组。
推导式用于**构建集合**——若将其用于副作用，编译器会发出警告；应改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

对包含函数在尾位置的调用，会变为 `while True:` 内部的参数重绑定——无论深度如何，栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万帧也没问题
```

`recur(args)` 是同一跳转的**受检**写法：它必须位于尾位置，且与某个子句的元数匹配，否则构建失败——当你需要恒定的栈空间（而非依赖希望）时，请使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保留为真实调用，并使用 Python 栈（约 1000 帧）；编译器会在定义处**发出警告**。
结构递归——深度受数据限制，如树遍历——是合理的：通过显式声明可消除警告：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态均可查看：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 会打印该信息，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

`@spec` 声明函数的类型；编译器会根据子句对其进行验证，并在生成的 Python 中生成 **PEP 484 注解**，以便 `pyright`/`mypy` 检查调用者，悬停时显示真实类型。每个定义组一个 `@spec`，与其他注解一起放在第一个子句之前。

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

具体容器声明“正是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器声明“任何行为类似的对象”——在**参数**中优先使用它们，这样调用者可以传入元组、范围或生成器，并且因为 Python 类型系统中 `list` 是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  —— 将被遍历一次
sequence(t)                # collections.abc.Sequence[t]  —— 可索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] —— 只读的类字典结构
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**抽象输入，具体输出**——接受 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个**一或两个字符**的裸小写名称是类型变量；每个变量会在输出中成为模块级别的 `typing.TypeVar`。相同的字母在 spec 中表示“相同的类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸名称会报错，并附带提示（“你是想写 `name()` 吗？”），因此拼写错误的 `intger` 不会悄无声息地变成泛型。

### 命名参数

`name :: type` 可以在 spec 中命名参数——既自文档化，也用于签名帮助显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 使用；圆括号可以参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并生成对应的导入语句
```

### 结构类型

`Mod.t()` 是由该模块中 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### 与默认参数及多参数数的交互

一个 spec 覆盖整个组；请为完整的参数列表（包括默认值）编写 spec。生成的委托和分发器会携带这些注解。如果 `@spec` 指定了一个不存在的函数或参数数，会编译报错，因此 spec 不会默默腐烂。

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

- `@param` 的名称必须与子句头部的变量名匹配 —— 在编译时验证。
- `@example` 是唯一的文档测试通道：`gan test` 会将 `gan>` 行编译成原生 Python 文档测试并执行。期望输出为 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子打印为 `'ok'`，映射打印为 `{'k': 1}`，布尔值打印为 `True`。
- 标准：**每个公开的 `def` 都带有 `@doc` + `@spec`**（如果有参数，则再加上 `@param`）；面向用户的表面会增加 `_trans` 配对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，位于 `.gitignore` 中，与 `gandora.jsonc` 同级）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 默认语言。`gan doc Mod.fun --locale zh-CN` 显式指定语言；`gan lsc doc` 始终以 JSON 格式返回所有语言。

## Python 互操作

`$module` 是一个一等模块对象；`$` 显式标记互操作边界。

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
| 链式启发式猜测错误时的一次性引用 | `$(os.path).sep`, `$(sys).stderr` |
| 在文件中反复使用的模块 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 经常使用的深层属性 | `@environ $(os).environ` 模块属性 |

在同一个文件中重复使用 `$(...)` 拼写是一种不良模式——应声明一个 `pyimport`。绝不要为 Python API 编写包装模块；不编写包装模块正是其设计所在。

装饰器通过 `@decorate` 附加；模块属性持有导入时的状态：

```elixir
@app $fastapi.FastAPI(title: "API")

@decorate @app.get("/")
def root(), do: %{hello: "world"}

@decorate $functools.lru_cache(maxsize: 64)
def fib(n), do: ...
```

生成的模块是一个标准的 ASGI 目标：`uvicorn app.api:app --app-dir dist`。

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

`try` 是一个表达式。`try` 内部的尾调用**不会**被优化（栈帧必须保留以供处理程序使用）——在其中使用 `recur` 会导致编译错误，而非静默消耗栈空间。

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

编译时、卫生的、类似 Elixir 的。宏在编译器的确定性沙箱中运行，不留下任何运行时痕迹。

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

模板变量在每次展开时被重命名（卫生性）；`var!(name)` 有意触及调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *Expand Macros* 命令检查结果。

## 符号与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\\d+/                     # re.compile("\\d+") — string escapes apply
~python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

小写形式的嵌入式语言符号可以携带整个文档，并通过 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内嵌语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

`~python` 是用于仅 Python 拼写的逃生出口；拼接部分会被编译为 Gandora 表达式，其余部分则原样传递。

## Compiler lints — warnings are provable facts

Each fires only on something statically certain, lands on the
definition line in `gan check`/`gan build`/editor squiggles, and has a
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

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理依赖和 `.venv`；`gandora.local.jsonc` 存放开发者个人偏好，不纳入 git 版本控制。

```console
gan init my-app          # 新建项目        gan check      # 仅分析和 lint
gan run src/main.gan     # 编译并执行      gan build      # 编译到 outDir
gan test                 # 运行 @example doctests
gan fmt src              # 原地格式化      gan fmt --check src   # CI 检查
gan fmt --diff src       # 显示差异        echo ... | gan fmt -  # 标准输入 -> 标准输出
gan doc Enum.take        # 文档（+ --locale）  gan repl       # 交互式 REPL
gan expand src/x.gan     # 宏展开输出      gan init --package name
```

包以普通 wheel 格式发布（`gan build && uv build && uv publish`），携带编译后的 Python 代码、一个 `gandora.toml` 标记文件以及宏展开所需的 `.gan` 源文件——消费者使用 `uv add` 添加它们，无需引入 Gandora 运行时。

## AI 工具集：`gan lsc`

每个语言事实都是 stdout 上的一条 JSON 值——专为智能体构建：

```console
gan lsc check --root .                  # 全项目诊断，包含 lint
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格、说明（所有语言环境）、参数、tco 形状
gan lsc references Stats.mean --root .  # 每个调用点（+ 定义）
gan lsc wsymbols mean --root .          # 项目范围内的符号搜索
gan lsc symbols Stats --root .          # 单个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python，以文本形式输出
gan lsc expand src/x.gan --root .       # 宏展开后的引号 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取 Python 侧文档
```

以及**沙箱**——在生成的代码触及项目之前对其进行验证（GEP-0023）：

```console
echo 'Enum.mpa([1,2], fn x -> x end)' | gan lsc try - --root .
# -> {"ok": false, ..., "suggestions": [{"kind": "did_you_mean",
#     "message": "`Enum.mpa` 不是 Enum 的函数——你是否想用 `Enum.map`？"}]}
```

`try` 会编译、lint、对名称与真实符号进行拼写检查（编辑距离）、标记跨语言习惯（`return`、`lambda`、`None`、Python `def ...():` ……）并给出 Gandora 拼写，然后在临时目录中带超时运行——返回生成的 Python、stdout 以及代码片段的最后一个值。`--no-run` 跳过执行。

一个高效的智能体循环：**生成 → `gan lsc try -` → 应用建议 → 再次 `try` → 写入项目** → 然后 `gan lsc check`（修复所有发现）→ `gan test` → `gan fmt src`。

## 代理风格检查清单

1. 每个公开的 `def`：`@doc` + `@spec`（+ 每个参数的 `@param`）；对于任何具有有趣行为的内容，添加 `@example` —— `gan test` 会确保它们保持正确。
2. 规格：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正通用的流程；结构体使用 `Mod.t()`；Python 边界处使用 `$mod.Type()`。
3. 迭代：`for`/`Enum` 用于映射形状的工作；累加器尾递归用于无界循环（当需要恒定栈时使用 `recur`）；仅对结构边界递归使用 `@allow :stack_recursion`，且原因必须显而易见。
4. 互操作：`$` 一次性使用，重复使用采用 `pyimport`，无需包装器。
5. 零警告，`gan fmt` 清理干净，doctest 通过 —— 工具链、标准库、教程和 playground 均保持此标准；与之保持一致。

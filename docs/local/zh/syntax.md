# The Gandora Language Manual

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

Gandora 的实用编写指南——面向人类，并且有意地也面向 AI 代理。规范性定义位于 GEPs 中（[`geps/`](../geps/)）；本手册展示各组件如何使用，以及当存在多种拼写时优先选择哪种。这里的每个构造都通过 [`examples/tour`](../examples/tour) 进行练习（其已签入的 [`generated/`](../examples/tour/generated/) 展示了每个章节编译成的确切 Python 代码），并经过游乐场自检套件的实战检验。

以下基本原则决定了所有内容：

- **零运行时。** 生成的 Python 代码是自包含且可读的——就像审阅者手写的一样。辅助函数按模块内联；部署从不依赖 Gandora。
- **Elixir 表面，Python 语义为底层。** 当 Elixir 有某个构造时，Gandora 以 Elixir 方式拼写；值则是普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非观点；悬停显示递归如何编译；`gan lsc` 将每个事实以 JSON 格式提供。

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

`=`, `case`, 函数头, `with` 和 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、pin（`^x` — 匹配 x 的*现有*值）以及结构体。

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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新的绑定——在其中创建的闭包会保留旧的值（见下文）。

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

**闭包在创建时按值捕获** (GEP-0021)，完全如 Elixir 中一样——后续的重新绑定、尾递归循环迭代或推导式步骤永远不会泄漏到之前创建的闭包中。编译器通过 Python 自身的惯用法 (`lambda x, *, n=n: x + n`) 实现这一点；调用元数保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

当管道以 `|>` 开始时，可以在下一行继续。

## 迭代：推导式与递归

没有 `loop` 和 `while`。迭代通过 `for`、`Enum` 系列或递归实现——编译器确保递归安全。

### `for` 推导式（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # multiple generators
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # dict comprehension
for {k, v} <- [{"a", 1}, :bad], do: k          # non-matching elements are SKIPPED
```

主体是一个表达式；`into: %{}` 需要 `{key, value}` 元组形式的主体。推导式**构建一个集合**——将其用于副作用会触发编译器警告；应改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

对尾位置中封闭函数的调用变为 `while True:` 内部的参数重绑定——在任何深度下栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # a million frames is fine
```

`recur(args)` 是同一跳转的**检查式**拼写：它必须位于尾位置且匹配某个子句的元数，否则构建失败——当恒定栈是要求而非期望时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用并使用 Python 栈（约 1000 帧）；编译器在定义处发出**警告**。结构递归——深度受数据约束，如树遍历——是合理的：通过声明使其通过，警告消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态可见：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 打印它，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

`@spec` 声明函数的类型；编译器会根据函数子句对其验证，并在生成的 Python 中生成 **PEP 484 注解**，因此 `pyright`/`mypy` 可检查调用方，悬停时显示真实类型。每个定义组只能有一个 `@spec`，放在第一个子句之前与其他注解一起。

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

具体容器表示“恰好是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为类似该类型的东西”——在**参数**中优先使用它们，这样调用方可以传入元组、区间或生成器，同时因为 Python 类型系统中 `list` 是不变的而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  —— 只能遍历一次
sequence(t)                # collections.abc.Sequence[t]  —— 支持索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] —— 只读类字典
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**抽象入，具体出**——接收 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

**一或两个字符**的裸小写名称是类型变量；每个变量在输出中成为模块级别的 `typing.TypeVar`。相同的字母在 spec 中表示“相同类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

较长的裸名称会报错并附带提示（“你是不是想写 `name()`？”），因此拼写错误的 `intger` 不会静默地成为泛型。

### 命名参数

`name :: type` 为 spec 中的参数命名——用于自我文档化，签名帮助也会显示该名称：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 出现；括号内可参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并自动生成导入语句
```

### 结构体类型

`Mod.t()` 表示该模块中由 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数数量的交互

一个 spec 覆盖整个定义组；请针对完整参数列表（包含默认值）编写 spec。生成的委托函数和调度器会携带这些注解。若 `@spec` 指向了不存在的函数或参数数量，则会产生编译错误，因此 spec 不会静默地腐烂。

## 文档与文档测试

在 `def` 之前的注解顺序：`@doc`、`@doc_trans`、
`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow`——
它们都会累积到下一个定义上。

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

- `@param` 的名称必须与子句头部变量匹配——在编译时验证。
- `@example` 是唯一的文档测试通道：`gan test` 会将 `gan>` 行编译成原生 Python 文档测试并运行。预期输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子显示为 `'ok'`，映射显示为 `{'k': 1}`，布尔值显示为 `True`。
- 标准：**每个公开的 `def` 都必须带有 `@doc` + `@spec`**（有参数时还需加上 `@param`）；面向用户的部分应添加 `_trans` 配对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，位于 `.gitignore` 中，与 `gandora.jsonc` 相邻）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 默认语言。`gan doc Mod.fun --locale zh-CN` 明确指定语言；`gan lsc doc` 始终以 JSON 格式返回所有语言环境。

## Python 互操作

`$module` 是一等模块对象；`$` 在视觉上标明了互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # 点链：导入 importlib.metadata
$(PIL.Image).open(f)                # $(...) 明确定义模块边界
$(sys).stderr                       # ...单段亦可：import sys, 属性 stderr
pyimport numpy, as: np              # 别名导入
pyimport sys                        # 裸导入将 `sys` 绑定为普通名称
np.array([1, 2]) * 10               # 运算符广播——它仍是 Python
sys.stderr.write("...")             # 裸名称链无导入歧义
$json.dumps(data, indent: 2)        # 尾部关键字变为关键字参数
```

何时使用何种写法：

| 情景 | 写法 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性引用，但链式启发式猜测错误 | `$(os.path).sep`、`$(sys).stderr` |
| 在文件中重复使用模块 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 经常使用的深层属性 | `@environ $(os).environ` 模块属性 |

在同一个文件中重复使用 `$(...)` 写法是一种不良信号——应声明 `pyimport`。切勿为 Python API 编写包装模块；无包装正是此设计原则。

装饰器通过 `@decorate` 附加；模块属性保存导入时的状态：

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

`try` 是一个表达式。`try` 内部的尾调用*不*会被优化（帧必须为处理程序保留）——在 `try` 中使用 `recur` 会导致编译错误，而非静默的栈耗尽。

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

## Macros

编译时、卫生的、类Elixir的宏。宏在编译器内部的确定性沙箱中运行，不留运行时痕迹。

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

模板变量每次展开都会重命名（卫生性）；`var!(name)` 有意地访问调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *Expand Macros* 命令检查结果。

## 文字符号与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # 字符串，自由分隔符选择
~r/\\d+/                     # re.compile("\\d+") — 字符串转义生效
~python(sum(i*i for i in range(n)))   # 一条逐字的 Python 表达式
```

无大写字母的嵌入式语言符号可携带整个文档，并支持 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

`~python` 是专用于 Python 语法的逃生舱；拼接部分是被编译的 Gandora 表达式，其余部分原样传递。

## Compiler lints — warnings are provable facts

Each fires only on something statically certain, lands on the
definition line in `gan check`/`gan build`/editor squiggles, and has a
mechanical remedy (often a one-click quick fix):

| 警告 | 含义 | 修复方法 |
| --- | --- | --- |
| undefined variable | 读取未绑定的变量 — 保证会引发 `NameError` | 修正名称 |
| unused binding | 已绑定但从未读取 | 加 `_` 前缀（如 `_meta`） |
| unreachable clause | 一个无守卫的全变量头部遮蔽了后续相同元数的分支（也包括 `case` 的通配符） | 重新排序或删除 |
| discarded comprehension | 在语句位置的 `for` | 使用 `Enum.each` |
| unused `defp` | 私有函数未被使用 | 删除，或添加 `@allow :unused_function` |
| stack recursion | 自递归但从未在尾位置 | 改为累加器形式、`recur`，或添加 `@allow :stack_recursion` |

`@allow` 目标会被检查 — 拼写错误会导致编译错误。将警告视为缺陷：代码库标准为零。

## Projects and the CLI（项目与命令行界面）

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理自身依赖和 `.venv`；`gandora.local.jsonc` 存储开发者个人偏好，且不纳入 git 管理。

```console
gan init my-app          # new project        gan check      # analyze + lints only
gan run src/main.gan     # compile + execute  gan build      # compile to outDir
gan test                 # run @example doctests
gan fmt src              # format in place    gan fmt --check src   # CI gate
gan fmt --diff src       # show the diff      echo ... | gan fmt -  # stdin -> stdout
gan doc Enum.take        # docs (+ --locale)  gan repl       # interactive
gan expand src/x.gan     # macro output       gan init --package name
```

包以普通 wheel 格式发布（`gan build && uv build && uv publish`），包含编译后的 Python 代码、一个 `gandora.toml` 标记文件，以及宏展开所需的 `.gan` 源文件——消费者通过 `uv add` 安装它们，且无需引入 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一个 JSON 值——专为智能体构建：

```console
gan lsc check --root .                  # whole-project diagnostics, lints included
gan lsc diagnostics src/x.gan --root .  # one file
gan lsc doc Enum.take --root .          # specs, prose (all locales), params, tco shape
gan lsc references Stats.mean --root .  # every call site (+ definitions)
gan lsc wsymbols mean --root .          # project-wide symbol search
gan lsc symbols Stats --root .          # one module's outline
gan lsc definition Stats.mean --root .  # where it's defined
gan lsc compile src/x.gan --root .      # the generated Python, as text
gan lsc expand src/x.gan --root .       # post-macro quoted AST
gan lsc ast src/x.gan --root .          # parse tree (Elixir encoding)
gan lsc pydoc numpy.array --root .      # Python-side docs via jedi
```

一个高效的智能体循环：编辑 → `gan lsc check`（修复所有发现）→ 当不确定某物生成什么时运行 `gan lsc compile` → `gan test` → `gan fmt src`。

## 代理风格检查清单

1. 每个公开的 `def`：`@doc` + `@spec` (+ 每个参数配 `@param`)；任何有有趣行为的代码都配上 `@example` —— `gan test` 能确保它们有效。
2. 类型规约：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正泛型的流程；结构体使用 `Mod.t()`；在 Python 边界处使用 `$mod.Type()`。
3. 迭代：映射形状的工作使用 `for`/`Enum`；无界循环使用累加器尾递归（当需要恒定栈时使用 `recur`）；`@allow :stack_recursion` 仅用于结构有界的递归，且原因必须显而易见。
4. 互操作：一次性使用 `$`，重复使用则用 `pyimport`，不编写包装器。
5. 零警告，`gan fmt` 整洁，文档测试通过 —— 工具链、标准库、教程、playground 都遵循此标准；请保持一致。

# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

编写 Gandora 的实用指南——面向人类，也特意面向 AI 智能体。规范性定义位于 GEP 中（[`geps/`](../geps/)）；本手册展示各部分的用法，以及在多个写法并存时应优先选用哪种拼写。此处每个构造都在 [`examples/tour`](../examples/tour) 中得到了实践（其签入的 [`generated/`](../examples/tour/generated/) 显示了每一章编译成的精确 Python 代码），并通过 playground 的自我检查套件进行了实战检验。

以下基本规则贯穿全文：

- **零运行时。** 生成的 Python 代码自包含且可读——就像审阅者手写的一样。辅助工具按模块内联；部署从不依赖 Gandora。
- **Elixir 表层，Python 语义底层。** 凡是 Elixir 有的构造，Gandora 就按 Elixir 的方式拼写；值则是普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非意见；悬停显示递归如何编译；`gan lsc` 以 JSON 格式提供每个事实。

## 模块与函数

每个文件一个 `defmodule`；模块名称必须与路径匹配
(`src/app/hello_web.gan` ↔ `App.HelloWeb` ↔ `app/hello_web.py`)。

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

原子是驻留字符串和**纯数据**——它们从不命名模块（那是`$module`的工作）。只有`false`和`nil`为假；`0`、`""`和`[]`为真（Elixir语义，而非Python的）。

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

`=`, `case`, 函数头, `with` 和 `for` 生成器都会匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、pin（`^x` — 匹配 *x 的现有值*）以及结构体。

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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新的绑定——期间创建的闭包会保留旧值（见下文）。

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

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 完全一致——后续的重新绑定、尾递归循环迭代或推导步骤永远不会泄漏到早期创建的闭包中。编译器通过 Python 自身的惯用法实现这一点（`lambda x, *, n=n: x + n`）；调用参数数量保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

管道可以在以 `|>` 开头时延续到下一行。

## 迭代：推导式和递归

没有 `loop` 和 `while`。迭代使用 `for`、`Enum` 系列或递归 —— 编译器使递归安全。

### `for` 推导式 (GEP-0020)

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被跳过
```

主体是一个表达式；`into: %{}` 需要一个 `{key, value}` 元组形式的主体。推导式**构建一个集合** —— 将其用于副作用会触发编译器警告；应改用 `Enum.each`。

### 尾递归编译为循环 (GEP-0019)

位于尾位置的封闭函数调用会变成 `while True:` 内部的参数重新绑定 —— 任意深度下栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万个帧也没问题
```

`recur(args)` 是同一跳转的**有检查**写法：它必须位于尾位置且与某个子句的参数数量匹配，否则构建失败 —— 当你需要恒定栈空间（而非仅希望如此）时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用并使用 Python 栈（约 1000 帧）；编译器会在定义处**发出警告**。结构递归 —— 深度由数据边界决定（如树遍历） —— 是合理的：声明后警告消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态都是可见的：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 打印该信息，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

`@spec` 声明函数的类型；编译器会将其与函数子句进行验证，并在生成的 Python 中输出 **PEP 484 类型注解**，从而让 `pyright`/`mypy` 检查调用方，且悬停提示显示真实类型。每个定义组只能有一个 `@spec`，放在第一个子句之前，与其他注解同位置。

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

### 容器——构造时具体，接收时抽象

具体容器表示“精确的 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为类似的东西”——建议在**参数**中优先使用，这样调用方可以传入元组、范围或生成器；同时因为 Python 类型系统中 `list` 是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 只会被遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 支持索引/可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读的类字典
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**入参抽象，出参具体**——接收 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个由**一或两个字符**组成的裸小写名称即为类型变量；每个变量在输出中会成为模块级别的 `typing.TypeVar`。同一字母在 spec 中表示“同一类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

较长的裸名称会报错，并提示“你是不是想用 `name()`？”，因此拼写错误的 `intger` 不会悄然成为泛型。

### 具名参数

`name :: type` 为 spec 中的参数命名——既可自文档化，也用于签名帮助显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型均可通过 `$module` 引入；括号可进行参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并在输出中自动生成导入
```

### 结构体类型

`Mod.t()` 表示该模块中由 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数数量的交互

一个 spec 覆盖整个函数组；应针对完整参数列表（包含默认值）编写。生成的委托函数和调度器会携带这些注解。若 `@spec` 命名了不存在的函数或参数数量，则会在编译时报错，因此 spec 不会悄然失效。

## 文档与文档测试（doctests）

`def` 之前的注解顺序：`@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow`——所有这些注解都会累积到下一个定义上。

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

- `@param` 的名称必须与子句头部的变量匹配——编译时验证。
- `@example` 是唯一的文档测试通道：`gan test` 会将 `gan>` 行编译为原生的 Python 文档测试并执行。期望的输出是 Python `repr`（即 `inspect/1` 显示的内容）：原子输出为 `'ok'`，映射输出为 `{'k': 1}`，布尔值输出为 `True`。
- 标准：**每个公开的 `def` 必须携带 `@doc` + `@spec`**（当有参数时还需携带 `@param`）；面向用户的表面需添加 `_trans` 对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，已加入 gitignore，位于 `gandora.jsonc` 旁）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 默认语言。`gan doc Mod.fun --locale zh-CN` 显式指定语言；`gan lsc doc` 始终以 JSON 格式返回所有语言。

## Python 互操作

`$module` 是一个一等模块对象；`$` 显式标记互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # 点链：导入 importlib.metadata
$(PIL.Image).open(f)                # $(...) 显式锁定模块边界
$(sys).stderr                       # ...单段也可：import sys，属性 stderr
pyimport numpy, as: np              # 别名导入
pyimport sys                        # 裸导入将 `sys` 绑定为普通名称
np.array([1, 2]) * 10               # 运算符广播——它只是 Python
sys.stderr.write("...")             # 裸名称链无导入歧义
$json.dumps(data, indent: 2)        # 尾部关键字变为关键字参数
```

何时使用哪种写法：

| 场景 | 写法 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性的链式启发猜测错误时 | `$(os.path).sep`、`$(sys).stderr` |
| 模块在文件中多次使用 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 深层属性经常使用 | `@environ $(os).environ` 模块属性 |

在一个文件中重复使用 `$(...)` 写法是一种坏味道——应声明一个 `pyimport`。永远不要为 Python API 编写包装模块；没有包装器正是设计所在。

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

`try/rescue/after` 映射到 Python 异常；rescue 子句按异常类匹配：

```elixir
try do
  risky()
rescue
  e in $builtins.ValueError -> {:bad_value, to_string(e)}
  e in $requests.HTTPError -> {:http, e.response.status_code}
  e -> {:error, to_string(e)}      # 裸变量：任何 Exception
after
  cleanup()                        # 始终执行，不贡献值
end

raise "message"                    # -> raise RuntimeError("message")
```

`try` 是一个表达式。`try` 内部的尾调用 **不会** 被优化（帧必须保留以供处理程序使用）—— 其中的 `recur` 会导致编译错误，而非静默地消耗栈空间。

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

编译时、卫生的、Elixir 风格的宏。宏在编译器内的确定性沙箱中运行，不会留下任何运行时痕迹。

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

模板变量在每次展开时都会重命名（卫生性）；`var!(name)` 可故意访问调用者的作用域；`unquote_splicing(list)` 用于拼接序列；`def unquote(head)` 用于构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *Expand Macros* 命令来检查展开结果。

## 符号与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\\d+/                     # re.compile("\\d+") — string escapes apply
~python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

无大写字母的嵌入式语言符号携带完整文档，通过 `<%= expr %>` 拼接回 Gandora（编辑器高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

`~python` 是用于纯 Python 拼写的转义出口；拼接部分是被编译的 Gandora 表达式，其余所有内容原样传递。

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

## 项目和 CLI

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理自身依赖和 `.venv`；`gandora.local.jsonc` 保存每个开发者的偏好设置，且不纳入 git 管理。

```console
gan init my-app          # 新项目        gan check      # 仅分析和 lint
gan run src/main.gan     # 编译并执行    gan build      # 编译到 outDir
gan test                 # 运行 @example 文档测试
gan fmt src              # 原地格式化    gan fmt --check src   # CI 门禁
gan fmt --diff src       # 显示差异      echo ... | gan fmt -  # 标准输入 -> 标准输出
gan doc Enum.take        # 文档（+ --locale）  gan repl       # 交互式
gan expand src/x.gan     # 宏输出        gan try <file|->  # 沙箱
gan init --package name
```

包以普通 wheel 形式发布（`gan build && uv build && uv publish`），包含编译后的 Python、一个 `gandora.toml` 标记文件以及宏展开所用的 `.gan` 源文件——使用者通过 `uv add` 引入它们，无需引入任何 Gandora 运行时。

## AI工具箱: `gan lsc`

每个语言事实都是标准输出上的一个JSON值——为代理而构建：

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

以及**沙箱**——验证生成的代码在接触项目之前 (GEP-0023)：

```console
echo 'Enum.mpa([1,2], fn x -> x end)' | gan try -
# -> {"ok": false, ..., "suggestions": [{"kind": "did_you_mean",
#     "message": "`Enum.mpa` is not a function of Enum — did you mean `Enum.map`?"}]}
```

`try` 编译、静态检查、对照真实符号进行拼写检查（编辑距离），标记跨语言习惯（`return`、`lambda`、`None`、Python `def ...():` …）使用 Gandora 拼写，然后在临时目录中在超时限制下运行——返回生成的 Python、标准输出和代码片段的最后一个值。`--no-run` 跳过执行。

一个对代理来说高效的工作循环：**生成 → `gan try -` → 应用建议 → 再次 `try` → 写入项目** → 然后 `gan lsc check`（修复所有发现的问题）→ `gan test` → `gan fmt src`。

## 智能体风格检查清单

1. 每个公开的 `def`：`@doc` + `@spec` (+ 每个参数对应的 `@param`)；对于任何有有趣行为的内容，加上 `@example` —— `gan test` 确保它们可靠。
2. 类型规范：输入使用抽象容器（`sequence`、`mapping`），输出使用具体容器；类型变量用于真正的通用流程；结构体使用 `Mod.t()`；Python 边界处使用 `$mod.Type()`。
3. 迭代：映射形态的工作使用 `for`/`Enum`；无限循环使用累加器尾递归（当需要恒定栈时使用 `recur`）；`@allow :stack_recursion` 仅用于结构有界递归，且原因必须显而易见。
4. 互操作：一次性使用用 `$`，重复使用用 `pyimport`，不要包装器。
5. 零警告，`gan fmt` 整洁，doctests 通过——工具链、标准库、教程和游乐场都遵循这一标准；请与之保持一致。

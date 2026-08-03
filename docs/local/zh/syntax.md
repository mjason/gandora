# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

编写 Gandora 的实用指南——面向人类，且有意为之，也面向 AI 智能体。规范性定义收录于 GEP（[`geps/`](../geps/)）中；本手册展示各部分的用法，并在存在多种写法时指明应优先选用哪一种。手册中的每个构造都在 [`examples/tour`](../examples/tour) 中得以实践（其已检入的 [`generated/`](../examples/tour/generated/) 目录展示了每章所编译出的精确 Python 代码），并通过沙盒的自检套件经过了实战检验。

以下基本原则贯穿全书：

- **零运行时。** 生成的 Python 代码自包含且可读——与审阅者手写的代码无异。辅助函数按模块内联；部署从不依赖 Gandora。
- **Elixir 语法，Python 语义于底层。** 凡 Elixir 中存在某个构造，Gandora 便以 Elixir 方式拼写；值则是普通的 Python 对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非观点；悬停提示显示递归是如何编译的；`gan lsc` 将每个事实以 JSON 形式提供。

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

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔型内建函数：`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`，比较操作符，`and or not`，算术运算。

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

`=`、`case`、函数头、`with` 以及 `for` 生成器都会匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、针（`^x` — 匹配 *现有* 值 x）以及结构体。

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "the pinned value"
  {:error, %{reason: r}} -> "failed: #{r}"
  _ -> "anything else"
end

cond do                       # 第一个为真的条件（非模式）
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

失败的 `=`/`case` 匹配将引发 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新绑定 —— 在此期间创建的闭包将保留旧值（见下文）。

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

**闭包在创建时按值捕获**（GEP-0021），与 Elixir 完全一致——后续的重新绑定、尾递归循环迭代或推导步骤绝不会泄露到先前创建的闭包中。编译器通过 Python 自身的惯用法实现这一点（`lambda x, *, n=n: x + n`）；调用元数保持严格。

## 管道

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

管道可以延续到下一行，只要该行以 `|>` 开头。

## 迭代：推导式与递归

没有 `loop` 和 `while`。迭代通过 `for`、`Enum` 族函数或递归实现——编译器确保递归安全。

### `for` 推导式（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被跳过
```

主体是一个表达式；`into: %{}` 要求主体返回 `{key, value}` 元组。推导式**构建一个集合**——若将其用于副作用，编译器会发出警告；请改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

对尾位置上的封闭函数的调用，会变成 `while True:` 内部的参数重新绑定——无论深度如何，栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 百万级帧数毫无问题
```

`recur(args)` 是**经过检查**的同一种跳转写法：它 MUST 位于尾位置并与某个子句的元数匹配，否则构建失败——在需要恒定栈（而非仅期望）时使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保留为真实调用，使用 Python 栈（约 1000 帧）；编译器会在定义处**发出警告**。结构递归——深度受数据限制，如树遍历——是合理的：通过显式声明即可消除警告：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态均可见：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 打印该信息，`gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

`@spec` 声明一个函数的类型；编译器会将其与子句进行校验，并在生成的 Python 中发出 **PEP 484 类型注解**，从而让 `pyright`/`mypy` 检查调用方，鼠标悬停时也会显示真实类型。每个定义组只能有一个 `@spec`，放在第一个子句之前的其他注解旁。

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

### 容器——构造时具体，接受时抽象

具体容器表示“就是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何表现类似它的东西”——在**参数**中优先使用它们，这样调用方可以传入元组、区间或生成器，而且因为 Python 类型系统中 `list` 是逆变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 只会被遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 支持索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读的类字典
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**接受时抽象，返回时具体**——接受 `sequence(t)`，返回 `list(t)`。

### 联合类型与 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个由**一到两个字符**组成的小写裸名称是一个类型变量；每个变量在输出中会成为模块级的 `typing.TypeVar`。相同的字母在规格中代表“相同的类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

较长的裸名称会报错，并附带提示（“你是想写 `name()` 吗？”），因此拼写错误的 `intger` 不会悄无声息地变成一个泛型。

### 命名参数

`name :: type` 为规格中的参数命名——既有自文档作用，也会在签名帮助中显示：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 出现；括号用于参数化：

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

### 与默认参数及多参数类型的交互

一个 `@spec` 覆盖整个定义组；请针对完整的参数列表（包括默认参数）编写。生成的委托函数和调度器会携带这些注解。如果 `@spec` 引用了一个不存在的函数或参数数量，编译时会报错，因此规格不会无声地腐烂。

## 文档与文档测试

`def` 之前的注解顺序：`@doc`、`@doc_trans`、`@param`、`@param_trans`、`@spec`、`@example`、`@decorate`、`@allow` —— 这些注解都会累积到下一个定义上。

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

- `@param` 的名称必须与子句头中的变量匹配 —— 在编译时验证。
- `@example` 是唯一的文档测试渠道：`gan test` 会将 `gan>` 行编译为原生 Python 文档测试并执行它们。期望的输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子打印为 `'ok'`，映射打印为 `{'k': 1}`，布尔值打印为 `True`。
- 标准：**每个公开的 `def` 都应带有 `@doc` + `@spec`**（如果有参数，还应带有 `@param`）；面向用户的接口需要添加 `_trans` 对应的注解对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，位于 `.gitignore` 中，与 `gandora.jsonc` 同级）> `GAN_DOC_LOCALE` 环境变量 / 编辑器设置 > 默认语言。`gan doc Mod.fun --locale zh-CN` 显式指定语言；`gan lsc doc` 始终以 JSON 格式返回所有语言。

## Testing (GEP-0024)

一个命令，两层：`gan test` 运行每个 `@example` 文档测试，然后运行 `tests/*.gan` 的每个 `test_*` 函数——这些函数使用项目的完整模块解析进行编译，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

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

ExUnit 表面：`test "name" do`（定义 `test_<slug>`），`describe`（为内部名称添加前缀），`assert`/`refute`（比较报告两个操作数），以及 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk` 作为普通函数。`tests/` 永远不会被分发——它位于源代码根目录之外。

## Python 互操作

`$module` 是一等模块对象；`$` 清晰地标记了互操作边界。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # 点式链：导入 importlib.metadata
$(PIL.Image).open(f)                # $(...) 显式锁定模块边界
$(sys).stderr                       # ...单段也一样：import sys, 属性 stderr
pyimport numpy, as: np              # 别名导入
pyimport sys                        # 裸导入将 `sys` 绑定为普通名称
np.array([1, 2]) * 10               # 运算符广播——它就是 Python
sys.stderr.write("...")             # 裸名称链无导入歧义
$json.dumps(data, indent: 2)        # 末尾关键字成为 kwargs
```

何时使用哪种拼写：

| 场景 | 拼写 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性引用且链式猜测错误时 | `$(os.path).sep`, `$(sys).stderr` |
| 在一个文件中多次使用的模块 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 经常使用的深层属性 | `@environ $(os).environ` 模块属性 |

在一个文件中重复使用 `$(...)` 拼写是一种坏味道——应声明 `pyimport`。永远不要为 Python API 编写包装模块；没有包装正是设计所在。

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

`try/rescue/after` 映射到 Python 异常；`rescue` 子句按异常类进行匹配：

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

raise "message"                    # -> 引发 RuntimeError("message")
```

`try` 是一个表达式。`try` 内部的尾调用 *不* 会进行优化（栈帧必须为处理程序保留）——其中的 `recur` 是编译错误，而非静默的栈消耗。

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

结构体类型在规范中以 `App.User.t()` 形式出现。

## Macros

编译时，卫生的，Elixir 风格的。宏在编译器内的确定性沙箱中运行，不留下任何运行时痕迹。

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

模板变量在每次展开时被重命名（卫生性）；`var!(name)` 有意地触及调用者的作用域；`unquote_splicing(list)` 拼接序列；`def unquote(head)` 构建定义。使用 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *Expand Macros* 命令检查结果。

## 符号与内嵌语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\\d+/                     # re.compile("\\d+") — string escapes apply
~python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

小写内嵌语言符号可承载整个文档，并通过 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内嵌语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

`~python` 是仅供 Python 专有写法的逃生通道；拼接处是编译后的 Gandora 表达式，其余所有内容原样传递。

## 编译器检查 — 警告是可证明的事实

每条警告仅针对静态确定的情况触发，定位在 `gan check`/`gan build`/编辑器波浪线中的定义行上，并具有机械性的修复方法（通常是一键快速修复）：

| 警告 | 含义 | 修复方法 |
| --- | --- | --- |
| undefined variable | 读取未绑定的变量 — 保证会引发 `NameError` | 修正名称 |
| unused binding | 已绑定但从未读取 | `_` 前缀 (`_meta`) |
| unreachable clause | 无守卫的全变量头部遮蔽了后面相同元数的子句（也包括 `case` 通配符） | 重新排序或删除 |
| discarded comprehension | 处于语句位置的 `for` | 使用 `Enum.each` |
| unused `defp` | 死私有函数 | 删除，或使用 `@allow :unused_function` |
| stack recursion | 自递归，但从未在尾位置 | 累加器形式、`recur`，或使用 `@allow :stack_recursion` |

`@allow` 目标会被检查 — 拼写错误会导致编译错误。将警告视为缺陷：代码库标准为零。

## 项目与命令行界面

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 负责管理依赖和 `.venv`；`gandora.local.jsonc` 保存每个开发者的偏好设置，并且不纳入 git 管理。

```console
gan init my-app          # new project        gan check      # analyze + lints only
gan run src/main.gan     # compile + execute  gan build      # compile to outDir
gan test                 # run @example doctests
gan fmt src              # format in place    gan fmt --check src   # CI gate
gan fmt --diff src       # show the diff      echo ... | gan fmt -  # stdin -> stdout
gan doc Enum.take        # docs (+ --locale)  gan repl       # interactive
gan expand src/x.gan     # macro output       gan try <file|->  # sandbox
gan init --package name
```

包以普通 wheel 的形式发布（`gan build && uv build && uv publish`），包含编译后的 Python、一个 `gandora.toml` 标记以及宏展开所用的 `.gan` 源文件——消费者通过 `uv add` 添加它们，无需引入 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一个 JSON 值——专为 agent 构建：

```console
gan lsc check --root .                  # 整个项目的诊断，包括 lint
gan lsc review --root .                 # check + 每个文件的实践/迁移建议
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格、说明（所有语言）、参数、tco 形状
gan lsc doc for --root .                # 语言结构也会回答（for/recur/with...）
gan lsc references Stats.mean --root .  # 所有调用点（+ 定义）
gan lsc wsymbols mean --root .          # 全项目符号搜索
gan lsc symbols Stats --root .          # 单个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python，以文本形式
gan lsc expand src/x.gan --root .       # 宏展开后的 quoted AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取 Python 侧文档
```

以及 **沙箱**——在生成的代码接触项目之前进行验证（GEP-0023）：

```console
echo 'Enum.mpa([1,2], fn x -> x end)' | gan try -
# -> {"ok": false, ..., "suggestions": [{"kind": "did_you_mean",
#     "message": "`Enum.mpa` is not a function of Enum — did you mean `Enum.map`?"}]}
```

`try` 会编译、lint、根据真实符号检查拼写（编辑距离）、标记跨语言习惯（`return`、`lambda`、`None`、Python `def ...():` 等）并给出 Gandora 拼写，然后在临时目录下运行并在超时后停止——返回生成的 Python、stdout 以及代码片段的最后一个值。`--no-run` 跳过执行。

一个高效的 agent 循环：**生成 → `gan try -` → 应用建议 → 再次 `try` → 写入项目** → 然后 `gan lsc check`（修复所有发现）→ `gan test` → `gan fmt src`。

## 代理风格检查清单

1. 每个公开的 `def`：`@doc` + `@spec`（每个参数加 `@param`）；任何具有有趣行为的代码都需加 `@example`——`gan test` 确保它们准确无误。
2. 规约：输入使用抽象容器（`sequence`、`mapping`），输出使用具体类型；真正通用的流程使用类型变量；结构体使用 `Mod.t()`；Python 边界处使用 `$mod.Type()`。
3. 迭代：映射类工作使用 `for`/`Enum`；无界循环使用累加器尾递归（当需要恒定栈时使用 `recur`）；仅对结构有界的递归使用 `@allow :stack_recursion`，且原因必须显而易见。
4. 互操作：一次性使用 `$`，重复使用 `pyimport`，不写包装器。
5. 零警告、`gan fmt` 清洁、文档测试通过——工具链、标准库、教程和游乐场都遵循这一准则；请与之保持一致。

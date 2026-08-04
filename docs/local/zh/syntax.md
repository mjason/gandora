# Gandora 语言手册

> 本文件是非规范性翻译，仅供参考；原文为英文版 [syntax.md](../../syntax.md)。

面向人类、同时也刻意面向AI代理的Gandora实用编写指南。规范性定义位于GEP（[`geps/`](../geps/)）中；本手册展示各部分的用法，并在多种写法并存时给出推荐的拼写方式。手册中的每个构造均在 [`examples/tour`](../examples/tour) 中得到演练（其已签入的 [`generated/`](../examples/tour/generated/) 展示了每章编译后的确切Python代码），并经过playground自检套件的实战检验。

以下基本规则贯穿全文：

- **零运行时。** 生成的Python代码是自包含且可读的——如同审阅者手写的一样。辅助函数按模块内联，部署从不依赖Gandora。
- **Elixir界面，Python语义底层。** 凡Elixir拥有某个构造，Gandora就按Elixir方式拼写；值均为普通的Python对象。
- **编译器会反馈。** 警告是静态可证明的事实，而非观点；悬停提示显示递归如何编译；`gan lsc` 以JSON形式提供所有事实。

## 模块与函数

每个文件一个 `defmodule`；模块名称必须与路径匹配
（`src/app/hello_web.gan` ↔ `App.HelloWeb` ↔ `app/hello_web.py`）。

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

函数返回其最后一个表达式。`def f(x), do: expr` 是单行形式。守卫（`when`）可以使用布尔型内建函数：
`is_list is_map is_tuple is_binary is_integer is_float is_number
is_atom is_nil is_function`、比较操作、`and or not`、算术运算。

## 数据

原子是 interned 字符串和**纯数据**——它们从不命名模块（那是 `$module` 的工作）。只有 `false` 和 `nil` 是假值；`0`、`""` 和 `[]` 是真值（Elixir 语义，而非 Python 的）。

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

`=`、`case`、函数头、`with` 以及 `for` 生成器都匹配模式：字面量、变量、`_`、元组、`[head | tail]`、映射、固定（`^x` — 匹配 `x` 的*现有*值）以及结构体。

```elixir
{:ok, value} = fetch()
[first | rest] = list

case result do
  {:ok, ^expected} -> "the pinned value"
  {:error, %{reason: r}} -> "failed: #{r}"
  _ -> "anything else"
end

cond do                       # 第一个真值条件（不是模式匹配）
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

失败的 `=`/`case` 匹配会引发 `GanMatchError`。重新绑定名称（`x = 1; x = 2`）会创建一个新绑定——中间创建的闭包保留旧值（见下文）。

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

**闭包在创建时按值捕获**（GEP-0021），完全如同在 Elixir 中一样——后续的重新绑定、尾递归循环迭代或推导步骤永远不会泄漏到先前创建的闭包中。编译器使用 Python 自身的惯用法实现这一点（`lambda x, *, n=n: x + n`）；调用参数数量保持严格。

## Pipelines

```elixir
xs |> normalize() |> $builtins.sum()   # first-argument pipe
df |> .groupby("k") |> .agg(spec)      # method pipe: calls ON the piped value
" gan " |> .strip() |> .upper()        # works on literals too
value.method(x).attr                   # postfix chains on anything
```

管道可在下一行以 `|>` 开头时继续。

## 迭代：推导式与递归

没有 `loop`，也没有 `while`。迭代使用 `for`、`Enum` 系列函数，或递归——而编译器会确保递归的安全性。

### `for` 推导式（GEP-0020）

编译为原生 Python 推导式：

```elixir
for x <- xs, x > 0, do: x * x                  # [x*x for x in xs if x > 0]
for x <- [1, 2], y <- [10, 20], do: {x, y}     # 多个生成器
for {k, v} <- pairs, into: %{}, do: {k, v * 2} # 字典推导式
for {k, v} <- [{"a", 1}, :bad], do: k          # 不匹配的元素会被跳过
```

主体是一个表达式；`into: %{}` 要求主体为 `{key, value}` 元组。
推导式 **构建一个集合**——若将其用于副作用，编译器会发出警告；请改用 `Enum.each`。

### 尾递归编译为循环（GEP-0019）

对封闭函数的尾位置调用会变成 `while True:` 内部的参数重新绑定——在任何深度下栈空间恒定：

```elixir
def sum_to(n), do: sum_to(n, 0)
def sum_to(0, acc), do: acc
def sum_to(n, acc), do: sum_to(n - 1, acc + n)   # 一百万个栈帧也没问题
```

`recur(args)` 是同一跳转的 **受检查的** 拼写形式：它必须位于尾位置且与某个子句的元数匹配，否则构建失败——当恒定栈空间是要求而非期望时，请使用它：

```elixir
def drain(q) do
  if empty?(q), do: :done, else: recur(pop(q))
end
```

非尾递归（`n * fact(n - 1)`）保持为真实调用，并使用 Python 栈（约 1000 帧）；编译器会在定义处 **发出警告**。结构递归——深度受数据本身限制，例如树遍历——是合理的：在声明中明确允许后，警告就会消失：

```elixir
@allow :stack_recursion
def depth([]), do: 1
def depth(x) when is_list(x), do: 1 + $builtins.max(for e <- x, do: depth(e))
def depth(_), do: 0
```

每个函数的编译形态都是可见的：悬停显示 `♻ tail recursion → while loop` 或 `⚠ native call stack`，`gan doc` 会打印它，而 `gan lsc doc` 返回 `"tco": "loop" | "stack"`。

## 类型系统（`@spec`）

`@spec` 声明函数的类型；编译器会根据子句对其进行验证，并在生成的 Python 中生成 **PEP 484 类型注解**，这样 `pyright`/`mypy` 可以检查调用者，悬停时也能显示真实类型。每个定义组一个 `@spec`，放在第一个子句之前，与其他注解一起。

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

具体容器表示“正是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为类似的东西”——在**参数**中优先使用它们，这样调用者可以传入元组、区间或生成器，而且因为 `list` 在 Python 类型系统中是不变的，而 `Sequence` 是协变的：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 将被遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 可索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读类字典
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**输入抽象，输出具体**——接受 `sequence(t)`，返回 `list(t)`。

### 联合类型和 `nil`

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec parse(string()) :: tuple(atom(), term()) | atom()
```

### 类型变量（泛型）

一个**一或两个字符**的裸小写名称是类型变量；每个变量在输出中成为模块级 `typing.TypeVar`。相同的字母在 spec 中表示“相同的类型”：

```elixir
@spec at(sequence(a), integer()) :: a | nil
@spec map(iterable(a), fun()) :: list(b)
@spec get(map(k, v), k, v) :: v
```

更长的裸名称会触发错误并给出提示（“您是不是想写 `name()`？”），这样拼写错误的 `intger` 不会悄悄成为泛型。

### 命名参数

`name :: type` 在 spec 中给参数命名——自文档化，也是签名帮助显示的内容：

```elixir
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

### 宿主（Python）类型

任何 Python 类型都可以通过 `$module` 出现；括号进行参数化：

```elixir
@spec sales() :: $pandas.DataFrame()
@spec compile(string()) :: $re.Pattern()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
# -> subprocess.CompletedProcess[str]，并生成对应的导入语句
```

### 结构体类型

`Mod.t()` 是由该模块中的 `defstruct` 定义的结构体类：

```elixir
@spec sale(App.Shop.t()) :: App.Shop.t()   # -> def sale(item: Shop) -> Shop
```

### 与默认值及多参数列表的交互

一个 spec 覆盖整个组；为完整的参数列表（包括默认值）编写 spec。生成的委托和分发器会携带注解。如果 `@spec` 命名了一个不存在的函数或参数数量，则会导致编译错误，因此 spec 不会无声地腐烂。

## 文档与文档测试

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

- `@param` 名称必须与子句头部变量匹配——在编译时验证。
- `@example` 是唯一的文档测试通道：`gan test` 会将 `gan>` 行编译为原生的 Python 文档测试并运行。预期输出是 Python 的 `repr`（即 `inspect/1` 显示的内容）：原子输出为 `'ok'`，映射为 `{'k': 1}`，布尔值为 `True`。
- 标准：**每个公开的 `def` 都带有 `@doc` + `@spec`**（当有参数时还包括 `@param`）；面向用户的接口添加 `_trans` 对。

文档语言是**开发者**的个人偏好，而非项目配置：`gandora.local.jsonc`（`{"docLocale": "zh-CN"}`，已加入 gitignore，位于 `gandora.jsonc` 旁边）> `GAN_DOC_LOCALE` 环境变量/编辑器设置 > 仅默认语言。`gan doc Mod.fun --locale zh-CN` 显式指定；`gan lsc doc` 始终以 JSON 格式返回所有区域设置。

## 测试（GEP-0024）

一条命令，两层结构：`gan test` 先运行所有 `@example` 文档测试，再运行 `tests/*.gan` 中所有 `test_*` 函数——这些函数在项目完整的模块解析下编译，并由 pytest 执行（只需添加一次：`uv add --dev pytest`）。

```elixir
# tests/test_stats.gan
defmodule TestStats do
  @moduledoc "文档测试未覆盖的边界情况。"

  use Test

  describe "mean" do
    test "平均计算均匀" do
      assert Stats.mean([1, 2, 3, 4]) == 2.5   # 失败时命名左右两侧
    end
  end

  test "成员关系与否定" do
    assert 16 in Stats.even_squares(1, 8)
    refute 9 in Stats.even_squares(1, 8)
  end

  test "类型化异常抛出" do
    _ = Test.assert_raise($builtins.KeyError, fn -> Map.fetch!(%{}, "no") end)
    nil
  end
end
```

ExUnit 表面：`test "name" do`（定义 `test_<slug>`），`describe`（为内部名称添加前缀），`assert`/`refute`（比较时报告两个操作数），以及作为普通函数的 `Test.assert_eq / assert_nil / assert_raise / assert_in_delta / flunk`。`tests/` 从不发布——它位于源代码根目录之外。

## Python 互操作

`$module` 是一等模块对象；`$` 明确标记了互操作边界。

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

何时使用何种写法：

| 场景 | 写法 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 一次性引用且链式启发式猜测错误时 | `$(os.path).sep`, `$(sys).stderr` |
| 模块在文件中反复使用 | `pyimport sys`（或 `, as:`）+ 裸名 |
| 深层属性频繁使用 | `@environ $(os).environ` 模块属性 |

在一个文件中重复使用 `$(...)` 写法是一种不好的做法——应声明 `pyimport`。永远不要为 Python API 编写包装模块；不设包装正是设计意图。

装饰器通过 `@decorate` 附加；模块属性持有导入时的状态：

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

`try` 是一个表达式。`try` 内部的尾调用 *不* 会被优化（帧必须保留以供处理程序使用）——其中的 `recur` 会导致编译错误，而不是默默地消耗栈。

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

编译期、卫生、Elixir 风格。宏在编译器的确定性沙盒中运行，不留运行时痕迹。

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

模板变量在每次展开时重命名（卫生）；`var!(name)` 有意触及调用者的作用域；`unquote_splicing(list)` 将序列展开；`def unquote(head)` 构建定义。通过 `require Mod`（或 `import`/`use`）引入宏。使用 `gan expand file` 或编辑器的 *Expand Macros* 命令检查结果。

## 符号与嵌入式语言

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\\d+/                     # re.compile("\\d+") — string escapes apply
~python(sum(i*i for i in range(n)))   # one verbatim Python expression
```

无大写字母的嵌入式语言符号承载整个文档，其中包含 `<%= expr %>` 拼接回 Gandora（编辑器会高亮内部语言）：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

`~python` 是用于仅限 Python 拼写的逃生舱；拼接内容是编译后的 Gandora 表达式，其余所有内容原样传递。

## 编译器 lint — 警告是可证明的事实

每条警告仅针对静态确定的情况触发，在 `gan check`/`gan build`/编辑器波浪线下落于定义行，并附带机械式的修复方式（通常是一键快速修复）：

| 警告 | 含义 | 补救措施 |
| --- | --- | --- |
| 未定义变量 | 读取一个未绑定的值 — 保证会引发 `NameError` | 修正名称 |
| 未使用的绑定 | 已绑定但从未读取 | 使用 `_` 前缀（`_meta`） |
| 不可达子句 | 一个无守卫的全变量头部覆盖了之后同元数的子句（也包括 `case` 通配符） | 重新排序或删除 |
| 废弃的推导式 | 语句位置的 `for` | 使用 `Enum.each` |
| 未使用的 `defp` | 死私有函数 | 删除，或使用 `@allow :unused_function` |
| 栈递归 | 自递归且从未在尾位置 | 累加器形式、`recur` 或使用 `@allow :stack_recursion` |

`@allow` 目标会被检查 — 拼写错误将导致编译错误。将警告视为缺陷：代码库标准为零。

## 项目与 CLI

`gandora.jsonc` 配置编译器（`source`、`outDir`、`targetPython`、`exclude`、`package`、`pyPackage`）；`pyproject.toml` + `uv` 管理依赖和 `.venv`；`gandora.local.jsonc` 保存开发者个人偏好，不纳入 Git 管理。

```console
gan init my-app          # 新建项目             gan check      # 仅分析与 lint
gan run src/main.gan     # 编译并执行           gan build      # 编译到 outDir
gan test                 # 运行 @example 文档测试
gan fmt src              # 原地格式化           gan fmt --check src   # CI 门禁
gan fmt --diff src       # 显示差异             echo ... | gan fmt -  # 标准输入 -> 标准输出
gan doc Enum.take        # 文档（+ --locale）   gan repl       # 交互式
gan expand src/x.gan     # 宏展开输出
gan init --package name
```

包以标准 wheel 格式发布（`gan build && uv build && uv publish`），附带编译后的 Python、一个 `gandora.toml` 标记文件以及宏展开所需的 `.gan` 源文件——使用者通过 `uv add` 添加，无需引入 Gandora 运行时。

## AI 工具箱：`gan lsc`

每个语言事实都是 stdout 上的一个 JSON 值——专为智能体构建：

```console
gan lsc check --root .                  # 裁决：{diagnostics, suggestions}
gan lsc diagnostics src/x.gan --root .  # 单个文件
gan lsc doc Enum.take --root .          # 规格说明、文档（所有语言）、参数、tco 形状
gan lsc doc for --root .                # 语言结构也能回答（for/recur/with...）
gan lsc references Stats.mean --root .  # 所有调用点（+ 定义）
gan lsc wsymbols mean --root .          # 项目范围的符号搜索
gan lsc symbols Stats --root .          # 单个模块的概要
gan lsc definition Stats.mean --root .  # 定义位置
gan lsc compile src/x.gan --root .      # 生成的 Python，以文本形式
gan lsc expand src/x.gan --root .       # 宏展开后的带引号 AST
gan lsc ast src/x.gan --root .          # 解析树（Elixir 编码）
gan lsc pydoc numpy.array --root .      # 通过 jedi 获取的 Python 侧文档
```

**检查裁决是编译器的教学传递**（GEP-0025）：  
`gan check` 输出编译器诊断信息 *和* Advisor 建议（实践差距、跨语言迁移提示、拼写错误名称的“你是不是想找”）；`gan lsc check` 将相同信息以 JSON 对象 `{diagnostics, suggestions}` 返回。**`gan build` 首先运行 check** —— 一个重型编译器，采用 Rust 方式：错误阻止构建，警告和建议打印后继续构建。

```console
gan lsc check --root .
# {"ok": true, "clean": false, "diagnostics": [...],
#  "suggestions": [{"kind": "did_you_mean", "line": 3,
#   "message": "`Enum.mpa` 不是 Enum 的函数——你是不是想找 `Enum.map`？", ...}]}
```

裁决以交通灯信号开始：`ok`（编译通过——无错误）和 `clean`（ok **且** 零警告 **且** 零建议）。  
红色 → 修复错误；黄色 → 阅读建议；绿色 → 提交。  
建议携带其第一个证据所在的行，跨多个文件的相同发现会合并为一条带注释的条目，作用范围覆盖 `src/` 以及顶层 `tests/*.gan`（测试模块会获得迁移和惯用提示，但豁免于库注解覆盖率检查）。

一个高效的智能体工作循环：**编写 → `gan check`（修复所有发现）→ `gan test` → `gan build`**。当不确定某物生成什么时，使用 `gan lsc compile file`；当不确定语法时，使用 `gan lsc doc <construct>`（`for`、`spec`、`test`、……）。

## 代理风格检查表

1. 每个公共 `def`：`@doc` + `@spec`（+ 每个参数的 `@param`）；对于任何有趣的行为，`@example` —— `gan test` 保持它们诚实。
2. 规范：输入的抽象容器（`sequence`、`mapping`），输出的具体容器；类型变量用于真正通用的流程；`Mod.t()` 用于结构体；在 Python 边界使用 `$mod.Type()`。
3. 迭代：`for`/`Enum` 用于映射形状的工作；累加器尾递归用于无限循环（当常量堆栈是要求时使用 `recur`）；`@allow :stack_recursion` 仅用于结构有界的递归，且原因显而易见。
4. 互操作：`$` 一次性使用，`pyimport` 用于重复使用，无包装器。
5. 零警告，`gan fmt` 干净，doctests 通过——工具链、标准库、教程和 playground 都坚持这一标准；与它们保持一致。

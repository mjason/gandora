# 类型系统

`@spec` 声明函数的类型；编译器会将其与函数子句进行验证，并在生成的 Python 代码中发出 **PEP 484 注解**，从而让 `pyright`/`mypy`/ty 能够检查调用方，悬停时也能显示真实类型。每个定义组最多一个 `@spec`，与其他注解一同置于第一个子句之前。

```elixir
@spec mean(xs :: sequence(number()), precision :: integer()) :: float()
def mean(xs, precision \\ 2) do ... end
# -> def mean(xs: collections.abc.Sequence[int | float], precision: int = 2) -> float:
```

## 标量

| Gandora | Python 注解 |
| --- | --- |
| `integer()` | `int` |
| `float()` | `float` |
| `number()` | `int \| float` |
| `string()` | `str` |
| `boolean()` | `bool` |
| `atom()` | `str`（原子是驻留字符串） |
| `nil` | `None` |
| `term()` | `object` |
| `fun()` | `Callable`（无参数化） |

## 容器——抽象入、具体出

具体容器明确指定“就是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器则表示“任何表现出类似行为的类型”——在**参数**中优先使用它们，以便调用者可以传入元组、区间或生成器（在 Python 的类型系统中，`list` 是不变的，而 `Sequence` 是协变的）：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 仅遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 可索引/可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读类字典
keyword()                  # 关键词列表：list[tuple[str, object]]
```

经验法则：**接受 `sequence(t)`，返回 `list(t)`。** 构建的实践检查会在参数为具体类型时提醒你。

## 联合类型、类型变量、命名参数

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec get(map(k, v), k, v) :: v               # 1–2 letter names are TypeVars
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

在同一个 spec 中，相同的字母表示“相同的类型”。较长的裸名称会引发错误，并附带针对性的提示（`Int` → "write `integer()`"；`{:ok, x}` 元组字面量 → "documents as `tuple(atom(), x)`"）——键入错误不会悄无声息地变成泛型。

## 宿主（Python）与结构体类型

```elixir
@spec sales() :: $pandas.DataFrame()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
@spec sale(App.Shop.t()) :: App.Shop.t()   # struct class from defstruct
```

导入会随注解一起发出。

## 规范不会腐烂

引用一个不存在的函数或元数的 `@spec` 会产生编译错误；一个规范覆盖整个默认参数组，生成的签名携带注解——因此 `gan build --strict`（full ty type-flow）始终有可靠的内容可供检查。

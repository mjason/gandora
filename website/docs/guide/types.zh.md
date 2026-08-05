# 类型系统

一条拼写规则：**类型是一种调用**——`integer()`、`list(t)`、`Mod.t()`、`$mod.Type()` 都带括号；唯一的裸拼写是类型变量（1–2 个小写字母）和字面量 `nil`。其他任何东西都是编译错误，并附带修复提示。

`@spec` 声明函数的类型；编译器会根据子句对其进行验证，并在生成的 Python 中发出 **PEP 484 注解**，以便 `pyright`/`mypy`/ty 检查调用者，悬停时显示真实类型。每个定义组一个 `@spec`，与其他注解一起放在第一个子句之前。

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
| `atom()` | `str`（原子是内部化的字符串） |
| `nil` | `None` |
| `term()` | `object` |
| `fun()` | `Callable`（未参数化） |

## 容器 — 入参抽象，出参具体

具体容器表示“就是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为像它的东西” — 在**参数**中优先使用它们，以便调用者可以传入元组、区间或生成器（`list` 在 Python 类型系统中是不变的；`Sequence` 是协变的）：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 只遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 可索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读的类字典
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**接受 `sequence(t)`，返回 `list(t)`。** 构建的实践检查会在参数为具体类型时提醒你。

## 联合类型、类型变量、命名参数

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec get(map(k, v), k, v) :: v               # 1–2 letter names are TypeVars
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

同一个字母在一个 spec 中表示“相同的类型”。更长的裸名会触发错误，并附带针对性的提示（`Int` → "写成 `integer()`"；`{:ok, x}` 元组字面量 → "应写为 `tuple(atom(), x)`"）——一个拼写错误不会悄然变成一个泛型。

## 宿主（Python）与结构体类型

```elixir
@spec sales() :: $pandas.DataFrame()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
@spec sale(App.Shop.t()) :: App.Shop.t()   # struct class from defstruct
```

导入语句随注解一同发出。

## 规范不可腐化

将 `@spec` 命名一个不存在的函数或元数视为编译错误；一个 spec 覆盖整个默认参数组，生成的签名携带注释——因此 `gan build --strict`（完全类型流）总是有诚实的东西可检查。

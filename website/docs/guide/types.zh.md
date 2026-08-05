# 类型系统

一条拼写规则：**类型即调用** —— `integer()`, `list(t)`, `Mod()`, `$mod.Type()` 都需要括号；唯一允许裸写的类型是类型变量（1–2个小写字母）和字面量 `nil`。其他任何写法都会导致编译错误，并附带修正建议。

`@spec` 声明函数的类型；编译器会根据子句对其进行验证，并在生成的 Python 代码中输出 **PEP 484 注解**，从而让 `pyright`/`mypy`/ty 检查调用者，悬停提示显示真实类型。每个定义组只能有一个 `@spec`，放在第一个子句之前，与其他注解放在一起。

```elixir
@spec mean(xs :: sequence(number()), precision :: integer()) :: float()
def mean(xs, precision \\ 2) do ... end
# -> def mean(xs: collections.abc.Sequence[int | float], precision: int = 2) -> float:
```

## Scalars

| Gandora | Python 注解 |
| --- | --- |
| `integer()` | `int` |
| `float()` | `float` |
| `number()` | `int \| float` |
| `string()` | `str` |
| `boolean()` | `bool` |
| `atom()` | `str` (原子是驻留字符串) |
| `nil` | `None` |
| `term()` | `object` |
| `fun()` | `Callable` (未参数化) |

## 容器 — 抽象入，具体出

具体容器说“正是这个 Python 类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器说“任何行为像它的东西”——优先用于**参数**，这样调用者可以传递元组、范围或生成器（在 Python 类型系统中 `list` 是不变的，`Sequence` 是协变的）：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 只遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 可索引 / 可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读的类字典
keyword()                  # 关键字列表：list[tuple[str, object]]
```

经验法则：**接受 `sequence(t)`，返回 `list(t)`。** 构建的实践检查会在参数是具体类型时提醒你。

## 命名类型（`@type`）

`@type` 命名一个类型——并且是**泛型的声明点**：参数在头部声明，引用进行元数检查，所有内容在编译时展开（零运行时）：

```elixir
@type age() :: integer()
@type result(t) :: tuple(atom(), t)
@type scores() :: map(string(), age())

@spec parse(string()) :: result(integer())    # -> tuple[str, int]
@spec load(string()) :: Mod.result(string())  # cross-module reference
```

`@type` 体中未声明的变量、引用处错误的元数、重复、遮蔽内置类型以及递归都是编译错误，并附带修复提示；`@type` 上方的 `@doc` 为其提供文档。在 `@spec` 内部，短类型变量保持隐式作用域——但若某个 spec 的整个返回类型是一个在其他地方未使用的变量，则会给出一个实践提示（因为它未约束任何内容）。

## 联合类型、类型变量、命名参数

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec get(map(k, v), k, v) :: v               # 1–2 letter names are TypeVars
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

相同的字母在同一规格说明中表示“相同的类型”。较长的裸名称会导致错误，并带有针对性提示（`Int` → “写成 `integer()`”；`{:ok, x}` 元组字面量 → “文档写成 `tuple(atom(), x)`”）——拼写错误不会悄然变成一个泛型。

## 宿主 (Python) 和结构体类型

```elixir
@spec sales() :: $pandas.DataFrame()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
@spec sale(App.Shop()) :: App.Shop()   # struct class from defstruct
```

导入随注解一起发出。

## 规范不会腐烂

一个命名了不存在的函数或元数的 `@spec` 会产生编译错误；一个 spec 覆盖整个默认参数组，而生成的签名携带注释——所以 `gan build --strict`（全类型流）总是有可靠的内容可以检查。

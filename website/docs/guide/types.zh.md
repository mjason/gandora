# 类型系统

一个拼写规则：**类型是一个调用** — `integer()`, `list(t)`, `Mod()`, `$mod.Type()` 都带有括号；唯一不带括号的拼写是类型变量（1-2个小写字母）和字面量 `nil`。其他任何形式都会导致编译错误，并附带修复建议。

`@spec` 声明函数的类型；编译器会将其与子句进行验证，并在生成的 Python 中发出 **PEP 484 注解**，以便 `pyright`/`mypy`/ty 检查调用者，悬停时显示真实类型。每个定义组只有一个 `@spec`，放在第一个子句之前的其他注解位置。

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
| `atom()` | `str` (原子是内部化字符串) |
| `nil` | `None` |
| `term()` | `object` |
| `fun()` | `Callable` (未参数化) |

## 容器 — 抽象输入，具体输出

具体容器表示“确切的这个Python类型”：

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

抽象容器表示“任何行为类似的事物”——优先用于**参数**，以便调用者可以传递元组、范围或生成器（`list`在Python类型系统中是不变的；`Sequence`是协变的）：

```elixir
iterable(t)                # collections.abc.Iterable[t]  — 可遍历一次
sequence(t)                # collections.abc.Sequence[t]  — 可索引/可重复遍历
mapping(k, v)              # collections.abc.Mapping[k, v] — 只读字典类
keyword()                  # 关键字列表: list[tuple[str, object]]
```

经验法则：**接受`sequence(t)`，返回`list(t)`。** 构建的实践检查会在参数为具体时提醒你。

## 联合类型、类型变量、命名参数

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec get(map(k, v), k, v) :: v               # 1–2 letter names are TypeVars
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

相同的字母在同一个规约中表示“相同的类型”。较长裸名会被视为错误，并附带针对性的提示（`Int` → “请写成 `integer()`”；`{:ok, x}` 元组字面量 → “文档化为 `tuple(atom(), x)`”）——打字错误不会静默地变成泛型。

## 宿主语言 (Python) 与结构体类型

```elixir
@spec sales() :: $pandas.DataFrame()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
@spec sale(App.Shop()) :: App.Shop()   # struct class from defstruct
```

这些导入随注解一同发出。

## 规格说明不会腐烂

`@spec` 引用了一个不存在的函数或参数数量，会导致编译错误；一条规格说明覆盖整个默认参数组，生成的签名携带这些注解——因此 `gan build --strict`（完整的 ty 类型流）始终有可检查的诚实信息。

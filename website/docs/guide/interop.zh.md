# Python 互操作

`$module` 是一等模块对象；`$` 可见地标记互操作边界——看到 `$`，想到 Python。

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # dotted chain: imports importlib.metadata
$(PIL.Image).open(f)                # $(...) locks the module boundary explicitly
$(sys).stderr                       # ...single-segment too
pyimport numpy, as: np              # aliased import
pyimport sys                        # bare import binds `sys` as a plain name
np.array([1, 2]) * 10               # operators broadcast — it's just Python
$json.dumps(data, indent: 2)        # trailing keywords become kwargs
```

## 何时使用哪种拼写

| 情况 | 拼写 |
| --- | --- |
| 一次性引用 | `$math.sqrt(x)` |
| 链式启发式猜测错误时 | `$(os.path).sep`, `$(sys).stderr` |
| 模块在文件中重复使用 | `pyimport sys`（或 `, as:`）+ 裸名称 |
| 深层属性经常使用 | `@environ $(os).environ` 模块属性 |
| 整个 Python 表达式 | `$python(...)` — 参见 [Sigils](sigils.md) |

在一个文件中重复使用 `$module` 是构建工具指出的不良做法 — 应声明 `pyimport`。**永远不要编写围绕 Python API 的包装模块**；不包装正是设计所在。

## 方法、管道、异常

```elixir
" gan " |> .strip() |> .upper()     # method pipe on the piped value
df |> .groupby("k") |> .agg(spec)   # the pandas fluent world, one form
rescue
  e in $builtins.ValueError -> ...  # exceptions are spelled through their module
```

Python 内置函数位于 `$builtins` 下：`$builtins.len(x)`、`$builtins.round(x, 2)`、`$builtins.list(range(n))`。一个名称类似内建函数（如 `$round`）的未使用的导入会引发构建错误，提示使用 `$builtins.` 拼写。

## 构建保证了什么

互操作性经过验证，而非期望：生成的 Python 的导入和成员引用在构建时使用 ty 进行检查——`$requests`（未安装依赖）、`Enum.mpa` 或参数个数错误的调用都会映射到你的源代码行，是一个**构建错误**，在任何运行之前。参见[构建判决](../tooling/build.md)。

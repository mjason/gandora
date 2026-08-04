# 模块与函数

一个文件一个 `defmodule`，且模块名必须与路径匹配：
`src/app/hello_web.gan` ↔ `App.HelloWeb` ↔ `app/hello_web.py`。

```elixir
defmodule App.Math do
  @moduledoc "Docstrings come from @moduledoc and @doc."

  @doc "Multi-clause dispatch, top to bottom, with guards."
  @spec fact(integer()) :: integer()
  @allow :stack_recursion
  def fact(0), do: 1
  def fact(n) when n > 0, do: n * fact(n - 1)

  def empty?(xs), do: length(xs) == 0   # ? 和 ! 是名称的一部分
  defp helper(x), do: x + 1             # 私有：在 Python 中表现为下划线开头
end
```

函数返回其**最后一个表达式**——没有 `return`。
`def f(x), do: expr` 是单行形式；`do ... end` 块则包含多个表达式。

## 多子句头部和守卫

相同名称和元数的子句按模式从上到下进行分发。守卫（`when`）可以使用布尔型内置函数——`is_list is_map is_tuple is_binary is_integer is_float is_number is_atom is_nil is_function`——以及比较、`and or not`和算术运算。

```elixir
def classify(n) when n < 0, do: :negative
def classify(0), do: :zero
def classify(_), do: :positive
```

编译器证明子句可达性：全匹配子句后的子句会触发警告（GEP-0022）。

## 默认参数

尾部默认参数使用 `\\`；调用者可省略后缀参数：

```elixir
def greet(name, greeting \\ "hi", mark \\ "!") do
  "#{greeting} #{name}#{mark}"
end

greet("Ada")            # "hi Ada!"
greet("Ada", "yo")      # "yo Ada!"
```

不可变字面量默认参数会编译为 **原生 Python 签名**（`def greet(name, greeting="hi", mark="!")`）——类型明确、诚实，且可由构建的验证层进行参数数量检查。可变默认参数（`\\ []`）保持调用时求值，采用 Elixir 风格。

## 跨模块调用

```elixir
alias App.Stats            # then Stats.mean(...)
import App.Stats           # bare mean(...) — sparingly
Stats.mean([1, 2, 3])
```

## 注释规范

每个公开的 `def` 都带有 `@doc` 和 `@spec`（以及在面向用户工具中每个参数对应的 `@param`）；`@example` 文档测试记录了有趣的行为，并由 `gan test` 执行。构建的实践检查能让您保持诚实——参见[构建裁决](../tooling/build.md)。

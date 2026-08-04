# Gandora

**一种带有Elixir风格的语言，编译为可读的Python——为AI编写代码的时代而构建。**

Gandora为你提供Elixir的表达式一切表面——模式匹配、管道、不可变绑定、宏——并将其编译成一位细心的审阅者会手写出的Python代码。没有运行时，没有框架：生成的代码独立运行，可在任何Python运行的地方运行，与numpy、pandas、FastAPI以及你所依赖的其他一切共存。

```elixir
defmodule Stats do
  @moduledoc "Descriptive statistics over plain lists."

  @doc "The arithmetic mean, rounded to `precision` decimals."
  @spec mean(xs :: sequence(number()), precision :: integer()) :: float()
  @example """
      gan> Stats.mean([1, 2, 3, 4])
      2.5
  """
  def mean(xs, precision \\ 2) do
    $builtins.round(Enum.sum(xs) / Enum.count(xs), precision)
  end
end
```

编译为：

```python
def mean(xs: collections.abc.Sequence[int | float], precision: int = 2) -> float:
    """The arithmetic mean, rounded to `precision` decimals. ..."""
    return round(gandora_std.enum.sum(xs) / gandora_std.enum.count(xs), precision)
```

## 为何选择 Gandora

**编译器会回话。** `gan build` 输出单一结论：编译错误、可证明事实的警告、最佳实践建议，以及*产物验证*——生成的 Python 会通过 [ty](https://docs.astral.sh/ty/) 检查，因此未定义的函数、死导入或参数数量错误的调用都会成为构建错误并附带“您是否想要”的提示，而非运行时的意外。一个符合惯用写法的项目报告**零噪音**；语言自身的标准库、工具链、示例教程和 playground 均遵循这一原则。

**为 AI 编写者打造，对人类读者诚实。** 每条诊断都教导正确的拼写。`gan lsc` 将整个结论——以及文档、符号、引用——以 JSON 格式提供给代理。在我们的定期评估中，一个从未见过手册的小型模型，仅通过遵循构建的教导，就能在 24 项任务的严苛测试中达到全绿。

**零运行时。** 部署永远不依赖 Gandora。从 Gandora 项目构建的 wheel 包含普通 Python 以及用于宏消费者的 `.gan` 源文件——像任何包一样发布到 PyPI。

**递归是安全的。** 尾递归编译为 `while` 循环（百万帧级没问题）；非尾递归会在编译时收到警告并附带累加器改写方案；每个函数的编译后形状在悬停时可见。

## The pieces（组成部分）

| | |
| --- | --- |
| `ganc` | 阶段0编译器（Rust，零依赖） |
| `gan` | 任务运行器 — 构建 · 运行 · 测试 · 格式化 · REPL（用 Gandora 编写） |
| `gan-lsp` / `gan lsc` | 语言服务器及其 JSON 控制台（用 Gandora 编写） |
| `Enum` `Map` `List` `Keyword` `String` `Test` | 标准库 — 每个函数都有文档、类型标注和 doctest |

从[快速入门](getting-started.md)开始，浏览[指南](guide/modules.md)，或让您的智能体访问[使用 AI 编写 Gandora](ai.md)。

# 使用 AI 编写 Gandora

Gandora 的设计基于这样一个假设：其大部分代码将由模型编写。这一假设以具体的方式塑造了工具链：单一裁决、教导式纠正、处处使用 JSON，以及提示作为一等文本。

## 循环

**write → `gan build`（修复每个发现）→ `gan test` → ship。**

对于 agent 而言，JSON 表面为 `gan lsc check`：

```json
{"ok": true, "clean": false, "diagnostics": [], "suggestions": [
  {"kind": "practice", "line": 3,
   "message": "Annotation coverage: missing @spec on: total — e.g. ..."}]}
```

将其视为交通信号灯：`ok: false` → 修复错误；`clean: false` → 应用每一条建议；`clean: true` → 提交。每条消息都包含正确的拼写——模型无需打开手册即可应用。

## 裁决为你捕获的内容

- **跨语言惯用写法**：`return`、`while`、`None`、f-strings、`Integer.to_string`、`Stream.map`、`|> then(...)`、裸异常名称 — 每个都在一行中获得 Gandora 拼写。
- **运行前必然致命的事实**：未定义的函数（针对真实符号表的“你是不是想找”功能）、死导入、参数数量错误的调用 — 从 ty 对*生成的* Python 进行检查，然后映射回你的源代码行。
- **懒惰，而非拼写错误**：模型很少拼写错误；它们跳过注解，在列表推导式可读性更好的地方编写 map+filter 链，并且五次使用 `$math` 而不是一次性 `pyimport`。练习环节会指出每个快捷方式及其修复方法。

## 语言中的提示与工具模式

`~prompt` 是原始文本——引号、花括号、反斜杠和内联 JSON 均无需转义；`<%= expr %>` 用来拼接值：

```elixir
@task ~prompt(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~prompt"""
You are a coding agent. The user's name is <%= name %>.
Reply with {"status": "ok"} when done.
"""
```

工具模式是普通的映射——粘贴的 JSON 文档通过将 `:` 替换为 `=>` 即可变成映射（构建工具会即时识别此转换）：

```elixir
@tools [
  %{"type" => "function", "function" => %{
    "name" => "build_code",
    "description" => "The build verdict for a Gandora module.",
    "parameters" => %{"type" => "object", "properties" => %{
      "code" => %{"type" => "string"}}, "required" => ["code"]}}}
]
```

## 在 Gandora 中的代理

该仓库自带的评估框架是一个基于工具调用的 DeepSeek 代理，使用 Gandora 编写——litellm 负责模型调用，`~prompt` 负责任务，尾递归负责循环：

```elixir
resp = litellm.completion(
  model: "openai/" <> model,
  api_base: base, api_key: key,
  messages: messages, tools: tools(), timeout: 120
)
msg = Enum.at(resp.choices, 0).message
```

这之所以重要，是因为它同时也是度量方式：一个从未见过手册的小模型，仅持有 `build_code` / `run_code` / `read_doc` / `list_symbols` / `submit`，便能在包含 24 项任务的严苛测试中收敛至绿灯——从 fizzbuzz 到尊重优先级的表达式求值器——完全依靠遵循裁判结果的教导。

## 值得接入的发现工具

```console
gan lsc doc spec        # the type-language cheat sheet
gan lsc doc with        # construct cards: shapes, not prose
gan lsc symbols Enum    # what actually exists
gan lsc compile f.gan   # what the Python will be
```

# 使用AI编写Gandora

Gandora 的设计假设其大部分代码将由模型编写。这一假设具体影响了工具链的形态：单一裁决、教导式修正、处处 JSON，以及将提示作为一等文本。

## 一次调用开始

`gan agent` 打印整个会话简报——工作循环以及**上下文包**（GEP-0026）：每个标准函数名称、每个项目签名、构造索引、规范速查表以及当前裁决，全部在一个提示词大小的输出中。一个从上下文包开始的模型会立即编写，而不是将前五次工具调用花费在发现上。`gan lsc pack --root .` 以 JSON 格式返回相同内容；`gan lsc pack Enum` 深入探索单个模块。

## 循环

**编写 → `gan build`（修复所有发现）→ `gan test` → 提交。**

对于智能体，JSON 接口为 `gan lsc check`：

```json
{"ok": true, "clean": false, "diagnostics": [], "suggestions": [
  {"kind": "practice", "line": 3,
   "message": "Annotation coverage: missing @spec on: total — e.g. ..."}]}
```

将其视为红绿灯：`ok: false` → 修复错误；`clean: false` → 应用所有建议；`clean: true` → 提交。每条消息均包含正确的拼写——模型无需查阅手册即可直接应用。

## 裁决为你捕获的内容

- **跨语言习惯性写法**：`return`、`while`、`None`、f-字符串、
  `Integer.to_string`、`Stream.map`、`|> then(...)`、裸异常
  名称——每个都在一行中以 Gandora 拼写给出。
- **运行前必然致命的错误**：未定义的函数
  （针对真实符号表给出“你是不是想找”建议）、死导入、
  参数数量错误的调用——从对*生成的* Python 的 `ty` 检查，映射回
  你的源代码行。
- **懒惰而非拼写错误**：模型很少拼写错误；它们会跳过
  注解，在列表推导式可读性更好的地方编写 map+filter 链，
  并且五次使用 `$math` 而不是一次 `pyimport`。
  练习环节会指出每个捷径及其修复方法。

## 语言中的提示词与工具模式

`~p` 是原始散文——引号、花括号、反斜杠以及内联 JSON 无需转义；`<%= expr %>` 用于拼接值：

```elixir
@task ~p(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~p"""
You are a coding agent. The user's name is <%= name %>.
Reply with {"status": "ok"} when done.
"""
```

工具模式是普通的映射——通过将 `:` 替换为 `=>`，粘贴的 JSON 文档即可转换为映射（构建工具会即时教导这一点）：

```elixir
@tools [
  %{"type" => "function", "function" => %{
    "name" => "build_code",
    "description" => "The build verdict for a Gandora module.",
    "parameters" => %{"type" => "object", "properties" => %{
      "code" => %{"type" => "string"}}, "required" => ["code"]}}}
]
```

## 一个代理，在 Gandora 中

该仓库自带的评估框架是一个用 Gandora 编写的工具调用型 DeepSeek 代理——其中 `litellm` 负责模型调用，`~p` 负责任务，尾递归负责循环：

```elixir
resp = litellm.completion(
  model: "openai/" <> model,
  api_base: base, api_key: key,
  messages: messages, tools: tools(), timeout: 120
)
msg = Enum.at(resp.choices, 0).message
```

这很重要，因为它同时也是衡量标准：一个从未见过手册的小模型，仅持有 `build_code` / `run_code` / `read_doc` / `list_symbols` / `submit` 这几个工具，就能在包含 24 个任务的考验中——从 fizzbuzz 到遵循优先级规则的表达式求值器——完全通过遵循评判结果的教导而收敛到绿色状态。

## 值得接入的发现工具

```console
gan agent               # 以上所有内容，一次输出，Markdown 格式
gan lsc pack Enum       # 单个模块的完整文档，一次调用
gan lsc doc with for spec --brief   # 多张卡片，每行一条
gan lsc compile f.gan   # Python 生成的代码是什么
```

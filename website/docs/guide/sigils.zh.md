# 符号与嵌入式语言

## 经典示例

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\d+/                      # re.compile(r"\d+")
```

## 两个层级，两个符号

嵌入内容根据主体**是什么**来划分（GEP-0009）：

**`~<name>` 是文本。** 任何名称——`~sql`、`~markdown`、`~html`，
等等——标记一个原始字符串模板。主体逐字节原样传递；
`<%= expr %>` 拼接运行时值；编辑器高亮内嵌语言：

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`$python(expr)` 是代码。** 一个逐字逐句的 Python 表达式进入
程序——属于 `$` 世界，如同 `$math.sqrt`：

```elixir
def evens_capped(xs, limit) do
  $python([x for x in <%= xs %> if x % 2 == 0][:<%= limit %>])
end
```

`$python` 中的插值是*编译后的 Gandora 表达式*；其余部分就是你的 Python 代码，不做任何改动。它是仅用 Python 语法的逃生舱——应谨慎使用。

## `~p` — 无需转义的 AI 文本

提示文本-sigil 的正式名称 (GEP-0009-R006): 引号、花括号、反斜杠和内联 JSON 都原始通过 — 再也不用与 `\\\"` 搏斗:

```elixir
@task ~p(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~p"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

## 数据并非嵌入——而是映射

刻意**没有** JSON 字面量：Gandora 映射是唯一的数据拼写（更丰富——原子键、任意表达式）。粘贴的 JSON 文档通过将 `:` 替换为 `=>` 成为映射（构建工具会即时识别）；运行时的 JSON 文本使用 `$json.loads(s)`。

Heredoc（`"""`）主体会像普通 heredoc 一样去除缩进，因此模板在模块内自然缩进，同时生成左对齐的值。
